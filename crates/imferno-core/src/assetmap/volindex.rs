//! SMPTE ST 429-9 / ST 2067-9 — VOLINDEX parser.
//!
//! VOLINDEX is a trivial document — a single volume index number indicating
//! how many volumes are in the package. In practice always `<Index>1</Index>`
//! for single-volume packages.
//!
//! Per the spec, VOLINDEX is optional. If absent, the pipeline assumes a
//! single-volume package.

// codes live in assetmap/codes.rs

use serde::Deserialize;

// ─── Raw deserialization layer ─────────────────────────────────────────────────

mod raw {
    use serde::Deserialize;

    #[derive(Deserialize)]
    pub struct VolumeIndex {
        #[serde(rename = "Index")]
        pub index: u32,
    }
}

// ─── Error ────────────────────────────────────────────────────────────────────

#[derive(Debug, thiserror::Error)]
pub enum VolindexError {
    #[error("XML parse error: {0}")]
    Xml(#[from] quick_xml::DeError),
}

// ─── VolumeIndex ──────────────────────────────────────────────────────────────

/// VOLINDEX document (SMPTE ST 2067-9 §5 / ST 429-9:2014).
///
/// In practice this is always `<Index>1</Index>` for single-volume IMF packages.
/// The spec allows multi-volume packages but they are rarely used.
#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, serde::Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "typescript", derive(ts_rs::TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
#[cfg_attr(feature = "wasm", derive(tsify::Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct VolumeIndex {
    /// The volume count. Almost always `1`.
    #[serde(rename = "Index")]
    pub index: u32,
}

// ─── Parse function ───────────────────────────────────────────────────────────

/// Parse a VOLINDEX.xml document (SMPTE ST 2067-9 §5).
///
/// Returns `VolindexError::Xml` if the document is malformed.
pub fn parse_volindex(xml_content: &str) -> Result<VolumeIndex, VolindexError> {
    let raw: raw::VolumeIndex = quick_xml::de::from_str(xml_content)?;
    Ok(VolumeIndex { index: raw.index })
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    /// SMPTE ST 2067-9 §5: VOLINDEX contains a single <Index> element.
    #[test]
    fn volindex_parses_index_element() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="no" ?>
<VolumeIndex xmlns="http://www.smpte-ra.org/schemas/429-9/2007/AM">
    <Index>1</Index>
</VolumeIndex>"#;
        let result = parse_volindex(xml).unwrap();
        assert_eq!(result.index, 1);
    }

    #[test]
    fn volindex_parses_multi_volume() {
        let xml = r#"<VolumeIndex xmlns="http://www.smpte-ra.org/ns/2067-9/2020">
    <Index>3</Index>
</VolumeIndex>"#;
        let result = parse_volindex(xml).unwrap();
        assert_eq!(result.index, 3);
    }

    #[test]
    fn volindex_rejects_malformed_xml() {
        assert!(parse_volindex("<not-closed").is_err());
    }
}
