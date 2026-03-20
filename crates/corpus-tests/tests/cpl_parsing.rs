//! CPL parse-fidelity tests against real Photon corpus files.
//!
//! Ported from: Netflix Photon `CompositionTest.java`
//! These tests verify that the parser correctly extracts every typed field
//! from real-world CPL documents. A parse failure here means we broke the
//! mapping, not just the validator.

use corpus_tests::read_cpl;
use imferno_core::assetmap::ImfUuid;
use imferno_core::cpl::{
    ContentKind, CplNamespace, EditRate, LanguageTag, McaTagSymbol, VideoCodec,
};

// ── MERIDIAN (Netflix Photon reference package, 2013 namespace) ─────────────

/// Mirrors Photon: `CompositionTest.compositionPositiveTest`
///
/// MERIDIAN is the primary Photon reference package.  Every typed field
/// must deserialise without loss.
#[test]
fn meridian_parses_all_fields() {
    let cpl =
        read_cpl("MERIDIAN_Netflix_Photon_161006/CPL_0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85.xml");

    assert_eq!(
        cpl.id,
        ImfUuid::parse("urn:uuid:0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85").unwrap()
    );
    assert_eq!(cpl.content_title.text, "MERIDIAN");
    assert_eq!(cpl.content_kind, ContentKind::Test);
    assert_eq!(cpl.edit_rate, Some(EditRate::new(60000, 1001)));

    let edl = cpl
        .essence_descriptor_list
        .as_ref()
        .expect("missing EssenceDescriptorList");
    assert_eq!(edl.essence_descriptors.len(), 2);

    // Video: RGBADescriptor — 3840×2160 JPEG 2000 with Dolby Vision sub-descriptor
    let video_ed = &edl.essence_descriptors[0];
    assert_eq!(
        video_ed.id,
        ImfUuid::parse("urn:uuid:4c109a4a-9711-4d8f-bd89-6449a0fc6738").unwrap()
    );
    let rgba = video_ed
        .rgba_descriptor
        .as_ref()
        .expect("missing RGBADescriptor");
    assert_eq!(rgba.display_width, Some(3840));
    assert_eq!(rgba.display_height, Some(2160));
    assert_eq!(rgba.picture_compression, Some(VideoCodec::Jpeg2000Imf4k));
    let phdr = rgba
        .sub_descriptors
        .as_ref()
        .and_then(|sd| sd.phdr_metadata_track_sub_descriptor.as_ref());
    assert!(
        phdr.is_some(),
        "missing PHDRMetadataTrackSubDescriptor (Dolby Vision)"
    );

    // Audio: WAVEPCMDescriptor — 5.1 / 24-bit / English
    let audio_ed = &edl.essence_descriptors[1];
    assert_eq!(
        audio_ed.id,
        ImfUuid::parse("urn:uuid:0a2446df-ed83-4a79-89e7-85e5c52d942f").unwrap()
    );
    let wave = audio_ed
        .wave_pcm_descriptor
        .as_ref()
        .expect("missing WAVEPCMDescriptor");
    assert_eq!(wave.channel_count, Some(6));
    assert_eq!(wave.quantization_bits, Some(24));
    let sf = wave
        .sub_descriptors
        .as_ref()
        .and_then(|sd| sd.soundfield_group_label_sub_descriptor.as_ref())
        .expect("missing SoundfieldGroupLabelSubDescriptor");
    assert_eq!(sf.mca_tag_symbol, Some(McaTagSymbol::Sg51));
    assert_eq!(sf.mca_tag_name.as_deref(), Some("5.1"));
    assert_eq!(sf.rfc5646_spoken_language, Some(LanguageTag::new("en")));

    // Timeline
    let segment = &cpl.segment_list.segments[0];
    assert!(!segment.sequence_list.marker_sequences.is_empty());
    assert_eq!(segment.sequence_list.main_image_sequences.len(), 1);
    assert_eq!(segment.sequence_list.main_audio_sequences.len(), 1);

    let video_resource = &segment.sequence_list.main_image_sequences[0]
        .resource_list
        .resources[0];
    assert_eq!(
        video_resource.source_encoding,
        Some(ImfUuid::parse("urn:uuid:4c109a4a-9711-4d8f-bd89-6449a0fc6738").unwrap())
    );
    assert_eq!(video_resource.source_duration, Some(40));

    let audio_resource = &segment.sequence_list.main_audio_sequences[0]
        .resource_list
        .resources[0];
    assert_eq!(audio_resource.edit_rate, Some(EditRate::new(48000, 1)));

    // ExtensionProperties
    let ext = cpl
        .extension_properties
        .as_ref()
        .expect("missing ExtensionProperties");
    assert_eq!(ext.max_cll, Some(0));
    assert_eq!(ext.max_fall, Some(0));

    // LocaleList
    let ll = cpl.locale_list.as_ref().expect("missing LocaleList");
    let lang_list = ll.locales[0]
        .language_list
        .as_ref()
        .expect("missing LanguageList");
    assert_eq!(lang_list.languages, vec![LanguageTag::new("en")]);
    let region_list = ll.locales[0]
        .region_list
        .as_ref()
        .expect("missing RegionList");
    assert_eq!(region_list.regions, vec!["021"]);
}

/// Mirrors Photon: `CompositionTest.compositionPositiveTest` (namespace check)
#[test]
fn meridian_namespace_is_2013() {
    let cpl =
        read_cpl("MERIDIAN_Netflix_Photon_161006/CPL_0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85.xml");
    assert_eq!(cpl.namespace, CplNamespace::Smpte2067_3_2013);
}

