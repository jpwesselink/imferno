//! SMPTE ST 2067-2 Core Constraints — AssetMap, PKL, and foundational IMF types.
//!
//! This crate covers:
//! - Foundational primitives: [`ImfUuid`], [`SmpteUl`], [`ImfTypeError`]
//! - PKL types: [`AssetHash`], [`HashAlgorithm`], [`MimeType`]
//! - Namespace detection: [`AssetMapNamespace`], [`PklNamespace`], [`CoreConstraintsNamespace`]
//! - Document parsers: [`parse_assetmap`], [`parse_pkl`], [`parse_opl`]
//! - Re-exports from [`st429_9`]: [`VolumeIndex`], [`parse_volindex`]

pub mod codes;
pub mod volindex;
pub mod volindex_codes;

// Re-export VOLINDEX types
pub use volindex::{parse_volindex, VolindexError, VolumeIndex};

use base64::Engine;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

// ─── Error ────────────────────────────────────────────────────────────────────

#[derive(Debug, Error, PartialEq)]
pub enum ImfTypeError {
    #[error("Invalid UUID '{0}': expected urn:uuid:<uuid> or bare UUID")]
    InvalidUuid(String),
    #[error("Invalid edit rate '{0}': expected 'numerator denominator'")]
    InvalidEditRate(String),
    #[error("Invalid hash: {0}")]
    InvalidHash(String),
    #[error("Invalid language tag '{0}': must be non-empty")]
    InvalidLanguageTag(String),
    #[error("Invalid SMPTE UL '{0}': expected 16 hex bytes in dotted groups")]
    InvalidUl(String),
}

// ─── SmpteUl ─────────────────────────────────────────────────────────────────

/// A SMPTE Universal Label — 16-byte identifier per ST 336M.
///
/// Byte layout:
/// ```text
/// Bytes 1-4:  Object Identifier (always 06.0E.2B.34)
/// Byte  5:    Category designator
/// Byte  6:    Registry designator
/// Byte  7:    Structure designator
/// Byte  8:    Version number  ← MUST BE IGNORED for comparison (ST 298M)
/// Bytes 9-16: Item-specific identification
/// ```
///
/// Per ST 298M, byte 8 (the registry version number) is masked when comparing
/// ULs for semantic identity. Two ULs that differ only in byte 8 are the same item.
#[derive(Debug, Clone, Copy)]
pub struct SmpteUl(pub [u8; 16]);

impl SmpteUl {
    /// Parse a UL from string form.
    ///
    /// Accepted formats:
    /// - `urn:smpte:ul:060e2b34.04010106.04010101.03030000` (4 groups of 4 bytes)
    /// - `060e2b34.04010106.04010101.03030000` (bare 4-group form)
    /// - `060e2b34.0401.0106.04010101.03030000` (5-group variant from some test data)
    pub fn parse(s: &str) -> Result<Self, ImfTypeError> {
        let hex_part = s.strip_prefix("urn:smpte:ul:").unwrap_or(s).trim();

        // Remove dots to get a contiguous hex string
        let hex_str: String = hex_part.chars().filter(|c| *c != '.').collect();

        if hex_str.len() != 32 {
            return Err(ImfTypeError::InvalidUl(s.to_string()));
        }

        let mut bytes = [0u8; 16];
        for i in 0..16 {
            bytes[i] = u8::from_str_radix(&hex_str[i * 2..i * 2 + 2], 16)
                .map_err(|_| ImfTypeError::InvalidUl(s.to_string()))?;
        }

        Ok(SmpteUl(bytes))
    }

    /// Compare two ULs ignoring byte 8 (index 7) — the registry version number.
    ///
    /// Per ST 298M, the version byte must be masked for semantic comparison.
    pub fn matches_ignoring_version(&self, other: &SmpteUl) -> bool {
        for i in 0..16 {
            if i == 7 {
                continue; // skip version byte
            }
            if self.0[i] != other.0[i] {
                return false;
            }
        }
        true
    }

    /// Return the UL with byte 8 zeroed for use as a canonical match key.
    pub fn normalized(&self) -> Self {
        let mut bytes = self.0;
        bytes[7] = 0;
        SmpteUl(bytes)
    }

    /// The discriminating bytes (bytes 9-16) that identify the specific item.
    pub fn item_bytes(&self) -> &[u8] {
        &self.0[8..]
    }
}

impl PartialEq for SmpteUl {
    /// Equality comparison ignores byte 8 (version), per ST 298M.
    fn eq(&self, other: &Self) -> bool {
        self.matches_ignoring_version(other)
    }
}

impl Eq for SmpteUl {}

impl std::hash::Hash for SmpteUl {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Hash with byte 8 zeroed so equal items hash identically
        let norm = self.normalized();
        norm.0.hash(state);
    }
}

impl std::fmt::Display for SmpteUl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "urn:smpte:ul:{:02x}{:02x}{:02x}{:02x}.{:02x}{:02x}{:02x}{:02x}.{:02x}{:02x}{:02x}{:02x}.{:02x}{:02x}{:02x}{:02x}",
            self.0[0], self.0[1], self.0[2], self.0[3],
            self.0[4], self.0[5], self.0[6], self.0[7],
            self.0[8], self.0[9], self.0[10], self.0[11],
            self.0[12], self.0[13], self.0[14], self.0[15],
        )
    }
}

// ─── ImfUuid ──────────────────────────────────────────────────────────────────

/// A SMPTE IMF UUID.
///
/// In XML documents UUIDs appear as `urn:uuid:<uuid>`. In JSON/WASM output
/// they serialise as bare UUID strings (`"0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85"`).
/// Deserialization accepts both forms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(type = "string"))]
pub struct ImfUuid(pub Uuid);

impl ImfUuid {
    /// Parse from `urn:uuid:...` or a bare UUID string.
    pub fn parse(s: &str) -> Result<Self, ImfTypeError> {
        let bare = s.strip_prefix("urn:uuid:").unwrap_or(s);
        Uuid::parse_str(bare)
            .map(ImfUuid)
            .map_err(|_| ImfTypeError::InvalidUuid(s.to_string()))
    }

    /// Return the URN form used in XML: `urn:uuid:<uuid>`.
    pub fn to_urn(&self) -> String {
        format!("urn:uuid:{}", self.0)
    }
}

impl std::fmt::Display for ImfUuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Serialize for ImfUuid {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.0.to_string())
    }
}

impl<'de> Deserialize<'de> for ImfUuid {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        ImfUuid::parse(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(feature = "jsonschema")]
impl schemars::JsonSchema for ImfUuid {
    fn schema_name() -> String {
        "ImfUuid".to_owned()
    }

    fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        let mut schema = gen.subschema_for::<String>().into_object();
        schema.metadata().description = Some(
            "A SMPTE IMF UUID, serialised as a bare UUID string (e.g. \"0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85\")".to_owned()
        );
        schema.format = Some("uuid".to_owned());
        schema.into()
    }
}

// ─── AssetHash ────────────────────────────────────────────────────────────────

/// A decoded asset hash from a Packing List, per SMPTE ST 2067-2 §9.
///
/// PKL files carry base64-encoded SHA-1 digests for each tracked asset.
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetHash {
    pub algorithm: HashAlgorithm,
    pub bytes: Vec<u8>,
}

/// The hash algorithm used for a PKL asset digest.
///
/// Per SMPTE ST 2067-2:2020 §9, SHA-1 is the default algorithm.
/// SHA-256 is supported via the `<HashAlgorithm>` element using
/// XML Digital Signature algorithm URIs.
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HashAlgorithm {
    /// SHA-1 (default per ST 2067-2 §9).
    /// URI: `http://www.w3.org/2000/09/xmldsig#sha1`
    Sha1,
    /// SHA-256.
    /// URI: `http://www.w3.org/2001/04/xmlenc#sha256`
    Sha256,
}

