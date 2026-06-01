//! Full MXF header metadata extraction via `regxml` —
//! `MxfFragmentBuilder` converts an MXF file's header metadata sets
//! (Preface, MaterialPackage, descriptors, sub-descriptors) to RegXML
//! using a SMPTE metadictionary.
//!
//! This is the substrate for the ST 2067-2 §5.3 (audio MCA),
//! §5.4 (timed text / IMSC profile) and §5.5 (IAB) rules that Photon
//! enforces against typed metadata. Native-only — wasm doesn't see
//! MXF binaries.

use std::io::Cursor;
use std::path::Path;
use std::sync::OnceLock;

use regxml::{MxfFragmentBuilder, MxfFragmentOptions, MxfFragmentError};
use regxml_dict::{MetaDictionary, MetaDictionaryCollection};

use crate::diagnostics::{Category, Location, Severity, ValidationIssue};

// ── Embedded SMPTE metadictionaries ─────────────────────────────────────────
//
// Pre-compiled metadictionary XMLs sourced from regxmllib-rs's
// `resources/regxml-dicts/`. The three baseline namespaces cover the
// vast majority of MXF descriptor / sub-descriptor types encountered
// in IMF deliveries:
//
// - 335-2012 — SMPTE Elements Register
// - 2003-2012 — SMPTE Element Container UL Register
// - 395-2014 — SMPTE Groups & Types extension for ST 377-4 audio
//
// Vendored with the regxmllib-rs author's permission (it's their crate
// + their data); same MIT license terms as the rest of imferno-core.

const DICT_335: &[u8] = include_bytes!("../../resources/regxml-dicts/www-smpte-ra-org-reg-335-2012.xml");
const DICT_2003: &[u8] = include_bytes!("../../resources/regxml-dicts/www-smpte-ra-org-reg-2003-2012.xml");
const DICT_395: &[u8] = include_bytes!("../../resources/regxml-dicts/www-smpte-ra-org-reg-395-2014.xml");

/// Lazily-initialised metadictionary collection. Built once on first
/// use, shared by every caller for the lifetime of the process.
///
/// Returns `None` only if one of the embedded dictionary XMLs fails to
/// parse — which is a build-time consistency error that would already
/// have surfaced via integration tests, so callers can treat `None` as
/// "engine misconfigured".
pub fn dictionaries() -> Option<&'static MetaDictionaryCollection> {
    static CELL: OnceLock<Option<MetaDictionaryCollection>> = OnceLock::new();
    CELL.get_or_init(|| {
        let mut coll = MetaDictionaryCollection::new();
        for bytes in [DICT_335, DICT_2003, DICT_395] {
            let dict = MetaDictionary::from_xml(bytes).ok()?;
            coll.add(dict).ok()?;
        }
        Some(coll)
    })
    .as_ref()
}

/// Convert an MXF file's header metadata into RegXML.
///
/// Wraps `regxml::MxfFragmentBuilder::from_reader` with imferno's
/// embedded metadictionary so callers don't have to plumb dictionaries
/// themselves. Returns the RegXML as a UTF-8 string ready for further
/// parsing (e.g. via `quick_xml`) when applying essence-layer rules.
///
/// `options` controls which partition is read (footer first by default,
/// header fallback) and whether the full Preface or just an
/// EssenceDescriptor is emitted.
pub fn parse_mxf_to_regxml(
    path: &Path,
    options: MxfFragmentOptions,
) -> Result<String, MxfFragmentError> {
    let dicts = dictionaries().ok_or_else(|| {
        MxfFragmentError::Xml(
            "imferno metadictionary failed to load — engine misconfigured".to_string(),
        )
    })?;
    let file = std::fs::File::open(path).map_err(|e| MxfFragmentError::Io(e.to_string()))?;
    let mut reader = std::io::BufReader::new(file);
    let mut buf: Vec<u8> = Vec::new();
    MxfFragmentBuilder::from_reader(&mut reader, Cursor::new(&mut buf), dicts, options)?;
    String::from_utf8(buf).map_err(|e| {
        MxfFragmentError::Xml(format!("RegXML output was not valid UTF-8: {e}"))
    })
}

/// Wrap a `regxml`-side error as a `ValidationIssue` so callers can
/// fold it into the unified `ValidationReport` alongside the
/// partition-pack diagnostics from `mxf::essence`.
pub fn regxml_error_issue(path: &Path, err: &MxfFragmentError) -> ValidationIssue {
    ValidationIssue::new(
        Severity::Warning,
        Category::Container,
        "IMFERNO:Mxf/RegXmlConversionFailed",
        format!(
            "Could not convert MXF header metadata of {} to RegXML: {}",
            path.display(),
            err
        ),
    )
    .with_location(Location::new().with_file(path.to_path_buf()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_dictionaries_load_successfully() {
        // Just proving the embedded XMLs parse and the collection
        // builds — getting an exhaustive resolution test right would
        // couple us to SMPTE register revisions. If the load itself
        // succeeded, the dictionaries are usable by
        // `MxfFragmentBuilder`; a real-fixture integration test
        // exercises end-to-end RegXML emission.
        assert!(
            dictionaries().is_some(),
            "embedded metadictionaries must parse"
        );
    }

    #[test]
    fn parse_mxf_to_regxml_surfaces_io_error_for_missing_file() {
        let opts = MxfFragmentOptions::default();
        let err = parse_mxf_to_regxml(
            std::path::Path::new("/nonexistent/imferno-metadata-test.mxf"),
            opts,
        )
        .expect_err("missing file must error");
        let msg = format!("{err}");
        assert!(
            msg.to_lowercase().contains("no such file")
                || msg.to_lowercase().contains("not found")
                || msg.to_lowercase().contains("os error"),
            "expected IO error, got: {msg}"
        );
    }
}
