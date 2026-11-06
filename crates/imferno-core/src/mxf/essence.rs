//! MXF essence-header validation backed by `smpte-mxf` (the
//! SMPTE-reference `regxmllib` port, verified semantically identical
//! to its Java counterpart across 75 real-world IMF MXFs).
//!
//! This module covers the partition-pack layer that enforces
//! ST 377-1 §8.3.3 (OP1a operational pattern) and ST 2067-5:2013
//! §5.1.5 (partition pack exclusivity) — the foundational essence
//! constraints SMPTE compliance requires.
//!
//! Full header-metadata-set parsing (Preface, MaterialPackage,
//! AudioChannelLabelSubDescriptor, etc.) lives in `mxf::metadata`
//! which pulls in the `regxml` crate; this module is partition-only
//! so the integration shape and dependency surface stay small.

use std::io::{Read, Seek};
use std::path::Path;

use smpte_klv::KlvStream;
use smpte_mxf::{
    is_partition_pack_key, seek_header_partition, MxfError, PartitionKind, PartitionPack,
    PartitionStatus,
};
use smpte_types::Ul;

use crate::diagnostics::{Location, ValidationIssue};
use crate::mxf::codes::{ImfernoMxf, St2067_2_2016, St377_1_2011};

/// SMPTE Operational Pattern 1A UL — bytes per ST 378:2004 §6.
///
/// IMF mandates OP1a via ST 2067-2 §5.2 (which cites ST 2067-5:2013 §5.2),
/// itself citing ST 377-1:2011 §8.3.3 for the constraint that an OP1a
/// file contains exactly one MaterialPackage with one essence.
const OP1A_UL_BYTES: [u8; 16] = [
    0x06, 0x0e, 0x2b, 0x34, 0x04, 0x01, 0x01, 0x01, 0x0d, 0x01, 0x02, 0x01, 0x01, 0x01, 0x01, 0x00,
];

/// Mask used when comparing the registry-byte (byte 7) and the version
/// byte (byte 16) of an OP UL — both vary across SMPTE registry
/// revisions but the operational-pattern identity is byte 13-14.
const OP_UL_MATCH_MASK: u128 = 0xffff_ffff_ff00_ffff_ffff_ffff_ffff_ff00;

/// Validate an MXF file against the partition-pack-level rules of
/// ST 2067-2 §5.2 and ST 377-1 §8.3.3. Returns a catalogue of
/// `ValidationIssue`s; the caller folds them into the usual
/// `ValidationReport`.
pub fn validate_mxf_essence(path: &Path) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();
    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(e) => {
            issues.push(open_failure_issue(path, &e));
            return issues;
        }
    };
    let mut reader = std::io::BufReader::new(file);
    match parse_header_pack(&mut reader) {
        Ok((header_offset, pack)) => {
            // ST 377-1 §6.2.4: an Open header partition is fine when a
            // Closed footer partition carries header metadata. Only look
            // for the footer when the header is Open. If the footer cannot
            // be located reliably, the file's status is undetermined and
            // the header is not flagged.
            let file_is_open = is_open(pack.status)
                && match locate_footer(&mut reader, header_offset, &pack) {
                    FooterSearch::Found(footer) => {
                        is_open(footer.status) || footer.header_byte_count == 0
                    }
                    FooterSearch::Absent => true,
                    FooterSearch::Unknown => false,
                };
            issues.extend(check_partition_pack(path, &pack, file_is_open));
        }
        Err(e) => {
            issues.push(parse_failure_issue(path, &e));
        }
    }
    issues
}

/// Returns the absolute byte offset of the header partition (after any
/// run-in) and its partition pack.
fn parse_header_pack<R: Read + Seek>(reader: &mut R) -> Result<(u64, PartitionPack), MxfError> {
    let header_offset = seek_header_partition(reader)?;
    let pack = read_partition_pack_at(reader, header_offset, None)?.ok_or(MxfError::Truncated(
        "first triplet was not a partition pack",
    ))?;
    Ok((header_offset, pack))
}

/// Upper bound for a partition pack value: 88 fixed bytes plus a batch
/// of 16-byte essence container ULs (ST 377-1 §7.1). Generous on purpose.
const MAX_PARTITION_PACK_VALUE_LEN: u64 = 64 * 1024;

