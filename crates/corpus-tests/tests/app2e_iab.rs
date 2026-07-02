//! ST 2067-201 (IAB Plug-in) constraint validation tests against corpus files.
//!
//! Tests cover both the 2019 and 2021 plug-in validators.
//!
//! Key difference between editions:
//!   - 2019: `ChannelCount` **shall** be the distinguished value `0` (non-zero → error).
//!   - 2021: `ChannelCount` is **ignored** (not checked at all).
//!
//! Canonical code shape: `ST2067-201:{year}:{clause}/{cause}`.

use corpus_tests::{errors, read_cpl};
use imferno_core::validation::{AppIabPlugin2019, AppIabPlugin2021, ConstraintsValidator};

// ── VALID ────────────────────────────────────────────────────────────────────

/// ST 2067-201 §5/§6: A fully conformant IAB CPL (ChannelCount = 0) is valid.
#[test]
fn iab2021_valid_iabsequence() {
    let cpl = read_cpl("IAB/CPL/IAB_CPL_valid_iabsequence.xml");
    let issues = AppIabPlugin2021.validate_cpl(&cpl);
    assert!(
        errors(&issues).is_empty(),
        "expected no errors; got: {:#?}",
        errors(&issues)
    );
}

/// ST 2067-201 §5.9: Missing `AudioSampleRate` produces a Warning, not an Error.
///
/// AUDIT-16: §5.9 says the Audio Sampling Rate item "shall be present" —
/// a missing value is an Error, not the advisory Warning it used to be.
///
/// Canonical code: `ST2067-201:2021:5.9/AudioSamplingRateMissing`
#[test]
fn iab2021_invalid_missing_audiosamplingrate() {
    let cpl = read_cpl("IAB/CPL/IAB_CPL_valid_missing_audiosamplingrate.xml");
    let issues = AppIabPlugin2021.validate_cpl(&cpl);
    assert!(
        errors(&issues)
            .iter()
            .any(|i| i.code.contains("5.9/AudioSamplingRateMissing")),
        "missing AudioSampleRate is a SHALL (§5.9) and must produce an Error; got: {:#?}",
        errors(&issues)
    );
}

/// ST 2067-201 §5.9 (2021): `ChannelCount` is **ignored** in the 2021 edition.
///
/// Non-zero ChannelCount (= 6) is valid under `AppIabPlugin2021`.
/// The same file tested with `AppIabPlugin2019` would produce
/// `ST2067-201:2019:5.9/ChannelCountNotZero`.
#[test]
fn iab2021_valid_non_zero_channel_count() {
    let cpl = read_cpl("IAB/CPL/IAB_CPL_valid_non_zero_essence_channelcount.xml");
    let issues = AppIabPlugin2021.validate_cpl(&cpl);
    assert!(
        errors(&issues).is_empty(),
        "ChannelCount should be ignored in 2021 edition; got errors: {:#?}",
        errors(&issues)
    );
}

// ── INVALID ──────────────────────────────────────────────────────────────────

/// ST 2067-201 §5.9: `Codec` item shall NOT be present.
///
/// Canonical code: `ST2067-201:2021:5.9/CodecForbidden`
#[test]
fn iab2021_invalid_codec_present() {
    let cpl = read_cpl("IAB/CPL/IAB_CPL_invalid_codec_present.xml");
    let issues = AppIabPlugin2021.validate_cpl(&cpl);
    assert!(
        errors(&issues)
            .iter()
            .any(|i| i.code.contains("5.9/CodecForbidden")),
        "expected CodecForbidden; got: {:#?}",
        errors(&issues)
    );
}

/// ST 2067-201 §5.9: Non-homogeneous IAB track files detected.
///
/// Two `IABEssenceDescriptor`s are present: one has `QuantizationBits = 36`
/// (should be 24) and both have `AudioSampleRate = 24000/1001` (not 48000/1).
#[test]
fn iab2021_invalid_homogeneous() {
    let cpl = read_cpl("IAB/CPL/IAB_CPL_invalid_homogeneous.xml");
    let issues = AppIabPlugin2021.validate_cpl(&cpl);
    assert!(
        errors(&issues)
            .iter()
            .any(|i| i.code.contains("5.9/QuantizationBitsInvalid")
                || i.code.contains("5.9/AudioSamplingRateInvalid")),
        "expected QuantizationBitsInvalid or AudioSamplingRateInvalid; got: {:#?}",
        errors(&issues)
    );
}

