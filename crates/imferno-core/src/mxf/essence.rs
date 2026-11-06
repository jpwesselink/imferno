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
    seek_footer_partition, seek_header_partition, MxfError, PartitionKind, PartitionPack,
    PartitionStatus,
};

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
        Ok(pack) => {
            // ST 377-1 §6.2.4: an Open header partition is fine when a
            // Closed footer partition carries header metadata. Only read
            // the footer when the header is Open.
            let closed_footer_with_metadata = is_open(pack.status)
                && parse_footer_pack(&mut reader)
                    .map(|footer| {
                        matches!(footer.kind, PartitionKind::Footer)
                            && !is_open(footer.status)
                            && footer.header_byte_count > 0
                    })
                    .unwrap_or(false);
            issues.extend(check_partition_pack(
                path,
                &pack,
                closed_footer_with_metadata,
            ));
        }
        Err(e) => {
            issues.push(parse_failure_issue(path, &e));
        }
    }
    issues
}

fn parse_header_pack<R: Read + Seek>(reader: &mut R) -> Result<PartitionPack, MxfError> {
    let header_offset = seek_header_partition(reader)?;
    reader.seek(std::io::SeekFrom::Start(header_offset))?;
    let mut stream = KlvStream::new(reader);
    let triplet = stream
        .read_triplet()
        .map_err(MxfError::from)?
        .ok_or(MxfError::Truncated(
            "expected partition pack triplet at header offset",
        ))?;
    PartitionPack::from_triplet(&triplet)?.ok_or(MxfError::Truncated(
        "first triplet was not a partition pack",
    ))
}

fn is_open(status: PartitionStatus) -> bool {
    matches!(
        status,
        PartitionStatus::OpenIncomplete | PartitionStatus::OpenComplete
    )
}

fn parse_footer_pack<R: Read + Seek>(reader: &mut R) -> Result<PartitionPack, MxfError> {
    let footer_offset = seek_footer_partition(reader)?;
    reader.seek(std::io::SeekFrom::Start(footer_offset))?;
    let mut stream = KlvStream::new(reader);
    let triplet = stream
        .read_triplet()
        .map_err(MxfError::from)?
        .ok_or(MxfError::Truncated(
            "expected partition pack triplet at footer offset",
        ))?;
    PartitionPack::from_triplet(&triplet)?.ok_or(MxfError::Truncated(
        "footer triplet was not a partition pack",
    ))
}

/// `closed_footer_with_metadata`: whether a Closed footer partition with
/// header metadata was found (only looked up when the header is Open).
fn check_partition_pack(
    path: &Path,
    pack: &PartitionPack,
    closed_footer_with_metadata: bool,
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
    if is_open(pack.status) && !closed_footer_with_metadata {
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
        let mut buf = Vec::new();
        let mut key = Vec::new();
        write_partition_key(&mut key, 0x02, 0x01); // header, open+incomplete
        write_klv(
            &mut buf,
            &key,
            &synthetic_partition(&OP1A_UL_BYTES, 1024, header_len),
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