/// Read the partition pack at `offset`. The 16-byte key is checked first,
/// so a non-partition KLV is rejected without reading its value.
/// `max_value_len` bounds the value for offsets taken from untrusted data
/// (FooterPartition, RIP entries); `None` for positions found by parsing
/// the file itself (the header partition, the KLV walk). In every case a
/// value that would run past the end of the file is rejected unread.
fn read_partition_pack_at<R: Read + Seek>(
    reader: &mut R,
    offset: u64,
    max_value_len: Option<u64>,
) -> Result<Option<PartitionPack>, MxfError> {
    reader.seek(std::io::SeekFrom::Start(offset))?;
    let mut key = [0u8; 16];
    reader.read_exact(&mut key)?;
    if !is_partition_pack_key(&Ul::from_bytes(key)) {
        return Ok(None);
    }
    let Some((len, len_size)) = read_ber_length(reader) else {
        return Err(MxfError::Truncated(
            "partition pack has an invalid BER length",
        ));
    };
    if max_value_len.is_some_and(|max| len > max) {
        return Ok(None);
    }
    // The declared length comes from the file: reject a value that runs past
    // the end of the file before reading (and allocating) it.
    let file_len = reader.seek(std::io::SeekFrom::End(0))?;
    let value_end = offset
        .checked_add(16 + len_size)
        .and_then(|v| v.checked_add(len));
    if value_end.is_none_or(|end| end > file_len) {
        return Err(MxfError::Truncated(
            "partition pack value runs past the end of the file",
        ));
    }
    reader.seek(std::io::SeekFrom::Start(offset))?;
    let mut stream = KlvStream::new(reader);
    let triplet = stream
        .read_triplet()
        .map_err(MxfError::from)?
        .ok_or(MxfError::Truncated("expected partition pack triplet"))?;
    PartitionPack::from_triplet(&triplet)
}

/// Random Index Pack key, ST 377-1:2011 §12.1 Table 29 (p110): structure
/// version 01h, structure kind 01h, pack kind 11h, version 01h, reserved
/// 00h. Byte 8 (registry version) is not compared.
///
/// smpte-mxf 0.1.1's own RIP key drops the structure-kind byte, so its
/// `RandomIndexPack::from_triplet` rejects real RIPs; the pack is parsed
/// here instead.
const RIP_KEY: [u8; 16] = [
    0x06, 0x0e, 0x2b, 0x34, 0x02, 0x05, 0x01, 0x01, 0x0d, 0x01, 0x02, 0x01, 0x01, 0x11, 0x01, 0x00,
];

fn is_rip_key(key: &[u8; 16]) -> bool {
    key.iter()
        .zip(RIP_KEY.iter())
        .enumerate()
        .all(|(i, (a, b))| i == 7 || a == b)
}

/// Upper bound for a RIP value. ST 377-1 §12.2: 12 bytes per partition
/// (4-byte BodySID + 8-byte offset) plus the 4-byte overall length. IMF
/// track files have body partitions about every 60 s (ST 2067-2 §5.1.3),
/// i.e. a few thousand partitions even for very long files; 16 MiB allows
/// about 1.4 million entries.
const MAX_RIP_VALUE_LEN: u64 = 16 * 1024 * 1024;

/// Outcome of looking for the footer partition.
enum FooterSearch {
    Found(PartitionPack),
    /// The whole file was walked and has no footer partition.
    Absent,
    /// The footer could not be located reliably (malformed KLV, I/O error).
    Unknown,
}

/// Locate the footer partition pack. ST 377-1:2011 §7.1 and §12.2: the
/// FooterPartition item and RIP offsets are relative to the start of the
/// Header Partition. §7.2.2: an Open header may carry FooterPartition = 0
/// even when a footer exists, and the RIP is optional (§12), so as a last
/// resort the KLV items are walked from the header partition.
fn locate_footer<R: Read + Seek>(
    reader: &mut R,
    header_offset: u64,
    header: &PartitionPack,
) -> FooterSearch {
    let footer_at = |reader: &mut R, rel: u64| -> Option<PartitionPack> {
        let abs = header_offset.checked_add(rel)?;
        read_partition_pack_at(reader, abs, Some(MAX_PARTITION_PACK_VALUE_LEN))
            .ok()
            .flatten()
            .filter(|p| matches!(p.kind, PartitionKind::Footer))
    };

    if header.footer_partition != 0 {
        if let Some(footer) = footer_at(reader, header.footer_partition) {
            return FooterSearch::Found(footer);
        }
    }
    if let Some(rel) = rip_last_partition_offset(reader) {
        if let Some(footer) = footer_at(reader, rel) {
            return FooterSearch::Found(footer);
        }
    }
    walk_to_footer(reader, header_offset)
}

