//! Delivery comparison — compare a SourceAsset against a DeliveryRequest
//!
//! Answers the question: "Does this IMF package contain everything the delivery spec requires?"

use super::source_asset::{AudioType, VideoDynamicRange, VideoQuality};
use serde::{Deserialize, Serialize};
#[cfg(feature = "typescript")]
use ts_rs::TS;

// =============================================================================
// DeliveryRequest
// =============================================================================

#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
pub struct DeliveryRequest {
    pub audio_languages: Vec<String>,
    pub subtitle_languages: Vec<String>,
    #[serde(default)]
    pub caption_languages: Vec<String>,
    #[serde(default)]
    pub forced_narrative_languages: Vec<String>,
    pub audio_type: AudioType,
    pub video_quality: VideoQuality,
    pub video_dynamic_range: VideoDynamicRange,
}

// =============================================================================
// Comparison result
// =============================================================================

#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
pub struct DeliveryComparison {
    pub matches: bool,
    pub missing_audio_languages: Vec<String>,
    pub missing_subtitle_languages: Vec<String>,
    pub missing_caption_languages: Vec<String>,
    pub missing_forced_narrative_languages: Vec<String>,
    pub extra_audio_languages: Vec<String>,
    pub extra_subtitle_languages: Vec<String>,
    pub audio_type_match: ComparisonResult,
    pub video_quality_match: ComparisonResult,
    pub video_dynamic_range_match: ComparisonResult,
}

#[cfg_attr(feature = "jsonschema", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "status", rename_all = "camelCase")]
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export, rename_all = "camelCase"))]
pub enum ComparisonResult {
    Match,
    Exceeded { requested: String, found: String },
    Insufficient { requested: String, found: String },
}

// =============================================================================
// Comparison logic
// =============================================================================

/// Compare a SourceAsset against a DeliveryRequest
pub fn compare(
    source: &super::source_asset::SourceAsset,
    request: &DeliveryRequest,
) -> DeliveryComparison {
    // Language comparisons (set difference)
    let missing_audio = find_missing(&request.audio_languages, &source.audio_languages);
    let missing_subtitles = find_missing(&request.subtitle_languages, &source.subtitle_languages);
    let missing_captions = find_missing(&request.caption_languages, &source.caption_languages);
    let missing_forced = find_missing(
        &request.forced_narrative_languages,
        &source.forced_narrative_languages,
    );
    let extra_audio = find_missing(&source.audio_languages, &request.audio_languages);
    let extra_subtitles = find_missing(&source.subtitle_languages, &request.subtitle_languages);

    // Ordered enum comparisons
    let audio_match = compare_ordered(request.audio_type, source.audio_type);
    let quality_match = compare_ordered(request.video_quality, source.video_quality);
    let range_match = compare_ordered(request.video_dynamic_range, source.video_dynamic_range);

    let is_match = missing_audio.is_empty()
        && missing_subtitles.is_empty()
        && missing_captions.is_empty()
        && missing_forced.is_empty()
        && !matches!(audio_match, ComparisonResult::Insufficient { .. })
        && !matches!(quality_match, ComparisonResult::Insufficient { .. })
        && !matches!(range_match, ComparisonResult::Insufficient { .. });

    DeliveryComparison {
        matches: is_match,
        missing_audio_languages: missing_audio,
        missing_subtitle_languages: missing_subtitles,
        missing_caption_languages: missing_captions,
        missing_forced_narrative_languages: missing_forced,
        extra_audio_languages: extra_audio,
        extra_subtitle_languages: extra_subtitles,
        audio_type_match: audio_match,
        video_quality_match: quality_match,
        video_dynamic_range_match: range_match,
    }
}

/// Find items in `required` that are not in `available`
fn find_missing(required: &[String], available: &[String]) -> Vec<String> {
    required
        .iter()
        .filter(|r| !available.iter().any(|a| a.eq_ignore_ascii_case(r)))
        .cloned()
        .collect()
}