/// ST 2067-201 §5.9: IABEssenceDescriptor has `AudioSampleRate = 23000/1` (not 48000/1).
///
/// This fixture also has a mismatched IAB/Main Image edit rate per §6.2,
/// but the edit-rate check is not yet implemented; the descriptor error fires first.
///
/// Canonical code: `ST2067-201:2021:5.9/AudioSamplingRateInvalid`
#[test]
fn iab2021_invalid_iabsequence_wrong_editrate_main() {
    let cpl = read_cpl("IAB/CPL/IAB_CPL_invalid_iabsequence_wrong_editrate_main.xml");
    let issues = AppIabPlugin2021.validate_cpl(&cpl);
    assert!(
        errors(&issues)
            .iter()
            .any(|i| i.code.contains("5.9/AudioSamplingRateInvalid")),
        "expected AudioSamplingRateInvalid; got: {:#?}",
        errors(&issues)
    );
}

/// ST 2067-201 §6.2: IABSequence `Resource.SourceEncoding` must reference an
/// `IABEssenceDescriptor`.
///
/// Here the IABSequence resource references an `RGBADescriptor` (no IABEssenceDescriptor
/// exists in the CPL at all).
///
/// Canonical code: `ST2067-201:2021:6.2/IABSequenceSourceEncodingInvalid`
#[test]
fn iab2021_invalid_iabsequence_wrong_trackfile() {
    let cpl = read_cpl("IAB/CPL/IAB_CPL_invalid_iabsequence_wrong_trackfile.xml");
    let issues = AppIabPlugin2021.validate_cpl(&cpl);
    assert!(
        errors(&issues)
            .iter()
            .any(|i| i.code.contains("6.2/IABSequenceSourceEncodingInvalid")),
        "expected IABSequenceSourceEncodingInvalid; got: {:#?}",
        errors(&issues)
    );
}

/// AUDIT-14 regression guard: an `IABSequence` with no `MainAudioSequence`
/// in the same `Segment` is CONFORMANT. The former `MainAudioMissing` rule
/// was invented — no edition of ST 2067-201 (nor ST 2067-2:2020 §6.3.2,
/// "zero or more Audio Virtual Tracks") requires the pairing — and was
/// removed. This fixture must no longer produce that error.
#[test]
fn iab2021_missing_audio_is_conformant() {
    let cpl = read_cpl("IAB/CPL/IAB_CPL_invalid_missing_audio.xml");
    let issues = AppIabPlugin2021.validate_cpl(&cpl);
    assert!(
        !errors(&issues)
            .iter()
            .any(|i| i.code.contains("MainAudioMissing")),
        "invented MainAudioMissing rule (AUDIT-14) must not fire; got: {:#?}",
        errors(&issues)
    );
}

/// ST 2067-201 §5.10.2: exactly one `IABSoundfieldLabelSubDescriptor` shall
/// be present in `IABEssenceDescriptor.SubDescriptors`.
///
/// Canonical code: `ST2067-201:2021:5.10.2/SubDescriptorMissing`
#[test]
fn iab2021_invalid_missing_subdescriptor() {
    let cpl = read_cpl("IAB/CPL/IAB_CPL_invalid_missing_subdescriptor.xml");
    let issues = AppIabPlugin2021.validate_cpl(&cpl);
    assert!(
        errors(&issues)
            .iter()
            .any(|i| i.code.contains("5.10.2/SubDescriptorMissing")),
        "expected SubDescriptorMissing; got: {:#?}",
        errors(&issues)
    );
}

/// ST 2067-201 §6.2: `IABSequence` shall contain at least one `Resource`.
///
/// Canonical code: `ST2067-201:2021:6.2/IABSequenceNoResources`
#[test]
fn iab2021_invalid_no_resource() {
    let cpl = read_cpl("IAB/CPL/IAB_CPL_invalid_no_resource.xml");
    let issues = AppIabPlugin2021.validate_cpl(&cpl);
    assert!(
        errors(&issues)
            .iter()
            .any(|i| i.code.contains("6.2/IABSequenceNoResources")),
        "expected IABSequenceNoResources; got: {:#?}",
        errors(&issues)
    );
}

/// ST 2067-201 §5.9: `QuantizationBits` shall be 24 (got 36).
///
/// Canonical code: `ST2067-201:2021:5.9/QuantizationBitsInvalid`
#[test]
fn iab2021_invalid_wrong_bitdepth() {
    let cpl = read_cpl("IAB/CPL/IAB_CPL_invalid_wrong_bitdepth.xml");
    let issues = AppIabPlugin2021.validate_cpl(&cpl);
    assert!(
        errors(&issues)
            .iter()
            .any(|i| i.code.contains("5.9/QuantizationBitsInvalid")),
        "expected QuantizationBitsInvalid; got: {:#?}",
        errors(&issues)
    );
}

/// ST 2067-201 §5.9 (2019 only): `ChannelCount` shall be the distinguished value `0`.
///
/// The same file passes under `AppIabPlugin2021` (ChannelCount is ignored in 2021).
///
/// Canonical code: `ST2067-201:2019:5.9/ChannelCountNotZero`
#[test]
fn iab2019_invalid_wrong_channel_count() {
    let cpl = read_cpl("IAB/CPL/IAB_CPL_invalid_wrong_channel_count.xml");
    let issues = AppIabPlugin2019.validate_cpl(&cpl);
    assert!(
        errors(&issues)
            .iter()
            .any(|i| i.code.contains("5.9/ChannelCountNotZero")),
        "expected ChannelCountNotZero (2019 edition only); got: {:#?}",
        errors(&issues)
    );
}

