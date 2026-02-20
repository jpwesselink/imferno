//! SourceAsset extraction from IMF Composition Playlists
//!
//! This module replaces the TypeScript compositionPlaylistMapper.ts logic,
//! extracting structured track and asset information from CPL XML.

#[cfg(feature = "typescript")]
use ts_rs::TS;
use serde::{Deserialize, Serialize};
use crate::assetmap::ImfUuid;
use crate::cpl::EditRate;
use crate::cpl::{
    CompositionPlaylist, EssenceDescriptor, RGBADescriptor, CDCIDescriptor,
};

// =============================================================================
// Domain enums with ordering for "best" selection
// =============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "SCREAMING_SNAKE_CASE"))]
pub enum AudioType {
    Stereo,
    DolbyDigital,
    DolbyDigitalPlus,
    DolbyAtmos,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "SCREAMING_SNAKE_CASE"))]
pub enum VideoQuality {
    Sd,
    Hd,
    Uhd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "SCREAMING_SNAKE_CASE"))]
pub enum VideoDynamicRange {
    Sdr,
    Hdr10,
    HdrDolbyVision,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "SCREAMING_SNAKE_CASE"))]
pub enum AudioContentKind {
    /// Primary
    Prm,
    /// Visually Impaired
    Vi,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "SCREAMING_SNAKE_CASE"))]
pub enum ContentKind {
    Feature,
    Trailer,
}

