//! Full MXF header metadata extraction via `regxml` —
//! `MxfFragmentBuilder` converts an MXF file's header metadata sets
//! (Preface, MaterialPackage, descriptors, sub-descriptors) to RegXML
//! using a SMPTE metadictionary.
//!
//! This is the substrate for the ST 2067-2 §5.3 (audio MCA),
//! §5.4 (timed text / IMSC profile) and §5.5 (IAB) rules applied
//! against typed metadata. Native-only — wasm doesn't see MXF
//! binaries.

use std::io::Cursor;
use std::path::Path;
use std::sync::OnceLock;

use regxml::{MxfFragmentBuilder, MxfFragmentError, MxfFragmentOptions};
use regxml_dict::{importer::import_registers, MetaDictionary};

use crate::diagnostics::{Category, Location, Severity, ValidationIssue};

// ── Embedded SMPTE registers ────────────────────────────────────────────────
//
// The Elements / Groups / Types registers are the SMPTE source-of-truth
// for AAF-class definitions used by MXF header metadata. Vendored from
// regxmllib-rs's `crates/regxmllib-cli/registers/`. Together they total
// ~5.5 MB which is meaningful payload — but they're the only complete
// definition source `MxfFragmentBuilder` accepts. The Labels register
// is omitted here because `MxfFragmentBuilder` doesn't consume it
// (only optional `AuidNamer` does); save the binary footprint.

const REG_ELEMENTS: &[u8] = include_bytes!("../../resources/registers/Elements.xml");
const REG_GROUPS: &[u8] = include_bytes!("../../resources/registers/Groups.xml");
const REG_TYPES: &[u8] = include_bytes!("../../resources/registers/Types.xml");

/// Lazily-initialised metadictionary. Built once on first use from
/// `import_registers`, shared by every caller for the lifetime of the
/// process.
///
/// Returns `None` only if `import_registers` fails — which is a
/// build-time consistency error that would already have surfaced via
/// integration tests, so callers can treat `None` as
/// "engine misconfigured".
pub fn dictionaries() -> Option<&'static MetaDictionary> {
    static CELL: OnceLock<Option<MetaDictionary>> = OnceLock::new();
    CELL.get_or_init(|| {
        import_registers(&[REG_ELEMENTS, REG_GROUPS, REG_TYPES]).ok()
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
