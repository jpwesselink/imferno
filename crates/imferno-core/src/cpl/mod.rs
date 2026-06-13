//! SMPTE ST 2067-3: Composition Playlist (CPL) parser
//!
//! This parser handles CPL files - the heart of IMF packages containing:
//! - Composition metadata and timeline structure
//! - Segment and sequence definitions
//! - Resource references to MXF essence files
//! - Edit rates and timing information
//! - EssenceDescriptors (RGBADescriptor, WAVEPCMDescriptor, DCTimedTextDescriptor, IABEssenceDescriptor)

pub mod types;
pub use types::{
    CodingEquations, ColorPrimaries, ContentKind, CplNamespace, EditRate, LanguageTag, MarkerLabel,
    McaTagSymbol, Resolution, TransferCharacteristic, VideoCodec,
};

pub mod validate;
pub use validate::validate_cpl as validate_cpl_constraints;

pub mod codes;

use crate::assetmap::{HashAlgorithm, ImfUuid};
use base64::Engine;
use quick_xml::events::Event;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use thiserror::Error;

#[cfg(all(feature = "xmlsec", not(target_arch = "wasm32")))]
use libxml::parser::Parser as XmlParser;
#[cfg(all(feature = "xmlsec1", not(target_arch = "wasm32")))]
use std::io::Write;
#[cfg(all(feature = "xmlsec1", not(target_arch = "wasm32")))]
use std::process::Command;
#[cfg(all(feature = "xmlsec", not(target_arch = "wasm32")))]
use xmlsec::{XmlSecKey, XmlSecKeyFormat, XmlSecSignatureContext};

#[cfg(feature = "typescript")]
use ts_rs::TS;

#[cfg(feature = "wasm")]
use tsify::Tsify;

// =============================================================================
// Error type
// =============================================================================

#[derive(Debug, Error)]
pub enum CplParseError {
    #[error("XML parse error: {0}")]
    Xml(#[from] quick_xml::DeError),

    #[error("strict unknown XML token(s): {0}")]
    StrictUnknownXml(String),

    #[error("strict schema violation: {0}")]
    StrictSchema(String),

    #[error("XMLDSIG verifier is required for selected signature mode")]
    SignatureVerifierRequired,

    #[error("XMLDSIG verification failed: {0}")]
    SignatureVerificationFailed(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnknownFieldMode {
    Ignore,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SchemaStrictMode {
    Off,
    Basic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignatureValidationMode {
    Ignore,
    RequirePresence,
    VerifyIfPresent,
    RequireValid,
}

pub trait XmlSignatureVerifier {
    fn verify(&self, xml_content: &str) -> Result<(), String>;
}

/// Concrete XMLDSIG verifier backend.
///
/// This verifier validates XMLDSIG structure and verifies `<Reference>` digest values.
/// For `URI=""` references it removes the first `<Signature>` element (enveloped
/// signature transform) and computes the digest over a normalized XML form.
///
/// Notes:
/// - This backend does not perform asymmetric key / certificate signature checks.
/// - `URI="#..."` references are validated for algorithm and digest value shape only.
#[derive(Debug, Default, Clone, Copy)]
pub struct ReferenceDigestXmlDsigVerifier;

impl XmlSignatureVerifier for ReferenceDigestXmlDsigVerifier {
    fn verify(&self, xml_content: &str) -> Result<(), String> {
        validate_signature_structure(xml_content)?;
        validate_reference_digests(xml_content)
    }
}

/// `xmlsec` crate-backed XMLDSIG verifier.
///
/// This backend uses the Rust `xmlsec` crate (libxml2/xmlsec bindings) and
/// verifies signatures against an explicitly supplied verification key.
#[cfg(all(feature = "xmlsec", not(target_arch = "wasm32")))]
#[derive(Debug, Clone)]
pub struct XmlSecCrateVerifier {
    key_data: Vec<u8>,
    key_format: XmlSecKeyFormat,
    key_password: Option<String>,
}

#[cfg(all(feature = "xmlsec", not(target_arch = "wasm32")))]
impl XmlSecCrateVerifier {
    pub fn from_key_data(key_data: Vec<u8>, key_format: XmlSecKeyFormat) -> Self {
        Self {
            key_data,
            key_format,
            key_password: None,
        }
    }

    pub fn from_pem(key_data: impl AsRef<[u8]>) -> Self {
        Self::from_key_data(key_data.as_ref().to_vec(), XmlSecKeyFormat::Pem)
    }

    pub fn with_password(mut self, password: impl Into<String>) -> Self {
        self.key_password = Some(password.into());
        self
    }
}

#[cfg(all(feature = "xmlsec", not(target_arch = "wasm32")))]
impl XmlSignatureVerifier for XmlSecCrateVerifier {
    fn verify(&self, xml_content: &str) -> Result<(), String> {
        validate_signature_structure(xml_content)?;

        let mut tmp = tempfile::NamedTempFile::new()
            .map_err(|e| format!("failed to create temp xml file: {}", e))?;
        tmp.write_all(xml_content.as_bytes())
            .map_err(|e| format!("failed to write temp xml file: {}", e))?;
        tmp.flush()
            .map_err(|e| format!("failed to flush temp xml file: {}", e))?;

        let doc = XmlParser::default()
            .parse_file(tmp.path().to_string_lossy().as_ref())
            .map_err(|e| format!("xml parse failed for xmlsec verifier: {}", e))?;

        let key = XmlSecKey::from_memory(
            &self.key_data,
            self.key_format,
            self.key_password.as_deref(),
        )
        .map_err(|e| format!("xmlsec key load failed: {}", e))?;

        let mut ctx = XmlSecSignatureContext::new();
        ctx.insert_key(key);

        let valid = ctx
            .verify_document(&doc)
            .map_err(|e| format!("xmlsec verify failed: {}", e))?;

        if valid {
            Ok(())
        } else {
            Err("xmlsec signature verification returned invalid".to_string())
        }
    }
}

/// `xmlsec1` CLI-backed XMLDSIG verifier.
///
/// This backend delegates full XML signature verification to the system
/// `xmlsec1` command line utility.
///
/// Enabled with the crate feature `xmlsec1` (non-WASM targets only).
#[cfg(all(feature = "xmlsec1", not(target_arch = "wasm32")))]
#[derive(Debug, Clone)]
pub struct XmlSec1Verifier {
    binary_path: String,
    extra_args: Vec<String>,
}

#[cfg(all(feature = "xmlsec1", not(target_arch = "wasm32")))]
impl Default for XmlSec1Verifier {
    fn default() -> Self {
        let binary_path =
            std::env::var("IMF_XMLSEC1_BIN").unwrap_or_else(|_| "xmlsec1".to_string());
        Self {
            binary_path,
            extra_args: Vec::new(),
        }
    }
}

#[cfg(all(feature = "xmlsec1", not(target_arch = "wasm32")))]
impl XmlSec1Verifier {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_binary_path(mut self, binary_path: impl Into<String>) -> Self {
        self.binary_path = binary_path.into();
        self
    }

    pub fn with_extra_args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.extra_args = args.into_iter().map(Into::into).collect();
        self
    }
}

#[cfg(all(feature = "xmlsec1", not(target_arch = "wasm32")))]
impl XmlSignatureVerifier for XmlSec1Verifier {
    fn verify(&self, xml_content: &str) -> Result<(), String> {
        let mut tmp = tempfile::NamedTempFile::new()
            .map_err(|e| format!("failed to create temp xml file: {}", e))?;
        tmp.write_all(xml_content.as_bytes())
            .map_err(|e| format!("failed to write temp xml file: {}", e))?;
        tmp.flush()
            .map_err(|e| format!("failed to flush temp xml file: {}", e))?;

        let mut command = Command::new(&self.binary_path);
        command.arg("--verify");
        for arg in &self.extra_args {
            command.arg(arg);
        }
        command.arg(tmp.path());

        let output = command
            .output()
            .map_err(|e| format!("failed to execute '{} --verify': {}", self.binary_path, e))?;

        if output.status.success() {
            return Ok(());
        }

        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let message = if !stderr.is_empty() {
            stderr
        } else if !stdout.is_empty() {
            stdout
        } else {
            format!("xmlsec1 exited with status {}", output.status)
        };

        Err(message)
    }
}

pub struct CplParseOptions<'a> {
    pub unknown_field_mode: UnknownFieldMode,
    pub schema_strict_mode: SchemaStrictMode,
    pub signature_validation_mode: SignatureValidationMode,
    pub signature_verifier: Option<&'a dyn XmlSignatureVerifier>,
}

impl Default for CplParseOptions<'_> {
    fn default() -> Self {
        Self {
            unknown_field_mode: UnknownFieldMode::Ignore,
            schema_strict_mode: SchemaStrictMode::Off,
            signature_validation_mode: SignatureValidationMode::Ignore,
            signature_verifier: None,
        }
    }
}

/// Build strict production-oriented parse options.
///
/// This enables strict unknown-token and basic schema checks, and requires
/// a valid XML signature using the provided verifier.
pub fn strict_production_parse_options<'a>(
    signature_verifier: &'a dyn XmlSignatureVerifier,
) -> CplParseOptions<'a> {
    CplParseOptions {
        unknown_field_mode: UnknownFieldMode::Error,
        schema_strict_mode: SchemaStrictMode::Basic,
        signature_validation_mode: SignatureValidationMode::RequireValid,
        signature_verifier: Some(signature_verifier),
    }
}

/// Create the recommended signature verifier backend for the current build.
///
/// Preference order:
/// 1. `xmlsec1` CLI backend when feature `xmlsec1` is enabled (non-WASM).
/// 2. Fallback digest verifier (`ReferenceDigestXmlDsigVerifier`) otherwise.
pub fn recommended_signature_verifier() -> Box<dyn XmlSignatureVerifier> {
    #[cfg(all(feature = "xmlsec1", not(target_arch = "wasm32")))]
    {
        Box::new(XmlSec1Verifier::default())
    }

    #[cfg(not(all(feature = "xmlsec1", not(target_arch = "wasm32"))))]
    {
        Box::new(ReferenceDigestXmlDsigVerifier)
    }
}

fn validate_signature_structure(xml_content: &str) -> Result<(), String> {
    let signature_xml = extract_first_element(xml_content, "Signature")
        .ok_or_else(|| "missing Signature element".to_string())?;

    if extract_first_element(&signature_xml, "SignedInfo").is_none() {
        return Err("missing SignedInfo element".to_string());
    }

    let signature_value_raw = extract_first_element_text(&signature_xml, "SignatureValue")
        .ok_or_else(|| "missing SignatureValue element".to_string())?;
    let signature_value = collapse_xml_text(&signature_value_raw);
    if signature_value.is_empty() {
        return Err("SignatureValue is empty".to_string());
    }
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(signature_value.as_bytes())
        .map_err(|e| format!("invalid SignatureValue base64: {}", e))?;
    if decoded.is_empty() {
        return Err("SignatureValue decodes to zero bytes".to_string());
    }

    if let Some(signature_method_alg) = extract_signature_method_algorithm(&signature_xml) {
        let is_supported = matches!(
            signature_method_alg.as_str(),
            "http://www.w3.org/2000/09/xmldsig#rsa-sha1"
                | "http://www.w3.org/2001/04/xmldsig-more#rsa-sha256"
                | "http://www.w3.org/2001/04/xmldsig-more#rsa-sha384"
                | "http://www.w3.org/2001/04/xmldsig-more#rsa-sha512"
        );
        if !is_supported {
            return Err(format!(
                "unsupported SignatureMethod algorithm: {}",
                signature_method_alg
            ));
        }
    }

    Ok(())
}

fn validate_reference_digests(xml_content: &str) -> Result<(), String> {
    let signature_xml = extract_first_element(xml_content, "Signature")
        .ok_or_else(|| "missing Signature element".to_string())?;

    let references = extract_reference_entries(&signature_xml)?;
    if references.is_empty() {
        return Err("SignedInfo contains no Reference elements".to_string());
    }

    for reference in references {
        let digest_algorithm = HashAlgorithm::from_uri(&reference.digest_method_algorithm)
            .ok_or_else(|| {
                format!(
                    "unsupported DigestMethod algorithm: {}",
                    reference.digest_method_algorithm
                )
            })?;

        let expected_digest = base64::engine::general_purpose::STANDARD
            .decode(reference.digest_value.as_bytes())
            .map_err(|e| format!("invalid DigestValue base64: {}", e))?;

        if expected_digest.len() != digest_algorithm.digest_len() {
            return Err(format!(
                "DigestValue length {} does not match {} digest length {}",
                expected_digest.len(),
                digest_algorithm,
                digest_algorithm.digest_len()
            ));
        }

        match reference.uri.as_deref().unwrap_or("") {
            "" => {
                let unsigned_xml = strip_first_signature_element(xml_content)
                    .ok_or_else(|| "failed to remove Signature element for URI=\"\"".to_string())?;
                let normalized = normalize_xml_for_digest(&unsigned_xml);
                let actual_digest = compute_hash(digest_algorithm, normalized.as_bytes());
                if actual_digest != expected_digest {
                    return Err(format!(
                        "DigestValue mismatch for Reference URI=\"\" (algorithm {})",
                        digest_algorithm
                    ));
                }
            }
            uri if uri.starts_with('#') => {}
            uri => {
                return Err(format!(
                    "unsupported Reference URI '{}'; only empty or fragment URIs are supported",
                    uri
                ));
            }
        }
    }

    Ok(())
}

fn compute_hash(algorithm: HashAlgorithm, bytes: &[u8]) -> Vec<u8> {
    match algorithm {
        HashAlgorithm::Sha1 => {
            use sha1::Digest;
            let mut hasher = sha1::Sha1::new();
            hasher.update(bytes);
            hasher.finalize().to_vec()
        }
        HashAlgorithm::Sha256 => {
            use sha2::Digest;
            let mut hasher = sha2::Sha256::new();
            hasher.update(bytes);
            hasher.finalize().to_vec()
        }
    }
}

#[derive(Debug, Clone)]
struct SignatureReferenceEntry {
    uri: Option<String>,
    digest_method_algorithm: String,
    digest_value: String,
}

fn extract_signature_method_algorithm(signature_xml: &str) -> Option<String> {
    use std::sync::LazyLock;
    static RE: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(
            r#"<(?:(?:\w+):)?SignatureMethod\b[^>]*\bAlgorithm\s*=\s*\"([^\"]+)\"[^>]*/?>"#,
        )
        .unwrap()
    });
    RE.captures(signature_xml)
        .and_then(|c| c.get(1).map(|m| m.as_str().trim().to_string()))
}

fn extract_reference_entries(signature_xml: &str) -> Result<Vec<SignatureReferenceEntry>, String> {
    use std::sync::LazyLock;
    static REFERENCE_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(r#"(?s)<(?:(?:\w+):)?Reference\b([^>]*)>(.*?)</(?:(?:\w+):)?Reference>"#)
            .unwrap()
    });
    static URI_RE: LazyLock<regex::Regex> =
        LazyLock::new(|| regex::Regex::new(r#"\bURI\s*=\s*\"([^\"]*)\""#).unwrap());
    static DIGEST_METHOD_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(
            r#"<(?:(?:\w+):)?DigestMethod\b[^>]*\bAlgorithm\s*=\s*\"([^\"]+)\"[^>]*/?>"#,
        )
        .unwrap()
    });
    static DIGEST_VALUE_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(
            r#"(?s)<(?:(?:\w+):)?DigestValue\b[^>]*>(.*?)</(?:(?:\w+):)?DigestValue>"#,
        )
        .unwrap()
    });

    let mut out = Vec::new();
    for captures in REFERENCE_RE.captures_iter(signature_xml) {
        let attrs = captures
            .get(1)
            .map(|m| m.as_str())
            .ok_or_else(|| "internal parse error while reading Reference attributes".to_string())?;
        let inner = captures
            .get(2)
            .map(|m| m.as_str())
            .ok_or_else(|| "internal parse error while reading Reference body".to_string())?;

        let uri = URI_RE
            .captures(attrs)
            .and_then(|c| c.get(1).map(|m| m.as_str().to_string()));
        let digest_method_algorithm = DIGEST_METHOD_RE
            .captures(inner)
            .and_then(|c| c.get(1).map(|m| m.as_str().trim().to_string()))
            .ok_or_else(|| "Reference missing DigestMethod/@Algorithm".to_string())?;
        let digest_value = DIGEST_VALUE_RE
            .captures(inner)
            .and_then(|c| c.get(1).map(|m| collapse_xml_text(m.as_str())))
            .ok_or_else(|| "Reference missing DigestValue".to_string())?;

        out.push(SignatureReferenceEntry {
            uri,
            digest_method_algorithm,
            digest_value,
        });
    }

    Ok(out)
}

