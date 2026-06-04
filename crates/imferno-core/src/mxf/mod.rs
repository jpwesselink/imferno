//! SMPTE ST 377-1: Material Exchange Format (MXF) header parser.
//!
//! Reads the header partition pack from an MXF file and extracts:
//! - Operational Pattern UL (OP1a, OP1b, etc.)
//! - Essence Container ULs (codec container labels)
//!
//! Scope: partition-pack level only. Full header metadata set parsing
//! (Preface, MaterialPackage, essence descriptors) is out of scope for
//! this phase — CPL EssenceDescriptors are the primary source of format info.

pub mod codes;
/// MXF essence-header validation backed by `smpte-mxf`. Native-only —
/// the wasm validator never sees MXF binaries (browser callers upload
/// the XML side of an IMF package), so this module isn't compiled for
/// `target_arch = "wasm32"`.
#[cfg(not(target_arch = "wasm32"))]
pub mod essence;
/// MXF header-metadata extraction via `regxml` — converts the full
/// Preface tree (MaterialPackage, descriptors, MCA sub-descriptors)
/// to RegXML for typed essence-rule application. Native-only.
#[cfg(not(target_arch = "wasm32"))]
pub mod metadata;
/// ST 2067-2 §5.3 audio MCA rules applied against the RegXML output
/// of `mxf::metadata`. WAVE PCM requirement, sample rate / quant-bits
/// whitelist, channel-label count match, SoundfieldGroupLabel
/// singleton. Native-only.
#[cfg(not(target_arch = "wasm32"))]
pub mod audio_mca;
/// ST 2067-2 §5.4 timed-text essence rules applied against RegXML.
/// UCSEncoding=UTF-8, NamespaceURI ∈ IMSC1, MIMEType whitelist.
/// Native-only.
#[cfg(not(target_arch = "wasm32"))]
pub mod timed_text;

use std::io::Read;
use std::path::Path;
use thiserror::Error;

/// A rational number representing a sample rate (numerator/denominator).
///
/// Used for `SampleRate` fields in MXF essence descriptors (ST 377-1).
/// Distinct from `st2067_3::EditRate` — same representation, different domain.
#[derive(Debug, Clone, PartialEq)]
pub struct SampleRate {
    pub numerator: i64,
    pub denominator: i64,
}

// ─── Error ────────────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum MxfParseError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Not a valid MXF file: invalid header partition pack key")]
    NotMxf,
    #[error("KLV parse error at byte offset {offset}: {message}")]
    KlvError { offset: u64, message: String },
    #[error("Header partition pack missing or too short (got {got} bytes, need ≥ {need})")]
    PartitionPackTooShort { got: usize, need: usize },
    /// The partition pack declares more bytes than the parser will read
    /// (`MAX_PP_BODY = 4096`). Real-world IMF header partition packs are
    /// well under 1 KiB; lengths above the cap suggest a corrupted file or
    /// an unexpected MXF dialect — we error rather than silently truncate.
    #[error("Header partition pack body too large (got {got} bytes, parser cap is {cap})")]
    PartitionPackTooLarge { got: usize, cap: usize },
}

type Result<T> = std::result::Result<T, MxfParseError>;

// ─── Public types ─────────────────────────────────────────────────────────────

/// Header-level information extracted from an MXF file.
///
/// Populated by parsing the Header Partition Pack KLV triplet only —
/// no header metadata sets are parsed.
#[derive(Debug, Clone)]
pub struct MxfHeaderInfo {
    /// MXF format version (major, minor) from the partition pack.
    pub version: (u16, u16),
    /// Operational Pattern UL as a `urn:smpte:ul:` string.
    ///
    /// Common values: `OP1a` = `urn:smpte:ul:060e2b34.04010102.0d010201.01010900`
    pub operational_pattern: String,
    /// Essence Container ULs from the partition pack's EssenceContainers batch.
    pub essence_containers: Vec<String>,
    /// Descriptor extracted from header metadata (currently always `None`).
    pub descriptor: Option<MxfDescriptor>,
}

