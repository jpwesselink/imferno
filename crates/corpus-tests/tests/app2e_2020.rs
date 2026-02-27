//! App2E 2020 constraint validation tests against real Photon corpus files.
//!
//! Ported from: Netflix Photon `IMFApp2E2020ConstraintsValidatorTest.java`
//!
//! These tests verify that the App2E 2020 validator correctly accepts valid CPLs
//! and rejects invalid ones. A failure here means the validation logic is wrong,
//! not just the mapping.

use corpus_tests::{errors, read_cpl};
use imferno_core::validation::{App2E2020, ConstraintsValidator};

// ── VALID ────────────────────────────────────────────────────────────────────

/// Mirrors Photon: `IMFApp2E2020ConstraintsValidatorTest.ValidCPL`
///
/// Baseline App2E 2020 CPL with MaxCLL/MaxFALL present, JPEG 2000, BT.2100 PQ.
#[test]
fn app2e2020_valid_cpl() {
    let cpl = read_cpl("Application2E2020/CPL_46154ef9-7b54-45eb-a85c-00efcb0d47a7.xml");
    let issues = App2E2020.validate_cpl(&cpl);
    assert!(
        errors(&issues).is_empty(),
        "expected no App2E errors; got: {:#?}",
        errors(&issues)
    );
}

// ── INVALID — JPEG 2000 profile ──────────────────────────────────────────────

/// Mirrors Photon: `IMFApp2E2020ConstraintsValidatorTest.InvalidCPLBadJ2kProfile_01`
///
/// 2K J2K profile (stored width ≤ 2048) is not valid for 4K content (3840×2160).
/// ST 2067-21 §6.2.5 / Photon `IMFApp2E2020ConstraintsValidator.validateJ2KProfile()`.
#[test]
fn app2e2020_bad_j2k_profile_2k() {
    let cpl =
        read_cpl("Application2E2020/CPL_46154ef9-7b54-45eb-a85c-00efcb0d47a7_2k_j2k_profile.xml");
    let issues = App2E2020.validate_cpl(&cpl);
    assert!(
        !errors(&issues).is_empty(),
        "expected App2E errors for bad 2K J2K profile"
    );
}

/// Mirrors Photon: `IMFApp2E2020ConstraintsValidatorTest.InvalidCPLBadJ2kProfile_02`
///
/// 8K J2K profile UL (byte[14]=07) is not a recognized App2E profile.
/// Photon rejects it; our validator maps it to Unknown → not J2K family → error.
#[test]
fn app2e2020_bad_j2k_profile_8k() {
    let cpl =
        read_cpl("Application2E2020/CPL_46154ef9-7b54-45eb-a85c-00efcb0d47a7_8k_j2k_profile.xml");
    let issues = App2E2020.validate_cpl(&cpl);
    assert!(
        !errors(&issues).is_empty(),
        "expected App2E errors for bad 8K J2K profile"
    );
}

/// Mirrors Photon: `IMFApp2E2020ConstraintsValidatorTest.InvalidCPLBadJ2kProfile_03`
///
/// HT-J2K (ISO 15444-15, UL byte[14]=08) is not permitted by App2E 2020.
/// Per Photon `IMFApp2E2020ConstraintsValidator`, HT was added only in 2021.
#[test]
fn app2e2020_bad_j2k_profile_ht() {
    let cpl =
        read_cpl("Application2E2020/CPL_46154ef9-7b54-45eb-a85c-00efcb0d47a7_ht_j2k_profile.xml");
    let issues = App2E2020.validate_cpl(&cpl);
    assert!(
        !errors(&issues).is_empty(),
        "expected App2E errors for bad HT J2K profile"
    );
}

/// Mirrors Photon: `IMFApp2E2020ConstraintsValidatorTest.InvalidCPLBadJ2kProfile_04`
///
/// 4K J2K profile (stored width > 2048) does not support HD resolution (1920×1080).
/// Photon: `validateJ2KProfile()` — IMF 4K requires width in (2048, 4096].
#[test]
fn app2e2020_bad_j2k_profile_hd_resolution() {
    let cpl =
        read_cpl("Application2E2020/CPL_46154ef9-7b54-45eb-a85c-00efcb0d47a7_HD_resolution.xml");
    let issues = App2E2020.validate_cpl(&cpl);
    assert!(
        !errors(&issues).is_empty(),
        "expected App2E errors for HD resolution in a 4K CPL"
    );
}

/// Mirrors Photon: `IMFApp2E2020ConstraintsValidatorTest.InvalidCPLBadJ2kProfile_05`
///
/// Broadcast Contribution Profile (BCP) does not support DCI 4K width (4096 > 3840).
/// Photon: `validateJ2KProfile()` — BCP requires width in (0, 3840].
#[test]
fn app2e2020_bad_j2k_profile_dci4k_bcp() {
    let cpl = read_cpl("Application2E2020/CPL_46154ef9-7b54-45eb-a85c-00efcb0d47a7_DCI4k_bcp.xml");
    let issues = App2E2020.validate_cpl(&cpl);
    assert!(
        !errors(&issues).is_empty(),
        "expected App2E errors for DCI 4K BCP"
    );
}

// ── INVALID — codec ───────────────────────────────────────────────────────────

/// Mirrors Photon: `IMFApp2E2020ConstraintsValidatorTest.InvalidCPLBadPictureCompression`
///
/// VC-5 picture coding is not permitted by App2E (JPEG 2000 required).
#[test]
fn app2e2020_bad_picture_compression_vc5() {
    let cpl = read_cpl(
        "Application2E2020/CPL_46154ef9-7b54-45eb-a85c-00efcb0d47a7_vc5_picture_coding.xml",
    );
    let issues = App2E2020.validate_cpl(&cpl);
    assert!(
        !errors(&issues).is_empty(),
        "expected App2E errors for VC-5 picture compression"
    );
}