fn strip_first_signature_element(xml: &str) -> Option<String> {
    use std::sync::LazyLock;
    static RE: LazyLock<regex::Regex> = LazyLock::new(|| {
        regex::Regex::new(r#"(?s)<(?:(?:\w+):)?Signature\b[^>]*>.*?</(?:(?:\w+):)?Signature\s*>"#)
            .unwrap()
    });
    let m = RE.find(xml)?;
    let mut out = String::with_capacity(xml.len() - (m.end() - m.start()));
    out.push_str(&xml[..m.start()]);
    out.push_str(&xml[m.end()..]);
    Some(out)
}

fn normalize_xml_for_digest(xml: &str) -> String {
    use std::sync::LazyLock;
    static DECL_RE: LazyLock<regex::Regex> =
        LazyLock::new(|| regex::Regex::new(r#"(?s)^\s*<\?xml[^>]*\?>"#).unwrap());
    static INTER_TAG_WS_RE: LazyLock<regex::Regex> =
        LazyLock::new(|| regex::Regex::new(r#">\s+<"#).unwrap());

    let no_decl = xml.strip_prefix("\u{FEFF}").unwrap_or(xml).trim();
    let without_decl = DECL_RE.replace(no_decl, "").to_string();
    INTER_TAG_WS_RE
        .replace_all(without_decl.trim(), "><")
        .to_string()
}

fn extract_first_element(xml: &str, local_name: &str) -> Option<String> {
    let escaped = regex::escape(local_name);
    let pattern = format!(
        r#"(?s)<(?:(?:\w+):)?{name}\b[^>]*>.*?</(?:(?:\w+):)?{name}\s*>"#,
        name = escaped
    );
    // Dynamic pattern from local_name — cannot use LazyLock
    let re = regex::Regex::new(&pattern).expect("valid regex pattern");
    re.find(xml).map(|m| m.as_str().to_string())
}

fn extract_first_element_text(xml: &str, local_name: &str) -> Option<String> {
    let escaped = regex::escape(local_name);
    let pattern = format!(
        r#"(?s)<(?:(?:\w+):)?{name}\b[^>]*>(.*?)</(?:(?:\w+):)?{name}\s*>"#,
        name = escaped
    );
    // Dynamic pattern from local_name — cannot use LazyLock
    let re = regex::Regex::new(&pattern).expect("valid regex pattern");
    re.captures(xml)
        .and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
}

fn collapse_xml_text(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}

// =============================================================================
// Serde deserialization helpers
// =============================================================================

mod de_helpers {
    use crate::cpl::{
        CodingEquations, ColorPrimaries, EditRate, LanguageTag, McaTagSymbol,
        TransferCharacteristic, VideoCodec,
    };
    use serde::{Deserialize, Deserializer};

    pub fn de_optional_edit_rate<'de, D: Deserializer<'de>>(
        d: D,
    ) -> Result<Option<EditRate>, D::Error> {
        let s = String::deserialize(d)?;
        let trimmed = s.trim();
        if trimmed.is_empty() {
            Ok(None)
        } else {
            // Support both space-separated ("60000 1001") and slash-separated ("60000/1001") formats
            let normalized = trimmed.replace('/', " ");
            EditRate::parse(&normalized)
                .map(Some)
                .map_err(serde::de::Error::custom)
        }
    }

    /// Shared helper: deserialize an optional string, trim, and apply a converter if non-empty.
    fn de_optional_ul_type<'de, D, T, F>(d: D, from_ul: F) -> Result<Option<T>, D::Error>
    where
        D: Deserializer<'de>,
        F: FnOnce(&str) -> T,
    {
        let s = String::deserialize(d)?;
        Ok(if s.trim().is_empty() {
            None
        } else {
            Some(from_ul(s.trim()))
        })
    }

    pub fn de_optional_color_primaries<'de, D: Deserializer<'de>>(
        d: D,
    ) -> Result<Option<ColorPrimaries>, D::Error> {
        de_optional_ul_type(d, ColorPrimaries::from_ul)
    }

    pub fn de_optional_transfer_characteristic<'de, D: Deserializer<'de>>(
        d: D,
    ) -> Result<Option<TransferCharacteristic>, D::Error> {
        de_optional_ul_type(d, TransferCharacteristic::from_ul)
    }

    pub fn de_optional_video_codec<'de, D: Deserializer<'de>>(
        d: D,
    ) -> Result<Option<VideoCodec>, D::Error> {
        de_optional_ul_type(d, VideoCodec::from_ul)
    }

    pub fn de_optional_coding_equations<'de, D: Deserializer<'de>>(
        d: D,
    ) -> Result<Option<CodingEquations>, D::Error> {
        de_optional_ul_type(d, CodingEquations::from_ul)
    }

    pub fn de_optional_mca_tag_symbol<'de, D: Deserializer<'de>>(
        d: D,
    ) -> Result<Option<McaTagSymbol>, D::Error> {
        let s = String::deserialize(d)?;
        Ok(if s.trim().is_empty() {
            None
        } else {
            Some(McaTagSymbol::parse(s.trim()))
        })
    }

    pub fn de_optional_language_tag<'de, D: Deserializer<'de>>(
        d: D,
    ) -> Result<Option<LanguageTag>, D::Error> {
        let s = String::deserialize(d)?;
        let trimmed = s.trim();
        if trimmed.is_empty() {
            Ok(None)
        } else {
            LanguageTag::parse(trimmed)
                .map(Some)
                .map_err(serde::de::Error::custom)
        }
    }

    /// ColorSiting may be numeric (0 = CoSiting) or a label string ("CoSiting").
    /// Maps known label strings to their MXF numeric values; unknown strings → None.
    pub fn de_optional_color_siting<'de, D: Deserializer<'de>>(
        d: D,
    ) -> Result<Option<u32>, D::Error> {
        let s = String::deserialize(d)?;
        let s = s.trim();
        if s.is_empty() {
            return Ok(None);
        }
        if let Ok(n) = s.parse::<u32>() {
            return Ok(Some(n));
        }
        let v = match s.to_lowercase().as_str() {
            "cositing" => 0,
            "horizcositing" => 1,
            "threetap" => 2,
            "quincunx" => 3,
            "rec709" => 4,
            "rec601" => 6,
            _ => return Ok(None),
        };
        Ok(Some(v))
    }

    pub fn de_language_tag_list<'de, D: Deserializer<'de>>(
        d: D,
    ) -> Result<Vec<LanguageTag>, D::Error> {
        let s = String::deserialize(d)?;
        s.split(',')
            .map(|part| part.trim())
            .filter(|part| !part.is_empty())
            .map(|part| LanguageTag::parse(part).map_err(serde::de::Error::custom))
            .collect()
    }
}

/// Default content kind when not specified
fn default_content_kind() -> ContentKindElement {
    ContentKindElement {
        kind: ContentKind::Other("unknown".to_string()),
        scope: None,
    }
}

// =============================================================================
// ContentKindElement — text + @scope attribute per CPL XSD ContentKindType
// =============================================================================

/// Default scope URI for ContentKind per CPL XSD (ST 2067-3).
pub const CONTENT_KIND_DEFAULT_SCOPE: &str =
    "http://www.smpte-ra.org/schemas/2067-3/2013#content-kind";

/// ContentKind element with optional `scope` attribute, per CPL XSD `ContentKindType`.
///
/// ```xml
/// <ContentKind scope="http://www.smpte-ra.org/schemas/2067-3/2013#content-kind">feature</ContentKind>
/// ```
///
/// When `scope` is `None`, the XSD default applies: [`CONTENT_KIND_DEFAULT_SCOPE`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct ContentKindElement {
    pub kind: ContentKind,
    /// Scope URI. `None` means the XSD default applies.
    pub scope: Option<String>,
}

impl ContentKindElement {
    /// Returns the effective scope, falling back to the XSD default.
    pub fn effective_scope(&self) -> &str {
        self.scope.as_deref().unwrap_or(CONTENT_KIND_DEFAULT_SCOPE)
    }
}

impl std::fmt::Display for ContentKindElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.kind.fmt(f)
    }
}

impl PartialEq<ContentKind> for ContentKindElement {
    fn eq(&self, other: &ContentKind) -> bool {
        self.kind == *other
    }
}

impl From<ContentKind> for ContentKindElement {
    fn from(kind: ContentKind) -> Self {
        Self { kind, scope: None }
    }
}

impl<'de> Deserialize<'de> for ContentKindElement {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;

        struct ContentKindElementVisitor;

        impl<'de> Visitor<'de> for ContentKindElementVisitor {
            type Value = ContentKindElement;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str(
                    "a string or an object with text content and optional @scope attribute",
                )
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(ContentKindElement {
                    kind: ContentKind::parse(value),
                    scope: None,
                })
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut text = None;
                let mut scope = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "$text" | "#text" | "$value" => {
                            if text.is_some() {
                                return Err(de::Error::duplicate_field("text"));
                            }
                            text = Some(map.next_value::<String>()?);
                        }
                        "@scope" | "scope" => {
                            if scope.is_some() {
                                return Err(de::Error::duplicate_field("scope"));
                            }
                            let raw: String = map.next_value()?;
                            let trimmed = raw.trim();
                            if !trimmed.is_empty() {
                                scope = Some(trimmed.to_string());
                            }
                        }
                        _ => {
                            let _ = map.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }

                let kind = ContentKind::parse(text.as_deref().unwrap_or(""));
                Ok(ContentKindElement { kind, scope })
            }
        }

        deserializer.deserialize_any(ContentKindElementVisitor)
    }
}

impl Serialize for ContentKindElement {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        if self.scope.is_some() {
            let mut state = serializer.serialize_struct("ContentKindElement", 2)?;
            state.serialize_field("$text", &self.kind.to_string())?;
            state.serialize_field("@scope", &self.scope)?;
            state.end()
        } else {
            serializer.serialize_str(&self.kind.to_string())
        }
    }
}

#[cfg(feature = "jsonschema")]
impl schemars::JsonSchema for ContentKindElement {
    fn schema_name() -> String {
        "ContentKindElement".to_owned()
    }

    fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        use schemars::schema::*;

        let string_schema = gen.subschema_for::<String>();
        let mut obj = SchemaObject {
            instance_type: Some(InstanceType::Object.into()),
            ..Default::default()
        };
        let obj_validation = obj.object();
        obj_validation
            .properties
            .insert("$text".to_owned(), gen.subschema_for::<String>());
        obj_validation
            .properties
            .insert("@scope".to_owned(), gen.subschema_for::<Option<String>>());
        obj_validation.required.insert("$text".to_owned());

        SchemaObject {
            subschemas: Some(Box::new(SubschemaValidation {
                any_of: Some(vec![string_schema, obj.into()]),
                ..Default::default()
            })),
            ..Default::default()
        }
        .into()
    }
}

// =============================================================================
// MarkerLabelElement — text + @scope attribute per CPL XSD LabelType
// =============================================================================

/// Default scope URI for MarkerLabel per CPL XSD (ST 2067-3).
pub const MARKER_LABEL_DEFAULT_SCOPE: &str =
    "http://www.smpte-ra.org/schemas/2067-3/2013#standard-markers";

/// Marker Label element with optional `scope` attribute, per CPL XSD `LabelType`.
///
/// ```xml
/// <Label scope="http://www.smpte-ra.org/schemas/2067-3/2013#standard-markers">FFOC</Label>
/// ```
///
/// When `scope` is `None`, the XSD default applies: [`MARKER_LABEL_DEFAULT_SCOPE`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct MarkerLabelElement {
    pub label: MarkerLabel,
    /// Scope URI. `None` means the XSD default applies.
    pub scope: Option<String>,
}

impl MarkerLabelElement {
    /// Returns the effective scope, falling back to the XSD default.
    pub fn effective_scope(&self) -> &str {
        self.scope.as_deref().unwrap_or(MARKER_LABEL_DEFAULT_SCOPE)
    }
}

impl std::fmt::Display for MarkerLabelElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.label.fmt(f)
    }
}

impl PartialEq<MarkerLabel> for MarkerLabelElement {
    fn eq(&self, other: &MarkerLabel) -> bool {
        self.label == *other
    }
}

impl From<MarkerLabel> for MarkerLabelElement {
    fn from(label: MarkerLabel) -> Self {
        Self { label, scope: None }
    }
}

impl<'de> Deserialize<'de> for MarkerLabelElement {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;

        struct MarkerLabelElementVisitor;

        impl<'de> Visitor<'de> for MarkerLabelElementVisitor {
            type Value = MarkerLabelElement;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str(
                    "a string or an object with text content and optional @scope attribute",
                )
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(MarkerLabelElement {
                    label: MarkerLabel::parse(value),
                    scope: None,
                })
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut text = None;
                let mut scope = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "$text" | "#text" | "$value" => {
                            if text.is_some() {
                                return Err(de::Error::duplicate_field("text"));
                            }
                            text = Some(map.next_value::<String>()?);
                        }
                        "@scope" | "scope" => {
                            if scope.is_some() {
                                return Err(de::Error::duplicate_field("scope"));
                            }
                            let raw: String = map.next_value()?;
                            let trimmed = raw.trim();
                            if !trimmed.is_empty() {
                                scope = Some(trimmed.to_string());
                            }
                        }
                        _ => {
                            let _ = map.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }

                let label = MarkerLabel::parse(text.as_deref().unwrap_or(""));
                Ok(MarkerLabelElement { label, scope })
            }
        }

        deserializer.deserialize_any(MarkerLabelElementVisitor)
    }
}

impl Serialize for MarkerLabelElement {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        if self.scope.is_some() {
            let mut state = serializer.serialize_struct("MarkerLabelElement", 2)?;
            state.serialize_field("$text", &self.label.to_string())?;
            state.serialize_field("@scope", &self.scope)?;
            state.end()
        } else {
            serializer.serialize_str(&self.label.to_string())
        }
    }
}

#[cfg(feature = "jsonschema")]
impl schemars::JsonSchema for MarkerLabelElement {
    fn schema_name() -> String {
        "MarkerLabelElement".to_owned()
    }

    fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        use schemars::schema::*;

        let string_schema = gen.subschema_for::<String>();
        let mut obj = SchemaObject {
            instance_type: Some(InstanceType::Object.into()),
            ..Default::default()
        };
        let obj_validation = obj.object();
        obj_validation
            .properties
            .insert("$text".to_owned(), gen.subschema_for::<String>());
        obj_validation
            .properties
            .insert("@scope".to_owned(), gen.subschema_for::<Option<String>>());
        obj_validation.required.insert("$text".to_owned());

        SchemaObject {
            subschemas: Some(Box::new(SubschemaValidation {
                any_of: Some(vec![string_schema, obj.into()]),
                ..Default::default()
            })),
            ..Default::default()
        }
        .into()
    }
}

// =============================================================================
// XML namespace stripping
// =============================================================================

/// Strip XML namespace prefixes so quick-xml/serde can match element names uniformly.
///
/// Converts `<r0:RGBADescriptor>` → `<RGBADescriptor>`, `</cc:MainImageSequence>` → `</MainImageSequence>`, etc.
/// Also strips `xmlns:prefix="..."` declarations (but preserves default `xmlns="..."`).
///
/// This is the same approach used by the TypeScript mapper (fast-xml-parser namespace stripping).
pub fn strip_xml_namespaces(xml: &str) -> String {
    use std::sync::LazyLock;
    // Strip namespace prefixes from element names: <ns:Element → <Element, </ns:Element → </Element
    static TAG_PREFIX_RE: LazyLock<regex::Regex> =
        LazyLock::new(|| regex::Regex::new(r"<(/?)(\w+):(\w)").unwrap());
    let result = TAG_PREFIX_RE.replace_all(xml, "<$1$3");
    // Strip xmlns:prefix="..." attribute declarations
    static XMLNS_PREFIX_RE: LazyLock<regex::Regex> =
        LazyLock::new(|| regex::Regex::new(r#"\s+xmlns:\w+="[^"]*""#).unwrap());
    XMLNS_PREFIX_RE.replace_all(&result, "").to_string()
}

// =============================================================================
// LanguageString - String with optional language attribute
// =============================================================================

/// String with optional language attribute
#[derive(Debug, Default, PartialEq, Clone)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct LanguageString {
    pub text: String,
    pub language: Option<LanguageTag>, // RFC 5646 language tag
}

impl std::fmt::Display for LanguageString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(lang) = &self.language {
            write!(f, "{} ({})", self.text, lang)
        } else {
            write!(f, "{}", self.text)
        }
    }
}

// Helper to deserialize plain strings as LanguageString
impl<'de> Deserialize<'de> for LanguageString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;

        struct LanguageStringVisitor;

        impl<'de> Visitor<'de> for LanguageStringVisitor {
            type Value = LanguageString;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string or an object with text and optional language")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(LanguageString {
                    text: value.to_string(),
                    language: None,
                })
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut text = None;
                let mut language = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "$text" | "#text" | "$value" => {
                            if text.is_some() {
                                return Err(de::Error::duplicate_field("text"));
                            }
                            text = Some(map.next_value()?);
                        }
                        "@language" | "language" => {
                            if language.is_some() {
                                return Err(de::Error::duplicate_field("language"));
                            }
                            let raw: String = map.next_value()?;
                            let trimmed = raw.trim();
                            if !trimmed.is_empty() {
                                language = Some(LanguageTag::new(trimmed));
                            }
                        }
                        _ => {
                            // Ignore unknown fields
                            let _ = map.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }

                Ok(LanguageString {
                    text: text.unwrap_or_default(),
                    language,
                })
            }
        }

        deserializer.deserialize_any(LanguageStringVisitor)
    }
}

impl Serialize for LanguageString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;

        if self.language.is_some() {
            let mut state = serializer.serialize_struct("LanguageString", 2)?;
            state.serialize_field("$text", &self.text)?;
            state.serialize_field("@language", &self.language)?;
            state.end()
        } else {
            serializer.serialize_str(&self.text)
        }
    }
}

#[cfg(feature = "jsonschema")]
impl schemars::JsonSchema for LanguageString {
    fn schema_name() -> String {
        "LanguageString".to_owned()
    }

    fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        use schemars::schema::*;

        let string_schema = gen.subschema_for::<String>();
        let mut obj = SchemaObject {
            instance_type: Some(InstanceType::Object.into()),
            ..Default::default()
        };
        let obj_validation = obj.object();
        obj_validation
            .properties
            .insert("$text".to_owned(), gen.subschema_for::<String>());
        obj_validation.properties.insert(
            "@language".to_owned(),
            gen.subschema_for::<Option<String>>(),
        );
        obj_validation.required.insert("$text".to_owned());

        SchemaObject {
            subschemas: Some(Box::new(SubschemaValidation {
                any_of: Some(vec![string_schema, obj.into()]),
                ..Default::default()
            })),
            ..Default::default()
        }
        .into()
    }
}

// =============================================================================
// Locale types
// =============================================================================

/// LocaleList - Content locale information
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct LocaleList {
    #[cfg_attr(not(feature = "wasm"), serde(rename = "Locale", default))]
    #[cfg_attr(feature = "wasm", serde(rename = "locales", alias = "Locale", default))]
    pub locales: Vec<Locale>,
}

#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct Locale {
    #[cfg_attr(not(feature = "wasm"), serde(rename = "LanguageList", default))]
    #[cfg_attr(
        feature = "wasm",
        serde(rename = "languageList", alias = "LanguageList", default)
    )]
    pub language_list: Option<LanguageList>,

    #[cfg_attr(not(feature = "wasm"), serde(rename = "RegionList", default))]
    #[cfg_attr(
        feature = "wasm",
        serde(rename = "regionList", alias = "RegionList", default)
    )]
    pub region_list: Option<RegionList>,

    #[cfg_attr(
        not(feature = "wasm"),
        serde(rename = "ContentMaturityRatingList", default)
    )]
    #[cfg_attr(
        feature = "wasm",
        serde(
            rename = "contentMaturityRatingList",
            alias = "ContentMaturityRatingList",
            default
        )
    )]
    pub content_maturity_rating_list: Option<ContentMaturityRatingList>,
}

#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct ContentMaturityRatingList {
    #[cfg_attr(not(feature = "wasm"), serde(rename = "ContentMaturityRating"))]
    #[cfg_attr(
        feature = "wasm",
        serde(rename = "contentMaturityRatings", alias = "ContentMaturityRating")
    )]
    pub ratings: Vec<ContentMaturityRating>,
}

/// A single content maturity rating entry per ST 2067-3 / ST 2067-21 §5.1.3.
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct ContentMaturityRating {
    #[cfg_attr(not(feature = "wasm"), serde(rename = "Agency"))]
    #[cfg_attr(feature = "wasm", serde(rename = "agency", alias = "Agency"))]
    pub agency: String,

    #[cfg_attr(not(feature = "wasm"), serde(rename = "Rating", default))]
    #[cfg_attr(feature = "wasm", serde(rename = "rating", alias = "Rating", default))]
    pub rating: Option<String>,

    #[cfg_attr(not(feature = "wasm"), serde(rename = "Audience", default))]
    #[cfg_attr(
        feature = "wasm",
        serde(rename = "audience", alias = "Audience", default)
    )]
    pub audience: Option<AudienceElement>,
}

/// The `<Audience>` element carries an optional `scope` attribute.
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct AudienceElement {
    #[serde(rename = "@scope", default)]
    pub scope: Option<String>,

    #[serde(rename = "$text", default)]
    pub text: Option<String>,
}

#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct LanguageList {
    #[cfg_attr(not(feature = "wasm"), serde(rename = "Language"))]
    #[cfg_attr(feature = "wasm", serde(rename = "languages", alias = "Language"))]
    pub languages: Vec<LanguageTag>, // RFC 5646 language tags
}

#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct RegionList {
    #[cfg_attr(not(feature = "wasm"), serde(rename = "Region"))]
    #[cfg_attr(feature = "wasm", serde(rename = "regions", alias = "Region"))]
    pub regions: Vec<String>, // ISO 3166-1 country codes
}

// =============================================================================
// Extension Properties
// =============================================================================

#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct ExtensionProperties {
    #[cfg_attr(
        not(feature = "wasm"),
        serde(rename = "ApplicationIdentification", default)
    )]
    #[cfg_attr(
        feature = "wasm",
        serde(
            rename = "applicationIdentification",
            alias = "ApplicationIdentification",
            default
        )
    )]
    pub application_identification: Option<String>,

    #[cfg_attr(not(feature = "wasm"), serde(rename = "MaxCLL", default))]
    #[cfg_attr(feature = "wasm", serde(rename = "maxCLL", alias = "MaxCLL", default))]
    pub max_cll: Option<u32>,

    #[cfg_attr(not(feature = "wasm"), serde(rename = "MaxFALL", default))]
    #[cfg_attr(
        feature = "wasm",
        serde(rename = "maxFALL", alias = "MaxFALL", default)
    )]
    pub max_fall: Option<u32>,
}

// =============================================================================
// EssenceDescriptor types - proper deserialization of MXF metadata
// =============================================================================

#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct EssenceDescriptorList {
    #[cfg_attr(not(feature = "wasm"), serde(rename = "EssenceDescriptor"))]
    #[cfg_attr(
        feature = "wasm",
        serde(rename = "essenceDescriptors", alias = "EssenceDescriptor")
    )]
    pub essence_descriptors: Vec<EssenceDescriptor>,
}

#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct EssenceDescriptor {
    #[cfg_attr(not(feature = "wasm"), serde(rename = "Id"))]
    #[cfg_attr(feature = "wasm", serde(rename = "id", alias = "Id"))]
    pub id: ImfUuid,

    #[cfg_attr(not(feature = "wasm"), serde(rename = "RGBADescriptor", default))]
    #[cfg_attr(
        feature = "wasm",
        serde(rename = "rgbaDescriptor", alias = "RGBADescriptor", default)
    )]
    pub rgba_descriptor: Option<RGBADescriptor>,

    #[cfg_attr(not(feature = "wasm"), serde(rename = "CDCIDescriptor", default))]
    #[cfg_attr(
        feature = "wasm",
        serde(rename = "cdciDescriptor", alias = "CDCIDescriptor", default)
    )]
    pub cdci_descriptor: Option<CDCIDescriptor>,

    #[cfg_attr(not(feature = "wasm"), serde(rename = "WAVEPCMDescriptor", default))]
    #[cfg_attr(
        feature = "wasm",
        serde(rename = "wavePCMDescriptor", alias = "WAVEPCMDescriptor", default)
    )]
    pub wave_pcm_descriptor: Option<WAVEPCMDescriptor>,

    #[cfg_attr(
        not(feature = "wasm"),
        serde(rename = "DCTimedTextDescriptor", default)
    )]
    #[cfg_attr(
        feature = "wasm",
        serde(
            rename = "dcTimedTextDescriptor",
            alias = "DCTimedTextDescriptor",
            default
        )
    )]
    pub dc_timed_text_descriptor: Option<DCTimedTextDescriptor>,

    #[cfg_attr(not(feature = "wasm"), serde(rename = "IABEssenceDescriptor", default))]
    #[cfg_attr(
        feature = "wasm",
        serde(
            rename = "iabEssenceDescriptor",
            alias = "IABEssenceDescriptor",
            default
        )
    )]
    pub iab_essence_descriptor: Option<IABEssenceDescriptor>,

    #[cfg_attr(
        not(feature = "wasm"),
        serde(rename = "ISXDDataEssenceDescriptor", default)
    )]
    #[cfg_attr(
        feature = "wasm",
        serde(
            rename = "isxdDataEssenceDescriptor",
            alias = "ISXDDataEssenceDescriptor",
            default
        )
    )]
    pub isxd_data_essence_descriptor: Option<ISXDDataEssenceDescriptor>,
}

/// RGBA video descriptor (JPEG 2000 RGB content)
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct RGBADescriptor {
    #[serde(rename = "InstanceID", default)]
    pub instance_id: Option<String>,

    #[serde(rename = "DisplayWidth", default)]
    pub display_width: Option<u32>,

    #[serde(rename = "DisplayHeight", default)]
    pub display_height: Option<u32>,

    #[serde(rename = "StoredWidth", default)]
    pub stored_width: Option<u32>,

    #[serde(rename = "StoredHeight", default)]
    pub stored_height: Option<u32>,

    #[serde(
        rename = "SampleRate",
        default,
        deserialize_with = "de_helpers::de_optional_edit_rate"
    )]
    pub sample_rate: Option<EditRate>,

    #[serde(rename = "ImageAspectRatio", default)]
    pub image_aspect_ratio: Option<String>,

    #[serde(
        rename = "ColorPrimaries",
        default,
        deserialize_with = "de_helpers::de_optional_color_primaries"
    )]
    pub color_primaries: Option<ColorPrimaries>,

    #[serde(
        rename = "TransferCharacteristic",
        default,
        deserialize_with = "de_helpers::de_optional_transfer_characteristic"
    )]
    pub transfer_characteristic: Option<TransferCharacteristic>,

    #[serde(
        rename = "CodingEquations",
        default,
        deserialize_with = "de_helpers::de_optional_coding_equations"
    )]
    pub coding_equations: Option<CodingEquations>,

    #[serde(
        rename = "PictureCompression",
        default,
        deserialize_with = "de_helpers::de_optional_video_codec"
    )]
    pub picture_compression: Option<VideoCodec>,

    /// Generic Picture Essence Descriptor: Frame Layout
    /// "FullFrame" (00h, progressive) or "SeparateFields" (01h, interlaced)
    #[serde(rename = "FrameLayout", default)]
    pub frame_layout: Option<String>,

    /// Generic Picture Essence Descriptor: DisplayF2Offset
    #[serde(rename = "DisplayF2Offset", default)]
    pub display_f2_offset: Option<i32>,

    /// RGBA Descriptor: Component Max Ref (Table 10/11)
    #[serde(rename = "ComponentMaxRef", default)]
    pub component_max_ref: Option<u32>,

    /// RGBA Descriptor: Component Min Ref (Table 10/11)
    #[serde(rename = "ComponentMinRef", default)]
    pub component_min_ref: Option<u32>,

    /// RGBA Descriptor: Scanning Direction (Table 10)
    /// Shall be "ScanningDirection_LeftToRightTopToBottom" (00h)
    #[serde(rename = "ScanningDirection", default)]
    pub scanning_direction: Option<String>,

    /// Table 8: StoredF2Offset — shall not be present
    #[serde(rename = "StoredF2Offset", default)]
    pub stored_f2_offset: Option<i32>,

    /// Table 8: SampledWidth — shall not be present or shall be equal to StoredWidth
    #[serde(rename = "SampledWidth", default)]
    pub sampled_width: Option<u32>,

    /// Table 8: SampledHeight — shall not be present or shall be equal to StoredHeight
    #[serde(rename = "SampledHeight", default)]
    pub sampled_height: Option<u32>,

    /// Table 8: SampledXOffset — shall not be present or shall be 0
    #[serde(rename = "SampledXOffset", default)]
    pub sampled_x_offset: Option<u32>,

    /// Table 8: SampledYOffset — shall not be present or shall be 0
    #[serde(rename = "SampledYOffset", default)]
    pub sampled_y_offset: Option<u32>,

    /// Table 8: AlphaTransparency — shall not be present
    #[serde(rename = "AlphaTransparency", default)]
    pub alpha_transparency: Option<String>,

    /// Table 8: ImageAlignmentOffset — shall not be present
    #[serde(rename = "ImageAlignmentOffset", default)]
    pub image_alignment_offset: Option<u32>,

    /// Table 8: ImageStartOffset — shall not be present
    #[serde(rename = "ImageStartOffset", default)]
    pub image_start_offset: Option<u32>,

    /// Table 8: ImageEndOffset — shall not be present
    #[serde(rename = "ImageEndOffset", default)]
    pub image_end_offset: Option<u32>,

    /// Table 8: FieldDominance — shall be present if interlaced, shall not be present if progressive
    #[serde(rename = "FieldDominance", default)]
    pub field_dominance: Option<u32>,

    /// Table 10: AlphaMaxRef — shall not be present
    #[serde(rename = "AlphaMaxRef", default)]
    pub alpha_max_ref: Option<u32>,

    /// Table 10: AlphaMinRef — shall not be present
    #[serde(rename = "AlphaMinRef", default)]
    pub alpha_min_ref: Option<u32>,

    /// Table 10: Palette — shall not be present
    #[serde(rename = "Palette", default)]
    pub palette: Option<String>,

    /// Table 10: PaletteLayout — shall not be present
    #[serde(rename = "PaletteLayout", default)]
    pub palette_layout: Option<String>,

    #[serde(rename = "LinkedTrackID", default)]
    pub linked_track_id: Option<u32>,

    #[serde(rename = "SubDescriptors", default)]
    pub sub_descriptors: Option<VideoSubDescriptors>,
}

/// CDCI video descriptor (YCbCr content)
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct CDCIDescriptor {
    #[serde(rename = "InstanceUID", alias = "InstanceID", default)]
    pub instance_id: Option<String>,

    #[serde(rename = "StoredWidth", default)]
    pub stored_width: Option<u32>,

    #[serde(rename = "StoredHeight", default)]
    pub stored_height: Option<u32>,

    #[serde(rename = "DisplayWidth", default)]
    pub display_width: Option<u32>,

    #[serde(rename = "DisplayHeight", default)]
    pub display_height: Option<u32>,

    #[serde(rename = "ActiveWidth", default)]
    pub active_width: Option<u32>,

    #[serde(rename = "ActiveHeight", default)]
    pub active_height: Option<u32>,

    #[serde(
        rename = "SampleRate",
        default,
        deserialize_with = "de_helpers::de_optional_edit_rate"
    )]
    pub sample_rate: Option<EditRate>,

    #[serde(rename = "ImageAspectRatio", default)]
    pub image_aspect_ratio: Option<String>,

    #[serde(
        rename = "ColorPrimaries",
        default,
        deserialize_with = "de_helpers::de_optional_color_primaries"
    )]
    pub color_primaries: Option<ColorPrimaries>,

    #[serde(
        rename = "TransferCharacteristic",
        default,
        deserialize_with = "de_helpers::de_optional_transfer_characteristic"
    )]
    pub transfer_characteristic: Option<TransferCharacteristic>,

    #[serde(
        rename = "CodingEquations",
        default,
        deserialize_with = "de_helpers::de_optional_coding_equations"
    )]
    pub coding_equations: Option<CodingEquations>,

    #[serde(
        rename = "PictureCompression",
        default,
        deserialize_with = "de_helpers::de_optional_video_codec"
    )]
    pub picture_compression: Option<VideoCodec>,

    #[serde(rename = "ComponentDepth", default)]
    pub component_depth: Option<u32>,

    /// Generic Picture Essence Descriptor: Frame Layout
    /// "FullFrame" (00h, progressive) or "SeparateFields" (01h, interlaced)
    #[serde(rename = "FrameLayout", default)]
    pub frame_layout: Option<String>,

    /// Generic Picture Essence Descriptor: DisplayF2Offset
    #[serde(rename = "DisplayF2Offset", default)]
    pub display_f2_offset: Option<i32>,

    /// CDCI Descriptor: Horizontal Subsampling (Table 12)
    /// 1 = 4:4:4, 2 = 4:2:2
    #[serde(rename = "HorizontalSubsampling", default)]
    pub horizontal_subsampling: Option<u32>,

    /// CDCI Descriptor: Vertical Subsampling (Table 12)
    /// Shall be 1
    #[serde(rename = "VerticalSubsampling", default)]
    pub vertical_subsampling: Option<u32>,

    /// CDCI Descriptor: Color Siting (Table 12)
    /// Shall be 0 (CoSiting) but some encoders write a label string (e.g. "CoSiting").
    #[serde(
        rename = "ColorSiting",
        default,
        deserialize_with = "de_helpers::de_optional_color_siting"
    )]
    pub color_siting: Option<u32>,

    /// CDCI Descriptor: Black Ref Level (Table 13)
    #[serde(rename = "BlackRefLevel", default)]
    pub black_ref_level: Option<u32>,

    /// CDCI Descriptor: White Ref Level (Table 13)
    #[serde(rename = "WhiteRefLevel", default)]
    pub white_ref_level: Option<u32>,

    /// CDCI Descriptor: Color Range (Table 13)
    #[serde(rename = "ColorRange", default)]
    pub color_range: Option<u32>,

    /// Table 8: StoredF2Offset — shall not be present
    #[serde(rename = "StoredF2Offset", default)]
    pub stored_f2_offset: Option<i32>,

    /// Table 8: SampledWidth — shall not be present or shall be equal to StoredWidth
    #[serde(rename = "SampledWidth", default)]
    pub sampled_width: Option<u32>,

    /// Table 8: SampledHeight — shall not be present or shall be equal to StoredHeight
    #[serde(rename = "SampledHeight", default)]
    pub sampled_height: Option<u32>,

    /// Table 8: SampledXOffset — shall not be present or shall be 0
    #[serde(rename = "SampledXOffset", default)]
    pub sampled_x_offset: Option<u32>,

    /// Table 8: SampledYOffset — shall not be present or shall be 0
    #[serde(rename = "SampledYOffset", default)]
    pub sampled_y_offset: Option<u32>,

    /// Table 8: AlphaTransparency — shall not be present
    #[serde(rename = "AlphaTransparency", default)]
    pub alpha_transparency: Option<String>,

    /// Table 8: ImageAlignmentOffset — shall not be present
    #[serde(rename = "ImageAlignmentOffset", default)]
    pub image_alignment_offset: Option<u32>,

    /// Table 8: ImageStartOffset — shall not be present
    #[serde(rename = "ImageStartOffset", default)]
    pub image_start_offset: Option<u32>,

    /// Table 8: ImageEndOffset — shall not be present
    #[serde(rename = "ImageEndOffset", default)]
    pub image_end_offset: Option<u32>,

    /// Table 8: FieldDominance — shall be present if interlaced, shall not be present if progressive
    #[serde(rename = "FieldDominance", default)]
    pub field_dominance: Option<u32>,

    /// Table 12: ReversedByteOrder — shall not be present
    #[serde(rename = "ReversedByteOrder", default)]
    pub reversed_byte_order: Option<String>,

    /// Table 12: PaddingBits — shall not be present
    #[serde(rename = "PaddingBits", default)]
    pub padding_bits: Option<i32>,

    /// Table 12: AlphaSampleDepth — shall not be present
    #[serde(rename = "AlphaSampleDepth", default)]
    pub alpha_sample_depth: Option<u32>,

    #[serde(rename = "LinkedTrackID", default)]
    pub linked_track_id: Option<u32>,

    #[serde(rename = "SubDescriptors", default)]
    pub sub_descriptors: Option<VideoSubDescriptors>,
}

