//! Unified IMF report — the single JSON document for UI consumption
//!
//! Combines package metadata, source asset extraction, validation, and delivery comparison
//! into one structure.

#[cfg(feature = "typescript")]
use ts_rs::TS;
use serde::{Deserialize, Serialize};
use super::source_asset::SourceAsset;
use super::delivery::DeliveryComparison;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
pub struct ImfReport {
    pub package: PackageSummary,
    pub cpls: Vec<CplReport>,
    pub validation: ValidationSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
pub struct CplReport {
    pub id: String,
    pub title: String,
    /// Application profile, e.g. "App2E_2021", "App2E_2014", "App5"
    pub application_profile: Option<String>,
    /// Number of segments in this CPL
    pub segment_count: usize,
    /// Timecode start address, e.g. "01:00:00:00"
    pub timecode_start: Option<String>,
    /// True if this CPL references track files not present in the current package (supplemental IMP)
    pub is_supplemental: bool,
    /// Track file UUIDs referenced in this CPL that are not in the current package's AssetMap.
    /// These must be resolved from an ancestor package.
    pub unresolved_ancestor_asset_ids: Vec<String>,
    pub source_asset: SourceAsset,
    pub markers: Vec<CplMarker>,
    pub delivery_comparison: Option<DeliveryComparison>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
pub struct CplMarker {
    pub label: String,
    pub offset: u64,
    pub annotation: Option<String>,
}

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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
pub struct ValidationSummary {
    pub valid: bool,
    pub issues: Vec<ValidationIssueEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
pub struct ValidationIssueEntry {
    pub severity: String,
    pub message: String,
}

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

/// Collect all TrackFileIds referenced in a CPL across all sequence types
fn collect_track_file_ids(cpl: &crate::cpl::CompositionPlaylist) -> Vec<String> {
    let mut ids = Vec::new();
    for seg in &cpl.segment_list.segments {
        let sl = &seg.sequence_list;
        for seq in &sl.main_image_sequences {
            for r in &seq.resource_list.resources {
                if let Some(ref id) = r.track_file_id { ids.push(id.to_string()); }
            }
        }
        for seq in &sl.main_audio_sequences {
            for r in &seq.resource_list.resources {
                if let Some(ref id) = r.track_file_id { ids.push(id.to_string()); }
            }
        }
        for seq in &sl.iab_sequences {
            for r in &seq.resource_list.resources {
                if let Some(ref id) = r.track_file_id { ids.push(id.to_string()); }
            }
        }
        for seq in &sl.isxd_sequences {
            for r in &seq.resource_list.resources {
                if let Some(ref id) = r.track_file_id { ids.push(id.to_string()); }
            }
        }
        for seq in &sl.subtitles_sequences {
            for r in &seq.resource_list.resources {
                if let Some(ref id) = r.track_file_id { ids.push(id.to_string()); }
            }
        }
        for seq in &sl.hearing_impaired_captions_sequences {
            for r in &seq.resource_list.resources {
                if let Some(ref id) = r.track_file_id { ids.push(id.to_string()); }
            }
        }
        for seq in &sl.forced_narrative_sequences {
            for r in &seq.resource_list.resources {
                if let Some(ref id) = r.track_file_id { ids.push(id.to_string()); }
            }
        }
    }
    ids
}

/// Build a full ImfReport from a package, with optional ancestor package for supplemental IMPs
pub fn build_report(
    package: &super::Imferno,
    ancestor: Option<&super::Imferno>,
    delivery_spec: Option<&super::delivery::DeliveryRequest>,
) -> Result<ImfReport, String> {
    // Package summary
    let inspection = package.inspect();
    let pkg_summary = PackageSummary {
        asset_map_id: inspection.asset_map_id.clone(),
        volume_index: inspection.volume_index,
        asset_count: inspection.asset_count,
        cpl_count: inspection.cpl_count,
        issue_date: inspection.asset_map_issue_date.clone(),
        issuer: inspection.asset_map_issuer.clone(),
        creator: inspection.asset_map_creator.clone(),
        pkl_count: package.packing_lists.len(),
    };

    if package.composition_playlists.is_empty() {
        return Err("No CPLs found in package".to_string());
    }

    // Extract source asset for every CPL
    let mut cpls = Vec::new();
    for cpl in package.composition_playlists.values() {
        // Detect supplemental: find track file IDs not in this package's AssetMap
        let cpl_track_ids = collect_track_file_ids(cpl);
        let unresolved_ancestor_asset_ids: Vec<String> = cpl_track_ids.iter()
            .filter(|id| {
                package.get_asset_path_str(id).is_none()
                    && ancestor.map_or(true, |a| a.get_asset_path_str(id).is_none())
            })
            .cloned()
            .collect();
        let is_supplemental = cpl_track_ids.iter().any(|id| {
            package.get_asset_path_str(id).is_none()
        });

        let source_asset = super::source_asset::extract_source_asset(cpl)?;
        let delivery_comparison = delivery_spec.map(|spec| {
            super::delivery::compare(&source_asset, spec)
        });

        // Collect all markers from all segments
        let markers: Vec<CplMarker> = cpl.segment_list.segments.iter()
            .flat_map(|seg| seg.sequence_list.marker_sequences.iter())
            .flat_map(|ms| ms.resource_list.resources.iter())
            .flat_map(|res| res.markers.iter())
            .map(|m| CplMarker {
                label: m.label.to_string(),
                offset: m.offset,
                annotation: m.annotation.clone().filter(|a| !a.is_empty()),
            })
            .collect();

        let application_profile = cpl.extension_properties
            .as_ref()
            .and_then(|ep| ep.application_identification.as_ref())
            .map(|url| parse_application_profile(url));

        let segment_count = cpl.segment_list.segments.len();

        let timecode_start = cpl.composition_timecode
            .as_ref()
            .and_then(|tc| tc.timecode_start_address.clone());

        cpls.push(CplReport {
            id: cpl.id.to_string(),
            title: cpl.content_title.text.clone(),
            application_profile,
            segment_count,
            timecode_start,
            is_supplemental,
            unresolved_ancestor_asset_ids,
            source_asset,
            markers,
            delivery_comparison,
        });
    }

    // Validation
    let val_report = package.validate(&super::ValidationOptions::default());
    let validation = if val_report.has_errors() || val_report.has_critical() {
        let issues = val_report.critical.iter()
            .chain(val_report.errors.iter())
            .map(|i| ValidationIssueEntry {
                severity: match i.severity {
                    crate::diagnostics::Severity::Critical => "critical".to_string(),
                    crate::diagnostics::Severity::Error    => "error".to_string(),
                    crate::diagnostics::Severity::Warning  => "warning".to_string(),
                    crate::diagnostics::Severity::Info     => "info".to_string(),
                },
                message: i.message.clone(),
            })
            .collect();
        ValidationSummary {
            valid: false,
            issues,
        }
    } else {
        ValidationSummary {
            valid: true,
            issues: vec![],
        }
    };

    Ok(ImfReport {
        package: pkg_summary,
        cpls,
        validation,
    })
}