// ── IAB (Immersive Audio Bitstream) ─────────────────────────────────────────

/// Mirrors Photon: `IABCompositionTest`
#[test]
fn iab_cpl_parses_all_descriptors() {
    let cpl = read_cpl("IAB/CompleteIMP/CPL_e0265fda-cb35-4e35-a4e4-4f44d82d2a52.xml");

    assert_eq!(
        cpl.id,
        ImfUuid::parse("urn:uuid:e0265fda-cb35-4e35-a4e4-4f44d82d2a52").unwrap()
    );
    assert_eq!(cpl.edit_rate, Some(EditRate::new(24000, 1001)));

    let edl = cpl
        .essence_descriptor_list
        .as_ref()
        .expect("missing EssenceDescriptorList");
    assert_eq!(edl.essence_descriptors.len(), 3);

    // Video: RGBA 1920×1080
    let rgba = edl.essence_descriptors[0]
        .rgba_descriptor
        .as_ref()
        .expect("missing RGBADescriptor");
    assert_eq!(rgba.display_width, Some(1920));
    assert_eq!(rgba.display_height, Some(1080));

    // Audio: stereo / English
    let wave = edl.essence_descriptors[1]
        .wave_pcm_descriptor
        .as_ref()
        .expect("missing WAVEPCMDescriptor");
    assert_eq!(wave.channel_count, Some(2));
    let sf = wave
        .sub_descriptors
        .as_ref()
        .and_then(|sd| sd.soundfield_group_label_sub_descriptor.as_ref())
        .expect("missing SoundfieldGroupLabelSubDescriptor");
    assert_eq!(sf.rfc5646_spoken_language, Some(LanguageTag::new("en")));

    // IAB (Atmos)
    let iab = edl.essence_descriptors[2]
        .iab_essence_descriptor
        .as_ref()
        .expect("missing IABEssenceDescriptor");
    assert_eq!(iab.channel_count, Some(0)); // object-based, no fixed channel count
    let iab_sf = iab
        .sub_descriptors
        .as_ref()
        .and_then(|sd| sd.iab_soundfield_label_sub_descriptor.as_ref())
        .expect("missing IABSoundfieldLabelSubDescriptor");
    assert_eq!(iab_sf.mca_tag_symbol, Some(McaTagSymbol::Iab));
    assert_eq!(iab_sf.rfc5646_spoken_language, Some(LanguageTag::new("en")));

    // Sequences
    let segment = &cpl.segment_list.segments[0];
    assert_eq!(segment.sequence_list.main_image_sequences.len(), 1);
    assert_eq!(segment.sequence_list.main_audio_sequences.len(), 1);
    assert_eq!(segment.sequence_list.iab_sequences.len(), 1);
}

// ── Netflix/Sony Plugfest 2015 (CDCI + timed text, 2016 namespace) ──────────

/// Mirrors Photon: `CompositionTest.compositionWithMultipleImageResourcesPositiveTest`
#[test]
fn plugfest_cpl_parses_cdci_and_timed_text() {
    let cpl = read_cpl("Netflix_Sony_Plugfest_2015/CPL_BLACKL_202_HD_REC709_178_ENG_fe8cf2f4-1bcd-4145-8f72-6775af4038c4.xml");

    assert_eq!(cpl.content_kind, ContentKind::Episode);

    let edl = cpl
        .essence_descriptor_list
        .as_ref()
        .expect("missing EssenceDescriptorList");
    // CDCI video, 5.1 audio, stereo audio, DCTimedText
    assert_eq!(edl.essence_descriptors.len(), 4);

    // CDCI video — 4K 10-bit
    let cdci = edl.essence_descriptors[0]
        .cdci_descriptor
        .as_ref()
        .expect("missing CDCIDescriptor");
    assert_eq!(cdci.stored_width, Some(3840));
    assert_eq!(cdci.stored_height, Some(2160));
    assert_eq!(cdci.component_depth, Some(10));

    // 5.1 audio
    let wave_51 = edl.essence_descriptors[1]
        .wave_pcm_descriptor
        .as_ref()
        .expect("missing 5.1 WAVEPCMDescriptor");
    assert_eq!(wave_51.channel_count, Some(6));
    let sf_51 = wave_51
        .sub_descriptors
        .as_ref()
        .and_then(|sd| sd.soundfield_group_label_sub_descriptor.as_ref())
        .expect("missing 5.1 SoundfieldGroupLabelSubDescriptor");
    assert_eq!(sf_51.mca_tag_symbol, Some(McaTagSymbol::Sg51));
    assert_eq!(
        sf_51.rfc5646_spoken_language,
        Some(LanguageTag::new("en-US"))
    );

    // Stereo audio
    let wave_stereo = edl.essence_descriptors[2]
        .wave_pcm_descriptor
        .as_ref()
        .expect("missing stereo WAVEPCMDescriptor");
    assert_eq!(wave_stereo.channel_count, Some(2));

    // Timed text
    let tt = edl.essence_descriptors[3]
        .dc_timed_text_descriptor
        .as_ref()
        .expect("missing DCTimedTextDescriptor");
    assert!(tt.linked_track_id.is_some());

    // Sequences
    let segment = &cpl.segment_list.segments[0];
    assert!(!segment.sequence_list.marker_sequences.is_empty());
    assert_eq!(segment.sequence_list.main_image_sequences.len(), 1);
    assert_eq!(segment.sequence_list.main_audio_sequences.len(), 2);
}