/// SubDescriptors for video (RGBA/CDCI) descriptors
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct VideoSubDescriptors {
    /// Presence indicates Dolby Vision HDR
    #[serde(rename = "PHDRMetadataTrackSubDescriptor", default)]
    pub phdr_metadata_track_sub_descriptor: Option<PHDRMetadataTrackSubDescriptor>,

    /// JPEG 2000 Picture Sub Descriptor — Table 14 constraints
    #[serde(rename = "JPEG2000SubDescriptor", default)]
    pub jpeg2000_sub_descriptor: Option<JPEG2000SubDescriptor>,
}

/// JPEG 2000 Picture Sub Descriptor (ST 422 / ST 2067-21 Table 14)
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct JPEG2000SubDescriptor {
    #[serde(rename = "InstanceID", default)]
    pub instance_id: Option<String>,

    /// Decoder capabilities (Rsiz)
    #[serde(rename = "Rsiz", default)]
    pub rsiz: Option<u32>,

    /// Image width (Xsiz)
    #[serde(rename = "Xsiz", default)]
    pub xsiz: Option<u32>,

    /// Image height (Ysiz)
    #[serde(rename = "Ysiz", default)]
    pub ysiz: Option<u32>,

    /// Image X offset (XOsiz)
    #[serde(rename = "XOsiz", default)]
    pub xo_siz: Option<u32>,

    /// Image Y offset (YOsiz)
    #[serde(rename = "YOsiz", default)]
    pub yo_siz: Option<u32>,

    /// Tile width (XTsiz)
    #[serde(rename = "XTsiz", default)]
    pub xt_siz: Option<u32>,

    /// Tile height (YTsiz)
    #[serde(rename = "YTsiz", default)]
    pub yt_siz: Option<u32>,

    /// Tile X offset (XTOsiz)
    #[serde(rename = "XTOsiz", default)]
    pub xto_siz: Option<u32>,

    /// Tile Y offset (YTOsiz)
    #[serde(rename = "YTOsiz", default)]
    pub yto_siz: Option<u32>,

    /// Number of components (Csiz)
    #[serde(rename = "Csiz", default)]
    pub csiz: Option<u32>,

    /// Table 14: Coding Style — shall be present
    #[serde(rename = "CodingStyleDefault", default)]
    pub coding_style_default: Option<String>,

    /// Quantization Default
    #[serde(rename = "QuantizationDefault", default)]
    pub quantization_default: Option<String>,

    /// Table 14: J2CLayout — shall be present (§6.5.2)
    #[serde(rename = "J2CLayout", default)]
    pub j2c_layout: Option<J2CLayout>,

    /// Table 14: J2KExtendedCapabilities — shall be present if ISO/IEC 15444-15 coding
    #[serde(rename = "J2KExtendedCapabilities", default)]
    pub j2k_extended_capabilities: Option<J2KExtendedCapabilities>,

    /// Picture component sizing information
    #[serde(rename = "PictureComponentSizing", default)]
    pub picture_component_sizing: Option<PictureComponentSizing>,
}

/// J2CLayout — pixel component layout for JPEG 2000 (§6.5.2)
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct J2CLayout {
    #[serde(rename = "RGBAComponent", default)]
    pub components: Vec<RGBALayoutComponent>,
}

/// RGBA component entry within J2CLayout or PixelLayout
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct RGBALayoutComponent {
    /// Absent in intentionally malformed corpus files (e.g. RGBAError1).
    /// Default to empty string so the parser succeeds; the validator flags missing codes.
    #[serde(rename = "Code", default)]
    pub code: String,

    #[serde(rename = "ComponentSize", default)]
    pub component_size: u32,
}

/// J2K Extended Capabilities (ISO/IEC 15444-15)
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct J2KExtendedCapabilities {
    /// Profile capabilities (Pcap)
    #[serde(rename = "Pcap", default)]
    pub pcap: Option<u64>,
}

/// Picture component sizing information
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct PictureComponentSizing {
    #[serde(rename = "J2KComponentSizing", default)]
    pub components: Vec<J2KComponentSizing>,
}

/// Individual J2K component sizing
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct J2KComponentSizing {
    /// Component bit depth minus 1
    #[serde(rename = "Ssiz", default)]
    pub ssiz: Option<u32>,

    /// Horizontal separation of sample
    #[serde(rename = "XRSiz", default)]
    pub xr_siz: Option<u32>,

    /// Vertical separation of sample
    #[serde(rename = "YRSiz", default)]
    pub yr_siz: Option<u32>,
}

/// PHDR (Dolby Vision) metadata track sub-descriptor
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct PHDRMetadataTrackSubDescriptor {
    #[serde(rename = "InstanceID", default)]
    pub instance_id: Option<String>,

    #[serde(rename = "PHDRMetadataTrackSubDescriptor_DataDefinition", default)]
    pub data_definition: Option<String>,

    #[serde(rename = "PHDRMetadataTrackSubDescriptor_SimplePayloadSID", default)]
    pub simple_payload_sid: Option<u32>,

    #[serde(rename = "PHDRMetadataTrackSubDescriptor_SourceTrackID", default)]
    pub source_track_id: Option<u32>,
}

/// WAVE PCM audio descriptor
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct WAVEPCMDescriptor {
    #[serde(rename = "InstanceID", alias = "InstanceUID", default)]
    pub instance_id: Option<String>,

    #[serde(
        rename = "SampleRate",
        default,
        deserialize_with = "de_helpers::de_optional_edit_rate"
    )]
    pub sample_rate: Option<EditRate>,

    #[serde(
        rename = "AudioSampleRate",
        default,
        deserialize_with = "de_helpers::de_optional_edit_rate"
    )]
    pub audio_sample_rate: Option<EditRate>,

    #[serde(rename = "ChannelCount", default)]
    pub channel_count: Option<u32>,

    #[serde(rename = "QuantizationBits", default)]
    pub quantization_bits: Option<u32>,

    #[serde(rename = "LinkedTrackID", default)]
    pub linked_track_id: Option<u32>,

    #[serde(rename = "SubDescriptors", default)]
    pub sub_descriptors: Option<AudioSubDescriptors>,
}

/// SubDescriptors for audio (WAVEPCMDescriptor)
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct AudioSubDescriptors {
    #[serde(rename = "SoundfieldGroupLabelSubDescriptor", default)]
    pub soundfield_group_label_sub_descriptor: Option<SoundfieldGroupLabelSubDescriptor>,
}

/// Soundfield group label sub-descriptor — contains language and audio content kind
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct SoundfieldGroupLabelSubDescriptor {
    #[serde(
        rename = "MCATagSymbol",
        default,
        deserialize_with = "de_helpers::de_optional_mca_tag_symbol"
    )]
    pub mca_tag_symbol: Option<McaTagSymbol>,

    #[serde(rename = "MCATagName", default)]
    pub mca_tag_name: Option<String>,

    #[serde(rename = "MCAAudioContentKind", default)]
    pub mca_audio_content_kind: Option<String>,

    /// RFC 5646 language tag — field name varies between vendors
    #[serde(
        rename = "RFC5646SpokenLanguage",
        alias = "RFC5646AudioLanguageCode",
        default,
        deserialize_with = "de_helpers::de_optional_language_tag"
    )]
    pub rfc5646_spoken_language: Option<LanguageTag>,
}

/// DC Timed Text descriptor (subtitles/captions)
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct DCTimedTextDescriptor {
    #[serde(rename = "InstanceID", alias = "InstanceUID", default)]
    pub instance_id: Option<String>,

    #[serde(rename = "LinkedTrackID", default)]
    pub linked_track_id: Option<u32>,

    #[serde(
        rename = "SampleRate",
        default,
        deserialize_with = "de_helpers::de_optional_edit_rate"
    )]
    pub sample_rate: Option<EditRate>,

    /// Comma-separated RFC 5646 language tags for this timed text track
    #[serde(
        rename = "RFC5646LanguageTagList",
        default,
        deserialize_with = "de_helpers::de_language_tag_list"
    )]
    pub rfc5646_language_tag_list: Vec<LanguageTag>,

    #[serde(rename = "NamespaceURI", default)]
    pub namespace_uri: Option<String>,
}

/// IAB (Immersive Audio Bitstream) essence descriptor — Dolby Atmos
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct IABEssenceDescriptor {
    #[serde(rename = "InstanceID", alias = "InstanceUID", default)]
    pub instance_id: Option<String>,

    #[serde(rename = "LinkedTrackID", default)]
    pub linked_track_id: Option<u32>,

    #[serde(
        rename = "SampleRate",
        default,
        deserialize_with = "de_helpers::de_optional_edit_rate"
    )]
    pub sample_rate: Option<EditRate>,

    #[serde(
        rename = "AudioSampleRate",
        default,
        deserialize_with = "de_helpers::de_optional_edit_rate"
    )]
    pub audio_sample_rate: Option<EditRate>,

    #[serde(rename = "ChannelCount", default)]
    pub channel_count: Option<u32>,

    /// ST 2067-201 §5.9: QuantizationBits shall be 24.
    #[serde(rename = "QuantizationBits", default)]
    pub quantization_bits: Option<u32>,

    /// ST 2067-201 §5.3: ContainerFormat shall be the IAB essence container UL.
    #[serde(rename = "ContainerFormat", default)]
    pub container_format: Option<String>,

    #[serde(rename = "SoundCompression", default)]
    pub sound_compression: Option<String>,

    /// ST 2067-201 §5.9: Codec item shall NOT be present.
    #[serde(rename = "Codec", default)]
    pub codec: Option<String>,

    /// ST 2067-201 §5.9: ElectrospatialFormulation shall NOT be present.
    #[serde(rename = "ElectrospatialFormulation", default)]
    pub electrospatial_formulation: Option<u32>,

    #[serde(rename = "SubDescriptors", default)]
    pub sub_descriptors: Option<IABSubDescriptors>,
}

/// SubDescriptors for IAB essence
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct IABSubDescriptors {
    #[serde(rename = "IABSoundfieldLabelSubDescriptor", default)]
    pub iab_soundfield_label_sub_descriptor: Option<IABSoundfieldLabelSubDescriptor>,

    /// ST 2067-201:2026 Annex E — IAB Channel SubDescriptor entries.
    /// Optional in 2021 (and earlier — silently dropped), recommended
    /// in 2026 ("should contain one instance for each channel of each
    /// BedDefinition"). Captured as a raw count via a bag struct so
    /// downstream code can probe presence without the parser needing
    /// to model every field defined in Annex E Table E.1.
    #[serde(rename = "IABChannelSubDescriptor", default)]
    pub iab_channel_sub_descriptors: Vec<IABChannelSubDescriptor>,
}

/// Presence-only stub for ST 2067-201:2026 Annex E `IABChannelSubDescriptor`.
///
/// The 2026 spec defines the full item set in Table E.1
/// (`IABBedMetaID`, `IABChannelID`, `IABAudioDescription`,
/// `IABAudioDescriptionText`); imferno's CPL parser only needs to count
/// occurrences to fire the `IabChannelSubDescriptorRecommended` warning,
/// so the inner shape is intentionally permissive — any nested content
/// deserialises into the catch-all map without affecting presence.
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Default)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct IABChannelSubDescriptor {
    /// Annex E §E.2 — IAB Bed MetaID of the associated BedDefinition.
    #[serde(rename = "IABBedMetaID", default)]
    pub bed_meta_id: Option<u32>,
    /// Annex E §E.2 — Channel ID within the bed.
    #[serde(rename = "IABChannelID", default)]
    pub channel_id: Option<u32>,
}

/// IAB soundfield label sub-descriptor — contains language for Atmos tracks
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct IABSoundfieldLabelSubDescriptor {
    #[serde(rename = "InstanceID", alias = "InstanceUID", default)]
    pub instance_id: Option<String>,

    #[serde(
        rename = "MCATagSymbol",
        default,
        deserialize_with = "de_helpers::de_optional_mca_tag_symbol"
    )]
    pub mca_tag_symbol: Option<McaTagSymbol>,

    #[serde(rename = "MCATagName", default)]
    pub mca_tag_name: Option<String>,

    /// ST 2067-201 §5.9: MCALabelDictionaryID shall be `urn:smpte:ul:060e2b34.0401010d.03020221.00000000`.
    #[serde(rename = "MCALabelDictionaryID", default)]
    pub mca_label_dictionary_id: Option<String>,

    #[serde(
        rename = "RFC5646SpokenLanguage",
        alias = "RFC5646AudioLanguageCode",
        default,
        deserialize_with = "de_helpers::de_optional_language_tag"
    )]
    pub rfc5646_spoken_language: Option<LanguageTag>,
}

/// SubDescriptors for ISXD essence descriptor
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Default)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct IsxdSubDescriptors {
    /// ST 2067-202: ContainerConstraintsSubDescriptor shall be present.
    #[serde(rename = "ContainerConstraintsSubDescriptor", default)]
    pub container_constraints_sub_descriptor: Option<ContainerConstraintsSubDescriptor>,
}

/// ContainerConstraintsSubDescriptor — presence required by ST 2067-202 §5
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct ContainerConstraintsSubDescriptor {
    #[serde(rename = "InstanceID", alias = "InstanceUID", default)]
    pub instance_id: Option<String>,
}

/// ISXD (Immersive Sound XML Data) essence descriptor — Dolby Atmos sidecar format
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct ISXDDataEssenceDescriptor {
    #[serde(rename = "InstanceID", alias = "InstanceUID", default)]
    pub instance_id: Option<String>,

    #[serde(rename = "LinkedTrackID", default)]
    pub linked_track_id: Option<u32>,

    #[serde(
        rename = "SampleRate",
        default,
        deserialize_with = "de_helpers::de_optional_edit_rate"
    )]
    pub sample_rate: Option<EditRate>,

    #[serde(rename = "DataEssenceCoding", default)]
    pub data_essence_coding: Option<String>,

    #[serde(rename = "NamespaceURI", default)]
    pub namespace_uri: Option<String>,

    #[serde(rename = "SubDescriptors", default)]
    pub sub_descriptors: Option<IsxdSubDescriptors>,
}

// =============================================================================
// Root CPL structure
// =============================================================================

