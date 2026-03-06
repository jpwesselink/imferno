//! Unified IMF report — the single JSON document for UI consumption
//!
//! Combines package metadata, validation, and structural analysis into one structure.
//! Also provides `format_report()` for pretty-printing to a terminal.

use crate::cpl::SequenceAccess;
use crate::diagnostics::{Severity, ValidationReport};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Write;
#[cfg(feature = "typescript")]
use ts_rs::TS;

// ── Report structs ───────────────────────────────────────────────────────────

#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
pub struct ImfReport {
    pub package: PackageSummary,
    pub cpls: Vec<CplReport>,
    pub validation: ValidationReport,
}

#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
pub struct CplReport {
    pub id: String,
    pub title: String,
    /// Application profile, e.g. "App2E_2021", "App2E_2014", "App5"
    pub application_profile: Option<String>,
    /// CPL-level edit rate, e.g. "24000/1001"
    pub edit_rate: Option<String>,
    /// Number of segments in this CPL
    pub segment_count: usize,
    /// Timecode start address, e.g. "01:00:00:00"
    pub timecode_start: Option<String>,
    /// True if this CPL references track files not present in the current package (supplemental IMP)
    pub is_supplemental: bool,
    /// Track file UUIDs referenced in this CPL that are not in the current package's AssetMap.
    /// These must be resolved from an ancestor package.
    pub unresolved_ancestor_asset_ids: Vec<String>,
    pub markers: Vec<CplMarker>,
    /// Virtual tracks (sequences) merged across all segments
    pub sequences: Vec<CplSequence>,
}

#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
pub struct CplMarker {
    pub label: String,
    pub offset: u64,
    pub annotation: Option<String>,
}

#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
pub struct CplSequence {
    /// Sequence type: "MainImage", "MainAudio", "Subtitles", etc.
    pub r#type: String,
    pub id: String,
    pub track_id: String,
    /// Language tag (e.g. "en", "fr") from EssenceDescriptor MCA or timed text metadata
    pub language: Option<String>,
    /// Channel layout (e.g. "5.1", "7.1", "Object-based") for audio tracks
    pub channels: Option<String>,
    /// Codec description (e.g. "JPEG 2000", "PCM 24-bit", "IAB (Dolby Atmos)")
    pub codec: Option<String>,
    /// Resolution (e.g. "3840x2160") for video tracks
    pub resolution: Option<String>,
    /// Subtitle kind: "standard", "hi" (hearing impaired), "forced"
    pub subtitle_type: Option<String>,
    pub resources: Vec<CplResource>,
}

#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
pub struct CplResource {
    pub id: String,
    /// Edit rate as "N/D" string, e.g. "24000/1001"
    pub edit_rate: Option<String>,
    pub intrinsic_duration: u64,
    pub source_duration: Option<u64>,
    pub entry_point: Option<u64>,
    pub source_encoding: Option<String>,
    pub track_file_id: Option<String>,
}

#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
pub struct PackageSummary {
    pub asset_map_id: String,
    pub volume_index: u32,
    pub asset_count: usize,
    pub cpl_count: usize,
    pub issue_date: String,
    pub issuer: Option<String>,
    pub creator: Option<String>,
    pub pkl_count: usize,
    pub scm_count: usize,
    pub sidecar_count: usize,
    pub unreferenced_assets: Vec<UnreferencedAsset>,
}

#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
pub struct UnreferencedAsset {
    pub id: String,
    pub path: String,
}

// ── build_report ─────────────────────────────────────────────────────────────

/// Map an ApplicationIdentification URL to a friendly profile name
fn parse_application_profile(url: &str) -> String {
    // SMPTE ST 2067-21 Application #2 Extended profiles
    if url.contains("2067-21") {
        if url.ends_with("2021") || url.contains("/2021") {
            return "App2E_2021".to_string();
        }
        if url.ends_with("2020") || url.contains("/2020") {
            return "App2E_2020".to_string();
        }
        if url.ends_with("2014") || url.contains("/2014") {
            return "App2E_2014".to_string();
        }
        return "App2E".to_string();
    }
    // SMPTE ST 2067-20 Application #2
    if url.contains("2067-20") {
        return "App2".to_string();
    }
    // SMPTE ST 2067-50 Application #5
    if url.contains("2067-50") || url.contains("2067-5/") {
        return "App5".to_string();
    }
    // Fallback: return the URL tail after the last '/'
    url.rsplit('/').next().unwrap_or(url).to_string()
}

