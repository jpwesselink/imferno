//! Unified IMF report — the single JSON document for UI consumption
//!
//! Combines package metadata, validation, and structural analysis into one structure.
//! Also provides `format_report()` for pretty-printing to a terminal.

use crate::assetmap::ImfUuid;
use crate::cpl::{EssenceDescriptor, SequenceAccess};
use crate::diagnostics::{Severity, ValidationReport};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::ValidationResult;
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
    /// RFC 5646 language tag extracted from the essence descriptor (e.g. "en", "fr")
    pub language: Option<String>,
    /// Audio channel count (e.g. 2, 6, 8) — only for audio sequences
    pub channel_count: Option<u32>,
    /// MCA soundfield label (e.g. "5.1", "7.1", "Atmos") — only for audio sequences
    pub soundfield: Option<String>,
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

/// Extract the language from an `EssenceDescriptor` by checking audio, timed-text, and IAB sub-descriptors.
fn language_from_descriptor(ed: &EssenceDescriptor) -> Option<String> {
    // Audio: WAVEPCMDescriptor → soundfield_group_label_sub_descriptor → rfc5646_spoken_language
    if let Some(wave) = &ed.wave_pcm_descriptor {
        if let Some(subs) = &wave.sub_descriptors {
            if let Some(sf) = &subs.soundfield_group_label_sub_descriptor {
                if let Some(lang) = &sf.rfc5646_spoken_language {
                    let s = lang.as_str();
                    if !s.is_empty() {
                        return Some(s.to_string());
                    }
                }
            }
        }
    }
    // IAB: IABEssenceDescriptor → iab_soundfield_label_sub_descriptor → rfc5646_spoken_language
    if let Some(iab) = &ed.iab_essence_descriptor {
        if let Some(subs) = &iab.sub_descriptors {
            if let Some(sf) = &subs.iab_soundfield_label_sub_descriptor {
                if let Some(lang) = &sf.rfc5646_spoken_language {
                    let s = lang.as_str();
                    if !s.is_empty() {
                        return Some(s.to_string());
                    }
                }
            }
        }
    }
    // Timed text: DCTimedTextDescriptor → rfc5646_language_tag_list
    if let Some(tt) = &ed.dc_timed_text_descriptor {
        let langs: Vec<&str> = tt
            .rfc5646_language_tag_list
            .iter()
            .map(|lt| lt.as_str())
            .filter(|s| !s.is_empty())
            .collect();
        if !langs.is_empty() {
            return Some(langs.join(","));
        }
    }
    None
}

/// Extract the audio channel count from an essence descriptor.
fn channel_count_from_descriptor(ed: &EssenceDescriptor) -> Option<u32> {
    if let Some(wave) = &ed.wave_pcm_descriptor {
        return wave.channel_count;
    }
    if let Some(iab) = &ed.iab_essence_descriptor {
        return iab.channel_count;
    }
    None
}

/// Extract the soundfield label (e.g. "5.1", "7.1", "Atmos") from an essence descriptor.
fn soundfield_from_descriptor(ed: &EssenceDescriptor) -> Option<String> {
    if let Some(wave) = &ed.wave_pcm_descriptor {
        if let Some(subs) = &wave.sub_descriptors {
            if let Some(sf) = &subs.soundfield_group_label_sub_descriptor {
                if let Some(mca) = &sf.mca_tag_symbol {
                    return Some(mca.to_string());
                }
                if let Some(name) = &sf.mca_tag_name {
                    return Some(name.clone());
                }
            }
        }
    }
    if let Some(iab) = &ed.iab_essence_descriptor {
        if let Some(subs) = &iab.sub_descriptors {
            if let Some(sf) = &subs.iab_soundfield_label_sub_descriptor {
                if let Some(mca) = &sf.mca_tag_symbol {
                    return Some(mca.to_string());
                }
            }
        }
        // IAB without a label is Dolby Atmos
        return Some("Atmos".to_string());
    }
    None
}