/// Offset (relative to the header partition) of the last partition listed
/// in a Random Index Pack at the end of the file, if there is one.
fn rip_last_partition_offset<R: Read + Seek>(reader: &mut R) -> Option<u64> {
    let file_len = reader.seek(std::io::SeekFrom::End(0)).ok()?;
    if file_len < 4 {
        return None;
    }
    reader.seek(std::io::SeekFrom::End(-4)).ok()?;
    let mut len_bytes = [0u8; 4];
    reader.read_exact(&mut len_bytes).ok()?;
    let rip_len = u64::from(u32::from_be_bytes(len_bytes));
    // Overall length = key (16) + BER length (at most 9) + value.
    if rip_len == 0 || rip_len > file_len || rip_len > 16 + 9 + MAX_RIP_VALUE_LEN {
        return None;
    }
    let rip_start = file_len - rip_len;
    // The trailing length may point at anything: check the RIP key and a
    // bounded BER length before reading the value.
    reader.seek(std::io::SeekFrom::Start(rip_start)).ok()?;
    let mut key = [0u8; 16];
    reader.read_exact(&mut key).ok()?;
    if !is_rip_key(&key) {
        return None;
    }
    let value_len = match read_ber_length(reader) {
        Some((len, len_size)) if len <= MAX_RIP_VALUE_LEN && 16 + len_size + len == rip_len => len,
        _ => return None,
    };
    // §12.2: (BodySID u32, ByteOffset u64) pairs, then the u32 overall length.
    if value_len < 4 + 12 || (value_len - 4) % 12 != 0 {
        return None;
    }
    let mut value = vec![0u8; usize::try_from(value_len).ok()?];
    reader.read_exact(&mut value).ok()?;
    let last_entry = &value[value.len() - 4 - 12..value.len() - 4];
    Some(u64::from_be_bytes(last_entry[4..12].try_into().ok()?))
}

/// Walk KLV items from the header partition to find the footer partition
/// pack, seeking over values without reading them.
fn walk_to_footer<R: Read + Seek>(reader: &mut R, header_offset: u64) -> FooterSearch {
    let Ok(file_len) = reader.seek(std::io::SeekFrom::End(0)) else {
        return FooterSearch::Unknown;
    };
    let mut pos = header_offset;
    while pos < file_len {
        if reader.seek(std::io::SeekFrom::Start(pos)).is_err() {
            return FooterSearch::Unknown;
        }
        let mut key = [0u8; 16];
        if reader.read_exact(&mut key).is_err() {
            return FooterSearch::Unknown;
        }
        let Some((len, len_size)) = read_ber_length(reader) else {
            return FooterSearch::Unknown;
        };
        if is_partition_pack_key(&Ul::from_bytes(key)) {
            match read_partition_pack_at(reader, pos, None) {
                Ok(Some(pack)) if matches!(pack.kind, PartitionKind::Footer) => {
                    return FooterSearch::Found(pack);
                }
                Ok(_) => {}
                Err(_) => return FooterSearch::Unknown,
            }
        }
        let Some(next) = pos
            .checked_add(16 + len_size)
            .and_then(|p| p.checked_add(len))
        else {
            return FooterSearch::Unknown;
        };
        pos = next;
    }
    if pos == file_len {
        FooterSearch::Absent
    } else {
        FooterSearch::Unknown
    }
}

/// Read a BER definite length (ST 336). Returns (length, bytes consumed).
fn read_ber_length<R: Read>(reader: &mut R) -> Option<(u64, u64)> {
    let mut first = [0u8; 1];
    reader.read_exact(&mut first).ok()?;
    if first[0] < 0x80 {
        return Some((u64::from(first[0]), 1));
    }
    let n = usize::from(first[0] & 0x7f);
    if n == 0 || n > 8 {
        return None;
    }
    let mut buf = [0u8; 8];
    reader.read_exact(&mut buf[8 - n..]).ok()?;
    Some((u64::from_be_bytes(buf), 1 + n as u64))
}

fn is_open(status: PartitionStatus) -> bool {
    matches!(
        status,
        PartitionStatus::OpenIncomplete | PartitionStatus::OpenComplete
    )
}