/// Map a single resource to a CplResource, falling back to the CPL edit rate
fn map_resource(r: &crate::cpl::Resource, cpl_er: &Option<String>) -> CplResource {
    CplResource {
        id: r.id.to_string(),
        edit_rate: r
            .edit_rate
            .as_ref()
            .map(|er| format!("{}/{}", er.numerator, er.denominator))
            .or_else(|| cpl_er.clone()),
        intrinsic_duration: r.intrinsic_duration,
        source_duration: r.source_duration,
        entry_point: r.entry_point,
        source_encoding: r.source_encoding.as_ref().map(|u| u.to_string()),
        track_file_id: r.track_file_id.as_ref().map(|u| u.to_string()),
    }
}

/// Metadata extracted from an EssenceDescriptor for a sequence
struct SequenceMeta {
    language: Option<String>,
    channels: Option<String>,
    codec: Option<String>,
    resolution: Option<String>,
    subtitle_type: Option<String>,
}

impl Default for SequenceMeta {
    fn default() -> Self {
        Self {
            language: None,
            channels: None,
            codec: None,
            resolution: None,
            subtitle_type: None,
        }
    }
}

/// Merge sequences of one type into the track map, accumulating resources by track_id
fn merge_sequences(
    track_map: &mut HashMap<String, CplSequence>,
    type_name: &str,
    sequences: &[impl SequenceAccess],
    cpl_er: &Option<String>,
    meta_fn: impl Fn(&crate::cpl::Resource) -> SequenceMeta,
) {
    for seq in sequences {
        let tid = seq.track_id().to_string();
        let resources: Vec<CplResource> = seq
            .resource_list()
            .resources
            .iter()
            .map(|r| map_resource(r, cpl_er))
            .collect();
        if let Some(existing) = track_map.get_mut(&tid) {
            existing.resources.extend(resources);
        } else {
            // Get metadata from the first resource that has a SourceEncoding
            let meta = seq
                .resource_list()
                .resources
                .first()
                .map(&meta_fn)
                .unwrap_or_default();
            track_map.insert(
                tid.clone(),
                CplSequence {
                    r#type: type_name.to_string(),
                    id: seq.id().to_string(),
                    track_id: tid,
                    language: meta.language,
                    channels: meta.channels,
                    codec: meta.codec,
                    resolution: meta.resolution,
                    subtitle_type: meta.subtitle_type,
                    resources,
                },
            );
        }
    }
}

/// Look up an EssenceDescriptor by the resource's SourceEncoding UUID
fn lookup_ed<'a>(
    r: &crate::cpl::Resource,
    eds: &'a HashMap<crate::assetmap::ImfUuid, &'a crate::cpl::EssenceDescriptor>,
) -> Option<&'a crate::cpl::EssenceDescriptor> {
    r.source_encoding
        .as_ref()
        .and_then(|se| eds.get(se).copied())
}