// =============================================================================
// Track types
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
pub struct VideoTrack {
    pub quality: VideoQuality,
    pub dynamic_range: VideoDynamicRange,
    pub hdr10_metadata: Option<Hdr10Metadata>,
    pub width: u32,
    pub height: u32,
    pub track_identifier: String,
    pub track_number: u32,
    pub fragment_duration: String,
    pub track_duration: u64,
    pub sequence_number: Option<u32>,
    pub sequence_track_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
pub struct Hdr10Metadata {
    pub max_cll: u32,
    pub max_fall: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
pub struct AudioTrack {
    #[serde(rename = "type")]
    pub audio_type: AudioType,
    pub audio_content_kind: AudioContentKind,
    pub channel_count: u32,
    pub language: String,
    pub track_file: Option<String>,
    pub atmos_type: Option<String>, // "DAMF" or "IAB" or "ISXD"
    /// MCA tag symbol, e.g. "sg51", "sg71", "sgAtmos", "sgLtRt"
    pub mca_tag_symbol: Option<String>,
    /// MCA tag name, e.g. "5.1", "7.1", "Atmos", "Lt-Rt"
    pub mca_tag_name: Option<String>,
    pub track_identifier: String,
    pub track_number: u32,
    pub fragment_duration: String,
    pub track_duration: u64,
    pub sequence_number: Option<u32>,
    pub sequence_track_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
pub struct TimedTextTrack {
    pub language: String,
    pub track_identifier: String,
    pub track_number: u32,
    pub fragment_duration: String,
    pub track_duration: u64,
    pub sequence_number: Option<u32>,
    pub sequence_track_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
pub struct Sequence {
    #[serde(rename = "type")]
    pub sequence_type: String,
    pub id: String,
    pub track_id: String,
    pub segment_id: String,
    pub sequence_number: u32,
    pub sequence_resources: Vec<SequenceResource>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
pub struct SequenceResource {
    pub id: String,
    pub intrinsic_duration: u64,
    pub source_duration: u64,
    pub source_encoding: String,
    pub track_file_id: String,
    pub annotation: Option<String>,
    pub edit_rate: Option<String>,
    pub entry_point: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
pub struct Tracks {
    #[serde(rename = "AUDIO")]
    pub audio: Vec<AudioTrack>,
    #[serde(rename = "VIDEO")]
    pub video: Vec<VideoTrack>,
    #[serde(rename = "SUBTITLES")]
    pub subtitles: Vec<TimedTextTrack>,
    #[serde(rename = "CAPTIONS")]
    pub captions: Vec<TimedTextTrack>,
    #[serde(rename = "FORCED_NARRATIVE")]
    pub forced_narrative: Vec<TimedTextTrack>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
pub struct SourceAsset {
    pub content_kind: String,
    pub content_title: String,
    pub territory: String,
    pub edit_rate: String,
    pub frame_rate: String, // e.g. "23.976", "25", "29.97"
    pub duration: String, // HH:MM:SS
    pub audio_languages: Vec<String>,
    pub subtitle_languages: Vec<String>,
    pub caption_languages: Vec<String>,
    pub forced_narrative_languages: Vec<String>,
    pub audio_type: AudioType,
    pub video_quality: VideoQuality,
    pub video_dynamic_range: VideoDynamicRange,
    pub tracks: Tracks,
    pub sequences: Vec<Sequence>,
}

// =============================================================================
// Lookup functions
// =============================================================================

fn determine_audio_type(channel_count: u32) -> AudioType {
    match channel_count {
        2 => AudioType::Stereo,
        6 => AudioType::DolbyDigitalPlus,
        12 => AudioType::DolbyAtmos,
        _ => AudioType::Stereo, // Default fallback
    }
}

fn determine_video_quality(width: u32) -> VideoQuality {
    match width {
        0..=640 => VideoQuality::Sd,
        641..=1920 => VideoQuality::Hd,
        _ => VideoQuality::Uhd,
    }
}

fn determine_dynamic_range(
    descriptor: &RGBADescriptor,
    extension_props: &Option<crate::cpl::ExtensionProperties>,
) -> (VideoDynamicRange, Option<Hdr10Metadata>) {
    // Check for Dolby Vision via PHDRMetadataTrackSubDescriptor
    if let Some(ref sub_descs) = descriptor.sub_descriptors {
        if sub_descs.phdr_metadata_track_sub_descriptor.is_some() {
            return (VideoDynamicRange::HdrDolbyVision, None);
        }
    }

    // Check for HDR10 via ExtensionProperties MaxCLL/MaxFALL
    if let Some(ref ext) = extension_props {
        let max_cll = ext.max_cll.unwrap_or(0);
        let max_fall = ext.max_fall.unwrap_or(0);
        if max_cll > 0 && max_fall > 0 {
            return (
                VideoDynamicRange::Hdr10,
                Some(Hdr10Metadata { max_cll, max_fall }),
            );
        }
    }

    (VideoDynamicRange::Sdr, Some(Hdr10Metadata { max_cll: 0, max_fall: 0 }))
}

fn determine_dynamic_range_cdci(
    _descriptor: &CDCIDescriptor,
    extension_props: &Option<crate::cpl::ExtensionProperties>,
) -> (VideoDynamicRange, Option<Hdr10Metadata>) {
    // CDCI doesn't have PHDR sub-descriptors, check HDR10 only
    if let Some(ref ext) = extension_props {
        let max_cll = ext.max_cll.unwrap_or(0);
        let max_fall = ext.max_fall.unwrap_or(0);
        if max_cll > 0 && max_fall > 0 {
            return (
                VideoDynamicRange::Hdr10,
                Some(Hdr10Metadata { max_cll, max_fall }),
            );
        }
    }

    (VideoDynamicRange::Sdr, Some(Hdr10Metadata { max_cll: 0, max_fall: 0 }))
}

/// Convert source duration in frames to seconds using edit rate
fn convert_source_duration_to_seconds(source_duration: u64, edit_rate: &EditRate) -> u64 {
    let numerator = edit_rate.numerator as u64;
    let denominator = edit_rate.denominator as u64;
    if numerator == 0 {
        return 0;
    }
    (source_duration * denominator) / numerator
}

/// Convert seconds to HH:MM:SS string
fn seconds_to_hhmmss(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, secs)
}

/// Determine fragment duration based on track type and edit rate
fn fragment_duration(track_type: &str, edit_rate: &EditRate) -> String {
    match track_type {
        "audio" | "iab" | "isxd" => "384000/48000".to_string(),
        "video" => {
            let n = edit_rate.numerator;
            let d = edit_rate.denominator;
            match (n, d) {
                (24, 1) => "192/24".to_string(),
                (25, 1) => "200/25".to_string(),
                (30, 1) => "240/30".to_string(),
                (24000, 1001) => "192192/24000".to_string(),
                (30000, 1001) => "240240/30000".to_string(),
                _ => edit_rate.to_string(),
            }
        },
        "timed_text" => "8/1".to_string(),
        _ => "".to_string(),
    }
}

// =============================================================================
// Extraction logic
// =============================================================================

/// Build an EssenceDescriptor lookup by ID
fn build_descriptor_map(cpl: &CompositionPlaylist) -> std::collections::HashMap<ImfUuid, &EssenceDescriptor> {
    let mut map = std::collections::HashMap::new();
    if let Some(ref desc_list) = cpl.essence_descriptor_list {
        for desc in &desc_list.essence_descriptors {
            map.insert(desc.id, desc);
        }
    }
    map
}

/// Extract a SourceAsset from a parsed CompositionPlaylist
pub fn extract_source_asset(cpl: &CompositionPlaylist) -> Result<SourceAsset, String> {
    let descriptor_map = build_descriptor_map(cpl);
    let edit_rate = cpl.edit_rate.unwrap_or(EditRate::new(24, 1));

    // Extract territory from LocaleList
    let territory = cpl.locale_list.as_ref()
        .and_then(|ll| ll.locales.first())
        .and_then(|locale| locale.region_list.as_ref())
        .and_then(|rl| rl.regions.first())
        .cloned()
        .unwrap_or_default();

    // Track counters
    let mut audio_track_num: u32 = 0;
    let mut video_track_num: u32 = 0;
    let mut timed_text_track_num: u32 = 0;

    let mut tracks = Tracks {
        audio: Vec::new(),
        video: Vec::new(),
        subtitles: Vec::new(),
        captions: Vec::new(),
        forced_narrative: Vec::new(),
    };
    let mut sequences = Vec::new();
    let mut sequence_counter: u32 = 0;

    // Process all segments
    for segment in &cpl.segment_list.segments {
        let seq_list = &segment.sequence_list;

        // Process MainImageSequence (video)
        for seq in &seq_list.main_image_sequences {
            sequence_counter += 1;
            let mut seq_resources = Vec::new();

            for resource in &seq.resource_list.resources {
                let source_encoding = resource.source_encoding.map(|u| u.to_string()).unwrap_or_default();
                let track_file_id = resource.track_file_id.map(|u| u.to_string()).unwrap_or_default();

                seq_resources.push(SequenceResource {
                    id: resource.id.to_string(),
                    intrinsic_duration: resource.intrinsic_duration,
                    source_duration: resource.source_duration.unwrap_or(resource.intrinsic_duration),
                    source_encoding: source_encoding.clone(),
                    track_file_id: track_file_id.clone(),
                    annotation: None,
                    edit_rate: resource.edit_rate.map(|er| er.to_string()),
                    entry_point: resource.entry_point,
                });

                if let Some(se_uuid) = resource.source_encoding {
                    if let Some(desc) = descriptor_map.get(&se_uuid) {
                        if let Some(ref rgba) = desc.rgba_descriptor {
                            let width = rgba.display_width.unwrap_or(0);
                            let height = rgba.display_height.unwrap_or(0);
                            let (dynamic_range, hdr10_meta) = determine_dynamic_range(rgba, &cpl.extension_properties);
                            video_track_num += 1;

                            tracks.video.push(VideoTrack {
                                quality: determine_video_quality(width),
                                dynamic_range,
                                hdr10_metadata: hdr10_meta,
                                width,
                                height,
                                track_identifier: track_file_id.clone(),
                                track_number: video_track_num,
                                fragment_duration: fragment_duration("video", &edit_rate),
                                track_duration: resource.source_duration.unwrap_or(resource.intrinsic_duration),
                                sequence_number: Some(sequence_counter),
                                sequence_track_id: Some(seq.track_id.to_string()),
                            });
                        } else if let Some(ref cdci) = desc.cdci_descriptor {
                            let width = cdci.active_width.or(cdci.display_width).unwrap_or(0);
                            let height = cdci.active_height.or(cdci.display_height).unwrap_or(0);
                            let (dynamic_range, hdr10_meta) = determine_dynamic_range_cdci(cdci, &cpl.extension_properties);
                            video_track_num += 1;

                            tracks.video.push(VideoTrack {
                                quality: determine_video_quality(width),
                                dynamic_range,
                                hdr10_metadata: hdr10_meta,
                                width,
                                height,
                                track_identifier: track_file_id.clone(),
                                track_number: video_track_num,
                                fragment_duration: fragment_duration("video", &edit_rate),
                                track_duration: resource.source_duration.unwrap_or(resource.intrinsic_duration),
                                sequence_number: Some(sequence_counter),
                                sequence_track_id: Some(seq.track_id.to_string()),
                            });
                        }
                    }
                }
            }

            sequences.push(Sequence {
                sequence_type: "MainImageSequence".to_string(),
                id: seq.id.to_string(),
                track_id: seq.track_id.to_string(),
                segment_id: segment.id.to_string(),
                sequence_number: sequence_counter,
                sequence_resources: seq_resources,
            });
        }

        // Process MainAudioSequence
        for seq in &seq_list.main_audio_sequences {
            sequence_counter += 1;
            let mut seq_resources = Vec::new();

            for resource in &seq.resource_list.resources {
                let source_encoding = resource.source_encoding.map(|u| u.to_string()).unwrap_or_default();
                let track_file_id = resource.track_file_id.map(|u| u.to_string()).unwrap_or_default();

                seq_resources.push(SequenceResource {
                    id: resource.id.to_string(),
                    intrinsic_duration: resource.intrinsic_duration,
                    source_duration: resource.source_duration.unwrap_or(resource.intrinsic_duration),
                    source_encoding: source_encoding.clone(),
                    track_file_id: track_file_id.clone(),
                    annotation: None,
                    edit_rate: resource.edit_rate.map(|er| er.to_string()),
                    entry_point: resource.entry_point,
                });

                if let Some(se_uuid) = resource.source_encoding {
                    if let Some(desc) = descriptor_map.get(&se_uuid) {
                        if let Some(ref wave) = desc.wave_pcm_descriptor {
                            let channel_count = wave.channel_count.unwrap_or(2);
                            let audio_type = determine_audio_type(channel_count);

                            // Extract language from sub-descriptors
                            let language = wave.sub_descriptors.as_ref()
                                .and_then(|sd| sd.soundfield_group_label_sub_descriptor.as_ref())
                                .and_then(|sg| sg.rfc5646_spoken_language.as_ref())
                                .map(|lt| lt.to_string())
                                .unwrap_or_default();

                            // Extract audio content kind
                            let audio_content_kind = wave.sub_descriptors.as_ref()
                                .and_then(|sd| sd.soundfield_group_label_sub_descriptor.as_ref())
                                .and_then(|sg| sg.mca_audio_content_kind.as_deref())
                                .map(|kind| match kind.to_lowercase().as_str() {
                                    "vi" => AudioContentKind::Vi,
                                    _ => AudioContentKind::Prm,
                                })
                                .unwrap_or(AudioContentKind::Prm);

                            let mca_tag_symbol = wave.sub_descriptors.as_ref()
                                .and_then(|sd| sd.soundfield_group_label_sub_descriptor.as_ref())
                                .and_then(|sg| sg.mca_tag_symbol.as_ref())
                                .map(|s| s.to_string());

                            let mca_tag_name = wave.sub_descriptors.as_ref()
                                .and_then(|sd| sd.soundfield_group_label_sub_descriptor.as_ref())
                                .and_then(|sg| sg.mca_tag_name.clone());

                            audio_track_num += 1;
                            let track = AudioTrack {
                                audio_type,
                                audio_content_kind,
                                channel_count,
                                language: language.clone(),
                                track_file: None,
                                atmos_type: None,
                                mca_tag_symbol: mca_tag_symbol.clone(),
                                mca_tag_name: mca_tag_name.clone(),
                                track_identifier: track_file_id.clone(),
                                track_number: audio_track_num,
                                fragment_duration: fragment_duration("audio", &edit_rate),
                                track_duration: resource.source_duration.unwrap_or(resource.intrinsic_duration),
                                sequence_number: Some(sequence_counter),
                                sequence_track_id: Some(seq.track_id.to_string()),
                            };
                            tracks.audio.push(track.clone());

                            // DD+ → DD copy business rule
                            if audio_type == AudioType::DolbyDigitalPlus {
                                audio_track_num += 1;
                                let mut dd_copy = track;
                                dd_copy.audio_type = AudioType::DolbyDigital;
                                dd_copy.track_number = audio_track_num;
                                tracks.audio.push(dd_copy);
                            }
                        }
                    }
                }
            }

            sequences.push(Sequence {
                sequence_type: "MainAudioSequence".to_string(),
                id: seq.id.to_string(),
                track_id: seq.track_id.to_string(),
                segment_id: segment.id.to_string(),
                sequence_number: sequence_counter,
                sequence_resources: seq_resources,
            });
        }

        // Process IABSequence (Dolby Atmos embedded in MXF)
        for seq in &seq_list.iab_sequences {
            sequence_counter += 1;
            let mut seq_resources = Vec::new();

            for resource in &seq.resource_list.resources {
                let source_encoding = resource.source_encoding.map(|u| u.to_string()).unwrap_or_default();
                let track_file_id = resource.track_file_id.map(|u| u.to_string()).unwrap_or_default();

                seq_resources.push(SequenceResource {
                    id: resource.id.to_string(),
                    intrinsic_duration: resource.intrinsic_duration,
                    source_duration: resource.source_duration.unwrap_or(resource.intrinsic_duration),
                    source_encoding: source_encoding.clone(),
                    track_file_id: track_file_id.clone(),
                    annotation: None,
                    edit_rate: resource.edit_rate.map(|er| er.to_string()),
                    entry_point: resource.entry_point,
                });

                if let Some(se_uuid) = resource.source_encoding {
                    if let Some(desc) = descriptor_map.get(&se_uuid) {
                        if let Some(ref iab) = desc.iab_essence_descriptor {
                            let language = iab.sub_descriptors.as_ref()
                                .and_then(|sd| sd.iab_soundfield_label_sub_descriptor.as_ref())
                                .and_then(|sg| sg.rfc5646_spoken_language.as_ref())
                                .map(|lt| lt.to_string())
                                .unwrap_or_default();

                            audio_track_num += 1;
                            tracks.audio.push(AudioTrack {
                                audio_type: AudioType::DolbyAtmos,
                                audio_content_kind: AudioContentKind::Prm,
                                channel_count: 0,
                                language,
                                track_file: None,
                                atmos_type: Some("IAB".to_string()),
                                mca_tag_symbol: Some("IAB".to_string()),
                                mca_tag_name: Some("Dolby Atmos IAB".to_string()),
                                track_identifier: track_file_id.clone(),
                                track_number: audio_track_num,
                                fragment_duration: fragment_duration("iab", &edit_rate),
                                track_duration: resource.source_duration.unwrap_or(resource.intrinsic_duration),
                                sequence_number: None,
                                sequence_track_id: None,
                            });
                        }
                    }
                }
            }

            sequences.push(Sequence {
                sequence_type: "IABSequence".to_string(),
                id: seq.id.to_string(),
                track_id: seq.track_id.to_string(),
                segment_id: segment.id.to_string(),
                sequence_number: sequence_counter,
                sequence_resources: seq_resources,
            });
        }

        // Process ISXDSequence (Dolby Atmos sidecar — XML spatial metadata alongside PCM)
        for seq in &seq_list.isxd_sequences {
            sequence_counter += 1;
            let mut seq_resources = Vec::new();

            for resource in &seq.resource_list.resources {
                let source_encoding = resource.source_encoding.map(|u| u.to_string()).unwrap_or_default();
                let track_file_id = resource.track_file_id.map(|u| u.to_string()).unwrap_or_default();

                seq_resources.push(SequenceResource {
                    id: resource.id.to_string(),
                    intrinsic_duration: resource.intrinsic_duration,
                    source_duration: resource.source_duration.unwrap_or(resource.intrinsic_duration),
                    source_encoding: source_encoding.clone(),
                    track_file_id: track_file_id.clone(),
                    annotation: None,
                    edit_rate: resource.edit_rate.map(|er| er.to_string()),
                    entry_point: resource.entry_point,
                });

                // ISXD carries no language in its descriptor — language comes from the
                // associated PCM audio track. Leave empty here.
                if let Some(se_uuid) = resource.source_encoding {
                    if descriptor_map.get(&se_uuid).is_some() {
                        audio_track_num += 1;
                        tracks.audio.push(AudioTrack {
                            audio_type: AudioType::DolbyAtmos,
                            audio_content_kind: AudioContentKind::Prm,
                            channel_count: 0,
                            language: String::new(),
                            track_file: None,
                            atmos_type: Some("ISXD".to_string()),
                            mca_tag_symbol: Some("ISXD".to_string()),
                            mca_tag_name: Some("Dolby Atmos ISXD".to_string()),
                            track_identifier: track_file_id.clone(),
                            track_number: audio_track_num,
                            fragment_duration: fragment_duration("isxd", &edit_rate),
                            track_duration: resource.source_duration.unwrap_or(resource.intrinsic_duration),
                            sequence_number: None,
                            sequence_track_id: None,
                        });
                    }
                }
            }

            sequences.push(Sequence {
                sequence_type: "ISXDSequence".to_string(),
                id: seq.id.to_string(),
                track_id: seq.track_id.to_string(),
                segment_id: segment.id.to_string(),
                sequence_number: sequence_counter,
                sequence_resources: seq_resources,
            });
        }

        // Process SubtitlesSequence
        process_timed_text_sequences(
            &seq_list.subtitles_sequences,
            "SubtitlesSequence",
            &descriptor_map,
            &edit_rate,
            &segment.id.to_string(),
            &mut timed_text_track_num,
            &mut sequence_counter,
            &mut tracks.subtitles,
            &mut sequences,
        );

        // Process HearingImpairedCaptionsSequence
        process_timed_text_sequences(
            &seq_list.hearing_impaired_captions_sequences,
            "HearingImpairedCaptionsSequence",
            &descriptor_map,
            &edit_rate,
            &segment.id.to_string(),
            &mut timed_text_track_num,
            &mut sequence_counter,
            &mut tracks.captions,
            &mut sequences,
        );

        // Process ForcedNarrativeSequence
        process_timed_text_sequences(
            &seq_list.forced_narrative_sequences,
            "ForcedNarrativeSequence",
            &descriptor_map,
            &edit_rate,
            &segment.id.to_string(),
            &mut timed_text_track_num,
            &mut sequence_counter,
            &mut tracks.forced_narrative,
            &mut sequences,
        );
    }

    // Validation: must have at least one video track
    if tracks.video.is_empty() {
        return Err("CPL must contain at least one MainImageSequence resource".to_string());
    }

    // Calculate duration from video tracks
    let total_seconds: u64 = tracks.video.iter()
        .map(|t| convert_source_duration_to_seconds(t.track_duration, &edit_rate))
        .sum();
    let duration = seconds_to_hhmmss(total_seconds);

    // Determine "best" summary values (highest in ordering)
    let best_audio_type = tracks.audio.iter()
        .map(|t| t.audio_type)
        .max()
        .unwrap_or(AudioType::Stereo);

    let best_video_quality = tracks.video.iter()
        .map(|t| t.quality)
        .max()
        .unwrap_or(VideoQuality::Sd);

    let best_dynamic_range = tracks.video.iter()
        .map(|t| t.dynamic_range)
        .max()
        .unwrap_or(VideoDynamicRange::Sdr);

    // Collect and deduplicate languages
    let audio_languages = collect_languages(tracks.audio.iter().map(|t| t.language.as_str()));
    let subtitle_languages = collect_languages(tracks.subtitles.iter().map(|t| t.language.as_str()));
    let caption_languages = collect_languages(tracks.captions.iter().map(|t| t.language.as_str()));
    let forced_narrative_languages = collect_languages(tracks.forced_narrative.iter().map(|t| t.language.as_str()));

    // Content kind mapping
    let content_kind = match cpl.content_kind.kind {
        crate::cpl::ContentKind::Feature => "FEATURE",
        crate::cpl::ContentKind::Trailer => "TRAILER",
        _ => &cpl.content_kind.kind.to_string(),
    };

    let frame_rate = crate::cpl::format_framerate(&edit_rate);

    Ok(SourceAsset {
        content_kind: content_kind.to_string(),
        content_title: cpl.content_title.text.clone(),
        territory,
        edit_rate: edit_rate.to_string(),
        frame_rate,
        duration,
        audio_languages,
        subtitle_languages,
        caption_languages,
        forced_narrative_languages,
        audio_type: best_audio_type,
        video_quality: best_video_quality,
        video_dynamic_range: best_dynamic_range,
        tracks,
        sequences,
    })
}

/// Process timed text sequence types (subtitles, captions, forced narrative)
fn process_timed_text_sequences<S: crate::cpl::SequenceAccess>(
    seqs: &[S],
    seq_type_name: &str,
    descriptor_map: &std::collections::HashMap<ImfUuid, &EssenceDescriptor>,
    edit_rate: &EditRate,
    segment_id: &str,
    track_num: &mut u32,
    sequence_counter: &mut u32,
    track_list: &mut Vec<TimedTextTrack>,
    sequences: &mut Vec<Sequence>,
) {
    for seq in seqs {
        *sequence_counter += 1;
        let mut seq_resources = Vec::new();

        for resource in &seq.resource_list().resources {
            let source_encoding = resource.source_encoding.map(|u| u.to_string()).unwrap_or_default();
            let track_file_id = resource.track_file_id.map(|u| u.to_string()).unwrap_or_default();

            seq_resources.push(SequenceResource {
                id: resource.id.to_string(),
                intrinsic_duration: resource.intrinsic_duration,
                source_duration: resource.source_duration.unwrap_or(resource.intrinsic_duration),
                source_encoding: source_encoding.clone(),
                track_file_id: track_file_id.clone(),
                annotation: None,
                edit_rate: resource.edit_rate.map(|er| er.to_string()),
                entry_point: resource.entry_point,
            });

            if let Some(se_uuid) = resource.source_encoding {
                if let Some(desc) = descriptor_map.get(&se_uuid) {
                    if let Some(ref timed_text) = desc.dc_timed_text_descriptor {
                        let language = timed_text.rfc5646_language_tag_list.iter()
                            .map(|lt| lt.to_string())
                            .collect::<Vec<_>>()
                            .join(",");
                        *track_num += 1;

                        track_list.push(TimedTextTrack {
                            language,
                            track_identifier: track_file_id.clone(),
                            track_number: *track_num,
                            fragment_duration: fragment_duration("timed_text", edit_rate),
                            track_duration: resource.source_duration.unwrap_or(resource.intrinsic_duration),
                            sequence_number: Some(*sequence_counter),
                            sequence_track_id: Some(seq.track_id().to_string()),
                        });
                    }
                }
            }
        }

        sequences.push(Sequence {
            sequence_type: seq_type_name.to_string(),
            id: seq.id().to_string(),
            track_id: seq.track_id().to_string(),
            segment_id: segment_id.to_string(),
            sequence_number: *sequence_counter,
            sequence_resources: seq_resources,
        });
    }
}

/// Collect, deduplicate, and sort language codes
fn collect_languages<'a>(iter: impl Iterator<Item = &'a str>) -> Vec<String> {
    let mut langs: Vec<String> = iter
        .filter(|l| !l.is_empty())
        .map(|l| l.to_string())
        .collect();
    langs.sort();
    langs.dedup();
    langs
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_type_ordering() {
        assert!(AudioType::DolbyAtmos > AudioType::DolbyDigitalPlus);
        assert!(AudioType::DolbyDigitalPlus > AudioType::DolbyDigital);
        assert!(AudioType::DolbyDigital > AudioType::Stereo);
    }

    #[test]
    fn test_video_quality_ordering() {
        assert!(VideoQuality::Uhd > VideoQuality::Hd);
        assert!(VideoQuality::Hd > VideoQuality::Sd);
    }

    #[test]
    fn test_dynamic_range_ordering() {
        assert!(VideoDynamicRange::HdrDolbyVision > VideoDynamicRange::Hdr10);
        assert!(VideoDynamicRange::Hdr10 > VideoDynamicRange::Sdr);
    }

    #[test]
    fn test_channel_count_mapping() {
        assert_eq!(determine_audio_type(2), AudioType::Stereo);
        assert_eq!(determine_audio_type(6), AudioType::DolbyDigitalPlus);
        assert_eq!(determine_audio_type(12), AudioType::DolbyAtmos);
        assert_eq!(determine_audio_type(1), AudioType::Stereo); // fallback
    }

    #[test]
    fn test_video_quality_mapping() {
        assert_eq!(determine_video_quality(640), VideoQuality::Sd);
        assert_eq!(determine_video_quality(1280), VideoQuality::Hd);
        assert_eq!(determine_video_quality(1920), VideoQuality::Hd);
        assert_eq!(determine_video_quality(3840), VideoQuality::Uhd);
    }

    #[test]
    fn test_duration_conversion() {
        // 24fps: 156613 frames at 24000/1001 ≈ 6532 seconds
        let secs = convert_source_duration_to_seconds(156613, &EditRate::new(24000, 1001));
        assert_eq!(secs, 6532); // floor((156613 * 1001) / 24000) = 6532

        // 25fps: 100 frames at 25/1 = 4 seconds
        let secs = convert_source_duration_to_seconds(100, &EditRate::new(25, 1));
        assert_eq!(secs, 4);
    }

    #[test]
    fn test_seconds_to_hhmmss() {
        assert_eq!(seconds_to_hhmmss(0), "00:00:00");
        assert_eq!(seconds_to_hhmmss(61), "00:01:01");
        assert_eq!(seconds_to_hhmmss(3661), "01:01:01");
        assert_eq!(seconds_to_hhmmss(6532), "01:48:52");
    }

    #[test]
    fn test_collect_languages() {
        let langs = collect_languages(["en", "nl", "en", "de", "nl"].iter().copied());
        assert_eq!(langs, vec!["de", "en", "nl"]);
    }

    #[test]
    fn test_extract_meridian_source_asset() {
        let cpl_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../test-data/MERIDIAN_Netflix_Photon_161006/CPL_0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85.xml"
        );
        let xml = std::fs::read_to_string(cpl_path).expect("Failed to read CPL");
        let cpl = crate::cpl::parse_cpl(&xml).expect("Failed to parse CPL");
        let result = extract_source_asset(&cpl).expect("Failed to extract source asset");

        // Check video
        assert!(!result.tracks.video.is_empty(), "Should have video tracks");
        let video = &result.tracks.video[0];
        assert_eq!(video.width, 3840);
        assert_eq!(video.height, 2160);
        assert_eq!(video.quality, VideoQuality::Uhd);
        assert_eq!(video.dynamic_range, VideoDynamicRange::HdrDolbyVision);

        // Check audio
        assert!(!result.tracks.audio.is_empty(), "Should have audio tracks");
        let audio_langs = &result.audio_languages;
        assert!(audio_langs.contains(&"en".to_string()), "Should have English audio");

        // Check summary
        assert_eq!(result.video_quality, VideoQuality::Uhd);
        assert_eq!(result.video_dynamic_range, VideoDynamicRange::HdrDolbyVision);
        assert_eq!(result.content_title, "MERIDIAN");

        // Check duration is in HH:MM:SS format
        assert!(
            result.duration.len() == 8 && result.duration.chars().nth(2) == Some(':'),
            "Duration should be HH:MM:SS format, got: {}", result.duration
        );

        // Check sequences
        assert!(!result.sequences.is_empty(), "Should have sequences");
    }