/// `file_is_open`: the header is Open and no Closed footer partition with
/// header metadata exists (ST 377-1 §6.2.4).
fn check_partition_pack(
    path: &Path,
    pack: &PartitionPack,
    file_is_open: bool,
) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    // ST 2067-5:2013 §5.1.5 / ST 377-1 §6.4: the header partition MUST
    // be the first partition in the file.
    if !matches!(pack.kind, PartitionKind::Header) {
        issues.push(
            ValidationIssue::from_code(St377_1_2011::NonHeaderFirstPartition,
                format!(
                    "First partition in MXF file at {} is {:?}, not the Header partition required by ST 377-1 §6.4",
                    path.display(),
                    pack.kind,
                ),
            )
            .with_location(Location::new().with_file(path.to_path_buf())),
        );
    }

    // ST 2067-2 §5.2 / ST 377-1 §8.3.3: IMF essence MXFs SHALL be
    // OP1a. Compare via the UL match mask so registry-byte and
    // version-byte revisions don't false-flag conformant files.
    let pack_op = u128::from_be_bytes(*pack.operational_pattern.as_bytes());
    let op1a = u128::from_be_bytes(OP1A_UL_BYTES);
    if (pack_op & OP_UL_MATCH_MASK) != (op1a & OP_UL_MATCH_MASK) {
        issues.push(
            ValidationIssue::from_code(St2067_2_2016::OperationalPatternNotOP1A,
                format!(
                    "MXF file at {} uses Operational Pattern UL {} — ST 2067-2 §5.2 requires OP1a (ST 378:2004)",
                    path.display(),
                    format_ul(pack.operational_pattern.as_bytes()),
                ),
            )
            .with_location(Location::new().with_file(path.to_path_buf())),
        );
    }

    // ST 377-1 §6.2.5: the Header Partition may be Open or Closed.
    // §6.2.4: a file is Closed if it has a Closed Header Partition or a
    // Closed Footer Partition containing Header Metadata; delivered
    // files shall be Closed. Flag only an Open header with no such
    // footer, i.e. an Open MXF file.
    if file_is_open {
        issues.push(
            ValidationIssue::from_code(
                St377_1_2011::HeaderPartitionOpen,
                format!(
                    "Header partition in MXF file at {} is {:?} and no Closed footer partition \
                     with header metadata was found, so the file is Open (ST 377-1 §6.2.4)",
                    path.display(),
                    pack.status,
                ),
            )
            .with_location(Location::new().with_file(path.to_path_buf())),
        );
    }

    // ST 377-1 §8.3.3: the IMF essence MXF MUST contain header
    // metadata in its header partition (header_byte_count > 0).
    if pack.header_byte_count == 0 {
        issues.push(
            ValidationIssue::from_code(St377_1_2011::MissingHeaderMetadata,
                format!(
                    "Header partition in MXF file at {} has no header metadata (HeaderByteCount = 0) — \
                     IMF requires header metadata in the header partition",
                    path.display(),
                ),
            )
            .with_location(Location::new().with_file(path.to_path_buf())),
        );
    }

    issues.push(
        ValidationIssue::from_code(
            ImfernoMxf::EssenceContainersDetected,
            format!(
                "MXF file at {} declares {} essence container UL(s) in its header partition",
                path.display(),
                pack.essence_containers.len(),
            ),
        )
        .with_location(Location::new().with_file(path.to_path_buf())),
    );

    issues
}

fn open_failure_issue(path: &Path, err: &std::io::Error) -> ValidationIssue {
    ValidationIssue::from_code(
        ImfernoMxf::OpenFailed,
        format!("Could not open MXF file {}: {}", path.display(), err),
    )
    .with_location(Location::new().with_file(path.to_path_buf()))
}

fn parse_failure_issue(path: &Path, err: &MxfError) -> ValidationIssue {
    ValidationIssue::from_code(
        ImfernoMxf::PartitionPackParseFailed,
        format!(
            "Failed to parse the header partition pack of {}: {}",
            path.display(),
            err
        ),
    )
    .with_location(Location::new().with_file(path.to_path_buf()))
}