/// Compare ordered enum values
fn compare_ordered<T: Ord + std::fmt::Debug>(requested: T, found: T) -> ComparisonResult {
    use std::cmp::Ordering;
    match found.cmp(&requested) {
        Ordering::Equal => ComparisonResult::Match,
        Ordering::Greater => ComparisonResult::Exceeded {
            requested: format!("{:?}", requested),
            found: format!("{:?}", found),
        },
        Ordering::Less => ComparisonResult::Insufficient {
            requested: format!("{:?}", requested),
            found: format!("{:?}", found),
        },
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::super::source_asset::{
        extract_source_asset, AudioType, VideoDynamicRange, VideoQuality,
    };
    use super::*;

    fn make_request(
        audio_langs: &[&str],
        sub_langs: &[&str],
        audio_type: AudioType,
        quality: VideoQuality,
        range: VideoDynamicRange,
    ) -> DeliveryRequest {
        DeliveryRequest {
            audio_languages: audio_langs.iter().map(|s| s.to_string()).collect(),
            subtitle_languages: sub_langs.iter().map(|s| s.to_string()).collect(),
            caption_languages: vec![],
            forced_narrative_languages: vec![],
            audio_type,
            video_quality: quality,
            video_dynamic_range: range,
        }
    }

    #[test]
    fn test_exact_match() {
        let cpl_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../test-data/MERIDIAN_Netflix_Photon_161006/CPL_0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85.xml"
        );
        let xml = std::fs::read_to_string(cpl_path).unwrap();
        let cpl = crate::cpl::parse_cpl(&xml).unwrap();
        let source = extract_source_asset(&cpl).unwrap();

        // Request exactly what the source has
        let request = make_request(
            &source
                .audio_languages
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>(),
            &source
                .subtitle_languages
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>(),
            source.audio_type,
            source.video_quality,
            source.video_dynamic_range,
        );

        let result = compare(&source, &request);
        assert!(result.matches, "Exact match should succeed");
        assert!(result.missing_audio_languages.is_empty());
        assert!(result.missing_subtitle_languages.is_empty());
        assert_eq!(result.audio_type_match, ComparisonResult::Match);
        assert_eq!(result.video_quality_match, ComparisonResult::Match);
        assert_eq!(result.video_dynamic_range_match, ComparisonResult::Match);
    }

    #[test]
    fn test_missing_language() {
        let cpl_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../test-data/MERIDIAN_Netflix_Photon_161006/CPL_0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85.xml"
        );
        let xml = std::fs::read_to_string(cpl_path).unwrap();
        let cpl = crate::cpl::parse_cpl(&xml).unwrap();
        let source = extract_source_asset(&cpl).unwrap();

        // Request a language that doesn't exist
        let request = make_request(
            &["nl", "fr"],
            &[],
            AudioType::Stereo,
            VideoQuality::Sd,
            VideoDynamicRange::Sdr,
        );

        let result = compare(&source, &request);

        // Should have missing languages (unless source happens to have nl/fr)
        let has_nl = source.audio_languages.contains(&"nl".to_string());
        let has_fr = source.audio_languages.contains(&"fr".to_string());
        if !has_nl {
            assert!(result.missing_audio_languages.contains(&"nl".to_string()));
        }
        if !has_fr {
            assert!(result.missing_audio_languages.contains(&"fr".to_string()));
        }
    }

    #[test]
    fn test_exceeded_quality() {
        let cpl_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../test-data/MERIDIAN_Netflix_Photon_161006/CPL_0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85.xml"
        );
        let xml = std::fs::read_to_string(cpl_path).unwrap();
        let cpl = crate::cpl::parse_cpl(&xml).unwrap();
        let source = extract_source_asset(&cpl).unwrap();

        // MERIDIAN is UHD Dolby Vision — request SD SDR
        let request = make_request(
            &[],
            &[],
            AudioType::Stereo,
            VideoQuality::Sd,
            VideoDynamicRange::Sdr,
        );

        let result = compare(&source, &request);
        assert!(result.matches, "Exceeded specs should still match");
        assert!(matches!(
            result.video_quality_match,
            ComparisonResult::Exceeded { .. }
        ));
        assert!(matches!(
            result.video_dynamic_range_match,
            ComparisonResult::Exceeded { .. }
        ));
    }

    #[test]
    fn test_insufficient_quality() {
        let cpl_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../test-data/MERIDIAN_Netflix_Photon_161006/CPL_0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85.xml"
        );
        let xml = std::fs::read_to_string(cpl_path).unwrap();
        let cpl = crate::cpl::parse_cpl(&xml).unwrap();
        let source = extract_source_asset(&cpl).unwrap();

        // Request Dolby Atmos but source only has DD+/Stereo (no Atmos in MERIDIAN audio sequences)
        let request = make_request(
            &[],
            &[],
            AudioType::DolbyAtmos,
            VideoQuality::Uhd,
            VideoDynamicRange::HdrDolbyVision,
        );

        let result = compare(&source, &request);
        // Audio type should be insufficient if source doesn't have Atmos
        if source.audio_type < AudioType::DolbyAtmos {
            assert!(!result.matches, "Insufficient audio should fail");
            assert!(matches!(
                result.audio_type_match,
                ComparisonResult::Insufficient { .. }
            ));
        }
    }

    #[test]
    fn test_find_missing() {
        let required = vec!["en".to_string(), "nl".to_string(), "fr".to_string()];
        let available = vec!["en".to_string(), "de".to_string()];
        let missing = find_missing(&required, &available);
        assert_eq!(missing, vec!["nl", "fr"]);
    }

    #[test]
    fn test_case_insensitive_language_match() {
        let required = vec!["EN".to_string()];
        let available = vec!["en".to_string()];
        let missing = find_missing(&required, &available);
        assert!(
            missing.is_empty(),
            "Language match should be case-insensitive"
        );
    }
}