impl HashAlgorithm {
    /// Parse a hash algorithm from an XML Digital Signature algorithm URI.
    ///
    /// Per ST 2067-2:2020 §9, the `<HashAlgorithm>` element uses
    /// `ds:DigestMethodType` which carries an `Algorithm` attribute URI.
    pub fn from_uri(uri: &str) -> Option<Self> {
        match uri.trim() {
            "http://www.w3.org/2000/09/xmldsig#sha1" => Some(Self::Sha1),
            "http://www.w3.org/2001/04/xmlenc#sha256" => Some(Self::Sha256),
            _ => None,
        }
    }

    /// Expected digest length in bytes for this algorithm.
    pub fn digest_len(&self) -> usize {
        match self {
            Self::Sha1 => 20,
            Self::Sha256 => 32,
        }
    }
}

impl std::fmt::Display for HashAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sha1 => write!(f, "SHA-1"),
            Self::Sha256 => write!(f, "SHA-256"),
        }
    }
}

impl AssetHash {
    /// Decode a base64-encoded SHA-1 digest as found in PKL `<Hash>` elements.
    ///
    /// Per SMPTE ST 2067-2 §9, SHA-1 produces a 20-byte digest.
    pub fn from_base64_sha1(b64: &str) -> Result<Self, ImfTypeError> {
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(b64)
            .map_err(|e| ImfTypeError::InvalidHash(e.to_string()))?;
        if bytes.len() != 20 {
            return Err(ImfTypeError::InvalidHash(format!(
                "SHA-1 digest must be 20 bytes, got {}",
                bytes.len()
            )));
        }
        Ok(Self {
            algorithm: HashAlgorithm::Sha1,
            bytes,
        })
    }

    /// Decode a base64-encoded SHA-256 digest.
    ///
    /// SHA-256 produces a 32-byte digest.
    pub fn from_base64_sha256(b64: &str) -> Result<Self, ImfTypeError> {
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(b64)
            .map_err(|e| ImfTypeError::InvalidHash(e.to_string()))?;
        if bytes.len() != 32 {
            return Err(ImfTypeError::InvalidHash(format!(
                "SHA-256 digest must be 32 bytes, got {}",
                bytes.len()
            )));
        }
        Ok(Self {
            algorithm: HashAlgorithm::Sha256,
            bytes,
        })
    }

    /// Decode a base64-encoded digest for the given algorithm.
    pub fn from_base64(b64: &str, algorithm: HashAlgorithm) -> Result<Self, ImfTypeError> {
        match algorithm {
            HashAlgorithm::Sha1 => Self::from_base64_sha1(b64),
            HashAlgorithm::Sha256 => Self::from_base64_sha256(b64),
        }
    }

    /// Encode the hash bytes as base64, as used in PKL XML.
    pub fn to_base64(&self) -> String {
        base64::engine::general_purpose::STANDARD.encode(&self.bytes)
    }
}

// ─── MimeType ─────────────────────────────────────────────────────────────────

/// MIME type as used in `<Type>` elements in PKL assets (SMPTE ST 2067-2 §9).
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MimeType {
    /// `text/xml` — CPL and other XML documents
    TextXml,
    /// `application/xml` — alternative XML MIME type
    ApplicationXml,
    /// `application/mxf` — MXF essence files
    ApplicationMxf,
    /// Unrecognised; the original string is preserved.
    Other(String),
}

impl MimeType {
    pub fn parse(s: &str) -> Self {
        match s.trim() {
            "text/xml" => Self::TextXml,
            "application/xml" => Self::ApplicationXml,
            "application/mxf" => Self::ApplicationMxf,
            other => Self::Other(other.to_string()),
        }
    }

    pub fn is_xml(&self) -> bool {
        matches!(self, Self::TextXml | Self::ApplicationXml)
    }

    pub fn is_mxf(&self) -> bool {
        matches!(self, Self::ApplicationMxf)
    }
}

impl std::fmt::Display for MimeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TextXml => write!(f, "text/xml"),
            Self::ApplicationXml => write!(f, "application/xml"),
            Self::ApplicationMxf => write!(f, "application/mxf"),
            Self::Other(s) => write!(f, "{}", s),
        }
    }
}

// ─── AssetMapNamespace ────────────────────────────────────────────────────────

/// The detected SMPTE spec version of an AssetMap document, derived from its root xmlns.
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssetMapNamespace {
    /// DCI era — `http://www.smpte-ra.org/schemas/429-9/2007/AM`
    Dci429_9,
    /// SMPTE ST 2067-9:2016 — `http://www.smpte-ra.org/schemas/2067-9/2016`
    Smpte2067_9_2016,
    /// SMPTE ST 2067-9:2020 — `http://www.smpte-ra.org/ns/2067-9/2020`
    Smpte2067_9_2020,
    /// Unrecognised namespace; the original URI is preserved.
    Unknown(String),
}

impl AssetMapNamespace {
    /// Detect AssetMap spec version from a namespace URI.
    pub fn from_uri(uri: &str) -> Self {
        match uri.trim() {
            "http://www.smpte-ra.org/schemas/429-9/2007/AM" => Self::Dci429_9,
            "http://www.smpte-ra.org/schemas/2067-9/2016" => Self::Smpte2067_9_2016,
            "http://www.smpte-ra.org/ns/2067-9/2020" => Self::Smpte2067_9_2020,
            other => Self::Unknown(other.to_string()),
        }
    }

    /// Returns the normative spec document identifier.
    pub fn spec_id(&self) -> &str {
        match self {
            Self::Dci429_9 => "ST 429-9:2007",
            Self::Smpte2067_9_2016 => "ST 2067-9:2016",
            Self::Smpte2067_9_2020 => "ST 2067-9:2020",
            Self::Unknown(_) => "Unknown",
        }
    }
}

impl Default for AssetMapNamespace {
    fn default() -> Self {
        Self::Dci429_9
    }
}

impl std::fmt::Display for AssetMapNamespace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dci429_9 => write!(f, "http://www.smpte-ra.org/schemas/429-9/2007/AM"),
            Self::Smpte2067_9_2016 => write!(f, "http://www.smpte-ra.org/schemas/2067-9/2016"),
            Self::Smpte2067_9_2020 => write!(f, "http://www.smpte-ra.org/ns/2067-9/2020"),
            Self::Unknown(s) => write!(f, "{}", s),
        }
    }
}

// ─── PklNamespace ─────────────────────────────────────────────────────────────

/// The detected SMPTE spec version of a PKL document, derived from its root xmlns.
///
/// PKL schema evolved across three eras: DCI 429-8, IMF 2067-2 (2013-2016), and 2020.
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PklNamespace {
    /// DCI era — `http://www.smpte-ra.org/schemas/429-8/2007/PKL`
    Dci429_8,
    /// SMPTE ST 2067-2:2013 — `http://www.smpte-ra.org/schemas/2067-2/2013`
    Smpte2067_2_2013,
    /// SMPTE ST 2067-2:2016 — `http://www.smpte-ra.org/schemas/2067-2/2016`
    Smpte2067_2_2016,
    /// SMPTE ST 2067-2:2016 (PKL variant) — `http://www.smpte-ra.org/schemas/2067-2/2016/PKL`
    Smpte2067_2_2016Pkl,
    /// SMPTE ST 2067-2:2020 — `http://www.smpte-ra.org/ns/2067-2/2020`
    Smpte2067_2_2020,
    /// Unrecognised namespace; the original URI is preserved.
    Unknown(String),
}

impl PklNamespace {
    /// Detect PKL spec version from a namespace URI.
    pub fn from_uri(uri: &str) -> Self {
        match uri.trim() {
            "http://www.smpte-ra.org/schemas/429-8/2007/PKL" => Self::Dci429_8,
            "http://www.smpte-ra.org/schemas/2067-2/2013" => Self::Smpte2067_2_2013,
            "http://www.smpte-ra.org/schemas/2067-2/2016" => Self::Smpte2067_2_2016,
            "http://www.smpte-ra.org/schemas/2067-2/2016/PKL" => Self::Smpte2067_2_2016Pkl,
            "http://www.smpte-ra.org/ns/2067-2/2020" => Self::Smpte2067_2_2020,
            other => Self::Unknown(other.to_string()),
        }
    }