fn format_ul(bytes: &[u8; 16]) -> String {
    let mut s = String::with_capacity(48);
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 && i % 4 == 0 {
            s.push('.');
        }
        s.push_str(&format!("{:02x}", b));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagnostics::Severity;

    fn write_partition_key(buf: &mut Vec<u8>, kind_byte: u8, status_byte: u8) {
        // ST 377-1 partition pack key with the partition-kind nibble
        // (byte 14) and status (byte 15) varied per call.
        buf.extend_from_slice(&[
            0x06,
            0x0e,
            0x2b,
            0x34,
            0x02,
            0x05,
            0x01,
            0x01,
            0x0d,
            0x01,
            0x02,
            0x01,
            0x01,
            kind_byte,
            status_byte,
            0x00,
        ]);
    }

    /// A minimal header partition pack value with OP1a UL and one
    /// essence container UL. Used to keep tests independent of any
    /// real MXF fixture (which we don't vendor).
    fn synthetic_header_partition(op_ul: &[u8; 16], header_byte_count: u64) -> Vec<u8> {
        synthetic_partition(op_ul, header_byte_count, 0)
    }

    /// Partition pack value with an explicit FooterPartition offset.
    fn synthetic_partition(
        op_ul: &[u8; 16],
        header_byte_count: u64,
        footer_partition: u64,
    ) -> Vec<u8> {
        let mut value = Vec::new();
        value.extend_from_slice(&0x0102u16.to_be_bytes()); // major.minor
        value.extend_from_slice(&0u16.to_be_bytes()); // minor (unused; major-minor laid out as u16 pair)
        value.extend_from_slice(&1u32.to_be_bytes()); // kag_size
        value.extend_from_slice(&0u64.to_be_bytes()); // this_partition
        value.extend_from_slice(&0u64.to_be_bytes()); // previous_partition
        value.extend_from_slice(&footer_partition.to_be_bytes());
        value.extend_from_slice(&header_byte_count.to_be_bytes());
        value.extend_from_slice(&0u64.to_be_bytes()); // index_byte_count
        value.extend_from_slice(&0u32.to_be_bytes()); // index_sid
        value.extend_from_slice(&0u64.to_be_bytes()); // body_offset
        value.extend_from_slice(&0u32.to_be_bytes()); // body_sid
        value.extend_from_slice(op_ul);
        // essence_containers: BatchHeader{count=1, item_len=16}+one UL
        value.extend_from_slice(&1u32.to_be_bytes());
        value.extend_from_slice(&16u32.to_be_bytes());
        value.extend_from_slice(op_ul);
        value
    }

    fn write_klv(out: &mut Vec<u8>, key: &[u8], value: &[u8]) {
        out.extend_from_slice(key);
        // BER long form, 4-byte length
        out.push(0x84);
        out.extend_from_slice(&(value.len() as u32).to_be_bytes());
        out.extend_from_slice(value);
    }

    fn dump_to_temp(buf: &[u8]) -> tempfile::NamedTempFile {
        use std::io::Write;
        let mut f = tempfile::NamedTempFile::new().unwrap();
        f.write_all(buf).unwrap();
        f.flush().unwrap();
        f
    }

    #[test]
    fn validate_mxf_essence_clean_header_emits_only_info_diagnostic() {
        // Synthetic Closed-Complete header partition with OP1a UL +
        // non-zero header metadata count → should fire only the
        // informational "essence containers detected" diagnostic.
        let mut buf = Vec::new();
        let mut key = Vec::new();
        write_partition_key(&mut key, 0x02, 0x04); // header, closed+complete
        let value = synthetic_header_partition(&OP1A_UL_BYTES, 1024);
        write_klv(&mut buf, &key, &value);
        let tmp = dump_to_temp(&buf);
        let issues = validate_mxf_essence(tmp.path());
        let non_info: Vec<_> = issues
            .iter()
            .filter(|i| i.severity != Severity::Info)
            .collect();
        assert!(
            non_info.is_empty(),
            "Clean header should emit only Info diagnostics, got: {:#?}",
            issues
        );
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("EssenceContainersDetected")),
            "expected EssenceContainersDetected info diagnostic"
        );
    }

    #[test]
    fn validate_mxf_essence_flags_non_op1a_pattern() {
        // Same partition pack but with an OP2a UL (byte 13 = 0x02).
        let mut op = OP1A_UL_BYTES;
        op[13] = 0x02;
        let mut buf = Vec::new();
        let mut key = Vec::new();
        write_partition_key(&mut key, 0x02, 0x04);
        let value = synthetic_header_partition(&op, 1024);
        write_klv(&mut buf, &key, &value);
        let tmp = dump_to_temp(&buf);
        let issues = validate_mxf_essence(tmp.path());
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("OperationalPatternNotOP1A")),
            "expected OperationalPatternNotOP1A, got: {:#?}",
            issues
        );
    }

    #[test]
    fn validate_mxf_essence_flags_zero_header_byte_count() {
        let mut buf = Vec::new();
        let mut key = Vec::new();
        write_partition_key(&mut key, 0x02, 0x04);
        let value = synthetic_header_partition(&OP1A_UL_BYTES, 0); // no metadata
        write_klv(&mut buf, &key, &value);
        let tmp = dump_to_temp(&buf);
        let issues = validate_mxf_essence(tmp.path());
        assert!(
            issues
                .iter()
                .any(|i| i.code.contains("MissingHeaderMetadata")),
            "expected MissingHeaderMetadata, got: {:#?}",
            issues
        );
    }

    /// Build a file with an Open header partition followed by a footer
    /// partition pack with the given status byte and HeaderByteCount.
    fn open_header_with_footer(footer_status: u8, footer_header_bytes: u64) -> Vec<u8> {
        open_header_with_footer_opts(footer_status, footer_header_bytes, 0, true)
    }

    /// `run_in`: bytes of run-in before the header partition.
    /// `header_points_to_footer`: whether the Open header's FooterPartition
    /// is set (relative to the header partition) or left at 0.
    fn open_header_with_footer_opts(
        footer_status: u8,
        footer_header_bytes: u64,
        run_in: usize,
        header_points_to_footer: bool,
    ) -> Vec<u8> {
        // Header value length is fixed, so compute the header KLV size
        // first to know where the footer starts.
        let header_len = {
            let mut probe = Vec::new();
            let mut key = Vec::new();
            write_partition_key(&mut key, 0x02, 0x01);
            write_klv(
                &mut probe,
                &key,
                &synthetic_partition(&OP1A_UL_BYTES, 1024, 0),
            );
            probe.len() as u64
        };
        let mut buf = vec![0xAAu8; run_in];
        let mut key = Vec::new();
        write_partition_key(&mut key, 0x02, 0x01); // header, open+incomplete
        let header_footer_field = if header_points_to_footer {
            header_len
        } else {
            0
        };
        write_klv(
            &mut buf,
            &key,
            &synthetic_partition(&OP1A_UL_BYTES, 1024, header_footer_field),
        );
        let mut key = Vec::new();
        write_partition_key(&mut key, 0x04, footer_status); // footer
        write_klv(
            &mut buf,
            &key,
            &synthetic_partition(&OP1A_UL_BYTES, footer_header_bytes, header_len),
        );
        buf
    }

    fn has_header_partition_open(issues: &[ValidationIssue]) -> bool {
        issues
            .iter()
            .any(|i| i.code.contains("HeaderPartitionOpen"))
    }

    /// ST 377-1:2011 §6.2.5: the Header Partition may be Open; §6.2.4: a
    /// file is Closed if it has a Closed Footer Partition with Header
    /// Metadata. Such a file must not be flagged.
    #[test]
    fn open_header_with_closed_footer_metadata_is_not_flagged() {
        let tmp = dump_to_temp(&open_header_with_footer(0x04, 2048));
        let issues = validate_mxf_essence(tmp.path());
        assert!(
            !has_header_partition_open(&issues),
            "Open header + Closed footer with header metadata is a Closed file: {issues:#?}",
        );
    }

    /// ST 377-1:2011 §6.2.4: without a Closed Header Partition or a Closed
    /// Footer Partition containing Header Metadata the file is Open.
    #[test]
    fn open_header_without_closed_footer_metadata_is_flagged() {
        // Closed footer but no header metadata in it.
        let tmp = dump_to_temp(&open_header_with_footer(0x04, 0));
        assert!(has_header_partition_open(&validate_mxf_essence(tmp.path())));

        // No footer at all.
        let mut buf = Vec::new();
        let mut key = Vec::new();
        write_partition_key(&mut key, 0x02, 0x01);
        write_klv(
            &mut buf,
            &key,
            &synthetic_header_partition(&OP1A_UL_BYTES, 1024),
        );
        let tmp = dump_to_temp(&buf);
        assert!(has_header_partition_open(&validate_mxf_essence(tmp.path())));
    }

    /// ST 377-1:2011 §7.2.2 (p45): an Open Header Partition Pack may carry
    /// FooterPartition = 0 even when a footer exists, and §12 (p110) makes the
    /// RIP optional. Such a file with a Closed footer carrying header
    /// metadata is Closed (§6.2.4) and must not be flagged.
    #[test]
    fn open_header_with_zero_footer_offset_and_no_rip_is_not_flagged() {
        let tmp = dump_to_temp(&open_header_with_footer_opts(0x04, 2048, 0, false));
        let issues = validate_mxf_essence(tmp.path());
        assert!(
            !has_header_partition_open(&issues),
            "footer found by walking partitions; file is Closed: {issues:#?}",
        );
    }

    /// ST 377-1:2011 §7.1 (p43): FooterPartition is relative to the start of
    /// the Header Partition, so a run-in shifts the absolute position.
    #[test]
    fn open_header_with_run_in_finds_footer_relative_to_header() {
        let tmp = dump_to_temp(&open_header_with_footer_opts(0x04, 2048, 100, true));
        let issues = validate_mxf_essence(tmp.path());
        assert!(
            !has_header_partition_open(&issues),
            "footer offset is relative to the header partition: {issues:#?}",
        );
    }

    /// Read+Seek wrapper counting the bytes actually read.
    struct CountingReader {
        inner: std::io::Cursor<Vec<u8>>,
        bytes_read: usize,
    }

    impl Read for CountingReader {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            let n = self.inner.read(buf)?;
            self.bytes_read += n;
            Ok(n)
        }
    }

    impl Seek for CountingReader {
        fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
            self.inner.seek(pos)
        }
    }

    /// A FooterPartition offset (or RIP entry) may point at arbitrary
    /// bytes. A non-partition KLV there with a huge declared length must be
    /// rejected from its 16-byte key, without reading its value.
    #[test]
    fn footer_offset_at_huge_non_partition_klv_is_not_read() {
        let mut header = Vec::new();
        let mut key = Vec::new();
        write_partition_key(&mut key, 0x02, 0x01); // header, open
        write_klv(
            &mut header,
            &key,
            &synthetic_partition(&OP1A_UL_BYTES, 1024, 0),
        );
        let junk_offset = header.len() as u64;
        // Rebuild the header with FooterPartition pointing at the junk KLV.
        let mut buf = Vec::new();
        write_klv(
            &mut buf,
            &key,
            &synthetic_partition(&OP1A_UL_BYTES, 1024, junk_offset),
        );
        assert_eq!(buf.len() as u64, junk_offset);
        // Non-partition key, declared length 32 MiB, 1 MiB actually present.
        buf.extend_from_slice(&[0x06, 0x0e, 0x2b, 0x34, 0x01, 0x02, 0x01, 0x01]);
        buf.extend_from_slice(&[0x0d, 0x01, 0x03, 0x01, 0x15, 0x01, 0x05, 0x00]);
        buf.push(0x84);
        buf.extend_from_slice(&(32u32 << 20).to_be_bytes());
        buf.extend(std::iter::repeat_n(0u8, 1 << 20));

        let mut reader = CountingReader {
            inner: std::io::Cursor::new(buf),
            bytes_read: 0,
        };
        let header_pack = read_partition_pack_at(&mut reader, 0, None)
            .unwrap()
            .unwrap();
        reader.bytes_read = 0;
        let result = locate_footer(&mut reader, 0, &header_pack);
        assert!(matches!(result, FooterSearch::Unknown));
        assert!(
            reader.bytes_read < 64 * 1024,
            "footer lookup read {} bytes; the junk value must not be read",
            reader.bytes_read
        );
    }

    /// The trailing 4 bytes of a file give the RIP's overall length
    /// (ST 377-1 §12.2) but may point at anything. A non-RIP KLV there with
    /// a huge declared length must be rejected from its key, unread.
    #[test]
    fn rip_length_at_huge_non_rip_klv_is_not_read() {
        let mut buf = Vec::new();
        let mut key = Vec::new();
        write_partition_key(&mut key, 0x02, 0x01);
        write_klv(
            &mut buf,
            &key,
            &synthetic_partition(&OP1A_UL_BYTES, 1024, 0),
        );
        let junk_start = buf.len();
        buf.extend_from_slice(&[0x06, 0x0e, 0x2b, 0x34, 0x01, 0x02, 0x01, 0x01]);
        buf.extend_from_slice(&[0x0d, 0x01, 0x03, 0x01, 0x15, 0x01, 0x05, 0x00]);
        buf.push(0x84);
        buf.extend_from_slice(&(32u32 << 20).to_be_bytes());
        buf.extend(std::iter::repeat_n(0u8, 1 << 20));
        let trailing = (buf.len() + 4 - junk_start) as u32;
        buf.extend_from_slice(&trailing.to_be_bytes());

        let mut reader = CountingReader {
            inner: std::io::Cursor::new(buf),
            bytes_read: 0,
        };
        assert_eq!(rip_last_partition_offset(&mut reader), None);
        assert!(
            reader.bytes_read < 64 * 1024,
            "RIP lookup read {} bytes; the junk value must not be read",
            reader.bytes_read
        );
    }

    /// A genuine RIP (ST 377-1 §12) is still found after the key check.
    #[test]
    fn rip_last_partition_offset_reads_valid_rip() {
        let mut buf = open_header_with_footer_opts(0x04, 2048, 0, false);
        let rip_start = buf.len();
        let mut value = Vec::new();
        for (sid, off) in [(0u32, 0u64), (0, 12345)] {
            value.extend_from_slice(&sid.to_be_bytes());
            value.extend_from_slice(&off.to_be_bytes());
        }
        // Overall length: key 16 + BER 5 + entries + 4-byte length field.
        let overall = (16 + 5 + value.len() + 4) as u32;
        value.extend_from_slice(&overall.to_be_bytes());
        // ST 377-1:2011 §12.1 Table 29 (p110), bytes 1-16 as listed there.
        let key = [
            0x06, 0x0e, 0x2b, 0x34, 0x02, 0x05, 0x01, 0x01, 0x0d, 0x01, 0x02, 0x01, 0x01, 0x11,
            0x01, 0x00,
        ];
        write_klv(&mut buf, &key, &value);
        assert_eq!(buf.len() - rip_start, overall as usize);
        let mut reader = std::io::Cursor::new(buf);
        assert_eq!(rip_last_partition_offset(&mut reader), Some(12345));
    }

    /// Real MXF files from the test fixtures end with a RIP; the RIP lookup
    /// alone must find the footer partition.
    #[test]
    fn rip_lookup_finds_footer_in_real_mxf_fixtures() {
        let fixtures: [(&str, &[u8]); 4] = [
            (
                "audio1",
                include_bytes!("../../tests/fixtures/mxf/audio1.mxf"),
            ),
            (
                "audio2",
                include_bytes!("../../tests/fixtures/mxf/audio2.mxf"),
            ),
            (
                "video1",
                include_bytes!("../../tests/fixtures/mxf/video1.mxf"),
            ),
            (
                "video2",
                include_bytes!("../../tests/fixtures/mxf/video2.mxf"),
            ),
        ];
        for (name, bytes) in fixtures {
            let mut reader = std::io::Cursor::new(bytes.to_vec());
            let header_offset = seek_header_partition(&mut reader).unwrap();
            let rel = rip_last_partition_offset(&mut reader)
                .unwrap_or_else(|| panic!("{name}: RIP not found"));
            let footer = read_partition_pack_at(
                &mut reader,
                header_offset + rel,
                Some(MAX_PARTITION_PACK_VALUE_LEN),
            )
            .unwrap()
            .unwrap_or_else(|| panic!("{name}: no partition pack at RIP offset"));
            assert!(
                matches!(footer.kind, PartitionKind::Footer),
                "{name}: RIP's last entry is not the footer"
            );
        }
    }

    /// A partition pack whose declared length runs past the end of the file
    /// (here ~256 MiB in a ~1 KiB file) is rejected from the key and length
    /// alone: the value is never read, so nothing is allocated for it.
    #[test]
    fn partition_pack_length_past_eof_is_rejected_unread() {
        let mut buf = Vec::new();
        write_partition_key(&mut buf, 0x02, 0x04); // header, closed+complete
        buf.push(0x84);
        buf.extend_from_slice(&(256u32 << 20).to_be_bytes());
        buf.extend(std::iter::repeat_n(0u8, 1000));

        let mut reader = CountingReader {
            inner: std::io::Cursor::new(buf.clone()),
            bytes_read: 0,
        };
        assert!(read_partition_pack_at(&mut reader, 0, None).is_err());
        assert!(
            reader.bytes_read <= 16 + 9,
            "read {} bytes; only the key and length may be read",
            reader.bytes_read
        );

        // End to end: reported as a parse failure, no panic.
        let tmp = dump_to_temp(&buf);
        let issues = validate_mxf_essence(tmp.path());
        assert!(issues
            .iter()
            .any(|i| i.code.contains("PartitionPackParseFailed")));
    }

    /// The header partition pack is at a trusted position (the start of the
    /// file), so a valid pack with a large essence-container batch (over
    /// the 64 KiB cap used for untrusted footer offsets) must still parse.
    #[test]
    fn oversized_valid_header_partition_pack_still_parses() {
        let mut value = synthetic_header_partition(&OP1A_UL_BYTES, 1024);
        // Replace the one-entry essence container batch with 5000 entries.
        let batch_at = value.len() - 24;
        value.truncate(batch_at);
        let count: u32 = 5000;
        value.extend_from_slice(&count.to_be_bytes());
        value.extend_from_slice(&16u32.to_be_bytes());
        for _ in 0..count {
            value.extend_from_slice(&OP1A_UL_BYTES);
        }
        assert!(value.len() as u64 > 64 * 1024);
        let mut buf = Vec::new();
        let mut key = Vec::new();
        write_partition_key(&mut key, 0x02, 0x04); // header, closed+complete
        write_klv(&mut buf, &key, &value);
        let tmp = dump_to_temp(&buf);
        let issues = validate_mxf_essence(tmp.path());
        assert!(
            !issues
                .iter()
                .any(|i| i.code.contains("PartitionPackParseFailed")),
            "valid oversized header pack must parse: {issues:#?}",
        );
        assert!(issues
            .iter()
            .any(|i| i.code.contains("EssenceContainersDetected")));
    }

    #[test]
    fn validate_mxf_essence_handles_missing_file_gracefully() {
        let issues =
            validate_mxf_essence(std::path::Path::new("/nonexistent/imferno-mxf-test.mxf"));
        assert!(
            issues.iter().any(|i| i.code == "IMFERNO:Mxf/OpenFailed"),
            "missing file should produce OpenFailed, got: {:#?}",
            issues
        );
    }
}
