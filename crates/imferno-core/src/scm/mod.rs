//! SMPTE ST 2067-9:2018 — Sidecar Composition Map (SCM) parser.
//!
//! The Sidecar Composition Map associates sidecar assets (e.g. IAB audio,
//! subtitle tracks) with a Composition Playlist by UUID, without modifying
//! the CPL itself.
//!
//! Normative namespace: `http://www.smpte-ra.org/ns/2067-9/2018`

pub mod codes;

use crate::assetmap::ImfUuid;
use thiserror::Error;

// ── Raw deserialization types ────────────────────────────────────────────────

mod raw {
    use serde::{Deserialize, Deserializer};

    /// Deserializes any XML element (with children) and discards the value.
    /// Used to detect presence of optional elements like `<Signer>` / `<Signature>`.
    #[derive(Debug)]
    pub struct Present;

    impl<'de> Deserialize<'de> for Present {
        fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
            serde::de::IgnoredAny::deserialize(d)?;
            Ok(Present)
        }
    }

    #[derive(Deserialize)]
    pub struct SidecarCompositionMap {
        #[serde(rename = "Id")]
        pub id: String,
        #[serde(rename = "IssueDate")]
        pub issue_date: String,
        #[serde(rename = "Issuer")]
        pub issuer: Option<String>,
        #[serde(rename = "Creator")]
        pub creator: Option<String>,
        #[serde(rename = "Annotation")]
        pub annotation: Option<String>,
        /// Presence flag for §7.2.4 Signer/Signature cross-check.
        #[serde(rename = "Signer")]
        pub signer: Option<Present>,
        /// Presence flag for §7.2.5 Signer/Signature cross-check.
        #[serde(rename = "Signature")]
        pub signature: Option<Present>,
        #[serde(rename = "SidecarAssetList")]
        pub sidecar_asset_list: Option<SidecarAssetList>,
    }

    #[derive(Deserialize)]
    pub struct SidecarAssetList {
        #[serde(rename = "SidecarAsset", default)]
        pub sidecar_assets: Vec<SidecarAsset>,
    }

    /// Per ST 2067-9:2018 §7.3 Table 3: each sidecar asset carries an
    /// `AssociatedCPLList` containing one or more `CPLId` elements.
    #[derive(Deserialize)]
    pub struct SidecarAsset {
        #[serde(rename = "Id")]
        pub id: String,
        #[serde(rename = "AssociatedCPLList")]
        pub associated_cpl_list: Option<AssociatedCplList>,
    }

    #[derive(Deserialize, Default)]
    pub struct AssociatedCplList {
        #[serde(rename = "CPLId", default)]
        pub cpl_ids: Vec<String>,
    }
}

// ── Domain types ─────────────────────────────────────────────────────────────

/// A parsed Sidecar Composition Map document (ST 2067-9:2018).
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SidecarCompositionMap {
    /// UUID of this SCM document.
    pub id: ImfUuid,
    pub issue_date: String,
    pub issuer: Option<String>,
    pub creator: Option<String>,
    pub annotation: Option<String>,
    /// True if a `<Signer>` element was present (§7.2.4).
    pub has_signer: bool,
    /// True if a `<Signature>` element was present (§7.2.5).
    pub has_signature: bool,
    /// Sidecar asset associations.
    pub sidecar_assets: Vec<SidecarAsset>,
}

/// A single sidecar asset association within an SCM document.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SidecarAsset {
    /// UUID of the sidecar asset (must be present in the package AssetMap).
    pub id: ImfUuid,
    /// CPLs this sidecar asset is associated with (§7.3.1.1: one or more).
    pub cpl_ids: Vec<ImfUuid>,
}

// ── Error type ───────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum ScmParseError {
    #[error("XML parse error: {0}")]
    Xml(#[from] quick_xml::DeError),
    #[error("Invalid UUID '{0}': {1}")]
    InvalidUuid(String, String),
}

// ── Parser ───────────────────────────────────────────────────────────────────

