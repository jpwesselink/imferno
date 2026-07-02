//! App2E 2021 constraint validation tests against the vendored corpus.
//!
//!
//! These tests verify that the full validation pipeline (core constraints +
//! App2E) correctly accepts valid CPLs and rejects invalid ones.

use corpus_tests::{errors, read_cpl};
use imferno_core::validation::validate_cpl;

// ── VALID ────────────────────────────────────────────────────────────────────

///
/// P3 D65 / PQ / JPEG 2000 HT / 10-bit RGB / FullFrame / 1920×1080.
/// Core constraints + App2E should both pass.
#[test]
fn app2e2021_valid_cpl() {
    let cpl = read_cpl("Application2E2021/CPL_b2e1ace2-9c7d-4c12-b2f7-24bde303869e.xml");
    let issues = validate_cpl(&cpl);
    assert!(
        errors(&issues).is_empty(),
        "expected no errors; got: {:#?}",
        errors(&issues)
    );
}

// ── INVALID ───────────────────────────────────────────────────────────────────

///
/// Interlaced frame layout is prohibited by App2E (progressive-scan only).
#[test]
fn app2e2021_bad_frame_structure() {
    let cpl = read_cpl(
        "Application2E2021/CPL_b2e1ace2-9c7d-4c12-b2f7-24bde303869e-bad-frame-structure.xml",
    );
    let issues = validate_cpl(&cpl);
    assert!(
        !errors(&issues).is_empty(),
        "expected errors for interlaced frame structure"
    );
}

///
/// The validator rejects PictureCompression UL `03010101` for this CPL.
/// UL `03010101` (byte[14]=01, byte[15]=01) is not a recognized BCP sub-level
/// (valid BCP sub-levels are 0x11–0x17). It maps to `Unknown` → not J2K family → error.
#[test]
fn app2e2021_bad_codec() {
    let cpl = read_cpl("Application2E2021/CPL_b2e1ace2-9c7d-4c12-b2f7-24bde303869e-bad-codec.xml");
    let issues = validate_cpl(&cpl);
    assert!(
        !errors(&issues).is_empty(),
        "expected errors for non-JPEG 2000 codec"
    );
}

///
/// Invalid colorimetry system (mismatched CP/TC/CE triplet per Table 3).
#[test]
fn app2e2021_bad_color() {
    let cpl = read_cpl("Application2E2021/CPL_b2e1ace2-9c7d-4c12-b2f7-24bde303869e-bad-color.xml");
    let issues = validate_cpl(&cpl);
    assert!(
        !errors(&issues).is_empty(),
        "expected errors for invalid colorimetry system"
    );
}

///
/// CC-Namespaces CPL uses prefixed XML namespaces (ns3/ns4/ns6) and declares an
/// App2E application identification. Namespace stripping must succeed and
/// App2E validation must pass.
///
/// Note: The reference test asserts 2 errors (namespace mismatch warnings counted as errors).
/// Our validator models namespace mismatches as warnings, not errors.
///
/// The fixture also carries an `ns6:IABSequence` with no accompanying
/// MainAudioSequence. Since IAB plug-in dispatch was wired up
/// (sequence-presence dispatch), the ST 2067-201 §6.2 rule correctly
/// fires on it — that error is expected and pinned here; anything else
/// is a regression.
#[test]
fn app2e2021_cc_namespaces() {
    let cpl =
        read_cpl("Application2E2021/CPL_3714715a-af0c-4a89-9cc9-c99f61e7eb6d_CC-Namespaces.xml");
    let issues = validate_cpl(&cpl);
    let errs = errors(&issues);
    assert!(
        errs.iter()
            .all(|i| i.code == "ST2067-201:2021:6.2/MainAudioMissing"),
        "only the genuine IAB §6.2 MainAudioMissing error is expected for the \
         CC-Namespaces CPL; got: {errs:#?}"
    );
    assert_eq!(
        errs.len(),
        1,
        "expected exactly the IAB §6.2 error; got: {errs:#?}"
    );
}