    /// Returns the normative spec document identifier.
    pub fn spec_id(&self) -> &str {
        match self {
            Self::Dci429_8 => "ST 429-8:2007",
            Self::Smpte2067_2_2013 => "ST 2067-2:2013",
            Self::Smpte2067_2_2016 | Self::Smpte2067_2_2016Pkl => "ST 2067-2:2016",
            Self::Smpte2067_2_2020 => "ST 2067-2:2020",
            Self::Unknown(_) => "Unknown",
        }
    }
}

impl Default for PklNamespace {
    fn default() -> Self {
        Self::Dci429_8
    }
}

impl std::fmt::Display for PklNamespace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dci429_8 => write!(f, "http://www.smpte-ra.org/schemas/429-8/2007/PKL"),
            Self::Smpte2067_2_2013 => write!(f, "http://www.smpte-ra.org/schemas/2067-2/2013"),
            Self::Smpte2067_2_2016 => write!(f, "http://www.smpte-ra.org/schemas/2067-2/2016"),
            Self::Smpte2067_2_2016Pkl => {
                write!(f, "http://www.smpte-ra.org/schemas/2067-2/2016/PKL")
            }
            Self::Smpte2067_2_2020 => write!(f, "http://www.smpte-ra.org/ns/2067-2/2020"),
            Self::Unknown(s) => write!(f, "{}", s),
        }
    }
}

// ─── CoreConstraintsNamespace ─────────────────────────────────────────────────

/// The detected SMPTE core constraints spec version, from inner xmlns declarations in CPLs.
///
/// CPL documents reference core constraints namespaces for elements defined in ST 2067-2.
/// This is distinct from the CPL namespace (ST 2067-3) and determines which core constraint
/// rules apply.
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CoreConstraintsNamespace {
    /// SMPTE ST 2067-2:2013 — `http://www.smpte-ra.org/schemas/2067-2/2013`
    Smpte2067_2_2013,
    /// SMPTE ST 2067-2:2016 — `http://www.smpte-ra.org/schemas/2067-2/2016`
    Smpte2067_2_2016,
    /// SMPTE ST 2067-2:2020 — `http://www.smpte-ra.org/ns/2067-2/2020`
    Smpte2067_2_2020,
    /// Unrecognised namespace; the original URI is preserved.
    Unknown(String),
}

impl CoreConstraintsNamespace {
    /// Detect core constraints spec version from a namespace URI.
    pub fn from_uri(uri: &str) -> Self {
        match uri.trim() {
            "http://www.smpte-ra.org/schemas/2067-2/2013" => Self::Smpte2067_2_2013,
            "http://www.smpte-ra.org/schemas/2067-2/2016" => Self::Smpte2067_2_2016,
            "http://www.smpte-ra.org/ns/2067-2/2020" => Self::Smpte2067_2_2020,
            other => Self::Unknown(other.to_string()),
        }
    }

    /// Returns the normative spec document identifier.
    pub fn spec_id(&self) -> &str {
        match self {
            Self::Smpte2067_2_2013 => "ST 2067-2:2013",
            Self::Smpte2067_2_2016 => "ST 2067-2:2016",
            Self::Smpte2067_2_2020 => "ST 2067-2:2020",
            Self::Unknown(_) => "Unknown",
        }
    }
}

impl Default for CoreConstraintsNamespace {
    fn default() -> Self {
        Self::Smpte2067_2_2016
    }
}

impl std::fmt::Display for CoreConstraintsNamespace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Smpte2067_2_2013 => write!(f, "http://www.smpte-ra.org/schemas/2067-2/2013"),
            Self::Smpte2067_2_2016 => write!(f, "http://www.smpte-ra.org/schemas/2067-2/2016"),
            Self::Smpte2067_2_2020 => write!(f, "http://www.smpte-ra.org/ns/2067-2/2020"),
            Self::Unknown(s) => write!(f, "{}", s),
        }
    }
}

// ─── detect_root_namespace ────────────────────────────────────────────────────