/// Parse a SidecarCompositionMap XML document.
pub fn parse_scm(xml: &str) -> Result<SidecarCompositionMap, ScmParseError> {
    let raw: raw::SidecarCompositionMap = quick_xml::de::from_str(xml)?;

    let id = ImfUuid::parse(&raw.id)
        .map_err(|e| ScmParseError::InvalidUuid(raw.id.clone(), e.to_string()))?;

    let sidecar_assets = raw
        .sidecar_asset_list
        .map(|list| list.sidecar_assets)
        .unwrap_or_default()
        .into_iter()
        .map(|a| {
            let asset_id = ImfUuid::parse(&a.id)
                .map_err(|e| ScmParseError::InvalidUuid(a.id.clone(), e.to_string()))?;
            let cpl_ids = a
                .associated_cpl_list
                .unwrap_or_default()
                .cpl_ids
                .into_iter()
                .map(|s| {
                    ImfUuid::parse(&s)
                        .map_err(|e| ScmParseError::InvalidUuid(s.clone(), e.to_string()))
                })
                .collect::<Result<Vec<_>, ScmParseError>>()?;
            Ok(SidecarAsset {
                id: asset_id,
                cpl_ids,
            })
        })
        .collect::<Result<Vec<_>, ScmParseError>>()?;

    Ok(SidecarCompositionMap {
        id,
        issue_date: raw.issue_date,
        issuer: raw.issuer,
        creator: raw.creator,
        annotation: raw.annotation,
        has_signer: raw.signer.is_some(),
        has_signature: raw.signature.is_some(),
        sidecar_assets,
    })
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    const MINIMAL_SCM: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<SidecarCompositionMap xmlns="http://www.smpte-ra.org/ns/2067-9/2018">
    <Id>urn:uuid:a1b2c3d4-e5f6-7890-abcd-ef1234567890</Id>
    <IssueDate>2024-01-01T00:00:00</IssueDate>
    <SidecarAssetList>
        <SidecarAsset>
            <Id>urn:uuid:11111111-2222-3333-4444-555555555555</Id>
            <AssociatedCPLList>
                <CPLId>urn:uuid:aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee</CPLId>
            </AssociatedCPLList>
        </SidecarAsset>
    </SidecarAssetList>
</SidecarCompositionMap>"#;

    #[test]
    fn parses_minimal_scm() {
        let scm = parse_scm(MINIMAL_SCM).unwrap();
        assert_eq!(scm.id.to_string(), "a1b2c3d4-e5f6-7890-abcd-ef1234567890");
        assert_eq!(scm.issue_date, "2024-01-01T00:00:00");
        assert_eq!(scm.sidecar_assets.len(), 1);
        assert_eq!(
            scm.sidecar_assets[0].id.to_string(),
            "11111111-2222-3333-4444-555555555555"
        );
        assert_eq!(scm.sidecar_assets[0].cpl_ids.len(), 1);
        assert_eq!(
            scm.sidecar_assets[0].cpl_ids[0].to_string(),
            "aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee"
        );
    }

    #[test]
    fn parses_multiple_cpl_ids() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<SidecarCompositionMap xmlns="http://www.smpte-ra.org/ns/2067-9/2018">
    <Id>urn:uuid:a1b2c3d4-e5f6-7890-abcd-ef1234567890</Id>
    <IssueDate>2024-01-01T00:00:00</IssueDate>
    <SidecarAssetList>
        <SidecarAsset>
            <Id>urn:uuid:11111111-2222-3333-4444-555555555555</Id>
            <AssociatedCPLList>
                <CPLId>urn:uuid:aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee</CPLId>
                <CPLId>urn:uuid:bbbbbbbb-cccc-dddd-eeee-ffffffffffff</CPLId>
            </AssociatedCPLList>
        </SidecarAsset>
    </SidecarAssetList>
</SidecarCompositionMap>"#;
        let scm = parse_scm(xml).unwrap();
        assert_eq!(scm.sidecar_assets[0].cpl_ids.len(), 2);
        assert_eq!(
            scm.sidecar_assets[0].cpl_ids[1].to_string(),
            "bbbbbbbb-cccc-dddd-eeee-ffffffffffff"
        );
    }

    #[test]
    fn parses_optional_fields() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<SidecarCompositionMap xmlns="http://www.smpte-ra.org/ns/2067-9/2018">
    <Id>urn:uuid:a1b2c3d4-e5f6-7890-abcd-ef1234567890</Id>
    <IssueDate>2024-06-15T12:00:00</IssueDate>
    <Issuer>Test Facility</Issuer>
    <Creator>Test Tool 1.0</Creator>
    <Annotation>IAB sidecar for main feature</Annotation>
    <SidecarAssetList/>
</SidecarCompositionMap>"#;
        let scm = parse_scm(xml).unwrap();
        assert_eq!(scm.issuer.as_deref(), Some("Test Facility"));
        assert_eq!(scm.creator.as_deref(), Some("Test Tool 1.0"));
        assert_eq!(
            scm.annotation.as_deref(),
            Some("IAB sidecar for main feature")
        );
        assert!(scm.sidecar_assets.is_empty());
        assert!(!scm.has_signer);
        assert!(!scm.has_signature);
    }

    #[test]
    fn detects_signer_presence() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<SidecarCompositionMap xmlns="http://www.smpte-ra.org/ns/2067-9/2018">
    <Id>urn:uuid:a1b2c3d4-e5f6-7890-abcd-ef1234567890</Id>
    <IssueDate>2024-01-01T00:00:00</IssueDate>
    <Signer><X509Data/></Signer>
    <SidecarAssetList/>
</SidecarCompositionMap>"#;
        let scm = parse_scm(xml).unwrap();
        assert!(scm.has_signer);
        assert!(!scm.has_signature);
    }

    #[test]
    fn rejects_malformed_xml() {
        assert!(parse_scm("<not valid xml").is_err());
    }

    #[test]
    fn rejects_invalid_uuid() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<SidecarCompositionMap xmlns="http://www.smpte-ra.org/ns/2067-9/2018">
    <Id>not-a-uuid</Id>
    <IssueDate>2024-01-01T00:00:00</IssueDate>
</SidecarCompositionMap>"#;
        assert!(parse_scm(xml).is_err());
    }
}