/// Root CPL structure — defines a complete IMF composition.
///
/// # Spec-required vs `Option<T>` policy (FIX-7 audit)
///
/// The parser is intentionally lenient: every spec-required element that
/// isn't strictly necessary to **construct** a valid `CompositionPlaylist`
/// is exposed as `Option<T>` (or via a `default = "…"` serde attribute).
/// Missing-required-field violations are surfaced as catalogue diagnostics
/// by the validator (`validate_cpl`) rather than as parse errors, so a
/// caller can still inspect the parsed structure of a non-conformant CPL.
///
/// Field-by-field map against ST 2067-3 §6 / §7 (2013 / 2016 — the 2020
/// edition reuses the 2016 schema verbatim):
///
/// | Field                     | Type             | Spec status                               |
/// |---------------------------|------------------|-------------------------------------------|
/// | `id`                      | `ImfUuid`        | required §6.1 — parse error if missing    |
/// | `annotation`              | `Option<…>`      | optional §6.2                             |
/// | `issue_date`              | `String`         | required §6.3 — parse error if missing    |
/// | `issuer`                  | `Option<…>`      | optional §6.4                             |
/// | `creator`                 | `Option<…>`      | optional §6.5                             |
/// | `content_originator`      | `Option<…>`      | optional §6.6                             |
/// | `content_title`           | `LanguageString` | required §6.7 — parse error if missing    |
/// | `content_kind`            | concrete (default) | required §6.8 — `default_content_kind`  |
/// | `content_version_list`    | `Option<…>`      | optional §6.10                            |
/// | `essence_descriptor_list` | `Option<…>`      | **required** per ST 2067-2 §6.1.5 —       |
/// |                           |                  | parser-lenient; absence is reported by    |
/// |                           |                  | `validate_cpl` as Error                   |
/// | `edit_rate`               | `Option<…>`      | required §6.13 — parser-lenient; absence  |
/// |                           |                  | reported by `validate_cpl`                |
/// | `total_running_time`      | `Option<String>` | optional §6.14                            |
/// | `locale_list`             | `Option<…>`      | optional §6.15                            |
/// | `extension_properties`    | `Option<…>`      | optional §6.16                            |
/// | `composition_timecode`    | `Option<…>`      | optional §6.9                             |
/// | `segment_list`            | `SegmentList`    | required §6.17 — parse error if missing   |
/// | `has_signer`/`has_signature` | `bool`        | reflect presence in raw XML (§8 signatures unparsed) |
/// | `source_xml`              | `Option<String>` | retained when parsed from XML; absent for JSON-deserialised |
///
/// Five fields are spec-required but stored as `Option<T>` (with `default`
/// on the serde side) to support the parser-lenient model:
/// `content_kind` (defaults via `default_content_kind`), `content_version_list`,
/// `essence_descriptor_list`, `edit_rate`, `locale_list`. The validator
/// surfaces missing-required findings against the ST 2067-3 prose.
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct CompositionPlaylist {
    /// The SMPTE spec version detected from the root xmlns of the CPL XML.
    /// Set after deserialization by `parse_cpl()`.
    #[serde(skip)]
    pub namespace: CplNamespace,

    #[cfg_attr(not(feature = "wasm"), serde(rename = "Id"))]
    #[cfg_attr(feature = "wasm", serde(rename = "id", alias = "Id"))]
    #[cfg_attr(feature = "typescript", ts(rename = "id"))]
    pub id: ImfUuid,

    #[cfg_attr(not(feature = "wasm"), serde(rename = "Annotation", default))]
    #[cfg_attr(
        feature = "wasm",
        serde(rename = "annotation", alias = "Annotation", default)
    )]
    #[cfg_attr(feature = "typescript", ts(rename = "annotation"))]
    pub annotation: Option<LanguageString>,

    #[cfg_attr(not(feature = "wasm"), serde(rename = "IssueDate"))]
    #[cfg_attr(feature = "wasm", serde(rename = "issueDate", alias = "IssueDate"))]
    #[cfg_attr(feature = "typescript", ts(rename = "issueDate"))]
    pub issue_date: String, // ISO 8601 datetime

    #[cfg_attr(not(feature = "wasm"), serde(rename = "Issuer", default))]
    #[cfg_attr(feature = "wasm", serde(rename = "issuer", alias = "Issuer", default))]
    #[cfg_attr(feature = "typescript", ts(rename = "issuer"))]
    pub issuer: Option<LanguageString>,

    #[cfg_attr(not(feature = "wasm"), serde(rename = "Creator", default))]
    #[cfg_attr(
        feature = "wasm",
        serde(rename = "creator", alias = "Creator", default)
    )]
    #[cfg_attr(feature = "typescript", ts(rename = "creator"))]
    pub creator: Option<LanguageString>,

    #[cfg_attr(not(feature = "wasm"), serde(rename = "ContentOriginator", default))]
    #[cfg_attr(
        feature = "wasm",
        serde(rename = "contentOriginator", alias = "ContentOriginator", default)
    )]
    #[cfg_attr(feature = "typescript", ts(rename = "contentOriginator"))]
    pub content_originator: Option<LanguageString>,

    #[cfg_attr(not(feature = "wasm"), serde(rename = "ContentTitle"))]
    #[cfg_attr(
        feature = "wasm",
        serde(rename = "contentTitle", alias = "ContentTitle")
    )]
    #[cfg_attr(feature = "typescript", ts(rename = "contentTitle"))]
    pub content_title: LanguageString,

    #[cfg_attr(
        not(feature = "wasm"),
        serde(rename = "ContentKind", default = "default_content_kind")
    )]
    #[cfg_attr(
        feature = "wasm",
        serde(
            rename = "contentKind",
            alias = "ContentKind",
            default = "default_content_kind"
        )
    )]
    #[cfg_attr(feature = "typescript", ts(rename = "contentKind"))]
    pub content_kind: ContentKindElement,

    #[cfg_attr(not(feature = "wasm"), serde(rename = "ContentVersionList", default))]
    #[cfg_attr(
        feature = "wasm",
        serde(rename = "contentVersionList", alias = "ContentVersionList", default)
    )]
    #[cfg_attr(feature = "typescript", ts(rename = "contentVersionList"))]
    pub content_version_list: Option<ContentVersionList>,

    #[cfg_attr(
        not(feature = "wasm"),
        serde(rename = "EssenceDescriptorList", default)
    )]
    #[cfg_attr(
        feature = "wasm",
        serde(
            rename = "essenceDescriptorList",
            alias = "EssenceDescriptorList",
            default
        )
    )]
    #[cfg_attr(feature = "typescript", ts(rename = "essenceDescriptorList"))]
    pub essence_descriptor_list: Option<EssenceDescriptorList>,

    #[cfg_attr(
        not(feature = "wasm"),
        serde(
            rename = "EditRate",
            default,
            deserialize_with = "de_helpers::de_optional_edit_rate"
        )
    )]
    #[cfg_attr(
        feature = "wasm",
        serde(
            rename = "editRate",
            alias = "EditRate",
            default,
            deserialize_with = "de_helpers::de_optional_edit_rate"
        )
    )]
    #[cfg_attr(feature = "typescript", ts(rename = "editRate"))]
    pub edit_rate: Option<EditRate>,

    #[cfg_attr(not(feature = "wasm"), serde(rename = "TotalRunningTime", default))]
    #[cfg_attr(
        feature = "wasm",
        serde(rename = "totalRunningTime", alias = "TotalRunningTime", default)
    )]
    #[cfg_attr(feature = "typescript", ts(rename = "totalRunningTime"))]
    pub total_running_time: Option<String>,

    #[cfg_attr(not(feature = "wasm"), serde(rename = "LocaleList", default))]
    #[cfg_attr(
        feature = "wasm",
        serde(rename = "localeList", alias = "LocaleList", default)
    )]
    #[cfg_attr(feature = "typescript", ts(rename = "localeList"))]
    pub locale_list: Option<LocaleList>,

    #[cfg_attr(not(feature = "wasm"), serde(rename = "ExtensionProperties", default))]
    #[cfg_attr(
        feature = "wasm",
        serde(rename = "extensionProperties", alias = "ExtensionProperties", default)
    )]
    #[cfg_attr(feature = "typescript", ts(rename = "extensionProperties"))]
    pub extension_properties: Option<ExtensionProperties>,

    #[cfg_attr(not(feature = "wasm"), serde(rename = "CompositionTimecode", default))]
    #[cfg_attr(
        feature = "wasm",
        serde(rename = "compositionTimecode", alias = "CompositionTimecode", default)
    )]
    #[cfg_attr(feature = "typescript", ts(rename = "compositionTimecode"))]
    pub composition_timecode: Option<CompositionTimecode>,

    /// Whether the original CPL XML contained a `<Signer>` element.
    /// Set by `parse_cpl()` from raw XML before namespace stripping.
    #[serde(skip)]
    pub has_signer: bool,

    /// Whether the original CPL XML contained a `<Signature>` element.
    /// Set by `parse_cpl()` from raw XML before namespace stripping.
    #[serde(skip)]
    pub has_signature: bool,

    /// The raw CPL XML as parsed, retained so callers running through
    /// `validate_cpl(&cpl)` can transparently invoke the runtime-XSD
    /// validator (which needs the unparsed source). Set by `parse_cpl()`.
    /// `None` when the struct was built via JSON deserialization or
    /// manual construction.
    #[serde(skip)]
    pub source_xml: Option<String>,

    #[cfg_attr(not(feature = "wasm"), serde(rename = "SegmentList"))]
    #[cfg_attr(feature = "wasm", serde(rename = "segmentList", alias = "SegmentList"))]
    #[cfg_attr(feature = "typescript", ts(rename = "segmentList"))]
    pub segment_list: SegmentList,
}

// =============================================================================
// CompositionTimecode
// =============================================================================

#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(not(feature = "wasm"), serde(rename_all = "PascalCase"))]
#[cfg_attr(feature = "wasm", serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct CompositionTimecode {
    #[cfg_attr(not(feature = "wasm"), serde(rename = "TimecodeDropFrame"))]
    #[cfg_attr(
        feature = "wasm",
        serde(rename = "timecodeDropFrame", alias = "TimecodeDropFrame")
    )]
    pub timecode_drop_frame: Option<bool>,

    #[cfg_attr(not(feature = "wasm"), serde(rename = "TimecodeRate"))]
    #[cfg_attr(
        feature = "wasm",
        serde(rename = "timecodeRate", alias = "TimecodeRate")
    )]
    pub timecode_rate: Option<u32>,

    #[cfg_attr(not(feature = "wasm"), serde(rename = "TimecodeStartAddress"))]
    #[cfg_attr(
        feature = "wasm",
        serde(rename = "timecodeStartAddress", alias = "TimecodeStartAddress")
    )]
    pub timecode_start_address: Option<String>,
}

// =============================================================================
// Content Version types
// =============================================================================

#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct ContentVersionList {
    #[cfg_attr(not(feature = "wasm"), serde(rename = "ContentVersion"))]
    #[cfg_attr(
        feature = "wasm",
        serde(rename = "contentVersions", alias = "ContentVersion")
    )]
    pub content_versions: Vec<ContentVersion>,
}

#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct ContentVersion {
    #[cfg_attr(not(feature = "wasm"), serde(rename = "Id"))]
    #[cfg_attr(feature = "wasm", serde(rename = "id", alias = "Id"))]
    pub id: String,

    #[cfg_attr(not(feature = "wasm"), serde(rename = "LabelText", default))]
    #[cfg_attr(
        feature = "wasm",
        serde(rename = "labelText", alias = "LabelText", default)
    )]
    pub label_text: Option<LanguageString>,
}

// =============================================================================
// Segment and Sequence types
// =============================================================================

#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct SegmentList {
    #[cfg_attr(not(feature = "wasm"), serde(rename = "Segment", default))]
    #[cfg_attr(
        feature = "wasm",
        serde(rename = "segments", alias = "Segment", default)
    )]
    pub segments: Vec<Segment>,
}

#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct Segment {
    #[cfg_attr(not(feature = "wasm"), serde(rename = "Id"))]
    #[cfg_attr(feature = "wasm", serde(rename = "id", alias = "Id"))]
    pub id: ImfUuid,

    #[cfg_attr(not(feature = "wasm"), serde(rename = "SequenceList"))]
    #[cfg_attr(
        feature = "wasm",
        serde(rename = "sequenceList", alias = "SequenceList")
    )]
    pub sequence_list: SequenceList,
}

#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct SequenceList {
    #[cfg_attr(not(feature = "wasm"), serde(rename = "MarkerSequence", default))]
    #[cfg_attr(
        feature = "wasm",
        serde(rename = "markerSequences", alias = "MarkerSequence", default)
    )]
    pub marker_sequences: Vec<MarkerSequence>,

    #[cfg_attr(not(feature = "wasm"), serde(rename = "MainImageSequence", default))]
    #[cfg_attr(
        feature = "wasm",
        serde(rename = "mainImageSequences", alias = "MainImageSequence", default)
    )]
    pub main_image_sequences: Vec<MainImageSequence>,

    #[cfg_attr(not(feature = "wasm"), serde(rename = "MainAudioSequence", default))]
    #[cfg_attr(
        feature = "wasm",
        serde(rename = "mainAudioSequences", alias = "MainAudioSequence", default)
    )]
    pub main_audio_sequences: Vec<MainAudioSequence>,

    #[cfg_attr(
        not(feature = "wasm"),
        serde(rename = "SubtitlesSequence", alias = "MainSubtitleSequence", default)
    )]
    #[cfg_attr(
        feature = "wasm",
        serde(
            rename = "subtitlesSequences",
            alias = "SubtitlesSequence",
            alias = "MainSubtitleSequence",
            default
        )
    )]
    pub subtitles_sequences: Vec<SubtitlesSequence>,

    #[cfg_attr(
        not(feature = "wasm"),
        serde(rename = "HearingImpairedCaptionsSequence", default)
    )]
    #[cfg_attr(
        feature = "wasm",
        serde(
            rename = "hearingImpairedCaptionsSequences",
            alias = "HearingImpairedCaptionsSequence",
            default
        )
    )]
    pub hearing_impaired_captions_sequences: Vec<HearingImpairedCaptionsSequence>,

    #[cfg_attr(
        not(feature = "wasm"),
        serde(rename = "ForcedNarrativeSequence", default)
    )]
    #[cfg_attr(
        feature = "wasm",
        serde(
            rename = "forcedNarrativeSequences",
            alias = "ForcedNarrativeSequence",
            default
        )
    )]
    pub forced_narrative_sequences: Vec<ForcedNarrativeSequence>,

    #[cfg_attr(not(feature = "wasm"), serde(rename = "IABSequence", default))]
    #[cfg_attr(
        feature = "wasm",
        serde(rename = "iabSequences", alias = "IABSequence", default)
    )]
    pub iab_sequences: Vec<IABSequence>,

    #[cfg_attr(not(feature = "wasm"), serde(rename = "ISXDSequence", default))]
    #[cfg_attr(
        feature = "wasm",
        serde(rename = "isxdSequences", alias = "ISXDSequence", default)
    )]
    pub isxd_sequences: Vec<ISXDSequence>,
}

impl SequenceList {
    /// Return all non-marker sequences as trait objects.
    pub fn all_sequences(&self) -> Vec<&dyn SequenceAccess> {
        let mut v: Vec<&dyn SequenceAccess> = Vec::new();
        for s in &self.main_image_sequences {
            v.push(s);
        }
        for s in &self.main_audio_sequences {
            v.push(s);
        }
        for s in &self.subtitles_sequences {
            v.push(s);
        }
        for s in &self.hearing_impaired_captions_sequences {
            v.push(s);
        }
        for s in &self.forced_narrative_sequences {
            v.push(s);
        }
        for s in &self.iab_sequences {
            v.push(s);
        }
        for s in &self.isxd_sequences {
            v.push(s);
        }
        v
    }

    /// Return all non-marker sequences paired with their type name.
    pub fn all_sequences_typed(&self) -> Vec<(&dyn SequenceAccess, &'static str)> {
        let mut v: Vec<(&dyn SequenceAccess, &'static str)> = Vec::new();
        for s in &self.main_image_sequences {
            v.push((s, "MainImage"));
        }
        for s in &self.main_audio_sequences {
            v.push((s, "MainAudio"));
        }
        for s in &self.subtitles_sequences {
            v.push((s, "Subtitles"));
        }
        for s in &self.hearing_impaired_captions_sequences {
            v.push((s, "HearingImpairedCaptions"));
        }
        for s in &self.forced_narrative_sequences {
            v.push((s, "ForcedNarrative"));
        }
        for s in &self.iab_sequences {
            v.push((s, "IAB"));
        }
        for s in &self.isxd_sequences {
            v.push((s, "ISXD"));
        }
        v
    }
}

// All sequence types share the same structure: Id, TrackId, ResourceList
/// Trait for accessing common sequence fields
pub trait SequenceAccess {
    fn id(&self) -> &ImfUuid;
    fn track_id(&self) -> &ImfUuid;
    fn resource_list(&self) -> &ResourceList;
}

macro_rules! define_sequence_type {
    ($name:ident) => {
        #[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        #[cfg_attr(feature = "typescript", derive(TS))]
        #[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
        #[cfg_attr(feature = "wasm", derive(Tsify))]
        #[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
        pub struct $name {
            #[cfg_attr(not(feature = "wasm"), serde(rename = "Id"))]
            #[cfg_attr(feature = "wasm", serde(rename = "id", alias = "Id"))]
            pub id: ImfUuid,

            #[cfg_attr(not(feature = "wasm"), serde(rename = "TrackId"))]
            #[cfg_attr(feature = "wasm", serde(rename = "trackId", alias = "TrackId"))]
            pub track_id: ImfUuid,

            #[cfg_attr(not(feature = "wasm"), serde(rename = "ResourceList"))]
            #[cfg_attr(
                feature = "wasm",
                serde(rename = "resourceList", alias = "ResourceList")
            )]
            pub resource_list: ResourceList,
        }

        impl SequenceAccess for $name {
            fn id(&self) -> &ImfUuid {
                &self.id
            }
            fn track_id(&self) -> &ImfUuid {
                &self.track_id
            }
            fn resource_list(&self) -> &ResourceList {
                &self.resource_list
            }
        }
    };
}

define_sequence_type!(MarkerSequence);
define_sequence_type!(MainImageSequence);
define_sequence_type!(MainAudioSequence);
define_sequence_type!(SubtitlesSequence);
define_sequence_type!(HearingImpairedCaptionsSequence);
define_sequence_type!(ForcedNarrativeSequence);
define_sequence_type!(IABSequence);
define_sequence_type!(ISXDSequence);

// =============================================================================
// Resource types
// =============================================================================

#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct ResourceList {
    #[cfg_attr(not(feature = "wasm"), serde(rename = "Resource", default))]
    #[cfg_attr(
        feature = "wasm",
        serde(rename = "resources", alias = "Resource", default)
    )]
    pub resources: Vec<Resource>,
}