/// Essence descriptor information from MXF header metadata.
///
/// Populated only if header metadata parsing is implemented. Currently always
/// `None` — CPL EssenceDescriptors are the source of truth.
#[derive(Debug, Clone)]
pub enum MxfDescriptor {
    Video(MxfVideoDescriptor),
    Audio(MxfAudioDescriptor),
    TimedText(MxfTimedTextDescriptor),
}

/// Video essence descriptor from MXF header metadata.
#[derive(Debug, Clone)]
pub struct MxfVideoDescriptor {
    pub stored_width: u32,
    pub stored_height: u32,
    pub sample_rate: SampleRate,
    /// Raw PictureCompression UL string — pass to `VideoCodec::from_ul`.
    pub picture_compression_ul: Option<String>,
    /// Raw ColorPrimaries UL string — pass to `ColorPrimaries::from_ul`.
    pub color_primaries_ul: Option<String>,
    /// Raw TransferCharacteristic UL string — pass to `TransferCharacteristic::from_ul`.
    pub transfer_characteristic_ul: Option<String>,
}

/// Audio essence descriptor from MXF header metadata.
#[derive(Debug, Clone)]
pub struct MxfAudioDescriptor {
    pub sample_rate: SampleRate,
    pub channel_count: u32,
    pub quantization_bits: u32,
}

/// Timed text (subtitle/caption) descriptor from MXF header metadata.
#[derive(Debug, Clone)]
pub struct MxfTimedTextDescriptor {
    pub namespace_uri: Option<String>,
}

// ─── Parser ───────────────────────────────────────────────────────────────────

/// Parse header-level information from an MXF file on disk.
pub fn parse_mxf_header_info(path: &Path) -> Result<MxfHeaderInfo> {
    let file = std::fs::File::open(path)?;
    let mut reader = std::io::BufReader::new(file);
    parse_mxf_header_info_from_reader(&mut reader)
}

