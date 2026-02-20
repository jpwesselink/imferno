//! Core Constraints validation tests against real Photon corpus files.
//!
//! Ported from:
//!   - Netflix Photon `IMFCoreConstraintsValidatorTest.java`
//!   - Netflix Photon `IMFCPLValidatorTest.java`
//!
//! These tests verify structural and referential integrity rules defined in
//! SMPTE ST 2067-2. A failure here means the core validator rejected a
//! condition it should reject (or accepted one it should not).

use corpus_tests::{errors, read_cpl};
use imferno_core::validation::validate_cpl;

// ── Ported from IMFCoreConstraintsValidatorTest ───────────────────────────────

/// Mirrors Photon: `IMFCoreConstraintsValidatorTest.invalidCPLmissingED`
///
/// CPL with an empty / absent EssenceDescriptorList must be rejected.
/// ST 2067-2:2020 requires at least one EssenceDescriptor (§9.7.1).
#[test]
fn missing_essence_descriptor_list() {
    let cpl =
        read_cpl("Application2E2020/CPL_46154ef9-7b54-45eb-a85c-00efcb0d47a7_missing_ed.xml");
    let issues = validate_cpl(&cpl);
    assert!(
        !errors(&issues).is_empty(),
        "expected errors for CPL with missing EssenceDescriptorList"
    );
}

// ── Ported from IMFCPLValidatorTest ───────────────────────────────────────────

/// Mirrors Photon: `IMFCPLValidatorTest.invalidCPLfragmentedVirtulTrack`
///
/// A virtual track that appears in some segments but not all is invalid
/// (fragmented virtual track constraint, ST 2067-3 §6.10).
#[test]
fn fragmented_virtual_track() {
    let cpl = read_cpl(
        "Application2E2020/CPL_46154ef9-7b54-45eb-a85c-00efcb0d47a7_fragmented_virtual_track.xml",
    );
    let issues = validate_cpl(&cpl);
    assert!(
        !errors(&issues).is_empty(),
        "expected errors for fragmented virtual track"
    );
}

/// Mirrors Photon: `IMFCPLValidatorTest.invalidCPLfragmentedVirtulTrack_02`
///
/// Second fragmented-virtual-track variant (different segmentation pattern).
#[test]
fn fragmented_virtual_track_02() {
    let cpl = read_cpl(
        "Application2E2020/CPL_46154ef9-7b54-45eb-a85c-00efcb0d47a7_fragmented_virtual_track_02.xml",
    );
    let issues = validate_cpl(&cpl);
    assert!(
        !errors(&issues).is_empty(),
        "expected errors for fragmented virtual track (variant 02)"
    );
}

/// Mirrors Photon: `IMFCPLValidatorTest.invalidCPLdanglingED`
///
/// EssenceDescriptor present in the EssenceDescriptorList but not referenced
/// by any Resource is a dangling-ED condition (ST 2067-2 §9.7.1).
#[test]
fn dangling_essence_descriptor() {
    let cpl =
        read_cpl("Application2E2020/CPL_46154ef9-7b54-45eb-a85c-00efcb0d47a7_dangling_ed.xml");
    let issues = validate_cpl(&cpl);
    assert!(
        !errors(&issues).is_empty(),
        "expected errors for CPL with a dangling EssenceDescriptor"
    );
}
