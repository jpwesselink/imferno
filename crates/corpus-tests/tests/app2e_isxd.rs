//! ST 2067-202 (ISXD Plug-in) constraint validation tests against corpus files.
//!
//! Canonical code shape: `ST2067-202:{year}:{clause}/{cause}`.

use corpus_tests::{errors, read_cpl};
use imferno_core::validation::{AppIsxdPlugin2022, ConstraintsValidator};

// ── VALID ─────────────────────────────────────────────────────────────────────

/// ST 2067-202: A fully conformant ISXD CPL produces no errors.
#[test]
fn isxd_valid_cpl() {
    let cpl = read_cpl("ISXD/CPL_ISXD_TEST_1.xml");
    let issues = AppIsxdPlugin2022.validate_cpl(&cpl);
    assert!(
        errors(&issues).is_empty(),
        "expected no errors; got: {:#?}",
        errors(&issues)
    );
}

// ── INVALID ───────────────────────────────────────────────────────────────────

/// ST 2067-202 §5: ISXDDataEssenceDescriptor shall have ContainerConstraintsSubDescriptor.
///
/// Canonical code: `ST2067-202:2022:5/SubDescriptorMissing`
#[test]
fn isxd_missing_sub_descriptor() {
    let cpl = read_cpl("ISXD/CPL_ISXD_TEST_SubDescriptorMissingTest.xml");
    let issues = AppIsxdPlugin2022.validate_cpl(&cpl);
    assert!(
        errors(&issues)
            .iter()
            .any(|i| i.code.contains("5/SubDescriptorMissing")),
        "expected SubDescriptorMissing; got: {:#?}",
        errors(&issues)
    );
}

/// ST 2067-202 §6: Resources in the same ISXDSequence shall reference descriptors
/// with the same NamespaceURI.
///
/// Canonical code: `ST2067-202:2022:6/NamespaceUriMismatch`
#[test]
fn isxd_namespace_uri_mismatch() {
    let cpl = read_cpl("ISXD/CPL_ISXD_TEST_NamespaceUriMismatch.xml");
    let issues = AppIsxdPlugin2022.validate_cpl(&cpl);
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
/// Canonical code: `ST2067-202:2022:6/ISXDSequenceNoResources`
#[test]
fn isxd_sequence_no_resources() {
    let cpl = read_cpl("ISXD/CPL_ISXD_TEST_EmptyIsxdTrack.xml");
    let issues = AppIsxdPlugin2022.validate_cpl(&cpl);
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
/// Canonical code: `ST2067-202:2022:6/ISXDSequenceSourceEncodingInvalid`
#[test]
fn isxd_sequence_wrong_source_encoding() {
    let cpl = read_cpl("ISXD/CPL_ISXD_TEST_NonIsxdResource.xml");
    let issues = AppIsxdPlugin2022.validate_cpl(&cpl);
    assert!(
        errors(&issues)
            .iter()
            .any(|i| i.code.contains("6/ISXDSequenceSourceEncodingInvalid")),
        "expected ISXDSequenceSourceEncodingInvalid; got: {:#?}",
        errors(&issues)
    );
}

/// Edit rate mismatch in ISXD resource is caught by core CPL validation.
///
/// The resource EditRate (48 1) does not produce an integer number of
/// Composition Edit Units at 24000/1001, so ST 2067-3 core fires an error.
#[test]
fn isxd_edit_rate_mismatch_produces_error() {
    let cpl = read_cpl("ISXD/CPL_ISXD_TEST_EditRateMismatch.xml");
    let issues = AppIsxdPlugin2022.validate_cpl(&cpl);
    assert!(
        !errors(&issues).is_empty(),
        "expected at least one error for edit rate mismatch; got no errors"
    );
}