#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct Resource {
    #[cfg_attr(not(feature = "wasm"), serde(rename = "Id"))]
    #[cfg_attr(feature = "wasm", serde(rename = "id", alias = "Id"))]
    pub id: ImfUuid,

    #[cfg_attr(not(feature = "wasm"), serde(rename = "Annotation", default))]
    #[cfg_attr(
        feature = "wasm",
        serde(rename = "annotation", alias = "Annotation", default)
    )]
    pub annotation: Option<LanguageString>,

    #[cfg_attr(
        not(feature = "wasm"),
        serde(
            rename = "EditRate",
            default,
            deserialize_with = "de_helpers::de_optional_edit_rate"
        )
    )]
    #[cfg_attr(
        feature = "wasm",
        serde(
            rename = "editRate",
            alias = "EditRate",
            default,
            deserialize_with = "de_helpers::de_optional_edit_rate"
        )
    )]
    pub edit_rate: Option<EditRate>,

    #[cfg_attr(not(feature = "wasm"), serde(rename = "IntrinsicDuration"))]
    #[cfg_attr(
        feature = "wasm",
        serde(rename = "intrinsicDuration", alias = "IntrinsicDuration")
    )]
    pub intrinsic_duration: u64,

    #[cfg_attr(not(feature = "wasm"), serde(rename = "EntryPoint", default))]
    #[cfg_attr(
        feature = "wasm",
        serde(rename = "entryPoint", alias = "EntryPoint", default)
    )]
    pub entry_point: Option<u64>,

    #[cfg_attr(not(feature = "wasm"), serde(rename = "SourceDuration", default))]
    #[cfg_attr(
        feature = "wasm",
        serde(rename = "sourceDuration", alias = "SourceDuration", default)
    )]
    pub source_duration: Option<u64>,

    #[cfg_attr(not(feature = "wasm"), serde(rename = "SourceEncoding", default))]
    #[cfg_attr(
        feature = "wasm",
        serde(rename = "sourceEncoding", alias = "SourceEncoding", default)
    )]
    pub source_encoding: Option<ImfUuid>, // UUID reference to EssenceDescriptor

    #[cfg_attr(not(feature = "wasm"), serde(rename = "TrackFileId", default))]
    #[cfg_attr(
        feature = "wasm",
        serde(rename = "trackFileId", alias = "TrackFileId", default)
    )]
    pub track_file_id: Option<ImfUuid>, // UUID reference to MXF file in AssetMap

    #[cfg_attr(not(feature = "wasm"), serde(rename = "RepeatCount", default))]
    #[cfg_attr(
        feature = "wasm",
        serde(rename = "repeatCount", alias = "RepeatCount", default)
    )]
    pub repeat_count: Option<u64>,

    #[cfg_attr(not(feature = "wasm"), serde(rename = "KeyId", default))]
    #[cfg_attr(feature = "wasm", serde(rename = "keyId", alias = "KeyId", default))]
    pub key_id: Option<ImfUuid>, // UUID reference to encryption key

    #[cfg_attr(not(feature = "wasm"), serde(rename = "Hash", default))]
    #[cfg_attr(feature = "wasm", serde(rename = "hash", alias = "Hash", default))]
    pub hash: Option<String>,

    #[cfg_attr(not(feature = "wasm"), serde(rename = "Marker", default))]
    #[cfg_attr(feature = "wasm", serde(rename = "markers", alias = "Marker", default))]
    pub markers: Vec<MarkerInfo>,
}

#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct MarkerInfo {
    #[cfg_attr(not(feature = "wasm"), serde(rename = "Annotation", default))]
    #[cfg_attr(
        feature = "wasm",
        serde(rename = "annotation", alias = "Annotation", default)
    )]
    pub annotation: Option<String>,

    #[cfg_attr(not(feature = "wasm"), serde(rename = "Label"))]
    #[cfg_attr(feature = "wasm", serde(rename = "label", alias = "Label"))]
    pub label: MarkerLabelElement,

    #[cfg_attr(not(feature = "wasm"), serde(rename = "Offset"))]
    #[cfg_attr(feature = "wasm", serde(rename = "offset", alias = "Offset"))]
    pub offset: u64,
}

// =============================================================================
// Track information (legacy, kept for backward compatibility)
// =============================================================================

/// Track information with codec details (legacy — use EssenceDescriptor parsing instead)
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct TrackInfo {
    pub track_id: String,
    pub track_type: String, // "video", "audio", "subtitle"
    pub codec: String,
    pub language: Option<String>,
    pub channels: Option<String>,
    pub format_details: Option<String>,
    pub resolution: Option<String>,
    pub framerate: Option<String>,
    pub bit_depth: Option<String>,
    pub subtitle_type: Option<String>,
}

// =============================================================================
// Parser functions
// =============================================================================

/// Parse CPL XML content with namespace stripping.
///
/// Detects the SMPTE spec version from the root `xmlns` attribute and stores it
/// on the returned `CompositionPlaylist.namespace` field. This enables downstream
/// code to apply version-specific validation rules.
pub fn parse_cpl(xml_content: &str) -> Result<CompositionPlaylist, CplParseError> {
    parse_cpl_with_options(xml_content, &CplParseOptions::default())
}

/// Parse CPL XML content with configurable hardening options.
pub fn parse_cpl_with_options(
    xml_content: &str,
    options: &CplParseOptions<'_>,
) -> Result<CompositionPlaylist, CplParseError> {
    // Detect namespace before stripping (stripping preserves default xmlns but
    // removes prefixed xmlns:xxx declarations). A document with no detectable
    // root xmlns falls into `Unknown(String::new())` so downstream validators
    // see "namespace unknown" instead of silently defaulting to the 2013
    // ruleset (the first enum variant).
    let namespace = crate::assetmap::detect_root_namespace(xml_content)
        .map(|uri| CplNamespace::from_uri(&uri))
        .unwrap_or_else(|| CplNamespace::Unknown(String::new()));

    // Detect Signer/Signature presence from raw XML before stripping
    let has_signer = xml_content.contains("<Signer") || xml_content.contains(":Signer");
    let has_signature = xml_content.contains("<Signature") || xml_content.contains(":Signature");

    match options.signature_validation_mode {
        SignatureValidationMode::Ignore => {}
        SignatureValidationMode::RequirePresence => {
            if !has_signature {
                return Err(CplParseError::StrictSchema(
                    "Signature element is required by selected signature mode".to_string(),
                ));
            }
        }
        SignatureValidationMode::VerifyIfPresent => {
            if has_signature {
                let verifier = options
                    .signature_verifier
                    .ok_or(CplParseError::SignatureVerifierRequired)?;
                verifier
                    .verify(xml_content)
                    .map_err(CplParseError::SignatureVerificationFailed)?;
            }
        }
        SignatureValidationMode::RequireValid => {
            if !has_signature {
                return Err(CplParseError::StrictSchema(
                    "Signature element is required by selected signature mode".to_string(),
                ));
            }
            let verifier = options
                .signature_verifier
                .ok_or(CplParseError::SignatureVerifierRequired)?;
            verifier
                .verify(xml_content)
                .map_err(CplParseError::SignatureVerificationFailed)?;
        }
    }

    let stripped = strip_xml_namespaces(xml_content);

    if options.unknown_field_mode == UnknownFieldMode::Error {
        let unknown = collect_unknown_xml_tokens(&stripped).map_err(|e| {
            CplParseError::StrictUnknownXml(format!("unknown token scan failed: {}", e))
        })?;
        if !unknown.is_empty() {
            let list = unknown.into_iter().collect::<Vec<_>>().join(", ");
            return Err(CplParseError::StrictUnknownXml(list));
        }
    }

    let mut cpl: CompositionPlaylist = quick_xml::de::from_str(&stripped)?;

    if options.schema_strict_mode == SchemaStrictMode::Basic {
        validate_basic_schema_constraints(&cpl)?;
    }

    cpl.namespace = namespace;
    cpl.has_signer = has_signer;
    cpl.has_signature = has_signature;
    cpl.source_xml = Some(xml_content.to_string());
    Ok(cpl)
}

fn validate_basic_schema_constraints(cpl: &CompositionPlaylist) -> Result<(), CplParseError> {
    if cpl.segment_list.segments.is_empty() {
        return Err(CplParseError::StrictSchema(
            "SegmentList must contain at least one Segment".to_string(),
        ));
    }

    for (segment_index, segment) in cpl.segment_list.segments.iter().enumerate() {
        let sequence_count = segment.sequence_list.marker_sequences.len()
            + segment.sequence_list.main_image_sequences.len()
            + segment.sequence_list.main_audio_sequences.len()
            + segment.sequence_list.subtitles_sequences.len()
            + segment
                .sequence_list
                .hearing_impaired_captions_sequences
                .len()
            + segment.sequence_list.forced_narrative_sequences.len()
            + segment.sequence_list.iab_sequences.len()
            + segment.sequence_list.isxd_sequences.len();

        if sequence_count == 0 {
            return Err(CplParseError::StrictSchema(format!(
                "Segment[{}] must contain at least one sequence",
                segment_index
            )));
        }
    }

    Ok(())
}

fn collect_unknown_xml_tokens(xml: &str) -> Result<BTreeSet<String>, String> {
    let mut reader = quick_xml::Reader::from_str(xml);
    reader.trim_text(true);

    let allowed_elements: BTreeSet<&'static str> = [
        "CompositionPlaylist",
        "Id",
        "Annotation",
        "IssueDate",
        "Issuer",
        "Creator",
        "ContentOriginator",
        "ContentTitle",
        "ContentKind",
        "ContentVersionList",
        "ContentVersion",
        "LabelText",
        "EssenceDescriptorList",
        "EssenceDescriptor",
        "EditRate",
        "TotalRunningTime",
        "LocaleList",
        "Locale",
        "LanguageList",
        "Language",
        "RegionList",
        "Region",
        "ContentMaturityRatingList",
        "ContentMaturityRating",
        "Agency",
        "Rating",
        "Audience",
        "ExtensionProperties",
        "ApplicationIdentification",
        "MaxCLL",
        "MaxFALL",
        "CompositionTimecode",
        "TimecodeDropFrame",
        "TimecodeRate",
        "TimecodeStartAddress",
        "SegmentList",
        "Segment",
        "SequenceList",
        "MarkerSequence",
        "MainImageSequence",
        "MainAudioSequence",
        "SubtitlesSequence",
        "MainSubtitleSequence",
        "HearingImpairedCaptionsSequence",
        "ForcedNarrativeSequence",
        "IABSequence",
        "ISXDSequence",
        "TrackId",
        "ResourceList",
        "Resource",
        "IntrinsicDuration",
        "EntryPoint",
        "SourceDuration",
        "SourceEncoding",
        "TrackFileId",
        "RepeatCount",
        "KeyId",
        "Hash",
        "Marker",
        "Label",
        "Offset",
        "RGBADescriptor",
        "CDCIDescriptor",
        "WAVEPCMDescriptor",
        "DCTimedTextDescriptor",
        "IABEssenceDescriptor",
        "ISXDDataEssenceDescriptor",
        "InstanceID",
        "InstanceUID",
        "DisplayWidth",
        "DisplayHeight",
        "StoredWidth",
        "StoredHeight",
        "SampleRate",
        "ImageAspectRatio",
        "ColorPrimaries",
        "TransferCharacteristic",
        "CodingEquations",
        "PictureCompression",
        "FrameLayout",
        "DisplayF2Offset",
        "ComponentMaxRef",
        "ComponentMinRef",
        "ScanningDirection",
        "StoredF2Offset",
        "SampledWidth",
        "SampledHeight",
        "SampledXOffset",
        "SampledYOffset",
        "AlphaTransparency",
        "ImageAlignmentOffset",
        "ImageStartOffset",
        "ImageEndOffset",
        "FieldDominance",
        "AlphaMaxRef",
        "AlphaMinRef",
        "Palette",
        "PaletteLayout",
        "LinkedTrackID",
        "SubDescriptors",
        "ActiveWidth",
        "ActiveHeight",
        "ComponentDepth",
        "HorizontalSubsampling",
        "VerticalSubsampling",
        "ColorSiting",
        "BlackRefLevel",
        "WhiteRefLevel",
        "ColorRange",
        "ReversedByteOrder",
        "PaddingBits",
        "AlphaSampleDepth",
        "PHDRMetadataTrackSubDescriptor",
        "JPEG2000SubDescriptor",
        "Rsiz",
        "Xsiz",
        "Ysiz",
        "XOsiz",
        "YOsiz",
        "XTsiz",
        "YTsiz",
        "XTOsiz",
        "YTOsiz",
        "Csiz",
        "CodingStyleDefault",
        "QuantizationDefault",
        "J2CLayout",
        "J2KExtendedCapabilities",
        "PictureComponentSizing",
        "RGBAComponent",
        "Code",
        "ComponentSize",
        "Pcap",
        "J2KComponentSizing",
        "Ssiz",
        "XRSiz",
        "YRSiz",
        "PHDRMetadataTrackSubDescriptor_DataDefinition",
        "PHDRMetadataTrackSubDescriptor_SimplePayloadSID",
        "PHDRMetadataTrackSubDescriptor_SourceTrackID",
        "AudioSampleRate",
        "ChannelCount",
        "QuantizationBits",
        "SoundfieldGroupLabelSubDescriptor",
        "MCATagSymbol",
        "MCATagName",
        "MCAAudioContentKind",
        "RFC5646SpokenLanguage",
        "RFC5646AudioLanguageCode",
        "RFC5646LanguageTagList",
        "NamespaceURI",
        "SoundCompression",
        "IABSoundfieldLabelSubDescriptor",
        "ContainerFormat",
        "Codec",
        "ElectrospatialFormulation",
        "MCALabelDictionaryID",
        "EssenceLength",
        "Locked",
        "MCALinkID",
        "MCAChannelID",
        "AudioChannelLabelSubDescriptor",
        "MCATitle",
        "MCATitleVersion",
        "MCAAudioElementKind",
        "SoundfieldGroupLinkID",
        "DataEssenceCoding",
        "ContainerConstraintsSubDescriptor",
        "Signer",
        "Signature",
    ]
    .into_iter()
    .collect();

    let allowed_attributes: BTreeSet<&'static str> =
        ["xmlns", "scope", "language"].into_iter().collect();

    let mut unknown = BTreeSet::new();

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                let name = std::str::from_utf8(e.name().as_ref())
                    .map_err(|e| e.to_string())?
                    .to_string();
                if !allowed_elements.contains(name.as_str()) {
                    unknown.insert(format!("element:{}", name));
                }
                for attr in e.attributes() {
                    let attr = attr.map_err(|e| e.to_string())?;
                    let key = std::str::from_utf8(attr.key.as_ref())
                        .map_err(|e| e.to_string())?
                        .to_string();
                    if !(allowed_attributes.contains(key.as_str()) || key.starts_with("xmlns:")) {
                        unknown.insert(format!("attribute:{}@{}", key, name));
                    }
                }
            }
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(e) => return Err(e.to_string()),
        }
    }

    Ok(unknown)
}

/// Extract all languages found in a CPL
pub fn extract_cpl_languages(cpl: &CompositionPlaylist) -> Vec<LanguageTag> {
    let mut languages: Vec<LanguageTag> = Vec::new();

    let add_lang = |languages: &mut Vec<LanguageTag>, lang_opt: &Option<LanguageTag>| {
        if let Some(lang) = lang_opt {
            if !lang.as_str().is_empty() && !languages.contains(lang) {
                languages.push(lang.clone());
            }
        }
    };

    let add_lang_string = |languages: &mut Vec<LanguageTag>,
                           lang_string: &Option<LanguageString>| {
        if let Some(ls) = lang_string {
            add_lang(languages, &ls.language);
        }
    };

    let add_required_lang_string =
        |languages: &mut Vec<LanguageTag>, lang_string: &LanguageString| {
            add_lang(languages, &lang_string.language);
        };

    // Extract from main CPL fields
    add_lang_string(&mut languages, &cpl.annotation);
    add_lang_string(&mut languages, &cpl.issuer);
    add_lang_string(&mut languages, &cpl.creator);
    add_lang_string(&mut languages, &cpl.content_originator);
    add_required_lang_string(&mut languages, &cpl.content_title);

    // Extract from content versions
    if let Some(content_version_list) = &cpl.content_version_list {
        for version in &content_version_list.content_versions {
            add_lang_string(&mut languages, &version.label_text);
        }
    }

    // Extract from LocaleList
    if let Some(locale_list) = &cpl.locale_list {
        for locale in &locale_list.locales {
            if let Some(language_list) = &locale.language_list {
                for lang in &language_list.languages {
                    if !lang.as_str().is_empty() && !languages.contains(lang) {
                        languages.push(lang.clone());
                    }
                }
            }
        }
    }

    // Extract from EssenceDescriptors
    if let Some(edl) = &cpl.essence_descriptor_list {
        for ed in &edl.essence_descriptors {
            // Audio language from WAVEPCMDescriptor
            if let Some(wave) = &ed.wave_pcm_descriptor {
                if let Some(subs) = &wave.sub_descriptors {
                    if let Some(sf) = &subs.soundfield_group_label_sub_descriptor {
                        add_lang(&mut languages, &sf.rfc5646_spoken_language);
                    }
                }
            }
            // Audio language from IABEssenceDescriptor
            if let Some(iab) = &ed.iab_essence_descriptor {
                if let Some(subs) = &iab.sub_descriptors {
                    if let Some(sf) = &subs.iab_soundfield_label_sub_descriptor {
                        add_lang(&mut languages, &sf.rfc5646_spoken_language);
                    }
                }
            }
            // Timed text language from DCTimedTextDescriptor
            if let Some(tt) = &ed.dc_timed_text_descriptor {
                for lang in &tt.rfc5646_language_tag_list {
                    if !lang.as_str().is_empty() && !languages.contains(lang) {
                        languages.push(lang.clone());
                    }
                }
            }
        }
    }

    languages.sort_by(|a, b| a.as_str().cmp(b.as_str()));
    languages.dedup();
    languages
}