/// ST 2067-201 §5.9: "If present, the Electro-Spatial Formulation item …
/// shall be set to a value of 15" — the fixture carries value 0.
/// (AUDIT-15: presence itself is legal; the old Forbidden rule inverted
/// the spec.)
///
/// Canonical code: `ST2067-201:2021:5.9/ElectrospatialFormulationInvalid`
#[test]
fn iab2021_invalid_wrong_electro_spatial_formulation() {
    let cpl = read_cpl("IAB/CPL/IAB_CPL_invalid_wrong_electro_spatial_formulation.xml");
    let issues = AppIabPlugin2021.validate_cpl(&cpl);
    assert!(
        errors(&issues)
            .iter()
            .any(|i| i.code.contains("5.9/ElectrospatialFormulationInvalid")),
        "expected ElectrospatialFormulationInvalid (value 0 != 15); got: {:#?}",
        errors(&issues)
    );
}

/// ST 2067-201 §5.9 Table 4.5: `ContainerFormat` shall be the IAB essence
/// container UL (`urn:smpte:ul:060e2b34.0401010d.0d010301.021d0101`).
///
/// Canonical code: `ST2067-201:2021:5.9/EssenceContainerInvalid`
#[test]
fn iab2021_invalid_wrong_essence_container_ul() {
    let cpl = read_cpl("IAB/CPL/IAB_CPL_invalid_wrong_essence_container_ul.xml");
    let issues = AppIabPlugin2021.validate_cpl(&cpl);
    assert!(
        errors(&issues)
            .iter()
            .any(|i| i.code.contains("5.9/EssenceContainerInvalid")),
        "expected EssenceContainerInvalid; got: {:#?}",
        errors(&issues)
    );
}

/// ST 2067-201 §5.9: `SoundCompression` shall be the IAB UL
/// (`urn:smpte:ul:060e2b34.04010105.0e090604.00000000`).
///
/// Canonical code: `ST2067-201:2021:5.9/SoundCompressionInvalid`
#[test]
fn iab2021_invalid_wrong_soundcompression() {
    let cpl = read_cpl("IAB/CPL/IAB_CPL_invalid_wrong_soundcompression.xml");
    let issues = AppIabPlugin2021.validate_cpl(&cpl);
    assert!(
        errors(&issues)
            .iter()
            .any(|i| i.code.contains("5.9/SoundCompressionInvalid")),
        "expected SoundCompressionInvalid; got: {:#?}",
        errors(&issues)
    );
}

/// ST 2067-201 §5.9: `IABSoundfieldLabelSubDescriptor` has wrong values:
/// `MCATagSymbol` absent, `MCATagName = "I A B"` (not `"IAB"`),
/// and `MCALabelDictionaryID` points to wrong UL.
///
/// Canonical codes: `ST2067-201:2021:5.10.4/MCATagSymbolMissing`,
/// `ST2067-201:2021:5.10.4/MCATagNameInvalid`, `ST2067-201:2021:5.10.4/MCALabelDictionaryIDInvalid`
#[test]
fn iab2021_invalid_wrong_subdescriptor_values() {
    let cpl = read_cpl("IAB/CPL/IAB_CPL_invalid_wrong_subdescriptor_values.xml");
    let issues = AppIabPlugin2021.validate_cpl(&cpl);
    assert!(
        errors(&issues).iter().any(|i| {
            i.code.contains("5.10.4/MCATagSymbolMissing")
                || i.code.contains("5.10.4/MCATagNameInvalid")
                || i.code.contains("5.10.4/MCALabelDictionaryIDInvalid")
        }),
        "expected subdescriptor value error (MCATagSymbolMissing / MCATagNameInvalid / \
         MCALabelDictionaryIDInvalid); got: {:#?}",
        errors(&issues)
    );
}

/// ST 2067-201 §5.9: `IABEssenceDescriptor.SubDescriptors` contains a
/// `SoundfieldGroupLabelSubDescriptor` (not `IABSoundfieldLabelSubDescriptor`).
///
/// Canonical code: `ST2067-201:2021:5.9/SubDescriptorMissing`
#[test]
fn iab2021_invalid_wrong_subdescriptor() {
    let cpl = read_cpl("IAB/CPL/IAB_CPL_invalid_wrong_subdescriptor.xml");
    let issues = AppIabPlugin2021.validate_cpl(&cpl);
    assert!(
        errors(&issues)
            .iter()
            .any(|i| i.code.contains("5.10.2/SubDescriptorMissing")),
        "expected SubDescriptorMissing (wrong sub-descriptor type); got: {:#?}",
        errors(&issues)
    );
}