fn video_meta(
    r: &crate::cpl::Resource,
    eds: &HashMap<crate::assetmap::ImfUuid, &crate::cpl::EssenceDescriptor>,
) -> SequenceMeta {
    let Some(ed) = lookup_ed(r, eds) else {
        return SequenceMeta::default();
    };
    let (codec, resolution) = if let Some(cdci) = &ed.cdci_descriptor {
        let w = cdci
            .active_width
            .or(cdci.display_width)
            .or(cdci.stored_width);
        let h = cdci
            .active_height
            .or(cdci.display_height)
            .or(cdci.stored_height);
        let res = match (w, h) {
            (Some(w), Some(h)) => Some(format!("{w}x{h}")),
            _ => None,
        };
        let c = cdci
            .picture_compression
            .as_ref()
            .map(|c| c.to_string())
            .unwrap_or_else(|| "JPEG 2000".into());
        (Some(c), res)
    } else if let Some(rgba) = &ed.rgba_descriptor {
        let w = rgba.display_width.or(rgba.stored_width);
        let h = rgba.display_height.or(rgba.stored_height);
        let res = match (w, h) {
            (Some(w), Some(h)) => Some(format!("{w}x{h}")),
            _ => None,
        };
        let c = rgba
            .picture_compression
            .as_ref()
            .map(|c| c.to_string())
            .unwrap_or_else(|| "JPEG 2000".into());
        (Some(c), res)
    } else {
        (None, None)
    };
    SequenceMeta {
        codec,
        resolution,
        ..Default::default()
    }
}

fn audio_meta(
    r: &crate::cpl::Resource,
    eds: &HashMap<crate::assetmap::ImfUuid, &crate::cpl::EssenceDescriptor>,
) -> SequenceMeta {
    let Some(ed) = lookup_ed(r, eds) else {
        return SequenceMeta::default();
    };
    if let Some(wave) = &ed.wave_pcm_descriptor {
        let codec = wave
            .quantization_bits
            .map(|b| format!("PCM {b}-bit"))
            .unwrap_or_else(|| "PCM".into());
        let channels = match wave.channel_count {
            Some(1) => Some("1.0".into()),
            Some(2) => Some("2.0".into()),
            Some(6) => Some("5.1".into()),
            Some(8) => Some("7.1".into()),
            Some(n) => Some(format!("{n}ch")),
            None => None,
        };
        let language = wave
            .sub_descriptors
            .as_ref()
            .and_then(|sd| sd.soundfield_group_label_sub_descriptor.as_ref())
            .and_then(|sf| sf.rfc5646_spoken_language.as_ref())
            .map(|lt| lt.0.clone());
        return SequenceMeta {
            codec: Some(codec),
            channels,
            language,
            ..Default::default()
        };
    }
    SequenceMeta::default()
}

fn iab_meta(
    r: &crate::cpl::Resource,
    eds: &HashMap<crate::assetmap::ImfUuid, &crate::cpl::EssenceDescriptor>,
) -> SequenceMeta {
    let Some(ed) = lookup_ed(r, eds) else {
        return SequenceMeta::default();
    };
    let language = ed
        .iab_essence_descriptor
        .as_ref()
        .and_then(|iab| iab.sub_descriptors.as_ref())
        .and_then(|sd| sd.iab_soundfield_label_sub_descriptor.as_ref())
        .and_then(|sf| sf.rfc5646_spoken_language.as_ref())
        .map(|lt| lt.0.clone());
    SequenceMeta {
        codec: Some("IAB (Dolby Atmos)".into()),
        channels: Some("Object-based".into()),
        language,
        ..Default::default()
    }
}

fn timed_text_meta(
    subtitle_type: &str,
    r: &crate::cpl::Resource,
    eds: &HashMap<crate::assetmap::ImfUuid, &crate::cpl::EssenceDescriptor>,
) -> SequenceMeta {
    let Some(ed) = lookup_ed(r, eds) else {
        return SequenceMeta {
            subtitle_type: Some(subtitle_type.into()),
            ..Default::default()
        };
    };
    let language = ed
        .dc_timed_text_descriptor
        .as_ref()
        .map(|tt| {
            tt.rfc5646_language_tag_list
                .iter()
                .map(|lt| lt.as_str())
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join(",")
        })
        .filter(|s| !s.is_empty());
    SequenceMeta {
        codec: Some("IMSC1 (Timed Text)".into()),
        language,
        subtitle_type: Some(subtitle_type.into()),
        ..Default::default()
    }
}