/// Extract the default namespace URI from an XML document's root element.
///
/// Searches for the first `xmlns="..."` (non-prefixed) declaration. This is used
/// by parsers to detect which spec version a document conforms to.
pub fn detect_root_namespace(xml: &str) -> Option<String> {
    // Match xmlns="..." but NOT xmlns:prefix="..."
    // We look for xmlns= preceded by whitespace (not by a colon)
    let re = regex::Regex::new(r#"(?:^|[\s<])xmlns="([^"]*)""#).unwrap();
    re.captures(xml).map(|cap| cap[1].to_string())
}

// ─── Parse error ──────────────────────────────────────────────────────────────

/// Errors that can occur when parsing an AssetMap, PKL, OPL, or VOLINDEX.
#[derive(Debug, Error)]
pub enum AssetMapParseError {
    /// The XML is structurally invalid or missing required elements.
    #[error("XML parse error: {0}")]
    Xml(#[from] quick_xml::DeError),
    /// A required field contains an invalid value (bad UUID, bad hash, etc.).
    #[error("Invalid field '{field}': {source}")]
    Field {
        field: &'static str,
        #[source]
        source: ImfTypeError,
    },
}

// ─── Private raw deserialization layer ────────────────────────────────────────

mod raw {
    use serde::Deserialize;

    fn default_volume_index() -> u32 {
        1
    }

    #[derive(Deserialize)]
    pub struct AssetMap {
        #[serde(rename = "Id")]
        pub id: String,
        #[serde(rename = "AnnotationText", default)]
        pub annotation_text: Option<String>,
        #[serde(rename = "Creator", default)]
        pub creator: Option<String>,
        #[serde(rename = "VolumeCount")]
        pub volume_count: u32,
        #[serde(rename = "IssueDate")]
        pub issue_date: String,
        #[serde(rename = "Issuer", default)]
        pub issuer: Option<String>,
        #[serde(rename = "AssetList")]
        pub asset_list: AssetList,
    }

    #[derive(Deserialize)]
    pub struct AssetList {
        #[serde(rename = "Asset")]
        pub assets: Vec<Asset>,
    }

    #[derive(Deserialize)]
    pub struct Asset {
        #[serde(rename = "Id")]
        pub id: String,
        #[serde(rename = "PackingList", default)]
        pub packing_list: Option<bool>,
        #[serde(rename = "ChunkList")]
        pub chunk_list: ChunkList,
    }

    #[derive(Deserialize)]
    pub struct ChunkList {
        #[serde(rename = "Chunk")]
        pub chunks: Vec<Chunk>,
    }

    #[derive(Deserialize)]
    pub struct Chunk {
        #[serde(rename = "Path")]
        pub path: String,
        #[serde(rename = "VolumeIndex", default = "default_volume_index")]
        pub volume_index: u32,
    }

    // ── OPL (ST 2067-100) ──────────────────────────────────────────────────

    #[derive(Deserialize)]
    pub struct OutputProfileList {
        #[serde(rename = "Id")]
        pub id: String,
        #[serde(rename = "Annotation", default)]
        pub annotation: Option<String>,
        #[serde(rename = "IssueDate")]
        pub issue_date: String,
        #[serde(rename = "Issuer", default)]
        pub issuer: Option<String>,
        #[serde(rename = "Creator", default)]
        pub creator: Option<String>,
        #[serde(rename = "CompositionPlaylistId")]
        pub composition_playlist_id: String,
    }

    // ── PKL ─────────────────────────────────────────────────────────────────

    #[derive(Deserialize)]
    pub struct PackingList {
        #[serde(rename = "Id")]
        pub id: String,
        #[serde(rename = "AnnotationText", default)]
        pub annotation_text: Option<String>,
        #[serde(rename = "IssueDate")]
        pub issue_date: String,
        #[serde(rename = "Issuer", default)]
        pub issuer: Option<String>,
        #[serde(rename = "Creator", default)]
        pub creator: Option<String>,
        /// SMPTE ST 2067-2 §9: Optional group identifier for partial deliveries.
        #[serde(rename = "GroupId", default)]
        pub group_id: Option<String>,
        #[serde(rename = "AssetList")]
        pub asset_list: PklAssetList,
    }

    #[derive(Deserialize)]
    pub struct PklAssetList {
        #[serde(rename = "Asset")]
        pub assets: Vec<PklAsset>,
    }

    /// `ds:DigestMethodType` — carries an `Algorithm` attribute URI.
    /// Used in `<HashAlgorithm Algorithm="..."/>` per SMPTE ST 2067-2 §9.
    #[derive(Deserialize)]
    pub struct DigestMethod {
        #[serde(rename = "@Algorithm")]
        pub algorithm: String,
    }

    #[derive(Deserialize)]
    pub struct PklAsset {
        #[serde(rename = "Id")]
        pub id: String,
        #[serde(rename = "AnnotationText", default)]
        pub annotation_text: Option<String>,
        #[serde(rename = "Hash")]
        pub hash: String,
        #[serde(rename = "Size")]
        pub size: u64,
        #[serde(rename = "Type")]
        pub mime_type: String,
        #[serde(rename = "OriginalFileName", default)]
        pub original_file_name: Option<String>,
        /// SMPTE ST 2067-2 §9: Optional hash algorithm override.
        /// When absent, SHA-1 is assumed (default per spec).
        #[serde(rename = "HashAlgorithm", default)]
        pub hash_algorithm: Option<DigestMethod>,
    }
}

// ─── Public domain types ───────────────────────────────────────────────────────

// VolumeIndex lives in st429-9 and is re-exported at the top of this file.

/// ASSETMAP.xml — maps UUIDs to physical file paths (ST 429-9 §6).
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct AssetMap {
    /// The SMPTE spec version detected from the root xmlns.
    #[serde(skip)]
    pub namespace: AssetMapNamespace,
    /// Unique identifier for this AssetMap (ST 429-9 §6.2).
    pub id: ImfUuid,
    pub annotation_text: Option<String>,
    pub creator: Option<String>,
    pub volume_count: u32,
    /// ISO 8601 issue date (e.g. `"2016-10-06T08:35:02-00:00"`).
    pub issue_date: String,
    pub issuer: Option<String>,
    pub asset_list: AssetList,
}

impl AssetMap {
    fn from_raw(
        raw: raw::AssetMap,
        namespace: AssetMapNamespace,
    ) -> Result<Self, AssetMapParseError> {
        Ok(Self {
            namespace,
            id: ImfUuid::parse(&raw.id).map_err(|source| AssetMapParseError::Field {
                field: "Id",
                source,
            })?,
            annotation_text: raw.annotation_text,
            creator: raw.creator,
            volume_count: raw.volume_count,
            issue_date: raw.issue_date,
            issuer: raw.issuer,
            asset_list: AssetList::from_raw(raw.asset_list)?,
        })
    }
}

/// The `<AssetList>` element in ASSETMAP.xml.
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct AssetList {
    pub assets: Vec<Asset>,
}

impl AssetList {
    fn from_raw(raw: raw::AssetList) -> Result<Self, AssetMapParseError> {
        let assets = raw
            .assets
            .into_iter()
            .map(Asset::from_raw)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { assets })
    }
}

/// A single asset entry in ASSETMAP.xml.
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct Asset {
    /// UUID identifying this asset.
    pub id: ImfUuid,
    /// Present and `true` when this entry refers to the Packing List file.
    pub packing_list: Option<bool>,
    pub chunk_list: ChunkList,
}

impl Asset {
    fn from_raw(raw: raw::Asset) -> Result<Self, AssetMapParseError> {
        Ok(Self {
            id: ImfUuid::parse(&raw.id).map_err(|source| AssetMapParseError::Field {
                field: "Id",
                source,
            })?,
            packing_list: raw.packing_list,
            chunk_list: ChunkList::from_raw(raw.chunk_list),
        })
    }
}

/// A list of file chunks for a single asset.
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct ChunkList {
    pub chunks: Vec<Chunk>,
}

impl ChunkList {
    fn from_raw(raw: raw::ChunkList) -> Self {
        Self {
            chunks: raw
                .chunks
                .into_iter()
                .map(|c| Chunk {
                    path: c.path,
                    volume_index: c.volume_index,
                })
                .collect(),
        }
    }
}

/// A single file path entry in a ChunkList.
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct Chunk {
    /// File path relative to the IMP root directory.
    pub path: String,
    pub volume_index: u32,
}

/// OPL XML — Output Profile List (SMPTE ST 2067-100).
///
/// Defines output processing instructions for a composition: image scaling,
/// cropping, pixel encoding, and audio routing/mixing macros. The macro list
/// is not deserialized (it uses `xsi:type` polymorphism with vendor-specific
/// extension types).
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct OutputProfileList {
    pub id: ImfUuid,
    pub annotation: Option<String>,
    /// ISO 8601 issue date.
    pub issue_date: String,
    pub issuer: Option<String>,
    pub creator: Option<String>,
    /// The CPL that this OPL targets.
    pub composition_playlist_id: ImfUuid,
}

impl OutputProfileList {
    fn from_raw(raw: raw::OutputProfileList) -> Result<Self, AssetMapParseError> {
        Ok(Self {
            id: ImfUuid::parse(&raw.id).map_err(|source| AssetMapParseError::Field {
                field: "Id",
                source,
            })?,
            annotation: raw.annotation,
            issue_date: raw.issue_date,
            issuer: raw.issuer,
            creator: raw.creator,
            composition_playlist_id: ImfUuid::parse(&raw.composition_playlist_id).map_err(
                |source| AssetMapParseError::Field {
                    field: "CompositionPlaylistId",
                    source,
                },
            )?,
        })
    }
}

/// PKL XML — Packing List (SMPTE ST 2067-2 §9).
///
/// Assets carry SHA-1 (default) or SHA-256 checksums. The algorithm is
/// determined by the optional `<HashAlgorithm>` element on each asset.
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct PackingList {
    /// The SMPTE spec version detected from the root xmlns.
    #[serde(skip)]
    pub namespace: PklNamespace,
    pub id: ImfUuid,
    pub annotation_text: Option<String>,
    /// ISO 8601 issue date.
    pub issue_date: String,
    pub issuer: Option<String>,
    pub creator: Option<String>,
    /// Optional group identifier for partial deliveries (SMPTE ST 2067-2 §9).
    pub group_id: Option<ImfUuid>,
    pub asset_list: PklAssetList,
}

impl PackingList {
    fn from_raw(
        raw: raw::PackingList,
        namespace: PklNamespace,
    ) -> Result<Self, AssetMapParseError> {
        let group_id = raw
            .group_id
            .map(|s| ImfUuid::parse(&s))
            .transpose()
            .map_err(|source| AssetMapParseError::Field {
                field: "GroupId",
                source,
            })?;

        Ok(Self {
            namespace,
            id: ImfUuid::parse(&raw.id).map_err(|source| AssetMapParseError::Field {
                field: "Id",
                source,
            })?,
            annotation_text: raw.annotation_text,
            issue_date: raw.issue_date,
            issuer: raw.issuer,
            creator: raw.creator,
            group_id,
            asset_list: PklAssetList::from_raw(raw.asset_list)?,
        })
    }
}

/// The `<AssetList>` element in a PKL.
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct PklAssetList {
    pub assets: Vec<PklAsset>,
}

impl PklAssetList {
    fn from_raw(raw: raw::PklAssetList) -> Result<Self, AssetMapParseError> {
        let assets = raw
            .assets
            .into_iter()
            .map(PklAsset::from_raw)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { assets })
    }
}

