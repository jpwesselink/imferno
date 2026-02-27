//! ST 2067-9:2018 (Sidecar Composition Map) integration tests.
//!
//! Tests exercise `Imferno::parse_and_validate` with in-memory file maps so
//! that file-manifest and MXF-header checks are skipped (no `root_path`).
//! Only structural and SCM reference validation runs.

use corpus_tests::validate_package;
use std::collections::HashMap;

// ── Shared UUIDs ─────────────────────────────────────────────────────────────

const CPL_ID: &str = "urn:uuid:cc000001-0000-0000-0000-000000000001";
const SCM_ID: &str = "urn:uuid:cc000002-0000-0000-0000-000000000002";
const SIDECAR_ID: &str = "urn:uuid:cc000003-0000-0000-0000-000000000003";
const UNKNOWN_ID: &str = "urn:uuid:dd000099-0000-0000-0000-000000000099";

// ── Fixtures ─────────────────────────────────────────────────────────────────

fn assetmap(extra_assets: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<AssetMap xmlns="http://www.smpte-ra.org/schemas/429-9/2007/AM">
  <Id>urn:uuid:aa000000-0000-0000-0000-000000000000</Id>
  <VolumeCount>1</VolumeCount>
  <IssueDate>2024-01-01T00:00:00</IssueDate>
  <AssetList>
    <Asset>
      <Id>{CPL_ID}</Id>
      <ChunkList><Chunk><Path>CPL_test.xml</Path></Chunk></ChunkList>
    </Asset>
    <Asset>
      <Id>{SCM_ID}</Id>
      <ChunkList><Chunk><Path>SCM_test.xml</Path></Chunk></ChunkList>
    </Asset>
    <Asset>
      <Id>{SIDECAR_ID}</Id>
      <ChunkList><Chunk><Path>IAB_sidecar.mxf</Path></Chunk></ChunkList>
    </Asset>
    {extra_assets}
  </AssetList>
</AssetMap>"#,
        CPL_ID = CPL_ID,
        SCM_ID = SCM_ID,
        SIDECAR_ID = SIDECAR_ID,
        extra_assets = extra_assets,
    )
}

fn cpl() -> &'static str {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<CompositionPlaylist xmlns="http://www.smpte-ra.org/schemas/2067-3/2016">
  <Id>urn:uuid:cc000001-0000-0000-0000-000000000001</Id>
  <IssueDate>2024-01-01T00:00:00</IssueDate>
  <ContentTitle>SCM Test CPL</ContentTitle>
  <SegmentList>
    <Segment>
      <Id>urn:uuid:bb000001-0000-0000-0000-000000000001</Id>
      <SequenceList></SequenceList>
    </Segment>
  </SegmentList>
</CompositionPlaylist>"#
}

fn scm_valid() -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<SidecarCompositionMap xmlns="http://www.smpte-ra.org/ns/2067-9/2018">
  <Id>{SCM_ID}</Id>
  <IssueDate>2024-01-01T00:00:00</IssueDate>
  <SidecarAssetList>
    <SidecarAsset>
      <Id>{SIDECAR_ID}</Id>
      <AssociatedCPLList>
        <CPLId>{CPL_ID}</CPLId>
      </AssociatedCPLList>
    </SidecarAsset>
  </SidecarAssetList>
</SidecarCompositionMap>"#,
        SCM_ID = SCM_ID,
        SIDECAR_ID = SIDECAR_ID,
        CPL_ID = CPL_ID,
    )
}

fn scm_unknown_asset() -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<SidecarCompositionMap xmlns="http://www.smpte-ra.org/ns/2067-9/2018">
  <Id>{SCM_ID}</Id>
  <IssueDate>2024-01-01T00:00:00</IssueDate>
  <SidecarAssetList>
    <SidecarAsset>
      <Id>{UNKNOWN_ID}</Id>
      <AssociatedCPLList>
        <CPLId>{CPL_ID}</CPLId>
      </AssociatedCPLList>
    </SidecarAsset>
  </SidecarAssetList>
</SidecarCompositionMap>"#,
        SCM_ID = SCM_ID,
        UNKNOWN_ID = UNKNOWN_ID,
        CPL_ID = CPL_ID,
    )
}

fn scm_unknown_cpl() -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<SidecarCompositionMap xmlns="http://www.smpte-ra.org/ns/2067-9/2018">
  <Id>{SCM_ID}</Id>
  <IssueDate>2024-01-01T00:00:00</IssueDate>
  <SidecarAssetList>
    <SidecarAsset>
      <Id>{SIDECAR_ID}</Id>
      <AssociatedCPLList>
        <CPLId>{UNKNOWN_ID}</CPLId>
      </AssociatedCPLList>
    </SidecarAsset>
  </SidecarAssetList>
</SidecarCompositionMap>"#,
        SCM_ID = SCM_ID,
        SIDECAR_ID = SIDECAR_ID,
        UNKNOWN_ID = UNKNOWN_ID,
    )
}

fn scm_duplicate_asset() -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<SidecarCompositionMap xmlns="http://www.smpte-ra.org/ns/2067-9/2018">
  <Id>{SCM_ID}</Id>
  <IssueDate>2024-01-01T00:00:00</IssueDate>
  <SidecarAssetList>
    <SidecarAsset>
      <Id>{SIDECAR_ID}</Id>
      <AssociatedCPLList>
        <CPLId>{CPL_ID}</CPLId>
      </AssociatedCPLList>
    </SidecarAsset>
    <SidecarAsset>
      <Id>{SIDECAR_ID}</Id>
      <AssociatedCPLList>
        <CPLId>{CPL_ID}</CPLId>
      </AssociatedCPLList>
    </SidecarAsset>
  </SidecarAssetList>