/// Extract track-level codec information from CPL XML, returning an error on parse failure.
pub fn try_extract_cpl_track_codecs_from_xml(
    xml_content: &str,
) -> Result<Vec<TrackInfo>, CplParseError> {
    let cpl = parse_cpl(xml_content)?;
    Ok(extract_tracks_from_cpl(&cpl, xml_content))
}

/// Extract track-level codec information from CPL XML content.
///
/// Returns an empty `Vec` if the CPL fails to parse. Prefer
/// [`try_extract_cpl_track_codecs_from_xml`] to distinguish parse failure from empty tracks.
pub fn extract_cpl_track_codecs_from_xml(xml_content: &str) -> Vec<TrackInfo> {
    try_extract_cpl_track_codecs_from_xml(xml_content).unwrap_or_default()
}

/// Extract track info from a properly parsed CPL (replaces regex-based extraction)
fn extract_tracks_from_cpl(cpl: &CompositionPlaylist, _raw_xml: &str) -> Vec<TrackInfo> {
    let mut tracks = Vec::new();

    // Build essence descriptor lookup by ID
    let descriptors: std::collections::HashMap<ImfUuid, &EssenceDescriptor> =
        if let Some(edl) = &cpl.essence_descriptor_list {
            edl.essence_descriptors
                .iter()
                .map(|ed| (ed.id, ed))
                .collect()
        } else {
            std::collections::HashMap::new()
        };

    for segment in &cpl.segment_list.segments {
        let seq_list = &segment.sequence_list;

        // Video tracks from MainImageSequence
        for seq in &seq_list.main_image_sequences {
            for resource in &seq.resource_list.resources {
                if let Some(source_encoding) = &resource.source_encoding {
                    if let Some(ed) = descriptors.get(source_encoding) {
                        let (codec, resolution, bit_depth) = extract_video_info_from_descriptor(ed);
                        let framerate = resource
                            .edit_rate
                            .as_ref()
                            .or(cpl.edit_rate.as_ref())
                            .map(format_framerate);
                        tracks.push(TrackInfo {
                            track_id: seq.track_id.to_string(),
                            track_type: "video".to_string(),
                            codec,
                            language: None,
                            channels: None,
                            format_details: None,
                            resolution,
                            framerate,
                            bit_depth,
                            subtitle_type: None,
                        });
                    }
                }
            }
        }

        // Audio tracks from MainAudioSequence
        for seq in &seq_list.main_audio_sequences {
            for resource in &seq.resource_list.resources {
                if let Some(source_encoding) = &resource.source_encoding {
                    if let Some(ed) = descriptors.get(source_encoding) {
                        let (codec, channels, format_details, language) =
                            extract_audio_info_from_descriptor(ed);
                        tracks.push(TrackInfo {
                            track_id: seq.track_id.to_string(),
                            track_type: "audio".to_string(),
                            codec,
                            language,
                            channels,
                            format_details,
                            resolution: None,
                            framerate: None,
                            bit_depth: None,
                            subtitle_type: None,
                        });
                    }
                }
            }
        }

        // IAB (Atmos) tracks
        for seq in &seq_list.iab_sequences {
            for resource in &seq.resource_list.resources {
                if let Some(source_encoding) = &resource.source_encoding {
                    if let Some(ed) = descriptors.get(source_encoding) {
                        let language = ed
                            .iab_essence_descriptor
                            .as_ref()
                            .and_then(|iab| iab.sub_descriptors.as_ref())
                            .and_then(|sd| sd.iab_soundfield_label_sub_descriptor.as_ref())
                            .and_then(|sf| sf.rfc5646_spoken_language.as_ref())
                            .map(|lt| lt.as_str().to_string());
                        tracks.push(TrackInfo {
                            track_id: seq.track_id.to_string(),
                            track_type: "audio".to_string(),
                            codec: "IAB (Dolby Atmos)".to_string(),
                            language,
                            channels: Some("Object-based".to_string()),
                            format_details: Some("Immersive Audio".to_string()),
                            resolution: None,
                            framerate: None,
                            bit_depth: None,
                            subtitle_type: None,
                        });
                    }
                }
            }
        }

        // Subtitle tracks
        let subtitle_sequences: Vec<(&str, &[SubtitlesSequence])> = vec![
            // We need to handle each type separately due to different types
        ];
        let _ = subtitle_sequences; // suppress warning

        for seq in &seq_list.subtitles_sequences {
            if let Some(track) =
                extract_timed_text_track(seq.track_id, "standard", &seq.resource_list, &descriptors)
            {
                tracks.push(track);
            }
        }
        for seq in &seq_list.hearing_impaired_captions_sequences {
            if let Some(track) =
                extract_timed_text_track(seq.track_id, "hi", &seq.resource_list, &descriptors)
            {
                tracks.push(track);
            }
        }
        for seq in &seq_list.forced_narrative_sequences {
            if let Some(track) =
                extract_timed_text_track(seq.track_id, "forced", &seq.resource_list, &descriptors)
            {
                tracks.push(track);
            }
        }
    }

    tracks
}

fn extract_video_info_from_descriptor(
    ed: &EssenceDescriptor,
) -> (String, Option<String>, Option<String>) {
    if let Some(rgba) = &ed.rgba_descriptor {
        let width = rgba.display_width.or(rgba.stored_width);
        let height = rgba.display_height.or(rgba.stored_height);
        let resolution = match (width, height) {
            (Some(w), Some(h)) => Some(format!("{}x{}", w, h)),
            _ => None,
        };
        let codec = rgba
            .picture_compression
            .as_ref()
            .map(|c| c.to_string())
            .unwrap_or_else(|| "JPEG 2000".to_string());
        return (codec, resolution, None);
    }
    if let Some(cdci) = &ed.cdci_descriptor {
        let width = cdci
            .active_width
            .or(cdci.display_width)
            .or(cdci.stored_width);
        let height = cdci
            .active_height
            .or(cdci.display_height)
            .or(cdci.stored_height);
        let resolution = match (width, height) {
            (Some(w), Some(h)) => Some(format!("{}x{}", w, h)),
            _ => None,
        };
        let codec = cdci
            .picture_compression
            .as_ref()
            .map(|c| c.to_string())
            .unwrap_or_else(|| "JPEG 2000".to_string());
        let bit_depth = cdci.component_depth.map(|d| format!("{}-bit", d));
        return (codec, resolution, bit_depth);
    }
    ("Unknown".to_string(), None, None)
}

fn extract_audio_info_from_descriptor(
    ed: &EssenceDescriptor,
) -> (String, Option<String>, Option<String>, Option<String>) {
    if let Some(wave) = &ed.wave_pcm_descriptor {
        let codec = wave
            .quantization_bits
            .map(|b| format!("PCM {}-bit", b))
            .unwrap_or_else(|| "PCM".to_string());
        let (channels, format_details) = match wave.channel_count {
            Some(1) => (Some("1.0".to_string()), Some("Mono".to_string())),
            Some(2) => (Some("2.0".to_string()), Some("Stereo".to_string())),
            Some(6) => (Some("5.1".to_string()), Some("Surround".to_string())),
            Some(8) => (Some("7.1".to_string()), Some("Surround".to_string())),
            Some(n) => (Some(format!("{}.0", n)), Some(format!("{} Channel", n))),
            None => (None, None),
        };
        let language = wave
            .sub_descriptors
            .as_ref()
            .and_then(|sd| sd.soundfield_group_label_sub_descriptor.as_ref())
            .and_then(|sf| sf.rfc5646_spoken_language.as_ref())
            .map(|lt| lt.as_str().to_string());
        return (codec, channels, format_details, language);
    }
    ("Unknown".to_string(), None, None, None)
}

fn extract_timed_text_track(
    track_id: ImfUuid,
    subtitle_type: &str,
    resource_list: &ResourceList,
    descriptors: &std::collections::HashMap<ImfUuid, &EssenceDescriptor>,
) -> Option<TrackInfo> {
    for resource in &resource_list.resources {
        if let Some(source_encoding) = &resource.source_encoding {
            if let Some(ed) = descriptors.get(source_encoding) {
                let language = ed
                    .dc_timed_text_descriptor
                    .as_ref()
                    .map(|tt| {
                        tt.rfc5646_language_tag_list
                            .iter()
                            .map(|lt| lt.as_str())
                            .filter(|s| !s.is_empty())
                            .collect::<Vec<_>>()
                            .join(",")
                    })
                    .filter(|s| !s.is_empty());
                return Some(TrackInfo {
                    track_id: track_id.to_string(),
                    track_type: "subtitle".to_string(),
                    codec: "IMSC1 (Timed Text)".to_string(),
                    language,
                    channels: None,
                    format_details: None,
                    resolution: None,
                    framerate: None,
                    bit_depth: None,
                    subtitle_type: Some(subtitle_type.to_string()),
                });
            }
        }
    }
    None
}

pub fn format_framerate(edit_rate: &EditRate) -> String {
    let fps = edit_rate.as_f64();
    if (fps - 23.976).abs() < 0.01 {
        "23.976".to_string()
    } else if (fps - 29.97).abs() < 0.01 {
        "29.97".to_string()
    } else if (fps - 59.94).abs() < 0.01 {
        "59.94".to_string()
    } else if fps == fps.round() {
        format!("{}", fps as u32)
    } else {
        format!("{:.3}", fps)
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assetmap::ImfUuid;

    fn make_seq_list_with_all_types() -> SequenceList {
        let uuid = || ImfUuid::parse("urn:uuid:00000000-0000-0000-0000-000000000001").unwrap();
        let rl = || ResourceList { resources: vec![] };
        SequenceList {
            marker_sequences: vec![MarkerSequence {
                id: uuid(),
                track_id: uuid(),
                resource_list: rl(),
            }],
            main_image_sequences: vec![MainImageSequence {
                id: uuid(),
                track_id: uuid(),
                resource_list: rl(),
            }],
            main_audio_sequences: vec![MainAudioSequence {
                id: uuid(),
                track_id: uuid(),
                resource_list: rl(),
            }],
            subtitles_sequences: vec![SubtitlesSequence {
                id: uuid(),
                track_id: uuid(),
                resource_list: rl(),
            }],
            hearing_impaired_captions_sequences: vec![HearingImpairedCaptionsSequence {
                id: uuid(),
                track_id: uuid(),
                resource_list: rl(),
            }],
            forced_narrative_sequences: vec![ForcedNarrativeSequence {
                id: uuid(),
                track_id: uuid(),
                resource_list: rl(),
            }],
            iab_sequences: vec![IABSequence {
                id: uuid(),
                track_id: uuid(),
                resource_list: rl(),
            }],
            isxd_sequences: vec![ISXDSequence {
                id: uuid(),
                track_id: uuid(),
                resource_list: rl(),
            }],
        }
    }

    #[test]
    fn all_sequences_excludes_markers() {
        let sl = make_seq_list_with_all_types();
        // 7 non-marker sequence types, 1 of each
        assert_eq!(sl.all_sequences().len(), 7);
    }

    #[test]
    fn all_sequences_typed_returns_type_names() {
        let sl = make_seq_list_with_all_types();
        let typed = sl.all_sequences_typed();
        assert_eq!(typed.len(), 7);
        let names: Vec<&str> = typed.iter().map(|(_, n)| *n).collect();
        assert!(names.contains(&"MainImage"));
        assert!(names.contains(&"MainAudio"));
        assert!(names.contains(&"Subtitles"));
        assert!(names.contains(&"HearingImpairedCaptions"));
        assert!(names.contains(&"ForcedNarrative"));
        assert!(names.contains(&"IAB"));
        assert!(names.contains(&"ISXD"));
    }

    #[test]
    fn all_sequences_empty_list() {
        let sl = SequenceList {
            marker_sequences: vec![],
            main_image_sequences: vec![],
            main_audio_sequences: vec![],
            subtitles_sequences: vec![],
            hearing_impaired_captions_sequences: vec![],
            forced_narrative_sequences: vec![],
            iab_sequences: vec![],
            isxd_sequences: vec![],
        };
        assert!(sl.all_sequences().is_empty());
        assert!(sl.all_sequences_typed().is_empty());
    }

    #[test]
    fn try_extract_cpl_track_codecs_invalid_xml() {
        let result = try_extract_cpl_track_codecs_from_xml("<not-a-cpl/>");
        assert!(result.is_err());
    }

    struct AcceptAllSignatureVerifier;
    impl XmlSignatureVerifier for AcceptAllSignatureVerifier {
        fn verify(&self, _xml_content: &str) -> Result<(), String> {
            Ok(())
        }
    }

    struct RejectingSignatureVerifier;
    impl XmlSignatureVerifier for RejectingSignatureVerifier {
        fn verify(&self, _xml_content: &str) -> Result<(), String> {
            Err("bad signature".to_string())
        }
    }

    #[test]
    fn strict_production_options_enable_all_strict_checks() {
        let verifier = ReferenceDigestXmlDsigVerifier;
        let options = strict_production_parse_options(&verifier);
        assert_eq!(options.unknown_field_mode, UnknownFieldMode::Error);
        assert_eq!(options.schema_strict_mode, SchemaStrictMode::Basic);
        assert_eq!(
            options.signature_validation_mode,
            SignatureValidationMode::RequireValid
        );
        assert!(options.signature_verifier.is_some());
    }

    #[test]
    fn recommended_signature_verifier_rejects_unsigned_xml() {
        let xml = minimal_cpl_with_ns("http://www.smpte-ra.org/schemas/2067-3/2013");
        let verifier = recommended_signature_verifier();
        assert!(verifier.verify(&xml).is_err());
    }

    #[test]
    fn test_strip_xml_namespaces() {
        let input = r#"<r0:RGBADescriptor xmlns:r0="http://example.com"><r1:DisplayWidth>3840</r1:DisplayWidth></r0:RGBADescriptor>"#;
        let result = strip_xml_namespaces(input);
        assert!(result.contains("<RGBADescriptor"));
        assert!(result.contains("<DisplayWidth>3840</DisplayWidth>"));
        assert!(result.contains("</RGBADescriptor>"));
        assert!(!result.contains("xmlns:r0"));
    }

    #[test]
    fn test_strip_preserves_content_with_colons() {
        let input = r#"<PictureCompression>urn:smpte:ul:060e2b34</PictureCompression>"#;
        let result = strip_xml_namespaces(input);
        assert_eq!(result, input); // No namespace prefixes to strip
    }

    #[test]
    fn test_parse_simple_cpl() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8" ?>
<CompositionPlaylist xmlns="http://www.smpte-ra.org/schemas/2067-3/2013">
<Id>urn:uuid:0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85</Id>
<Annotation>Test CPL</Annotation>
<IssueDate>2016-10-06T08:35:02-00:00</IssueDate>
<ContentTitle>Test Content</ContentTitle>
<ContentKind>Test</ContentKind>
<SegmentList>
<Segment>
<Id>urn:uuid:00000000-0000-0000-0000-000000000001</Id>
<SequenceList>
</SequenceList>
</Segment>
</SegmentList>
</CompositionPlaylist>"#;

        let result = parse_cpl(xml);
        match result {
            Ok(cpl) => {
                assert_eq!(
                    cpl.id,
                    ImfUuid::parse("urn:uuid:0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85").unwrap()
                );
                assert_eq!(cpl.content_title.text, "Test Content");
                assert_eq!(cpl.content_kind, ContentKind::Test);
                assert!(!cpl.segment_list.segments.is_empty());
            }
            Err(e) => panic!("Failed to parse CPL: {:?}", e),
        }
    }

    #[test]
    fn test_content_kind_scope_attribute() {
        // Verify scope attribute is captured from ContentKind XML element
        let xml = r#"<?xml version="1.0" encoding="UTF-8" ?>
<CompositionPlaylist xmlns="http://www.smpte-ra.org/schemas/2067-3/2013">
<Id>urn:uuid:0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85</Id>
<IssueDate>2024-01-01T00:00:00Z</IssueDate>
<ContentTitle>Test</ContentTitle>
<ContentKind scope="http://www.smpte-ra.org/schemas/2067-3/2013#content-kind">feature</ContentKind>
<SegmentList>
<Segment>
<Id>urn:uuid:00000000-0000-0000-0000-000000000001</Id>
<SequenceList>
</SequenceList>
</Segment>
</SegmentList>
</CompositionPlaylist>"#;

        let cpl = parse_cpl(xml).expect("Failed to parse CPL with ContentKind scope");
        assert_eq!(cpl.content_kind.kind, ContentKind::Feature);
        assert_eq!(
            cpl.content_kind.scope.as_deref(),
            Some("http://www.smpte-ra.org/schemas/2067-3/2013#content-kind")
        );
        assert_eq!(
            cpl.content_kind.effective_scope(),
            "http://www.smpte-ra.org/schemas/2067-3/2013#content-kind"
        );
    }

    #[test]
    fn test_content_kind_custom_scope() {
        // Verify custom (non-default) scope is preserved
        let xml = r#"<?xml version="1.0" encoding="UTF-8" ?>
<CompositionPlaylist xmlns="http://www.smpte-ra.org/schemas/2067-3/2013">
<Id>urn:uuid:0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85</Id>
<IssueDate>2024-01-01T00:00:00Z</IssueDate>
<ContentTitle>Test</ContentTitle>
<ContentKind scope="http://example.com/custom-kinds">my-custom-kind</ContentKind>
<SegmentList>
<Segment>
<Id>urn:uuid:00000000-0000-0000-0000-000000000001</Id>
<SequenceList>
</SequenceList>
</Segment>
</SegmentList>
</CompositionPlaylist>"#;

        let cpl = parse_cpl(xml).expect("Failed to parse CPL with custom scope");
        assert_eq!(
            cpl.content_kind.kind,
            ContentKind::Other("my-custom-kind".to_string())
        );
        assert_eq!(
            cpl.content_kind.scope.as_deref(),
            Some("http://example.com/custom-kinds")
        );
    }

    #[test]
    fn test_content_kind_no_scope_uses_default() {
        // When no scope attribute is present, effective_scope() returns the XSD default
        let xml = r#"<?xml version="1.0" encoding="UTF-8" ?>
<CompositionPlaylist xmlns="http://www.smpte-ra.org/schemas/2067-3/2013">
<Id>urn:uuid:0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85</Id>
<IssueDate>2024-01-01T00:00:00Z</IssueDate>
<ContentTitle>Test</ContentTitle>
<ContentKind>Test</ContentKind>
<SegmentList>
<Segment>
<Id>urn:uuid:00000000-0000-0000-0000-000000000001</Id>
<SequenceList>
</SequenceList>
</Segment>
</SegmentList>
</CompositionPlaylist>"#;

        let cpl = parse_cpl(xml).expect("Failed to parse CPL without scope");
        assert_eq!(cpl.content_kind.kind, ContentKind::Test);
        assert!(cpl.content_kind.scope.is_none());
        assert_eq!(
            cpl.content_kind.effective_scope(),
            CONTENT_KIND_DEFAULT_SCOPE
        );
    }

    #[test]
    fn test_malformed_xml_handling() {
        let malformed_xml = r#"<?xml version="1.0" encoding="UTF-8" ?>
<CompositionPlaylist xmlns="http://www.smpte-ra.org/schemas/2067-3/2013">
<Id>urn:uuid:test</Id>
<ContentTitle>Broken XML"#;

        let result: Result<CompositionPlaylist, CplParseError> = parse_cpl(malformed_xml);
        assert!(result.is_err(), "Should fail with malformed XML");
    }

    // ── Namespace compatibility ──────────────────────────────────────────────

    /// Helper: build a minimal CPL XML with the given xmlns.
    fn minimal_cpl_with_ns(ns: &str) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8" ?>
<CompositionPlaylist xmlns="{ns}">
<Id>urn:uuid:0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85</Id>
<IssueDate>2024-01-01T00:00:00Z</IssueDate>
<ContentTitle>NS Test</ContentTitle>
<ContentKind>Test</ContentKind>
<SegmentList>
<Segment>
<Id>urn:uuid:00000000-0000-0000-0000-000000000001</Id>
<SequenceList></SequenceList>
</Segment>
</SegmentList>
</CompositionPlaylist>"#
        )
    }

    /// SMPTE ST 2067-3:2013 namespace (original).
    #[test]
    fn cpl_parses_with_2067_3_2013_namespace() {
        let xml = minimal_cpl_with_ns("http://www.smpte-ra.org/schemas/2067-3/2013");
        let cpl = parse_cpl(&xml).expect("2013 namespace should parse");
        assert_eq!(cpl.content_title.text, "NS Test");
        assert_eq!(cpl.namespace, CplNamespace::Smpte2067_3_2013);
        assert_eq!(cpl.namespace.spec_id(), "ST 2067-3:2013");
        assert_eq!(cpl.namespace.year(), Some(2013));
    }

    /// SMPTE ST 2067-3:2016 namespace.
    #[test]
    fn cpl_parses_with_2067_3_2016_namespace() {
        let xml = minimal_cpl_with_ns("http://www.smpte-ra.org/schemas/2067-3/2016");
        let cpl = parse_cpl(&xml).expect("2016 namespace should parse");
        assert_eq!(cpl.content_title.text, "NS Test");
        assert_eq!(cpl.namespace, CplNamespace::Smpte2067_3_2016);
        assert_eq!(cpl.namespace.year(), Some(2016));
    }

    /// `http://www.smpte-ra.org/ns/2067-3/2020` is not a registered namespace —
    /// ST 2067-3:2020 reuses the 2016 namespace per the canonical XSD. Documents
    /// declaring the fake URI parse but resolve to `Unknown`.
    #[test]
    fn cpl_parses_with_fake_2020_namespace_as_unknown() {
        let xml = minimal_cpl_with_ns("http://www.smpte-ra.org/ns/2067-3/2020");
        let cpl = parse_cpl(&xml).expect("CPL should still parse, namespace just unknown");
        assert_eq!(cpl.content_title.text, "NS Test");
        assert!(matches!(cpl.namespace, CplNamespace::Unknown(_)));
        assert_eq!(cpl.namespace.year(), None);
    }

    /// FIX-3 regression: a CPL with no detectable root xmlns lands in
    /// `Unknown(String::new())` rather than silently defaulting to the 2013
    /// ruleset (the first enum variant).
    #[test]
    fn cpl_without_root_xmlns_lands_in_unknown_not_2013() {
        // No xmlns attribute on the root element at all.
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<CompositionPlaylist>
    <Id>urn:uuid:0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85</Id>
    <IssueDate>2024-01-01T00:00:00Z</IssueDate>
    <ContentTitle>NS Test</ContentTitle>
    <EditRate>24 1</EditRate>
    <SegmentList></SegmentList>
</CompositionPlaylist>"#;
        let cpl = parse_cpl(xml).expect("CPL should still parse without xmlns");
        assert!(
            matches!(cpl.namespace, CplNamespace::Unknown(ref s) if s.is_empty()),
            "expected Unknown(\"\") for missing xmlns, got {:?}",
            cpl.namespace
        );
    }

    /// DCI CPL namespace compatibility (pre-IMF era, ST 429 series).
    #[test]
    fn cpl_parses_with_dci_429_7_namespace() {
        let xml = minimal_cpl_with_ns("http://www.smpte-ra.org/schemas/429-7/2006/CPL");
        let cpl = parse_cpl(&xml).expect("DCI 429-7 namespace should parse");
        assert_eq!(cpl.content_title.text, "NS Test");
        assert_eq!(cpl.namespace, CplNamespace::Dci429_7);
        assert_eq!(cpl.namespace.year(), Some(2006));
    }

    /// Real test corpus: MERIDIAN CPL should detect 2013 namespace.
    #[test]
    fn cpl_meridian_detects_2013_namespace() {
        let xml = include_str!("../../../../test-data/MERIDIAN_Netflix_Photon_161006/CPL_0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85.xml");
        let cpl = parse_cpl(xml).expect("MERIDIAN CPL should parse");
        assert_eq!(cpl.namespace, CplNamespace::Smpte2067_3_2013);
    }

    #[test]
    fn strict_unknown_mode_rejects_unknown_elements() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8" ?>