/// Extract all virtual tracks from a CPL, merged across segments
fn extract_sequences(cpl: &crate::cpl::CompositionPlaylist) -> (Option<String>, Vec<CplSequence>) {
    let edit_rate = cpl
        .edit_rate
        .as_ref()
        .map(|er| format!("{}/{}", er.numerator, er.denominator));

    // Build essence descriptor lookup
    let eds: HashMap<crate::assetmap::ImfUuid, &crate::cpl::EssenceDescriptor> =
        if let Some(edl) = &cpl.essence_descriptor_list {
            edl.essence_descriptors
                .iter()
                .map(|ed| (ed.id, ed))
                .collect()
        } else {
            HashMap::new()
        };

    let mut track_map: HashMap<String, CplSequence> = HashMap::new();

    for seg in &cpl.segment_list.segments {
        let sl = &seg.sequence_list;
        merge_sequences(
            &mut track_map,
            "MainImage",
            &sl.main_image_sequences,
            &edit_rate,
            |r| video_meta(r, &eds),
        );
        merge_sequences(
            &mut track_map,
            "MainAudio",
            &sl.main_audio_sequences,
            &edit_rate,
            |r| audio_meta(r, &eds),
        );
        merge_sequences(
            &mut track_map,
            "Subtitles",
            &sl.subtitles_sequences,
            &edit_rate,
            |r| timed_text_meta("standard", r, &eds),
        );
        merge_sequences(
            &mut track_map,
            "HearingImpairedCaptions",
            &sl.hearing_impaired_captions_sequences,
            &edit_rate,
            |r| timed_text_meta("hi", r, &eds),
        );
        merge_sequences(
            &mut track_map,
            "ForcedNarrative",
            &sl.forced_narrative_sequences,
            &edit_rate,
            |r| timed_text_meta("forced", r, &eds),
        );
        merge_sequences(&mut track_map, "IAB", &sl.iab_sequences, &edit_rate, |r| {
            iab_meta(r, &eds)
        });
        merge_sequences(
            &mut track_map,
            "ISXD",
            &sl.isxd_sequences,
            &edit_rate,
            |_| SequenceMeta::default(),
        );
    }

    (edit_rate, track_map.into_values().collect())
}

/// Collect all TrackFileIds referenced in a CPL across all sequence types
fn collect_track_file_ids(cpl: &crate::cpl::CompositionPlaylist) -> Vec<String> {
    let mut ids = Vec::new();
    for seg in &cpl.segment_list.segments {
        let sl = &seg.sequence_list;
        for seq in &sl.main_image_sequences {
            for r in &seq.resource_list.resources {
                if let Some(ref id) = r.track_file_id {
                    ids.push(id.to_string());
                }
            }
        }
        for seq in &sl.main_audio_sequences {
            for r in &seq.resource_list.resources {
                if let Some(ref id) = r.track_file_id {
                    ids.push(id.to_string());
                }
            }
        }
        for seq in &sl.iab_sequences {
            for r in &seq.resource_list.resources {
                if let Some(ref id) = r.track_file_id {
                    ids.push(id.to_string());
                }
            }
        }
        for seq in &sl.isxd_sequences {
            for r in &seq.resource_list.resources {
                if let Some(ref id) = r.track_file_id {
                    ids.push(id.to_string());
                }
            }
        }
        for seq in &sl.subtitles_sequences {
            for r in &seq.resource_list.resources {
                if let Some(ref id) = r.track_file_id {
                    ids.push(id.to_string());
                }
            }
        }
        for seq in &sl.hearing_impaired_captions_sequences {
            for r in &seq.resource_list.resources {
                if let Some(ref id) = r.track_file_id {
                    ids.push(id.to_string());
                }
            }
        }
        for seq in &sl.forced_narrative_sequences {
            for r in &seq.resource_list.resources {
                if let Some(ref id) = r.track_file_id {
                    ids.push(id.to_string());
                }
            }
        }
    }
    ids
}