/// A single asset entry in a PKL.
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct PklAsset {
    pub id: ImfUuid,
    pub annotation_text: Option<String>,
    /// SHA-1 digest decoded from the base64 `<Hash>` element (SMPTE ST 2067-2 §9.3).
    pub hash: AssetHash,
    /// File size in bytes.
    pub size: u64,
    /// MIME type of the asset file (SMPTE ST 2067-2 §9.4).
    pub mime_type: MimeType,
    pub original_file_name: Option<String>,
}

impl PklAsset {
    fn from_raw(raw: raw::PklAsset) -> Result<Self, AssetMapParseError> {
        // Determine hash algorithm from <HashAlgorithm Algorithm="..."/> element.
        // Per ST 2067-2 §9, SHA-1 is the default when the element is absent.
        let algorithm = match &raw.hash_algorithm {
            Some(dm) => {
                HashAlgorithm::from_uri(&dm.algorithm).ok_or_else(|| AssetMapParseError::Field {
                    field: "HashAlgorithm",
                    source: ImfTypeError::InvalidHash(format!(
                        "unsupported hash algorithm URI: {}",
                        dm.algorithm
                    )),
                })?
            }
            None => HashAlgorithm::Sha1,
        };

        Ok(Self {
            id: ImfUuid::parse(&raw.id).map_err(|source| AssetMapParseError::Field {
                field: "Id",
                source,
            })?,
            annotation_text: raw.annotation_text,
            hash: AssetHash::from_base64(&raw.hash, algorithm).map_err(|source| {
                AssetMapParseError::Field {
                    field: "Hash",
                    source,
                }
            })?,
            size: raw.size,
            mime_type: MimeType::parse(&raw.mime_type),
            original_file_name: raw.original_file_name,
        })
    }
}

// ─── Parse functions ──────────────────────────────────────────────────────────

// parse_volindex is re-exported from st429-9 at the top of this file.

/// Parse ASSETMAP.xml (ST 429-9 §6).
///
/// Detects the SMPTE spec version from the root `xmlns` attribute and stores it
/// on the returned `AssetMap.namespace` field.
pub fn parse_assetmap(xml_content: &str) -> Result<AssetMap, AssetMapParseError> {
    let namespace = detect_root_namespace(xml_content)
        .map(|uri| AssetMapNamespace::from_uri(&uri))
        .unwrap_or_default();
    let raw: raw::AssetMap = quick_xml::de::from_str(xml_content)?;
    AssetMap::from_raw(raw, namespace)
}

/// Parse PKL XML (SMPTE ST 2067-2 §9).
///
/// Detects the SMPTE spec version from the root `xmlns` attribute and stores it
/// on the returned `PackingList.namespace` field.
pub fn parse_pkl(xml_content: &str) -> Result<PackingList, AssetMapParseError> {
    let namespace = detect_root_namespace(xml_content)
        .map(|uri| PklNamespace::from_uri(&uri))
        .unwrap_or_default();
    let raw: raw::PackingList = quick_xml::de::from_str(xml_content)?;
    PackingList::from_raw(raw, namespace)
}