    #[test]
    fn test_extract_isxd_source_asset() {
        let cpl_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../test-data/ISXD/CompleteIMP/CPL_ISXD_TEST_1.xml"
        );
        let xml = std::fs::read_to_string(cpl_path).expect("Failed to read CPL");
        let cpl = crate::cpl::parse_cpl(&xml).expect("Failed to parse CPL");
        let result = extract_source_asset(&cpl).expect("Failed to extract source asset");

        let has_isxd = result.tracks.audio.iter().any(|t| t.atmos_type.as_deref() == Some("ISXD"));
        assert!(has_isxd, "Should have ISXD Dolby Atmos track");

        let isxd_track = result.tracks.audio.iter().find(|t| t.atmos_type.as_deref() == Some("ISXD")).unwrap();
        assert_eq!(isxd_track.audio_type, AudioType::DolbyAtmos);

        // Sequences should include the ISXD sequence
        let has_isxd_seq = result.sequences.iter().any(|s| s.sequence_type == "ISXDSequence");
        assert!(has_isxd_seq, "Should have ISXDSequence in sequences");
    }

    #[test]
    fn test_extract_iab_source_asset() {
        let cpl_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../test-data/IAB/CompleteIMP/CPL_e0265fda-cb35-4e35-a4e4-4f44d82d2a52.xml"
        );
        let xml = std::fs::read_to_string(cpl_path).expect("Failed to read CPL");
        let cpl = crate::cpl::parse_cpl(&xml).expect("Failed to parse CPL");
        let result = extract_source_asset(&cpl).expect("Failed to extract source asset");