/// Merge a single trait-object sequence into the track map
fn merge_sequences_dyn(
    track_map: &mut HashMap<String, CplSequence>,
    type_name: &str,
    seq: &dyn SequenceAccess,
    cpl_er: &Option<String>,
    descriptors: &HashMap<ImfUuid, &EssenceDescriptor>,
) {
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
        // Resolve metadata from essence descriptor
        let ed = seq
            .resource_list()
            .resources
            .first()
            .and_then(|r| r.source_encoding.as_ref())
            .and_then(|se| descriptors.get(se).copied());
        let language = ed.and_then(language_from_descriptor);
        let channel_count = ed.and_then(channel_count_from_descriptor);
        let soundfield = ed.and_then(soundfield_from_descriptor);
        track_map.insert(
            tid.clone(),
            CplSequence {
                r#type: type_name.to_string(),
                id: seq.id().to_string(),
                track_id: tid,
                language,
                channel_count,
                soundfield,
                resources,
            },
        );
    }
}

/// Extract all virtual tracks from a CPL, merged across segments
fn extract_sequences(cpl: &crate::cpl::CompositionPlaylist) -> (Option<String>, Vec<CplSequence>) {
    let edit_rate = cpl
        .edit_rate
        .as_ref()
        .map(|er| format!("{}/{}", er.numerator, er.denominator));

    // Build essence descriptor lookup by ID for language resolution
    let descriptors: HashMap<ImfUuid, &EssenceDescriptor> =
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
        for (seq, type_name) in seg.sequence_list.all_sequences_typed() {
            merge_sequences_dyn(&mut track_map, type_name, seq, &edit_rate, &descriptors);
        }
    }

    (edit_rate, track_map.into_values().collect())
}