/// Parse OPL XML (SMPTE ST 2067-100).
///
/// Extracts the core metadata (Id, Annotation, IssueDate, Issuer, Creator,
/// CompositionPlaylistId). The MacroList is not deserialized because it uses
/// `xsi:type` polymorphism with vendor-specific extension namespaces.
pub fn parse_opl(xml_content: &str) -> Result<OutputProfileList, AssetMapParseError> {
    let raw: raw::OutputProfileList = quick_xml::de::from_str(xml_content)?;
    OutputProfileList::from_raw(raw)
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::path::PathBuf;

    fn test_data(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../test-data")
            .join(name)
    }

    // ── ImfUuid ──────────────────────────────────────────────────────────────

    /// SMPTE ST 2067-2 §7: UUIDs are serialized as urn:uuid: URNs in XML.
    #[test]
    fn uuid_parse_urn_form() {
        let id = ImfUuid::parse("urn:uuid:0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85").unwrap();
        assert_eq!(id.to_string(), "0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85");
        assert_eq!(id.to_urn(), "urn:uuid:0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85");
    }

    #[test]
    fn uuid_parse_bare_form() {
        let id = ImfUuid::parse("0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85").unwrap();
        assert_eq!(id.to_string(), "0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85");
    }

    #[test]
    fn uuid_parse_invalid() {
        assert!(ImfUuid::parse("not-a-uuid").is_err());
        assert!(ImfUuid::parse("").is_err());
        assert!(ImfUuid::parse("urn:uuid:not-valid").is_err());
    }

    #[test]
    fn uuid_roundtrip_serde() {
        let id = ImfUuid::parse("urn:uuid:0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85").unwrap();
        let json = serde_json::to_string(&id).unwrap();
        // Serializes as bare UUID (no urn: prefix) for JSON
        assert_eq!(json, r#""0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85""#);
        let back: ImfUuid = serde_json::from_str(&json).unwrap();
        assert_eq!(id, back);
    }

    #[test]
    fn uuid_deserialize_urn_from_json() {
        // Deserializer must accept urn: form too
        let back: ImfUuid =
            serde_json::from_str(r#""urn:uuid:0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85""#).unwrap();
        assert_eq!(back.to_string(), "0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85");
    }

    // ── SmpteUl ──────────────────────────────────────────────────────────────

    /// ST 298M: Byte 8 (registry version) must be ignored for semantic identity.
    #[test]
    fn smpte_ul_parse_4group() {
        let ul = SmpteUl::parse("060e2b34.04010106.04010101.03030000").unwrap();
        assert_eq!(ul.0[0], 0x06);
        assert_eq!(ul.0[7], 0x06); // byte 8 = version
        assert_eq!(ul.0[12], 0x03);
    }

    #[test]
    fn smpte_ul_parse_urn_form() {
        let ul = SmpteUl::parse("urn:smpte:ul:060e2b34.04010106.04010101.03030000").unwrap();
        assert_eq!(ul.0[0], 0x06);
    }

    #[test]
    fn smpte_ul_parse_5group_variant() {
        // Variant format from some test data
        let ul = SmpteUl::parse("urn:smpte:ul:060e2b34.0401.0101.04010101.01020000").unwrap();
        assert_eq!(ul.0[4], 0x04);
        assert_eq!(ul.0[5], 0x01);
    }

    #[test]
    fn smpte_ul_version_agnostic_equality() {
        // Same UL at different registry versions
        let v1 = SmpteUl::parse("060e2b34.04010101.04010101.03030000").unwrap();
        let v6 = SmpteUl::parse("060e2b34.04010106.04010101.03030000").unwrap();
        let vd = SmpteUl::parse("060e2b34.0401010d.04010101.03030000").unwrap();
        assert_eq!(v1, v6, "version 01 == version 06");
        assert_eq!(v6, vd, "version 06 == version 0d");
    }

    #[test]
    fn smpte_ul_different_items_not_equal() {
        let a = SmpteUl::parse("060e2b34.04010106.04010101.03030000").unwrap();
        let b = SmpteUl::parse("060e2b34.04010106.04010101.03040000").unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn smpte_ul_display_roundtrip() {
        let ul = SmpteUl::parse("urn:smpte:ul:060e2b34.04010106.04010101.03030000").unwrap();
        let s = ul.to_string();
        assert!(s.starts_with("urn:smpte:ul:"));
        let ul2 = SmpteUl::parse(&s).unwrap();
        assert_eq!(ul, ul2);
    }

    #[test]
    fn smpte_ul_parse_invalid() {
        assert!(SmpteUl::parse("not-a-ul").is_err());
        assert!(SmpteUl::parse("060e2b34.04010106").is_err()); // too short
    }

    // ── AssetHash ────────────────────────────────────────────────────────────

    /// SMPTE ST 2067-2 §9: PKL assets carry base64-encoded SHA-1 digests.
    #[test]
    fn asset_hash_sha1_roundtrip() {
        // SHA-1 of empty bytes
        let b64 = "2jmj7l5rSw0yVb/vlWAYkK/YBwk=";
        let h = AssetHash::from_base64_sha1(b64).unwrap();
        assert_eq!(h.algorithm, HashAlgorithm::Sha1);
        assert_eq!(h.bytes.len(), 20);
        assert_eq!(h.to_base64(), b64);
    }

    /// SMPTE ST 2067-2 §9: SHA-1 digest must be exactly 20 bytes.
    #[test]
    fn asset_hash_sha1_wrong_length_rejected() {
        let err = AssetHash::from_base64_sha1("AAAA").unwrap_err();
        assert!(err.to_string().contains("20 bytes"));
    }

    #[test]
    fn asset_hash_sha256_roundtrip() {
        let b64 = "47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU=";
        let h = AssetHash::from_base64_sha256(b64).unwrap();
        assert_eq!(h.algorithm, HashAlgorithm::Sha256);
        assert_eq!(h.bytes.len(), 32);
        assert_eq!(h.to_base64(), b64);
    }

    #[test]
    fn asset_hash_sha256_wrong_length_rejected() {
        let err = AssetHash::from_base64_sha256("2jmj7l5rSw0yVb/vlWAYkK/YBwk=").unwrap_err();
        assert!(err.to_string().contains("32 bytes"));
    }

    #[test]
    fn asset_hash_from_base64_routes_correctly() {
        let sha1_b64 = "2jmj7l5rSw0yVb/vlWAYkK/YBwk=";
        let sha256_b64 = "47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU=";

        let h1 = AssetHash::from_base64(sha1_b64, HashAlgorithm::Sha1).unwrap();
        assert_eq!(h1.algorithm, HashAlgorithm::Sha1);

        let h2 = AssetHash::from_base64(sha256_b64, HashAlgorithm::Sha256).unwrap();
        assert_eq!(h2.algorithm, HashAlgorithm::Sha256);
    }

    #[test]
    fn asset_hash_invalid_base64() {
        assert!(AssetHash::from_base64_sha1("not-valid-base64!!!").is_err());
    }

    // ── HashAlgorithm ─────────────────────────────────────────────────────────

    /// SMPTE ST 2067-2 §9: HashAlgorithm URI parsing.
    #[test]
    fn hash_algorithm_from_uri() {
        assert_eq!(
            HashAlgorithm::from_uri("http://www.w3.org/2000/09/xmldsig#sha1"),
            Some(HashAlgorithm::Sha1)
        );
        assert_eq!(
            HashAlgorithm::from_uri("http://www.w3.org/2001/04/xmlenc#sha256"),
            Some(HashAlgorithm::Sha256)
        );
        assert_eq!(HashAlgorithm::from_uri("http://example.com/unknown"), None);
    }

    #[test]
    fn hash_algorithm_digest_len() {
        assert_eq!(HashAlgorithm::Sha1.digest_len(), 20);
        assert_eq!(HashAlgorithm::Sha256.digest_len(), 32);
    }

    #[test]
    fn hash_algorithm_display() {
        assert_eq!(HashAlgorithm::Sha1.to_string(), "SHA-1");
        assert_eq!(HashAlgorithm::Sha256.to_string(), "SHA-256");
    }

    // ── VOLINDEX ──────────────────────────────────────────────────────────────

    /// ST 429-9 §5: VOLINDEX.xml contains a single <Index> element.
    #[test]
    fn volindex_parses_index_element() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="no" ?>
<VolumeIndex xmlns="http://www.smpte-ra.org/schemas/429-9/2007/AM">
    <Index>1</Index>
</VolumeIndex>"#;
        let result = parse_volindex(xml).unwrap();
        assert_eq!(result.index, 1);
    }

    // ── ASSETMAP ──────────────────────────────────────────────────────────────

    /// ST 429-9 §6.2: AssetMap Id must be a valid UUID URN.
    #[test]
    fn assetmap_id_is_imf_uuid() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="no" ?>
<AssetMap xmlns="http://www.smpte-ra.org/schemas/429-9/2007/AM">
    <Id>urn:uuid:75864667-c65e-4aae-a5b2-fa5ea5fe31b7</Id>
    <AnnotationText>MERIDIAN</AnnotationText>
    <Creator>Clipster 6.1.0.0 Beta (build 111500)</Creator>
    <VolumeCount>1</VolumeCount>
    <IssueDate>2016-10-06T08:35:02-00:00</IssueDate>
    <Issuer>R&amp;S</Issuer>
    <AssetList>
        <Asset>
            <Id>urn:uuid:0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85</Id>
            <ChunkList>
                <Chunk>
                    <Path>CPL_0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85.xml</Path>
                    <VolumeIndex>1</VolumeIndex>
                </Chunk>
            </ChunkList>
        </Asset>
        <Asset>
            <Id>urn:uuid:f5e93462-aed2-44ad-a4ba-2adb65823e7c</Id>
            <PackingList>true</PackingList>
            <ChunkList>
                <Chunk>
                    <Path>PKL_f5e93462-aed2-44ad-a4ba-2adb65823e7c.xml</Path>
                    <VolumeIndex>1</VolumeIndex>
                </Chunk>
            </ChunkList>
        </Asset>
    </AssetList>
</AssetMap>"#;

        let result = parse_assetmap(xml).unwrap();
        assert_eq!(
            result.id,
            ImfUuid::parse("urn:uuid:75864667-c65e-4aae-a5b2-fa5ea5fe31b7").unwrap()
        );
        assert_eq!(result.annotation_text, Some("MERIDIAN".to_string()));
        assert_eq!(result.volume_count, 1);
        assert_eq!(result.asset_list.assets.len(), 2);

        // ST 429-9 §6.3: Asset entries carry UUID references to package files.
        let cpl_asset = &result.asset_list.assets[0];
        assert_eq!(
            cpl_asset.id,
            ImfUuid::parse("urn:uuid:0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85").unwrap()
        );
        assert_eq!(cpl_asset.packing_list, None);
        assert_eq!(
            cpl_asset.chunk_list.chunks[0].path,
            "CPL_0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85.xml"
        );

        // ST 429-9 §6.3: PackingList flag marks the PKL asset.
        let pkl_asset = &result.asset_list.assets[1];
        assert_eq!(pkl_asset.packing_list, Some(true));
        assert_eq!(
            pkl_asset.chunk_list.chunks[0].path,
            "PKL_f5e93462-aed2-44ad-a4ba-2adb65823e7c.xml"
        );
    }

    /// ST 429-9 §6.2: Invalid UUID in AssetMap <Id> yields a typed error.
    #[test]
    fn assetmap_invalid_uuid_returns_field_error() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<AssetMap xmlns="http://www.smpte-ra.org/schemas/429-9/2007/AM">
    <Id>not-a-valid-uuid</Id>
    <VolumeCount>1</VolumeCount>
    <IssueDate>2024-01-01T00:00:00Z</IssueDate>
    <AssetList><Asset>
        <Id>urn:uuid:00000000-0000-0000-0000-000000000001</Id>
        <ChunkList><Chunk><Path>foo.xml</Path></Chunk></ChunkList>
    </Asset></AssetList>
</AssetMap>"#;
        let err = parse_assetmap(xml).unwrap_err();
        assert!(
            matches!(err, AssetMapParseError::Field { field: "Id", .. }),
            "expected Field error for Id, got: {err}"
        );
    }

    // ── PKL ───────────────────────────────────────────────────────────────────

    /// SMPTE ST 2067-2 §9: PKL carries SHA-1 hashes, sizes, and MIME types.
    #[test]
    fn pkl_parses_assets_with_strong_types() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="no" ?>
<PackingList xmlns="http://www.smpte-ra.org/schemas/429-8/2007/PKL">
    <Id>urn:uuid:f5e93462-aed2-44ad-a4ba-2adb65823e7c</Id>
    <AnnotationText>MERIDIAN</AnnotationText>
    <IssueDate>2016-10-06T08:35:02-00:00</IssueDate>
    <Issuer>R&amp;S</Issuer>
    <Creator>Clipster 6.1.0.0 Beta (build 111500)</Creator>
    <AssetList>
        <Asset>
            <Id>urn:uuid:0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85</Id>
            <AnnotationText>Meridian UHD 5994P</AnnotationText>
            <Hash>IW0J5IZBsAxLMCCmWtHvfHhjVUw=</Hash>
            <Size>15214</Size>
            <Type>text/xml</Type>
            <OriginalFileName>CPL_0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85.xml</OriginalFileName>
        </Asset>
        <Asset>
            <Id>urn:uuid:61d91654-2650-4abf-abbc-ad2c7f640bf8</Id>
            <Hash>fL7SnTeNskm71I4otXqr/T0D5LQ=</Hash>
            <Size>79486353</Size>
            <Type>application/mxf</Type>
            <OriginalFileName>MERIDIAN_Netflix_Photon_161006_00.mxf</OriginalFileName>
        </Asset>
    </AssetList>
</PackingList>"#;

        let result = parse_pkl(xml).unwrap();
        assert_eq!(
            result.id,
            ImfUuid::parse("urn:uuid:f5e93462-aed2-44ad-a4ba-2adb65823e7c").unwrap()
        );
        assert_eq!(result.annotation_text, Some("MERIDIAN".to_string()));
        assert_eq!(result.issuer, Some("R&S".to_string()));
        assert_eq!(result.asset_list.assets.len(), 2);

        let cpl_asset = &result.asset_list.assets[0];
        assert_eq!(cpl_asset.hash.algorithm, HashAlgorithm::Sha1);
        assert_eq!(cpl_asset.hash.bytes.len(), 20);
        assert_eq!(cpl_asset.hash.to_base64(), "IW0J5IZBsAxLMCCmWtHvfHhjVUw=");
        assert_eq!(cpl_asset.size, 15214);
        assert_eq!(cpl_asset.mime_type, MimeType::TextXml);
        assert!(cpl_asset.mime_type.is_xml());

        let mxf_asset = &result.asset_list.assets[1];
        assert_eq!(mxf_asset.mime_type, MimeType::ApplicationMxf);
        assert!(mxf_asset.mime_type.is_mxf());
    }

    /// SMPTE ST 2067-2 §9: PKL with explicit SHA-1 <HashAlgorithm> element.
    #[test]
    fn pkl_explicit_sha1_hash_algorithm() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<PackingList xmlns="http://www.smpte-ra.org/schemas/429-8/2007/PKL">
    <Id>urn:uuid:f5e93462-aed2-44ad-a4ba-2adb65823e7c</Id>
    <IssueDate>2024-01-01T00:00:00Z</IssueDate>
    <AssetList><Asset>
        <Id>urn:uuid:00000000-0000-0000-0000-000000000001</Id>
        <Hash>2jmj7l5rSw0yVb/vlWAYkK/YBwk=</Hash>
        <Size>1024</Size>
        <Type>application/mxf</Type>
        <HashAlgorithm Algorithm="http://www.w3.org/2000/09/xmldsig#sha1"/>
    </Asset></AssetList>