        // Should have IAB Atmos audio track
        let has_atmos = result.tracks.audio.iter().any(|t| t.audio_type == AudioType::DolbyAtmos);
        assert!(has_atmos, "Should have Dolby Atmos track from IAB");

        let atmos_track = result.tracks.audio.iter().find(|t| t.audio_type == AudioType::DolbyAtmos).unwrap();
        assert_eq!(atmos_track.atmos_type, Some("IAB".to_string()));
    }

    #[test]
    fn test_extract_plugfest_source_asset() {
        let cpl_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../test-data/Netflix_Sony_Plugfest_2015/CPL_BLACKL_202_HD_REC709_178_ENG_fe8cf2f4-1bcd-4145-8f72-6775af4038c4.xml"
        );
        let xml = std::fs::read_to_string(cpl_path).expect("Failed to read CPL");
        let cpl = crate::cpl::parse_cpl(&xml).expect("Failed to parse CPL");
        let result = extract_source_asset(&cpl).expect("Failed to extract source asset");

        // Plugfest has CDCI descriptor — despite filename saying HD, content is 3840x2160 UHD SDR
        assert!(!result.tracks.video.is_empty());
        let video = &result.tracks.video[0];
        assert_eq!(video.width, 3840);
        assert_eq!(video.quality, VideoQuality::Uhd);
        assert_eq!(video.dynamic_range, VideoDynamicRange::Sdr);

        // Timed text sequence is commented out in this CPL XML
        assert!(result.tracks.subtitles.is_empty());

        // DD+ tracks should produce DD copies
        let dd_plus_count = result.tracks.audio.iter().filter(|t| t.audio_type == AudioType::DolbyDigitalPlus).count();
        let dd_count = result.tracks.audio.iter().filter(|t| t.audio_type == AudioType::DolbyDigital).count();
        if dd_plus_count > 0 {
            assert_eq!(dd_plus_count, dd_count, "Each DD+ track should have a DD copy");
        }
    }
}