/// Parse header-level information from an MXF byte stream.
///
/// Reads only the Header Partition Pack KLV triplet. Does not seek.
pub fn parse_mxf_header_info_from_reader<R: Read>(reader: &mut R) -> Result<MxfHeaderInfo> {
    // ── Step 1: Read KLV key (16 bytes) ──────────────────────────────────────
    let mut key = [0u8; 16];
    reader.read_exact(&mut key).map_err(|e| {
        if e.kind() == std::io::ErrorKind::UnexpectedEof {
            MxfParseError::NotMxf
        } else {
            MxfParseError::Io(e)
        }
    })?;

    // Verify it is an MXF Header Partition Pack key.
    // SMPTE ST 377-1 §7.1 — all partition pack keys share the same 12-byte prefix:
    // 06 0E 2B 34 02 05 01 01 0D 01 02 01
    // Byte 12 = 01 (header), 02 (body), 03 (footer)
    // We only accept header partition packs.
    const MXF_PP_PREFIX: [u8; 12] = [
        0x06, 0x0E, 0x2B, 0x34, 0x02, 0x05, 0x01, 0x01, 0x0D, 0x01, 0x02, 0x01,
    ];
    if key[..12] != MXF_PP_PREFIX || key[12] != 0x01 {
        return Err(MxfParseError::NotMxf);
    }

    // ── Step 2: BER-decode the length ─────────────────────────────────────────
    let length = read_ber_length(reader, 16)?;

    // Minimum valid partition pack body is 88 bytes (0 essence containers).
    const MIN_PP_BODY: u64 = 88;
    if length < MIN_PP_BODY {
        return Err(MxfParseError::PartitionPackTooShort {
            got: length as usize,
            need: MIN_PP_BODY as usize,
        });
    }

    // ── Step 3: Read partition pack body ─────────────────────────────────────
    // Cap at 4 KiB to avoid absurd allocations on corrupt input. Real IMF
    // header partition packs are well under 1 KiB, so lengths above the cap
    // are a signal of a malformed file rather than a legitimate edge case.
    const MAX_PP_BODY: u64 = 4096;
    if length > MAX_PP_BODY {
        return Err(MxfParseError::PartitionPackTooLarge {
            got: length as usize,
            cap: MAX_PP_BODY as usize,
        });
    }
    let body_len = length as usize;
    let mut body = vec![0u8; body_len];
    reader.read_exact(&mut body)?;

    // ── Step 4: Parse the fixed fields ───────────────────────────────────────
    // SMPTE ST 377-1:2011, Table 13 — Partition Pack value layout (all big-endian)
    // Offset  0  MajorVersion       UInt16
    // Offset  2  MinorVersion       UInt16
    // Offset  4  KAGSize            UInt32
    // Offset  8  ThisPartition      UInt64
    // Offset 16  PreviousPartition  UInt64
    // Offset 24  FooterPartition    UInt64
    // Offset 32  HeaderByteCount    UInt64
    // Offset 40  IndexByteCount     UInt64
    // Offset 48  IndexSID           UInt32
    // Offset 52  BodyOffset         UInt64
    // Offset 60  BodySID            UInt32
    // Offset 64  OperationalPattern UL[16]
    // Offset 80  EssenceContainers  batch(count:u32, size:u32, UL[16]...)

    let major_version = u16::from_be_bytes([body[0], body[1]]);
    let minor_version = u16::from_be_bytes([body[2], body[3]]);

    // OperationalPattern is at offset 64 in the partition pack value.
    let operational_pattern = format_ul(&body[64..80]);

    // ── Step 5: Parse EssenceContainers batch at offset 80 ───────────────────
    let mut essence_containers = Vec::new();
    if body.len() >= 88 {
        // Batch header: 4-byte count + 4-byte element size
        let count = u32::from_be_bytes([body[80], body[81], body[82], body[83]]) as usize;
        let elem_size = u32::from_be_bytes([body[84], body[85], body[86], body[87]]) as usize;

        if elem_size == 16 {
            let mut offset = 88;
            for _ in 0..count {
                if offset + 16 <= body.len() {
                    essence_containers.push(format_ul(&body[offset..offset + 16]));
                    offset += 16;
                } else {
                    break;
                }
            }
        }
    }

    Ok(MxfHeaderInfo {
        version: (major_version, minor_version),
        operational_pattern,
        essence_containers,
        descriptor: None,
    })
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Read a BER-encoded length from `reader`.
/// `key_offset` is used for error messages (byte offset of the key start).
fn read_ber_length<R: Read>(reader: &mut R, key_offset: u64) -> Result<u64> {
    let mut first = [0u8; 1];
    reader.read_exact(&mut first)?;
    let first = first[0];

    if first < 0x80 {
        return Ok(first as u64);
    }

    if first == 0x80 {
        return Err(MxfParseError::KlvError {
            offset: key_offset + 16,
            message: "Indefinite BER length not supported in partition packs".to_string(),
        });
    }

    let num_bytes = (first & 0x7F) as usize;
    if num_bytes > 8 {
        return Err(MxfParseError::KlvError {
            offset: key_offset + 16,
            message: format!("BER length too wide: {num_bytes} bytes"),
        });
    }

    let mut buf = [0u8; 8];
    reader.read_exact(&mut buf[8 - num_bytes..])?;
    Ok(u64::from_be_bytes(buf))
}

/// Format 16 raw UL bytes as `urn:smpte:ul:xxxxxxxx.xxxxxxxx.xxxxxxxx.xxxxxxxx`.
fn format_ul(bytes: &[u8]) -> String {
    if bytes.len() < 16 {
        return format!("(invalid-ul:{}-bytes)", bytes.len());
    }
    format!(
        "urn:smpte:ul:{:02x}{:02x}{:02x}{:02x}.{:02x}{:02x}{:02x}{:02x}.\
         {:02x}{:02x}{:02x}{:02x}.{:02x}{:02x}{:02x}{:02x}",
        bytes[0],
        bytes[1],
        bytes[2],
        bytes[3],
        bytes[4],
        bytes[5],
        bytes[6],
        bytes[7],
        bytes[8],
        bytes[9],
        bytes[10],
        bytes[11],
        bytes[12],
        bytes[13],
        bytes[14],
        bytes[15],
    )
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    /// Helper: build a minimal valid MXF header partition pack byte stream.
    /// Key (16) + BER length (1) + partition pack body (88).
    fn make_minimal_mxf_stream(op_ul: [u8; 16]) -> Vec<u8> {
        let mut stream = Vec::new();

        // Key: Header Partition Pack (Closed and Complete = 01 02 04 00)
        stream.extend_from_slice(&[
            0x06, 0x0E, 0x2B, 0x34, 0x02, 0x05, 0x01, 0x01, 0x0D, 0x01, 0x02, 0x01, 0x01, 0x02,
            0x04, 0x00,
        ]);
        // BER length = 88 (fits in 1 byte)
        stream.push(88);

        // Partition pack body (88 bytes):
        // MajorVersion = 1
        stream.extend_from_slice(&[0x00, 0x01]);
        // MinorVersion = 3
        stream.extend_from_slice(&[0x00, 0x03]);
        // KAGSize = 512
        stream.extend_from_slice(&[0x00, 0x00, 0x02, 0x00]);
        // ThisPartition = 0
        stream.extend_from_slice(&[0u8; 8]);
        // PreviousPartition = 0
        stream.extend_from_slice(&[0u8; 8]);
        // FooterPartition = 0
        stream.extend_from_slice(&[0u8; 8]);
        // HeaderByteCount = 0
        stream.extend_from_slice(&[0u8; 8]);
        // IndexByteCount = 0
        stream.extend_from_slice(&[0u8; 8]);
        // IndexSID = 0
        stream.extend_from_slice(&[0u8; 4]);
        // BodyOffset = 0
        stream.extend_from_slice(&[0u8; 8]);
        // BodySID = 0
        stream.extend_from_slice(&[0u8; 4]);
        // OperationalPattern UL
        stream.extend_from_slice(&op_ul);
        // EssenceContainers batch: count=0, element_size=16
        stream.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // count
        stream.extend_from_slice(&[0x00, 0x00, 0x00, 0x10]); // element_size

        assert_eq!(stream.len(), 16 + 1 + 88);
        stream
    }

    /// SMPTE ST 377-1 §7.1: a valid MXF file starts with a Header Partition Pack key.
    #[test]
    fn valid_header_partition_pack_parsed() {
        // OP1a UL: 060E2B34.04010102.0D010201.01010900
        let op1a: [u8; 16] = [
            0x06, 0x0E, 0x2B, 0x34, 0x04, 0x01, 0x01, 0x02, 0x0D, 0x01, 0x02, 0x01, 0x01, 0x01,
            0x09, 0x00,
        ];
        let stream = make_minimal_mxf_stream(op1a);
        let mut cursor = Cursor::new(stream);
        let info = parse_mxf_header_info_from_reader(&mut cursor).unwrap();

        assert_eq!(info.version, (1, 3));
        assert_eq!(
            info.operational_pattern,
            "urn:smpte:ul:060e2b34.04010102.0d010201.01010900"
        );
        assert!(info.essence_containers.is_empty());
        assert!(info.descriptor.is_none());
    }

    /// SMPTE ST 377-1 §7.1: non-MXF files must be rejected.
    #[test]
    fn non_mxf_data_rejected() {
        let data = vec![0u8; 105];
        let mut cursor = Cursor::new(data);
        assert!(matches!(
            parse_mxf_header_info_from_reader(&mut cursor),
            Err(MxfParseError::NotMxf)
        ));
    }

    /// Body-type partition pack key (key[12] = 0x02) must be rejected — we
    /// only accept header partition packs.
    #[test]
    fn body_partition_pack_rejected() {
        let mut key = vec![
            0x06, 0x0E, 0x2B, 0x34, 0x02, 0x05, 0x01, 0x01, 0x0D, 0x01, 0x02, 0x01, 0x02, 0x02,
            0x04, 0x00, // key[12] = 0x02 = body
        ];
        key.extend_from_slice(&[0u8; 89]);
        let mut cursor = Cursor::new(key);
        assert!(matches!(
            parse_mxf_header_info_from_reader(&mut cursor),
            Err(MxfParseError::NotMxf)
        ));
    }

    /// FIX-4 regression: an oversized partition pack returns
    /// `PartitionPackTooLarge` rather than silently truncating to 4096 bytes.
    /// Pre-fix behaviour was a silent `min(4096)` clamp that could swallow
    /// essence-container data.
    #[test]
    fn oversized_partition_pack_returns_too_large() {
        let mut bytes = Vec::new();
        // Valid header partition pack key.
        bytes.extend_from_slice(&[
            0x06, 0x0E, 0x2B, 0x34, 0x02, 0x05, 0x01, 0x01, 0x0D, 0x01, 0x02, 0x01, 0x01, 0x01,
            0x09, 0x00,
        ]);
        // BER long-form length = 5000 (above the 4096 cap).
        // 4-byte BER encoding: 0x84 followed by 0x00001388 (5000).
        bytes.extend_from_slice(&[0x84, 0x00, 0x00, 0x13, 0x88]);
        // Body padding so read_exact has bytes to consume if the cap check
        // didn't trip — we only ever need to hit the length check, so the
        // body content doesn't matter.
        bytes.extend(std::iter::repeat_n(0u8, 5000));

        let mut cursor = Cursor::new(bytes);
        assert!(
            matches!(
                parse_mxf_header_info_from_reader(&mut cursor),
                Err(MxfParseError::PartitionPackTooLarge { got: 5000, cap: 4096 })
            ),
            "expected PartitionPackTooLarge {{ got: 5000, cap: 4096 }}"
        );
    }

    /// SMPTE ST 377-1 §7.1: EssenceContainers batch is correctly parsed.
    #[test]
    fn essence_containers_parsed() {
        let op: [u8; 16] = [
            0x06, 0x0E, 0x2B, 0x34, 0x04, 0x01, 0x01, 0x02, 0x0D, 0x01, 0x02, 0x01, 0x01, 0x01,
            0x09, 0x00,
        ];
        // JPEG 2000 Frame-wrapped container UL
        let ec: [u8; 16] = [
            0x06, 0x0E, 0x2B, 0x34, 0x04, 0x01, 0x01, 0x0D, 0x0D, 0x01, 0x03, 0x01, 0x02, 0x0C,
            0x01, 0x00,
        ];

        let mut stream = Vec::new();
        // Key: Header Partition Pack (Closed and Complete)
        stream.extend_from_slice(&[
            0x06, 0x0E, 0x2B, 0x34, 0x02, 0x05, 0x01, 0x01, 0x0D, 0x01, 0x02, 0x01, 0x01, 0x02,
            0x04, 0x00,
        ]);
        // BER length = 88 + 16 = 104 (one essence container)
        stream.push(104);

        // Fixed fields (80 bytes): versions + padding to OP
        stream.extend_from_slice(&[0x00, 0x01]); // MajorVersion = 1
        stream.extend_from_slice(&[0x00, 0x03]); // MinorVersion = 3
        stream.extend_from_slice(&[0x00, 0x00, 0x02, 0x00]); // KAGSize
        stream.extend_from_slice(&[0u8; 8 * 5 + 4 + 8 + 4]); // padding to OP offset
        stream.extend_from_slice(&op); // OperationalPattern at offset 68
                                       // EssenceContainers batch: count=1, element_size=16, then 1 UL
        stream.extend_from_slice(&[0x00, 0x00, 0x00, 0x01]); // count
        stream.extend_from_slice(&[0x00, 0x00, 0x00, 0x10]); // element_size
        stream.extend_from_slice(&ec);

        let mut cursor = Cursor::new(stream);
        let info = parse_mxf_header_info_from_reader(&mut cursor).unwrap();

        assert_eq!(info.essence_containers.len(), 1);
        assert_eq!(
            info.essence_containers[0],
            "urn:smpte:ul:060e2b34.0401010d.0d010301.020c0100"
        );
    }

    /// Real MXF files from the test corpus parse without error.
    #[test]
    #[ignore = "requires test-data MXF files (large)"]
    fn real_meridian_mxf_parses() {
        let path = std::path::Path::new(
            "../../test-data/MERIDIAN_Netflix_Photon_161006/MERIDIAN_Netflix_Photon_161006_00.mxf",
        );
        if !path.exists() {
            return; // skip if test data not present
        }
        let info = parse_mxf_header_info(path).unwrap();
        assert!(!info.operational_pattern.is_empty());
        println!("OP: {}", info.operational_pattern);
        for ec in &info.essence_containers {
            println!("EC: {ec}");
        }
    }
}
