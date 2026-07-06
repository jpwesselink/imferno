//! ST 2067-202 (ISXD Plug-in) constraint validation tests against corpus files.
//!
//! Canonical code shape: `ST2067-202:{year}:{clause}/{cause}`.

use corpus_tests::{errors, read_cpl};
use imferno_core::validation::{AppIsxdPlugin2023, ConstraintsValidator};

// ── VALID ─────────────────────────────────────────────────────────────────────

/// ST 2067-202: A fully conformant ISXD CPL produces no errors.
#[test]
fn isxd_valid_cpl() {
    let cpl = read_cpl("ISXD/CPL_ISXD_TEST_1.xml");
    let issues = AppIsxdPlugin2023.validate_cpl(&cpl);
    assert!(
        errors(&issues).is_empty(),
        "expected no errors; got: {:#?}",
        errors(&issues)
    );
}

// ── INVALID ───────────────────────────────────────────────────────────────────

/// AUDIT-19 regression guard: an ISXDDataEssenceDescriptor WITHOUT a
/// `ContainerConstraintsSubDescriptor` is CONFORMANT to ST 2067-202.
/// "ContainerConstraints" appears nowhere in -202 prose; §9.2 says
/// implementations "may extend" the descriptor and "shall ignore
/// unrecognized SubDescriptors" — the deleted `SubDescriptorMissing`
/// rule belonged to the ST 2127 lineage (ST 2067-203), not -202.
#[test]
fn isxd_missing_sub_descriptor_is_conformant() {
    let cpl = read_cpl("ISXD/CPL_ISXD_TEST_SubDescriptorMissingTest.xml");
    let issues = AppIsxdPlugin2023.validate_cpl(&cpl);
    assert!(
        !errors(&issues)
            .iter()
            .any(|i| i.code.contains("SubDescriptorMissing")),
        "deleted SubDescriptorMissing rule (AUDIT-19) must not fire; got: {:#?}",
        errors(&issues)
    );
}

/// ST 2067-202 §6: Resources in the same ISXDSequence shall reference descriptors
/// with the same NamespaceURI.
///
/// Canonical code: `ST2067-202:2023:6/NamespaceUriMismatch`
#[test]
fn isxd_namespace_uri_mismatch() {
    let cpl = read_cpl("ISXD/CPL_ISXD_TEST_NamespaceUriMismatch.xml");
    let issues = AppIsxdPlugin2023.validate_cpl(&cpl);
    assert!(
        errors(&issues)
            .iter()
            .any(|i| i.code.contains("6/NamespaceUriMismatch")),
        "expected NamespaceUriMismatch; got: {:#?}",
        errors(&issues)
    );
}

/// ST 2067-202 §6: ISXDSequence shall contain at least one Resource.
///
/// Canonical code: `ST2067-202:2023:6/ISXDSequenceNoResources`
#[test]
fn isxd_sequence_no_resources() {
    let cpl = read_cpl("ISXD/CPL_ISXD_TEST_EmptyIsxdTrack.xml");
    let issues = AppIsxdPlugin2023.validate_cpl(&cpl);
    assert!(
        errors(&issues)
            .iter()
            .any(|i| i.code.contains("6/ISXDSequenceNoResources")),
        "expected ISXDSequenceNoResources; got: {:#?}",
        errors(&issues)
    );
}

/// ST 2067-202 §6: ISXDSequence Resource.SourceEncoding shall reference an
/// ISXDDataEssenceDescriptor (here it references a WAVEPCMDescriptor).
///
/// Canonical code: `ST2067-202:2023:6/ISXDSequenceSourceEncodingInvalid`
#[test]
fn isxd_sequence_wrong_source_encoding() {
    let cpl = read_cpl("ISXD/CPL_ISXD_TEST_NonIsxdResource.xml");
    let issues = AppIsxdPlugin2023.validate_cpl(&cpl);
    assert!(
        errors(&issues)
            .iter()
            .any(|i| i.code.contains("6/ISXDSequenceSourceEncodingInvalid")),
        "expected ISXDSequenceSourceEncodingInvalid; got: {:#?}",
        errors(&issues)
    );
}

/// ST 2067-202 §9.3: DataEssenceCoding shall be the UTF-8 Text Data Essence
/// Coding UL (Table 6). The fixture carries the ISXD *container* UL instead
/// (AUDIT-21 gap fix).
///
/// Canonical code: `ST2067-202:2023:9.3/DataEssenceCodingInvalid`
#[test]
fn isxd_wrong_data_essence_coding() {
    let cpl = read_cpl("ISXD/CPL_ISXD_TEST_WrongDataEssenceCoding.xml");
    let issues = AppIsxdPlugin2023.validate_cpl(&cpl);
    assert!(
        errors(&issues)
            .iter()
            .any(|i| i.code.contains("9.3/DataEssenceCodingInvalid")),
        "expected DataEssenceCodingInvalid (§9.3 Table 6); got: {:#?}",
        errors(&issues)
    );
}

/// ST 2067-202 §6: "A Composition ... that references an ISXD Track File,
/// shall contain one or more ISXD Virtual Tracks." The fixture references
/// the ISXDDataEssenceDescriptor from a MainAudioSequence and carries no
/// ISXDSequence (AUDIT-21 gap fix).
///
/// Canonical code: `ST2067-202:2023:6/ISXDVirtualTrackMissing`
#[test]
fn isxd_referenced_without_isxd_virtual_track() {
    let cpl = read_cpl("ISXD/CPL_ISXD_TEST_NoIsxdTrack.xml");
    let issues = AppIsxdPlugin2023.validate_cpl(&cpl);
    assert!(
        errors(&issues)
            .iter()
            .any(|i| i.code.contains("6/ISXDVirtualTrackMissing")),
        "expected ISXDVirtualTrackMissing (§6); got: {:#?}",
        errors(&issues)
    );
}

/// ST 2067-202 §6: "The Edit Rate of an ISXD Virtual Track shall be equal
/// to the Edit Rate of the Main Image Virtual Track." The resource declares
/// EditRate 48/1 against a 24/1 Main Image track (AUDIT-21 gap fix — this
/// fixture previously only errored via the invented AUDIT-19 rule).
///
/// Canonical code: `ST2067-202:2023:6/EditRateMismatch`
#[test]
fn isxd_edit_rate_mismatch_produces_error() {
    let cpl = read_cpl("ISXD/CPL_ISXD_TEST_EditRateMismatch.xml");
    let issues = AppIsxdPlugin2023.validate_cpl(&cpl);
    assert!(
        errors(&issues)
            .iter()
            .any(|i| i.code.contains("6/EditRateMismatch")),
        "expected ISXD §6 EditRateMismatch; got: {:#?}",
        errors(&issues)
    );
}