/// Collect all TrackFileIds referenced in a CPL across all sequence types
fn collect_track_file_ids(cpl: &crate::cpl::CompositionPlaylist) -> Vec<String> {
    let mut ids = Vec::new();
    for seg in &cpl.segment_list.segments {
        for seq in seg.sequence_list.all_sequences() {
            for r in &seq.resource_list().resources {
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

fn ansi(code: &str, text: &str, enabled: bool) -> String {
    if enabled {
        format!("\x1b[{}m{}\x1b[0m", code, text)
    } else {
        text.to_string()
    }
}

fn c_red(s: &str, on: bool) -> String {
    ansi("31", s, on)
}
fn c_yellow(s: &str, on: bool) -> String {
    ansi("33", s, on)
}
fn c_cyan(s: &str, on: bool) -> String {
    ansi("36", s, on)
}
fn c_green(s: &str, on: bool) -> String {
    ansi("32", s, on)
}
fn c_bold(s: &str, on: bool) -> String {
    ansi("1", s, on)
}
fn c_dim(s: &str, on: bool) -> String {
    ansi("2", s, on)
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

    // Timeline / virtual tracks per CPL
    for cpl in &report.cpls {
        if !cpl.sequences.is_empty() {
            let _ = writeln!(
                out,
                "{}",
                c_bold(
                    &format!("Timeline [CPL:{}]:", &cpl.id[..cpl.id.len().min(8)]),
                    color,
                )
            );
            for seq in &cpl.sequences {
                let label = match seq.language.as_deref() {
                    Some(lang) => format!("{} ({})", seq.r#type, lang),
                    None => seq.r#type.clone(),
                };
                let resource_count = seq.resources.len();
                let _ = writeln!(
                    out,
                    "  {}  {} — {} resource(s)",
                    c_cyan("track", color),
                    c_bold(&label, color),
                    resource_count,
                );
            }
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
                let s = c.to_string();
                c_dim(&format!(" [CPL:{}]", &s[..s.len().min(8)]), color)
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

// ── format_validation_result ────────────────────────────────────────────────

/// Output format for `format_validation_result`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportFormat {
    /// Human-readable plain text (with optional ANSI color).
    Text,
    /// Markdown — headers, tables, code blocks. Embeddable in PRs, Slack, Notion.
    Markdown,
    /// CSV — one row per validation issue. Importable into Excel, dashboards.
    Csv,
}

/// Format options for `format_validation_result`.
#[derive(Debug, Clone)]
pub struct FormatOptions {
    pub format: ReportFormat,
    pub color: bool,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            format: ReportFormat::Text,
            color: false,
        }
    }
}

/// Format a `ValidationResult` (full package + validation) as text, markdown, or CSV.
///
/// This is the recommended formatting function — it reads directly from the
/// `Imferno` struct and `ValidationReport` without the lossy `ImfReport` intermediate.
pub fn format_validation_result(result: &ValidationResult, opts: &FormatOptions) -> String {
    match opts.format {
        ReportFormat::Text => format_text(result, opts.color),
        ReportFormat::Markdown => format_markdown(result),
        ReportFormat::Csv => format_csv(result),
    }
}

fn format_text(result: &ValidationResult, color: bool) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    let pkg = &result.package;
    let v = &result.validation;

    // Package structure
    let _ = writeln!(
        out,
        "  {}  ASSETMAP.xml — {} assets",
        c_green("ok", color),
        pkg.asset_map.asset_list.assets.len()
    );
    let _ = writeln!(
        out,
        "  {}  {} CPL(s), {} PKL(s)",
        c_green("ok", color),
        pkg.composition_playlists.len(),
        pkg.packing_lists.len()
    );
    if !pkg.sidecar_composition_maps.is_empty() {
        let _ = writeln!(
            out,
            "  {}  {} SCM(s)",
            c_green("ok", color),
            pkg.sidecar_composition_maps.len()
        );
    }

    // CPL timeline
    for (uuid, cpl) in &pkg.composition_playlists {
        let title = &cpl.content_title;
        let _ = writeln!(
            out,
            "\n{}",
            c_bold(
                &format!("CPL [{}] {}", &uuid.to_string()[..8], title),
                color
            )
        );

        for seg in &cpl.segment_list.segments {
            for seq in seg.sequence_list.all_sequences_typed() {
                let (s, type_name) = seq;
                let resource_count = s.resource_list().resources.len();
                let lang = language_from_descriptor_lookup(s, cpl);
                let label = match lang {
                    Some(l) => format!("{} ({})", type_name, l),
                    None => type_name.to_string(),
                };

                // Per-track media info from essence descriptor
                let media_detail = descriptor_lookup(s, cpl).and_then(|ed| {
                    if let Some(vi) = video_info_from_descriptor(ed) {
                        Some(format!(
                            "{} {} {} {} {}",
                            vi.resolution, vi.frame_rate, vi.codec, vi.bit_depth, vi.dynamic_range
                        ))
                    } else if ed.iab_essence_descriptor.is_some() {
                        Some("IAB (Dolby Atmos)".into())
                    } else if let Some(ai) = audio_info_from_descriptor(ed) {
                        Some(format!("{} {} {}", ai.format, ai.sample_rate, ai.bit_depth))
                    } else {
                        None
                    }
                });

                let resources_str = s
                    .resource_list()
                    .resources
                    .iter()
                    .filter_map(|r| {
                        r.track_file_id
                            .as_ref()
                            .map(|id| id.to_string()[..8].to_string())
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                let resources_display = if resources_str.is_empty() {
                    format!("{} resource(s)", resource_count)
                } else {
                    resources_str
                };

                let detail_str = match media_detail {
                    Some(d) => format!(
                        " — {} — {}",
                        c_dim(&d, color),
                        c_dim(&resources_display, color)
                    ),
                    None => format!(" — {}", c_dim(&resources_display, color)),
                };
                let _ = writeln!(
                    out,
                    "  {}  {}{}",
                    c_cyan("track", color),
                    c_bold(&label, color),
                    detail_str,
                );
            }
        }
    }

    // Validation findings
    format_issues_text(&mut out, v, color);
    format_summary_text(&mut out, v, color);

    out
}

fn format_markdown(result: &ValidationResult) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    let pkg = &result.package;
    let v = &result.validation;

    let _ = writeln!(out, "# IMF Package Validation Report\n");
    let _ = writeln!(
        out,
        "**AssetMap:** {} | **CPLs:** {} | **PKLs:** {}\n",
        pkg.asset_map.id,
        pkg.composition_playlists.len(),
        pkg.packing_lists.len()
    );

    // CPLs
    for (uuid, cpl) in &pkg.composition_playlists {
        let _ = writeln!(
            out,
            "## CPL: {} (`{}`)\n",
            cpl.content_title,
            &uuid.to_string()[..8]
        );

        let _ = writeln!(out, "| Track | Language | Resources |");
        let _ = writeln!(out, "|-------|----------|-----------|");
        for seg in &cpl.segment_list.segments {
            for (s, type_name) in seg.sequence_list.all_sequences_typed() {
                let lang =
                    language_from_descriptor_lookup(s, cpl).unwrap_or_else(|| "—".to_string());
                let _ = writeln!(
                    out,
                    "| {} | {} | {} |",
                    type_name,
                    lang,
                    s.resource_list().resources.len()
                );
            }
        }
        let _ = writeln!(out);
    }

    // Validation issues
    let all_issues = collect_all_issues(v);
    if !all_issues.is_empty() {
        let _ = writeln!(out, "## Validation Findings\n");
        let _ = writeln!(out, "| Severity | Code | Message |");
        let _ = writeln!(out, "|----------|------|---------|");
        for issue in &all_issues {
            let sev = match issue.severity {
                Severity::Critical => "🔴 Critical",
                Severity::Error => "🔴 Error",
                Severity::Warning => "🟡 Warning",
                Severity::Info => "🔵 Info",
            };
            let msg = issue.message.replace('|', "\\|");
            let _ = writeln!(out, "| {} | `{}` | {} |", sev, issue.code, msg);
        }
        let _ = writeln!(out);
    }

    // Summary
    let total_errors = v.critical.len() + v.errors.len();
    if total_errors > 0 {
        let _ = writeln!(
            out,
            "**Result: FAILED** — {} error(s), {} warning(s)",
            total_errors,
            v.warnings.len()
        );
    } else if !v.warnings.is_empty() {
        let _ = writeln!(out, "**Result: VALID** — {} warning(s)", v.warnings.len());
    } else {
        let _ = writeln!(out, "**Result: VALID**");
    }

    out
}

fn format_csv(result: &ValidationResult) -> String {
    use std::fmt::Write;
    let mut out = String::new();

    // Header
    let _ = writeln!(out, "severity,code,message,cpl_id,suggestion");

    // One row per issue
    for issue in collect_all_issues(&result.validation) {
        let cpl_id = issue
            .location
            .cpl_id
            .as_ref()
            .map(|u| u.to_string())
            .unwrap_or_default();
        let suggestion = issue.suggestion.as_deref().unwrap_or("");
        let _ = writeln!(
            out,
            "{},{},\"{}\",{},\"{}\"",
            match issue.severity {
                Severity::Critical => "critical",
                Severity::Error => "error",
                Severity::Warning => "warning",
                Severity::Info => "info",
            },
            issue.code,
            issue.message.replace('"', "\"\""),
            cpl_id,
            suggestion.replace('"', "\"\""),
        );
    }

    out
}

// ── Shared helpers for new formatter ────────────────────────────────────────

fn collect_all_issues(v: &ValidationReport) -> Vec<&crate::diagnostics::ValidationIssue> {
    v.critical
        .iter()
        .chain(v.errors.iter())
        .chain(v.warnings.iter())
        .chain(v.info.iter())
        .collect()
}

fn format_issues_text(out: &mut String, v: &ValidationReport, color: bool) {
    use std::fmt::Write;
    let all_issues = collect_all_issues(v);
    if all_issues.is_empty() {
        return;
    }

    let _ = writeln!(out, "\n{}", c_bold("Validation findings:", color));
    for issue in &all_issues {
        let (label, colorize): (&str, fn(&str, bool) -> String) = match issue.severity {
            Severity::Critical => ("error", c_red),
            Severity::Error => ("error", c_red),
            Severity::Warning => ("warning", c_yellow),
            Severity::Info => ("info", c_cyan),
        };
        let location = if let Some(ref c) = issue.location.cpl_id {
            let s = c.to_string();
            c_dim(&format!(" [CPL:{}]", &s[..s.len().min(8)]), color)
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

fn format_summary_text(out: &mut String, v: &ValidationReport, color: bool) {
    use std::fmt::Write;
    let total_errors = v.critical.len() + v.errors.len();
    let total_warnings = v.warnings.len();
    if total_errors > 0 {
        let mut reasons = Vec::new();
        reasons.push(format!("{} error(s)", total_errors));
        if total_warnings > 0 {
            reasons.push(format!("{} warning(s)", total_warnings));
        }
        let _ = writeln!(
            out,
            "\n{} {}",
            c_red("failed", color),
            c_bold(&reasons.join(", "), color)
        );
    } else if total_warnings > 0 {
        let _ = writeln!(
            out,
            "\n{}",
            c_yellow(&format!("valid  {} warning(s)", total_warnings), color)
        );
    } else {
        let _ = writeln!(out, "\n{}", c_green("valid", color));
    }
}

// ── Media info helpers ──────────────────────────────────────────────────────

struct VideoInfo {
    resolution: String,
    frame_rate: String,
    codec: String,
    dynamic_range: String,
    bit_depth: String,
}

struct AudioInfo {
    format: String,
    sample_rate: String,
    bit_depth: String,
}

fn video_info_from_descriptor(ed: &EssenceDescriptor) -> Option<VideoInfo> {
    // Try CDCI first (most common for YCbCr), then RGBA
    let (width, height, sample_rate, tc, codec, bit_depth, sub_descs) =
        if let Some(ref d) = ed.cdci_descriptor {
            (
                d.stored_width,
                d.stored_height,
                d.sample_rate.as_ref(),
                d.transfer_characteristic.as_ref(),
                d.picture_compression.as_ref(),
                d.component_depth,
                d.sub_descriptors.as_ref(),
            )
        } else if let Some(ref d) = ed.rgba_descriptor {
            (
                d.stored_width,
                d.stored_height,
                d.sample_rate.as_ref(),
                d.transfer_characteristic.as_ref(),
                d.picture_compression.as_ref(),
                None,
                d.sub_descriptors.as_ref(),
            )
        } else {
            return None;
        };

    let resolution = match (width, height) {
        (Some(w), Some(h)) => format!("{}x{}", w, h),
        _ => "?".into(),
    };

    let frame_rate = match sample_rate {
        Some(r) => {
            let fps = r.numerator as f64 / r.denominator as f64;
            if r.denominator == 1 {
                format!("{}fps", r.numerator)
            } else {
                format!("{:.2}fps", fps)
            }
        }
        None => "?".into(),
    };

    use crate::cpl::types::TransferCharacteristic;
    let has_dolby_vision = sub_descs
        .map(|s| s.phdr_metadata_track_sub_descriptor.is_some())
        .unwrap_or(false);
    let dynamic_range = if has_dolby_vision {
        "Dolby Vision".into()
    } else {
        match tc {
            Some(TransferCharacteristic::PqSt2084) => "HDR10 (PQ)".into(),
            Some(TransferCharacteristic::Hlg) => "HLG".into(),
            Some(TransferCharacteristic::Bt709) => "SDR".into(),
            Some(TransferCharacteristic::Bt2020) => "SDR (BT.2020)".into(),
            Some(TransferCharacteristic::Linear) => "Linear".into(),
            Some(TransferCharacteristic::Smpte240M) => "SDR (240M)".into(),
            Some(TransferCharacteristic::XvYcc709) => "SDR (xvYCC)".into(),
            Some(TransferCharacteristic::Unknown(s)) => format!("Unknown ({})", s),
            None => "?".into(),
        }
    };

    use crate::cpl::types::VideoCodec;
    let codec = match codec {
        Some(VideoCodec::Jpeg2000) => "JPEG 2000".into(),
        Some(VideoCodec::Jpeg2000Imf2k) => "JPEG 2000 (2K)".into(),
        Some(VideoCodec::Jpeg2000Imf4k) => "JPEG 2000 (4K)".into(),
        Some(VideoCodec::Jpeg2000Broadcast) => "JPEG 2000 (BCP)".into(),
        Some(VideoCodec::Jpeg2000Ht) => "JPEG 2000 HT".into(),
        Some(VideoCodec::Vc5) => "VC-5".into(),
        Some(VideoCodec::Mpeg2) => "MPEG-2".into(),
        Some(VideoCodec::H264) => "H.264".into(),
        Some(VideoCodec::H265) => "H.265".into(),
        Some(VideoCodec::ProRes) => "ProRes".into(),
        Some(VideoCodec::Av1) => "AV1".into(),
        Some(VideoCodec::Unknown(s)) => format!("Unknown ({})", s),
        None => "?".into(),
    };

    let bit_depth = match bit_depth {
        Some(d) => format!("{}-bit", d),
        None => "?".into(),
    };

    Some(VideoInfo {
        resolution,
        frame_rate,
        codec,
        dynamic_range,
        bit_depth,
    })
}

fn audio_info_from_descriptor(ed: &EssenceDescriptor) -> Option<AudioInfo> {
    if let Some(ref _iab) = ed.iab_essence_descriptor {
        return Some(AudioInfo {
            format: "IAB (Dolby Atmos)".into(),
            sample_rate: "—".into(),
            bit_depth: "—".into(),
        });
    }

    let d = ed.wave_pcm_descriptor.as_ref()?;
    let channel_count = d.channel_count.unwrap_or(0);

    use crate::cpl::types::McaTagSymbol;
    let mca = d
        .sub_descriptors
        .as_ref()
        .and_then(|s| s.soundfield_group_label_sub_descriptor.as_ref())
        .and_then(|s| s.mca_tag_symbol.as_ref());

    let format = match mca {
        Some(McaTagSymbol::Sg51) => "5.1 Surround".into(),
        Some(McaTagSymbol::Sg71) => "7.1 Surround".into(),
        Some(McaTagSymbol::Sg71Ds) => "7.1 Dolby Surround".into(),
        Some(McaTagSymbol::SgSt) => "Stereo".into(),
        Some(McaTagSymbol::SgMono) => "Mono".into(),
        Some(McaTagSymbol::Iab) => "IAB (Dolby Atmos)".into(),
        Some(McaTagSymbol::Other(s)) => s.clone(),
        _ => match channel_count {
            1 => "Mono".into(),
            2 => "Stereo".into(),
            6 => "5.1".into(),
            8 => "7.1".into(),
            n => format!("{}ch", n),
        },
    };

    let sample_rate = match d.audio_sample_rate.as_ref().or(d.sample_rate.as_ref()) {
        Some(r) => {
            let hz = r.numerator as f64 / r.denominator as f64;
            if hz >= 1000.0 {
                format!("{:.1}kHz", hz / 1000.0)
            } else {
                format!("{}Hz", hz as u32)
            }
        }
        None => "?".into(),
    };

    let bit_depth = match d.quantization_bits {
        Some(b) => format!("{}-bit", b),
        None => "?".into(),
    };

    Some(AudioInfo {
        format,
        sample_rate,
        bit_depth,
    })
}

/// Look up the essence descriptor for a sequence from the CPL.
fn descriptor_lookup<'a>(
    seq: &dyn SequenceAccess,
    cpl: &'a crate::cpl::CompositionPlaylist,
) -> Option<&'a EssenceDescriptor> {
    let se = seq
        .resource_list()
        .resources
        .first()?
        .source_encoding
        .as_ref()?;
    let edl = cpl.essence_descriptor_list.as_ref()?;
    edl.essence_descriptors.iter().find(|e| &e.id == se)
}

/// Look up language for a sequence from the CPL's essence descriptor list.
fn language_from_descriptor_lookup(
    seq: &dyn SequenceAccess,
    cpl: &crate::cpl::CompositionPlaylist,
) -> Option<String> {
    let se = seq
        .resource_list()
        .resources
        .first()?
        .source_encoding
        .as_ref()?;
    let edl = cpl.essence_descriptor_list.as_ref()?;
    let ed = edl.essence_descriptors.iter().find(|e| &e.id == se)?;
    language_from_descriptor(ed)
}