/// Build a full ImfReport from a package, with optional ancestor package for supplemental IMPs
pub fn build_report(
    package: &super::Imferno,
    options: &super::ValidationOptions,
    ancestor: Option<&super::Imferno>,
) -> Result<ImfReport, String> {
    // Unreferenced assets
    let unreferenced_assets: Vec<UnreferencedAsset> = package
        .unreferenced_assets()
        .iter()
        .map(|a| {
            let path = a
                .chunk_list
                .chunks
                .first()
                .map(|c| c.path.as_str())
                .unwrap_or("")
                .to_string();
            UnreferencedAsset {
                id: a.id.to_string(),
                path,
            }
        })
        .collect();

    // SCM counts
    let scm_count = package.sidecar_composition_maps.len();
    let sidecar_count: usize = package
        .sidecar_composition_maps
        .values()
        .map(|s| s.sidecar_assets.len())
        .sum();

    let pkg_summary = PackageSummary {
        asset_map_id: package.asset_map.id.to_string(),
        volume_index: package.volume_index.index,
        asset_count: package.asset_map.asset_list.assets.len(),
        cpl_count: package.composition_playlists.len(),
        issue_date: package.asset_map.issue_date.clone(),
        issuer: package.asset_map.issuer.clone(),
        creator: package.asset_map.creator.clone(),
        pkl_count: package.packing_lists.len(),
        scm_count,
        sidecar_count,
        unreferenced_assets,
    };

    // CPL reports
    let mut cpls = Vec::new();
    for cpl in package.composition_playlists.values() {
        let cpl_track_ids = collect_track_file_ids(cpl);
        let unresolved_ancestor_asset_ids: Vec<String> = cpl_track_ids
            .iter()
            .filter(|id| {
                package.get_asset_path_str(id).is_none()
                    && ancestor.is_none_or(|a| a.get_asset_path_str(id).is_none())
            })
            .cloned()
            .collect();
        let is_supplemental = cpl_track_ids
            .iter()
            .any(|id| package.get_asset_path_str(id).is_none());

        let markers: Vec<CplMarker> = cpl
            .segment_list
            .segments
            .iter()
            .flat_map(|seg| seg.sequence_list.marker_sequences.iter())
            .flat_map(|ms| ms.resource_list.resources.iter())
            .flat_map(|res| res.markers.iter())
            .map(|m| CplMarker {
                label: m.label.to_string(),
                offset: m.offset,
                annotation: m.annotation.clone().filter(|a| !a.is_empty()),
            })
            .collect();

        let application_profile = cpl
            .extension_properties
            .as_ref()
            .and_then(|ep| ep.application_identification.as_ref())
            .map(|url| parse_application_profile(url));

        let segment_count = cpl.segment_list.segments.len();

        let timecode_start = cpl
            .composition_timecode
            .as_ref()
            .and_then(|tc| tc.timecode_start_address.clone());

        let (edit_rate, sequences) = extract_sequences(cpl);

        cpls.push(CplReport {
            id: cpl.id.to_string(),
            title: cpl.content_title.text.clone(),
            application_profile,
            edit_rate,
            segment_count,
            timecode_start,
            is_supplemental,
            unresolved_ancestor_asset_ids,
            markers,
            sequences,
        });
    }

    // Validation
    let validation = package.validate(options);

    Ok(ImfReport {
        package: pkg_summary,
        cpls,
        validation,
    })
}

// ── ANSI colour helpers ──────────────────────────────────────────────────────

fn c_red(s: &str, on: bool) -> String {
    if on {
        format!("\x1b[31m{}\x1b[0m", s)
    } else {
        s.to_string()
    }
}
fn c_yellow(s: &str, on: bool) -> String {
    if on {
        format!("\x1b[33m{}\x1b[0m", s)
    } else {
        s.to_string()
    }
}
fn c_cyan(s: &str, on: bool) -> String {
    if on {
        format!("\x1b[36m{}\x1b[0m", s)
    } else {
        s.to_string()
    }
}
fn c_green(s: &str, on: bool) -> String {
    if on {
        format!("\x1b[32m{}\x1b[0m", s)
    } else {
        s.to_string()
    }
}
fn c_bold(s: &str, on: bool) -> String {
    if on {
        format!("\x1b[1m{}\x1b[0m", s)
    } else {
        s.to_string()
    }
}
fn c_dim(s: &str, on: bool) -> String {
    if on {
        format!("\x1b[2m{}\x1b[0m", s)
    } else {
        s.to_string()
    }
}