</PackingList>"#;
        let result = parse_pkl(xml).unwrap();
        assert_eq!(
            result.asset_list.assets[0].hash.algorithm,
            HashAlgorithm::Sha1
        );
    }

    /// SMPTE ST 2067-2 §9: PKL with SHA-256 <HashAlgorithm>.
    #[test]
    fn pkl_sha256_hash_algorithm() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<PackingList xmlns="http://www.smpte-ra.org/schemas/429-8/2007/PKL">
    <Id>urn:uuid:f5e93462-aed2-44ad-a4ba-2adb65823e7c</Id>
    <IssueDate>2024-01-01T00:00:00Z</IssueDate>
    <AssetList><Asset>
        <Id>urn:uuid:00000000-0000-0000-0000-000000000001</Id>
        <Hash>47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU=</Hash>
        <Size>1024</Size>
        <Type>application/mxf</Type>
        <HashAlgorithm Algorithm="http://www.w3.org/2001/04/xmlenc#sha256"/>
    </Asset></AssetList>
</PackingList>"#;
        let result = parse_pkl(xml).unwrap();
        assert_eq!(
            result.asset_list.assets[0].hash.algorithm,
            HashAlgorithm::Sha256
        );
        assert_eq!(result.asset_list.assets[0].hash.bytes.len(), 32);
    }

    /// SMPTE ST 2067-2 §9: PKL without <HashAlgorithm> defaults to SHA-1.
    #[test]
    fn pkl_missing_hash_algorithm_defaults_to_sha1() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<PackingList xmlns="http://www.smpte-ra.org/schemas/429-8/2007/PKL">
    <Id>urn:uuid:f5e93462-aed2-44ad-a4ba-2adb65823e7c</Id>
    <IssueDate>2024-01-01T00:00:00Z</IssueDate>
    <AssetList><Asset>
        <Id>urn:uuid:00000000-0000-0000-0000-000000000001</Id>
        <Hash>2jmj7l5rSw0yVb/vlWAYkK/YBwk=</Hash>
        <Size>1024</Size>
        <Type>application/mxf</Type>
    </Asset></AssetList>
</PackingList>"#;
        let result = parse_pkl(xml).unwrap();
        assert_eq!(
            result.asset_list.assets[0].hash.algorithm,
            HashAlgorithm::Sha1
        );
    }

    /// SMPTE ST 2067-2 §9: PKL with <GroupId> for partial deliveries.
    #[test]
    fn pkl_with_group_id() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<PackingList xmlns="http://www.smpte-ra.org/schemas/429-8/2007/PKL">
    <Id>urn:uuid:f5e93462-aed2-44ad-a4ba-2adb65823e7c</Id>
    <IssueDate>2024-01-01T00:00:00Z</IssueDate>
    <GroupId>urn:uuid:aabbccdd-1122-3344-5566-778899aabbcc</GroupId>
    <AssetList><Asset>
        <Id>urn:uuid:00000000-0000-0000-0000-000000000001</Id>
        <Hash>2jmj7l5rSw0yVb/vlWAYkK/YBwk=</Hash>
        <Size>1024</Size>
        <Type>application/mxf</Type>
    </Asset></AssetList>
</PackingList>"#;
        let result = parse_pkl(xml).unwrap();
        assert_eq!(
            result.group_id,
            Some(ImfUuid::parse("urn:uuid:aabbccdd-1122-3344-5566-778899aabbcc").unwrap())
        );
    }

    /// SMPTE ST 2067-2 §9: Unrecognised MIME type is preserved as MimeType::Other.
    #[test]
    fn pkl_unknown_mime_type_preserved() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<PackingList xmlns="http://www.smpte-ra.org/schemas/429-8/2007/PKL">
    <Id>urn:uuid:f5e93462-aed2-44ad-a4ba-2adb65823e7c</Id>
    <IssueDate>2024-01-01T00:00:00Z</IssueDate>
    <AssetList><Asset>
        <Id>urn:uuid:00000000-0000-0000-0000-000000000001</Id>
        <Hash>2jmj7l5rSw0yVb/vlWAYkK/YBwk=</Hash>
        <Size>512</Size>
        <Type>application/octet-stream</Type>
    </Asset></AssetList>