</SidecarCompositionMap>"#,
        SCM_ID = SCM_ID,
        SIDECAR_ID = SIDECAR_ID,
        CPL_ID = CPL_ID,
    )
}

fn scm_duplicate_cpl_id() -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<SidecarCompositionMap xmlns="http://www.smpte-ra.org/ns/2067-9/2018">
  <Id>{SCM_ID}</Id>
  <IssueDate>2024-01-01T00:00:00</IssueDate>
  <SidecarAssetList>
    <SidecarAsset>
      <Id>{SIDECAR_ID}</Id>
      <AssociatedCPLList>
        <CPLId>{CPL_ID}</CPLId>
        <CPLId>{CPL_ID}</CPLId>
      </AssociatedCPLList>
    </SidecarAsset>
  </SidecarAssetList>
</SidecarCompositionMap>"#,
        SCM_ID = SCM_ID,
        SIDECAR_ID = SIDECAR_ID,
        CPL_ID = CPL_ID,
    )
}

fn scm_signer_without_signature() -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<SidecarCompositionMap xmlns="http://www.smpte-ra.org/ns/2067-9/2018">
  <Id>{SCM_ID}</Id>
  <IssueDate>2024-01-01T00:00:00</IssueDate>
  <Signer><X509Data/></Signer>
  <SidecarAssetList>
    <SidecarAsset>
      <Id>{SIDECAR_ID}</Id>
      <AssociatedCPLList>
        <CPLId>{CPL_ID}</CPLId>
      </AssociatedCPLList>
    </SidecarAsset>
  </SidecarAssetList>
</SidecarCompositionMap>"#,
        SCM_ID = SCM_ID,
        SIDECAR_ID = SIDECAR_ID,
        CPL_ID = CPL_ID,
    )
}

fn base_package(scm_xml: String) -> HashMap<String, String> {
    HashMap::from([
        ("ASSETMAP.xml".into(), assetmap("")),
        ("CPL_test.xml".into(), cpl().into()),
        ("SCM_test.xml".into(), scm_xml),
    ])
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn scm_issues(report: &imferno_core::ValidationReport) -> Vec<&imferno_core::ValidationIssue> {
    report
        .critical
        .iter()
        .chain(report.errors.iter())
        .chain(report.warnings.iter())
        .chain(report.info.iter())
        .filter(|i| i.code.starts_with("ST2067-9:"))
        .collect()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

/// ST 2067-9:2018 §7.3: A valid SCM with all references resolved produces no SCM errors.
#[test]
fn scm_valid_no_errors() {
    let report = validate_package(base_package(scm_valid()));
    let issues = scm_issues(&report);
    assert!(
        issues.is_empty(),
        "expected no ST2067-9 issues; got: {:#?}",
        issues
    );
}

/// ST 2067-9:2018 §7.3.1: SCM SidecarAsset.Id is not in the AssetMap.
///
/// Canonical code: `ST2067-9:2018:7.3.1/SidecarAssetNotFound`
#[test]
fn scm_invalid_sidecar_asset_not_found() {
    let report = validate_package(base_package(scm_unknown_asset()));
    let issues = scm_issues(&report);
    assert!(
        issues
            .iter()
            .any(|i| i.code.contains("SidecarAssetNotFound")),
        "expected SidecarAssetNotFound; got: {:#?}",
        issues
    );
}

/// ST 2067-9:2018 §7.3.1.1: SCM CPLId does not reference a known CPL.
///
/// Canonical code: `ST2067-9:2018:7.3.1.1/CplNotFound`
#[test]
fn scm_invalid_cpl_not_found() {
    let report = validate_package(base_package(scm_unknown_cpl()));
    let issues = scm_issues(&report);
    assert!(
        issues.iter().any(|i| i.code.contains("CplNotFound")),
        "expected CplNotFound; got: {:#?}",
        issues
    );
}

/// ST 2067-9:2018 §7.2.3: The same SidecarAsset.Id appears twice in one SCM.
///
/// Canonical code: `ST2067-9:2018:7.2.3/DuplicateAssetId`
#[test]
fn scm_invalid_duplicate_asset_id() {
    let report = validate_package(base_package(scm_duplicate_asset()));
    let issues = scm_issues(&report);
    assert!(
        issues.iter().any(|i| i.code.contains("DuplicateAssetId")),
        "expected DuplicateAssetId; got: {:#?}",
        issues
    );
}

/// ST 2067-9:2018 §7.3.1.1: The same CPLId appears twice in one AssociatedCPLList.
///
/// Canonical code: `ST2067-9:2018:7.3.1.1/DuplicateCplId`
#[test]
fn scm_invalid_duplicate_cpl_id() {
    let report = validate_package(base_package(scm_duplicate_cpl_id()));
    let issues = scm_issues(&report);
    assert!(
        issues.iter().any(|i| i.code.contains("DuplicateCplId")),
        "expected DuplicateCplId; got: {:#?}",
        issues
    );
}

/// ST 2067-9:2018 §7.2.4: Signer element present but Signature element absent.
///
/// Canonical code: `ST2067-9:2018:7.2.4/SignerWithoutSignature`
#[test]
fn scm_invalid_signer_without_signature() {
    let report = validate_package(base_package(scm_signer_without_signature()));
    let issues = scm_issues(&report);
    assert!(
        issues
            .iter()
            .any(|i| i.code.contains("SignerWithoutSignature")),
        "expected SignerWithoutSignature; got: {:#?}",
        issues
    );
}