<CompositionPlaylist xmlns="http://www.smpte-ra.org/schemas/2067-3/2013">
<Id>urn:uuid:0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85</Id>
<IssueDate>2024-01-01T00:00:00Z</IssueDate>
<ContentTitle>Strict Unknown</ContentTitle>
<ContentKind>Test</ContentKind>
<UnknownElement>oops</UnknownElement>
<SegmentList><Segment><Id>urn:uuid:00000000-0000-0000-0000-000000000001</Id><SequenceList></SequenceList></Segment></SegmentList>
</CompositionPlaylist>"#;

        let options = CplParseOptions {
            unknown_field_mode: UnknownFieldMode::Error,
            ..Default::default()
        };
        let result = parse_cpl_with_options(xml, &options);
        assert!(matches!(result, Err(CplParseError::StrictUnknownXml(_))));
    }

    #[test]
    fn strict_schema_mode_rejects_empty_sequence_list_per_segment() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8" ?>
<CompositionPlaylist xmlns="http://www.smpte-ra.org/schemas/2067-3/2013">
<Id>urn:uuid:0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85</Id>
<IssueDate>2024-01-01T00:00:00Z</IssueDate>
<ContentTitle>Strict Schema</ContentTitle>
<ContentKind>Test</ContentKind>
<SegmentList><Segment><Id>urn:uuid:00000000-0000-0000-0000-000000000001</Id><SequenceList></SequenceList></Segment></SegmentList>
</CompositionPlaylist>"#;

        let options = CplParseOptions {
            schema_strict_mode: SchemaStrictMode::Basic,
            ..Default::default()
        };
        let result = parse_cpl_with_options(xml, &options);
        assert!(matches!(result, Err(CplParseError::StrictSchema(_))));
    }

    #[test]
    fn signature_mode_require_presence_rejects_unsigned_cpl() {
        let xml = minimal_cpl_with_ns("http://www.smpte-ra.org/schemas/2067-3/2013");
        let options = CplParseOptions {
            signature_validation_mode: SignatureValidationMode::RequirePresence,
            ..Default::default()
        };
        let result = parse_cpl_with_options(&xml, &options);
        assert!(matches!(result, Err(CplParseError::StrictSchema(_))));
    }

    #[test]
    fn signature_mode_require_valid_needs_verifier() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8" ?>
<CompositionPlaylist xmlns="http://www.smpte-ra.org/schemas/2067-3/2013">
<Id>urn:uuid:0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85</Id>
<IssueDate>2024-01-01T00:00:00Z</IssueDate>
<ContentTitle>Signed</ContentTitle>
<ContentKind>Test</ContentKind>
<Signature>dummy</Signature>
<SegmentList><Segment><Id>urn:uuid:00000000-0000-0000-0000-000000000001</Id><SequenceList></SequenceList></Segment></SegmentList>
</CompositionPlaylist>"#;

        let options = CplParseOptions {
            signature_validation_mode: SignatureValidationMode::RequireValid,
            ..Default::default()
        };
        let result = parse_cpl_with_options(xml, &options);
        assert!(matches!(
            result,
            Err(CplParseError::SignatureVerifierRequired)
        ));
    }

    #[test]
    fn signature_mode_require_valid_uses_verifier() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8" ?>
<CompositionPlaylist xmlns="http://www.smpte-ra.org/schemas/2067-3/2013">
<Id>urn:uuid:0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85</Id>
<IssueDate>2024-01-01T00:00:00Z</IssueDate>
<ContentTitle>Signed</ContentTitle>
<ContentKind>Test</ContentKind>
<Signature>dummy</Signature>
<SegmentList><Segment><Id>urn:uuid:00000000-0000-0000-0000-000000000001</Id><SequenceList><MainImageSequence><Id>urn:uuid:11111111-1111-1111-1111-111111111111</Id><TrackId>urn:uuid:22222222-2222-2222-2222-222222222222</TrackId><ResourceList><Resource><Id>urn:uuid:33333333-3333-3333-3333-333333333333</Id><IntrinsicDuration>1</IntrinsicDuration></Resource></ResourceList></MainImageSequence></SequenceList></Segment></SegmentList>
</CompositionPlaylist>"#;

        let verifier = AcceptAllSignatureVerifier;
        let options = CplParseOptions {
            signature_validation_mode: SignatureValidationMode::RequireValid,
            signature_verifier: Some(&verifier),
            ..Default::default()
        };
        let result = parse_cpl_with_options(xml, &options);
        assert!(
            result.is_ok(),
            "signature verifier should allow parse: {result:?}"
        );
    }

    #[test]
    fn signature_mode_require_valid_surfaces_verification_failure() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8" ?>
<CompositionPlaylist xmlns="http://www.smpte-ra.org/schemas/2067-3/2013">
<Id>urn:uuid:0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85</Id>
<IssueDate>2024-01-01T00:00:00Z</IssueDate>
<ContentTitle>Signed</ContentTitle>
<ContentKind>Test</ContentKind>
<Signature>dummy</Signature>
<SegmentList><Segment><Id>urn:uuid:00000000-0000-0000-0000-000000000001</Id><SequenceList></SequenceList></Segment></SegmentList>
</CompositionPlaylist>"#;

        let verifier = RejectingSignatureVerifier;
        let options = CplParseOptions {
            signature_validation_mode: SignatureValidationMode::RequireValid,
            signature_verifier: Some(&verifier),
            ..Default::default()
        };
        let result = parse_cpl_with_options(xml, &options);
        assert!(matches!(
            result,
            Err(CplParseError::SignatureVerificationFailed(_))
        ));
    }

    fn build_signed_cpl_with_reference_digest(tamper_digest: bool) -> String {
        let unsigned_xml = r#"<?xml version="1.0" encoding="UTF-8" ?>
<CompositionPlaylist xmlns="http://www.smpte-ra.org/schemas/2067-3/2013">
<Id>urn:uuid:0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85</Id>
<IssueDate>2024-01-01T00:00:00Z</IssueDate>
<ContentTitle>Signed Digest CPL</ContentTitle>
<ContentKind>Test</ContentKind>
<SegmentList>
<Segment>
<Id>urn:uuid:00000000-0000-0000-0000-000000000001</Id>
<SequenceList></SequenceList>
</Segment>
</SegmentList>
</CompositionPlaylist>"#;

        let normalized = normalize_xml_for_digest(unsigned_xml);
        let digest = compute_hash(HashAlgorithm::Sha256, normalized.as_bytes());
        let mut digest_b64 = base64::engine::general_purpose::STANDARD.encode(digest);
        if tamper_digest {
            digest_b64 = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=".to_string();
        }

        format!(
            r#"<?xml version="1.0" encoding="UTF-8" ?>
<CompositionPlaylist xmlns="http://www.smpte-ra.org/schemas/2067-3/2013">
<Id>urn:uuid:0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85</Id>
<IssueDate>2024-01-01T00:00:00Z</IssueDate>
<ContentTitle>Signed Digest CPL</ContentTitle>
<ContentKind>Test</ContentKind>
<Signature xmlns="http://www.w3.org/2000/09/xmldsig#">
  <SignedInfo>
    <CanonicalizationMethod Algorithm="http://www.w3.org/TR/2001/REC-xml-c14n-20010315"/>
    <SignatureMethod Algorithm="http://www.w3.org/2001/04/xmldsig-more#rsa-sha256"/>
    <Reference URI="">
      <Transforms>
        <Transform Algorithm="http://www.w3.org/2000/09/xmldsig#enveloped-signature"/>
      </Transforms>
      <DigestMethod Algorithm="http://www.w3.org/2001/04/xmlenc#sha256"/>
      <DigestValue>{}</DigestValue>
    </Reference>
  </SignedInfo>
  <SignatureValue>AQ==</SignatureValue>
</Signature>
<SegmentList>
<Segment>
<Id>urn:uuid:00000000-0000-0000-0000-000000000001</Id>
<SequenceList></SequenceList>
</Segment>
</SegmentList>
</CompositionPlaylist>"#,
            digest_b64
        )
    }

    #[test]
    fn reference_digest_verifier_accepts_valid_uri_empty_digest() {
        let xml = build_signed_cpl_with_reference_digest(false);
        let verifier = ReferenceDigestXmlDsigVerifier;
        assert!(verifier.verify(&xml).is_ok());
    }

    #[test]
    fn reference_digest_verifier_rejects_mismatched_digest() {
        let xml = build_signed_cpl_with_reference_digest(true);
        let verifier = ReferenceDigestXmlDsigVerifier;
        let result = verifier.verify(&xml);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(
            error.contains("DigestValue mismatch"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn signature_mode_require_valid_with_reference_digest_verifier() {
        let xml = build_signed_cpl_with_reference_digest(false);
        let verifier = ReferenceDigestXmlDsigVerifier;
        let options = CplParseOptions {
            signature_validation_mode: SignatureValidationMode::RequireValid,
            signature_verifier: Some(&verifier),
            ..Default::default()
        };
        let result = parse_cpl_with_options(&xml, &options);
        assert!(
            result.is_ok(),
            "expected valid signature digest path to parse: {result:?}"
        );
    }

    #[cfg(all(feature = "xmlsec1", not(target_arch = "wasm32")))]
    #[test]
    fn xmlsec_verifier_surfaces_missing_binary() {
        let xml = minimal_cpl_with_ns("http://www.smpte-ra.org/schemas/2067-3/2013");
        let verifier = XmlSec1Verifier::new().with_binary_path("xmlsec1-definitely-not-installed");
        let error = verifier
            .verify(&xml)
            .expect_err("expected missing binary error");
        assert!(
            error.contains("failed to execute"),
            "unexpected error message: {error}"
        );
    }

    #[cfg(all(feature = "xmlsec", not(target_arch = "wasm32")))]
    #[test]
    fn xmlsec_crate_verifier_rejects_invalid_key_material() {
        let xml = build_signed_cpl_with_reference_digest(false);
        let verifier = XmlSecCrateVerifier::from_pem("not-a-valid-key");
        let error = verifier
            .verify(&xml)
            .expect_err("expected xmlsec key load error");
        assert!(
            error.contains("xmlsec key load failed") || error.contains("xmlsec verify failed"),
            "unexpected error message: {error}"
        );
    }
}