// ── format_report ────────────────────────────────────────────────────────────

/// Render an `ImfReport` as a human-readable, optionally ANSI-coloured string.
pub fn format_report(report: &ImfReport, color: bool) -> String {
    let mut out = String::new();
    let pkg = &report.package;

    // Package structure
    let _ = writeln!(out, "  {}  VOLINDEX.xml found", c_green("ok", color));
    let _ = writeln!(out, "  {}  ASSETMAP.xml found", c_green("ok", color));
    let _ = writeln!(
        out,
        "  {}  {} assets mapped",
        c_green("ok", color),
        pkg.asset_count
    );
    let _ = writeln!(
        out,
        "  {}  {} CPL(s) parsed",
        c_green("ok", color),
        pkg.cpl_count
    );

    // SCMs
    if pkg.scm_count > 0 {
        let _ = writeln!(
            out,
            "  {}  {} SCM(s) parsed, {} sidecar asset(s) declared",
            c_green("ok", color),
            pkg.scm_count,
            pkg.sidecar_count
        );
    }

    // Unreferenced assets
    if !pkg.unreferenced_assets.is_empty() {
        let _ = writeln!(
            out,
            "  {}  {} unreferenced asset(s)",
            c_yellow("info", color),
            pkg.unreferenced_assets.len()
        );
        for asset in &pkg.unreferenced_assets {
            let _ = writeln!(out, "        {}", c_dim(&asset.path, color));
        }
    }

    // Validation findings
    let all_issues: Vec<_> = report
        .validation
        .critical
        .iter()
        .chain(report.validation.errors.iter())
        .chain(report.validation.warnings.iter())
        .chain(report.validation.info.iter())
        .collect();

    if !all_issues.is_empty() {
        let _ = writeln!(out, "{}", c_bold("Validation findings:", color));

        for issue in &all_issues {
            let (label, colorize): (&str, fn(&str, bool) -> String) = match issue.severity {
                Severity::Critical => ("error", c_red),
                Severity::Error => ("error", c_red),
                Severity::Warning => ("warning", c_yellow),
                Severity::Info => ("info", c_cyan),
            };
            let location = if let Some(ref c) = issue.location.cpl_id {
                c_dim(&format!(" [CPL:{}]", &c[..c.len().min(8)]), color)
            } else if let Some(ref f) = issue.location.file {
                let fname = f.file_name().and_then(|n| n.to_str()).unwrap_or("?");
                c_dim(&format!(" [{}]", fname), color)
            } else {
                String::new()
            };
            let _ = writeln!(
                out,
                "  {} {}{} {}",
                colorize(&format!("{:<7}", label), color),
                c_bold(&issue.code, color),
                location,
                issue.message,
            );
            if let Some(ref s) = issue.suggestion {
                let _ = writeln!(out, "          {} {}", c_dim("→", color), c_dim(s, color));
            }
        }
    }

    // Summary line
    let total_errors = report.validation.critical.len() + report.validation.errors.len();
    let total_warnings = report.validation.warnings.len();
    if total_errors > 0 {
        let mut reasons = Vec::new();
        reasons.push(format!("{} error(s)", total_errors));
        if total_warnings > 0 {
            reasons.push(format!("{} warning(s)", total_warnings));
        }
        let _ = writeln!(
            out,
            "{} {}",
            c_red("failed", color),
            c_bold(&reasons.join(", "), color)
        );
    } else if total_warnings > 0 {
        let _ = writeln!(
            out,
            "{}",
            c_yellow(&format!("valid  {} warning(s)", total_warnings), color)
        );
    } else {
        let _ = writeln!(out, "{}", c_green("valid", color));
    }

    out
}