</PackingList>"#;
        let result = parse_pkl(xml).unwrap();
        assert_eq!(
            result.asset_list.assets[0].mime_type,
            MimeType::Other("application/octet-stream".to_string())
        );
    }

    // ── Namespace compatibility ──────────────────────────────────────────────

    /// ST 2067-2: PKL namespace versions must all parse identically.
    #[test]
    fn pkl_parses_with_2067_2_2016_namespace() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<PackingList xmlns="http://www.smpte-ra.org/schemas/2067-2/2016">
    <Id>urn:uuid:f5e93462-aed2-44ad-a4ba-2adb65823e7c</Id>
    <IssueDate>2024-01-01T00:00:00Z</IssueDate>
    <AssetList><Asset>
        <Id>urn:uuid:00000000-0000-0000-0000-000000000001</Id>
        <Hash>2jmj7l5rSw0yVb/vlWAYkK/YBwk=</Hash>
        <Size>1024</Size>
        <Type>application/mxf</Type>
    </Asset></AssetList>
</PackingList>"#;
        let result = parse_pkl(xml).unwrap();
        assert_eq!(result.asset_list.assets.len(), 1);
        assert_eq!(result.namespace, PklNamespace::Smpte2067_2_2016);
        assert_eq!(result.namespace.spec_id(), "ST 2067-2:2016");
    }

    #[test]
    fn pkl_parses_with_2067_2_2020_namespace() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<PackingList xmlns="http://www.smpte-ra.org/ns/2067-2/2020">
    <Id>urn:uuid:f5e93462-aed2-44ad-a4ba-2adb65823e7c</Id>
    <IssueDate>2024-01-01T00:00:00Z</IssueDate>
    <AssetList><Asset>
        <Id>urn:uuid:00000000-0000-0000-0000-000000000001</Id>
        <Hash>2jmj7l5rSw0yVb/vlWAYkK/YBwk=</Hash>
        <Size>1024</Size>
        <Type>application/mxf</Type>
    </Asset></AssetList>
</PackingList>"#;
        let result = parse_pkl(xml).unwrap();
        assert_eq!(result.namespace, PklNamespace::Smpte2067_2_2020);
    }

    #[test]
    fn pkl_detects_dci_429_8_namespace() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<PackingList xmlns="http://www.smpte-ra.org/schemas/429-8/2007/PKL">
    <Id>urn:uuid:f5e93462-aed2-44ad-a4ba-2adb65823e7c</Id>
    <IssueDate>2024-01-01T00:00:00Z</IssueDate>
    <AssetList><Asset>
        <Id>urn:uuid:00000000-0000-0000-0000-000000000001</Id>
        <Hash>2jmj7l5rSw0yVb/vlWAYkK/YBwk=</Hash>
        <Size>1024</Size>
        <Type>application/mxf</Type>
    </Asset></AssetList>
</PackingList>"#;
        let result = parse_pkl(xml).unwrap();
        assert_eq!(result.namespace, PklNamespace::Dci429_8);
    }

    #[test]
    fn assetmap_parses_with_2067_9_namespace() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<AssetMap xmlns="http://www.smpte-ra.org/schemas/2067-9/2016">
    <Id>urn:uuid:75864667-c65e-4aae-a5b2-fa5ea5fe31b7</Id>
    <VolumeCount>1</VolumeCount>
    <IssueDate>2024-01-01T00:00:00Z</IssueDate>
    <AssetList>
        <Asset>
            <Id>urn:uuid:0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85</Id>
            <ChunkList><Chunk><Path>test.xml</Path></Chunk></ChunkList>
        </Asset>
    </AssetList>
</AssetMap>"#;
        let result = parse_assetmap(xml).unwrap();
        assert_eq!(result.asset_list.assets.len(), 1);
        assert_eq!(result.namespace, AssetMapNamespace::Smpte2067_9_2016);
    }

    #[test]
    fn assetmap_parses_with_2020_namespace() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<AssetMap xmlns="http://www.smpte-ra.org/ns/2067-9/2020">
    <Id>urn:uuid:75864667-c65e-4aae-a5b2-fa5ea5fe31b7</Id>
    <VolumeCount>1</VolumeCount>
    <IssueDate>2024-01-01T00:00:00Z</IssueDate>
    <AssetList>
        <Asset>
            <Id>urn:uuid:0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85</Id>
            <ChunkList><Chunk><Path>test.xml</Path></Chunk></ChunkList>
        </Asset>
    </AssetList>
</AssetMap>"#;
        let result = parse_assetmap(xml).unwrap();
        assert_eq!(result.namespace, AssetMapNamespace::Smpte2067_9_2020);
    }

    #[test]
    fn assetmap_detects_dci_429_9_namespace() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<AssetMap xmlns="http://www.smpte-ra.org/schemas/429-9/2007/AM">
    <Id>urn:uuid:75864667-c65e-4aae-a5b2-fa5ea5fe31b7</Id>
    <VolumeCount>1</VolumeCount>
    <IssueDate>2024-01-01T00:00:00Z</IssueDate>
    <AssetList>
        <Asset>
            <Id>urn:uuid:0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85</Id>
            <ChunkList><Chunk><Path>test.xml</Path></Chunk></ChunkList>
        </Asset>
    </AssetList>
</AssetMap>"#;
        let result = parse_assetmap(xml).unwrap();
        assert_eq!(result.namespace, AssetMapNamespace::Dci429_9);
    }

    // ── OPL ──────────────────────────────────────────────────────────────────

    /// SMPTE ST 2067-100: OPL metadata fields are parsed correctly.
    #[test]
    fn opl_parses_core_metadata() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<OutputProfileList xmlns="http://www.smpte-ra.org/schemas/2067-100/2014">
    <Id>urn:uuid:8cf83c32-4949-4f00-b081-01e12b18932f</Id>
    <Annotation>OPL Example</Annotation>
    <IssueDate>2016-06-14T19:22:37-00:00</IssueDate>
    <Issuer>Clipster</Issuer>
    <Creator>Clipster 5.9.3.7</Creator>
    <CompositionPlaylistId>urn:uuid:0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85</CompositionPlaylistId>
    <AliasList/>
    <MacroList/>
</OutputProfileList>"#;
        let result = parse_opl(xml).unwrap();
        assert_eq!(
            result.id.to_string(),
            "8cf83c32-4949-4f00-b081-01e12b18932f"
        );
        assert_eq!(result.annotation.as_deref(), Some("OPL Example"));
        assert_eq!(result.issuer.as_deref(), Some("Clipster"));
        assert_eq!(result.creator.as_deref(), Some("Clipster 5.9.3.7"));
        assert_eq!(
            result.composition_playlist_id.to_string(),
            "0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85"
        );
    }

    /// SMPTE ST 2067-100: OPL with complex macros parses without error.
    #[test]
    fn opl_parses_real_test_file() {
        let xml = std::fs::read_to_string(test_data(
            "OPL/OPL_8cf83c32-4949-4f00-b081-01e12b18932f.xml",
        ))
        .unwrap();
        let result = parse_opl(&xml).unwrap();
        assert_eq!(
            result.id.to_string(),
            "8cf83c32-4949-4f00-b081-01e12b18932f"
        );
        assert_eq!(
            result.composition_playlist_id.to_string(),
            "0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85"
        );
    }

    /// SMPTE ST 2067-100: OPL with simple preset macro from ISXD test data.
    #[test]
    fn opl_parses_isxd_test_file() {
        let xml = std::fs::read_to_string(test_data(
            "ISXD/CompleteIMP/OPL_af6b288d-27e8-441f-9a36-2c4ab9025d19.xml",
        ))
        .unwrap();
        let result = parse_opl(&xml).unwrap();
        assert_eq!(
            result.id.to_string(),
            "af6b288d-27e8-441f-9a36-2c4ab9025d19"
        );
        assert_eq!(
            result.composition_playlist_id.to_string(),
            "b2d74f92-1990-41e0-869f-2179a50f7090"
        );
    }
}
