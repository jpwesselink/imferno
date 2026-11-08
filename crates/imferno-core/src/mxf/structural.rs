//! ST 377-1 structural-metadata rules evaluated against the RegXML
//! produced by `mxf::metadata::parse_mxf_to_regxml`.
//!
//! - **Annex B.8** — a Structural Component strongly referenced from a
//!   Timeline Track has Duration as a Best Effort property (distinguished
//!   value -1), and §4.4 Table 3 says Best Effort properties shall always
//!   be encoded. Leaving Duration out is therefore non-conformant. The
//!   ST 377-1 working group confirmed this reading and that the
//!   requirement is normative already:
//!   <https://github.com/SMPTE/ST377-1-full-revision/issues/18>.
//!
//! In RegXML (AAF names) Duration is `ComponentLength`, Structural
//! Components hang off a track's `TrackSegment` and a Sequence's
//! `ComponentObjects`.

use std::path::Path;

use quick_xml::events::Event;

use crate::catalogue::st377_1::{self, St377_1Edition};
use crate::diagnostics::{Location, ValidationIssue};

struct Open {
    name: String,
    /// The element is a Structural Component on a Timeline Track.
    is_timeline_component: bool,
    has_duration: bool,
    /// The nearest track ancestor (or this element) is a Timeline Track.
    in_timeline_track: bool,
}

/// Report Structural Components on Timeline Tracks that have no
/// Duration (ST 377-1:2011 Annex B.8).
/// `st377_1` is the ST 377-1 edition of the track file.
pub fn check_structural_components(
    regxml: &str,
    path: &Path,
    st377_1: St377_1Edition,
) -> Vec<ValidationIssue> {
    let mut missing: Vec<String> = Vec::new();
    let mut stack: Vec<Open> = Vec::new();
    let mut reader = quick_xml::Reader::from_str(regxml);

    let open = |stack: &mut Vec<Open>, name: String| {
        let parent = stack.last();
        let in_timeline_track = match name.as_str() {
            "TimelineTrack" => true,
            "EventTrack" | "StaticTrack" => false,
            _ => parent.is_some_and(|p| p.in_timeline_track),
        };
        let is_timeline_component = in_timeline_track
            && parent.is_some_and(|p| p.name == "TrackSegment" || p.name == "ComponentObjects");
        stack.push(Open {
            name,
            is_timeline_component,
            has_duration: false,
            in_timeline_track,
        });
    };
    let close = |stack: &mut Vec<Open>, missing: &mut Vec<String>| {
        if let Some(el) = stack.pop() {
            if el.name == "ComponentLength" {
                if let Some(parent) = stack.last_mut() {
                    parent.has_duration = true;
                }
            }
            if el.is_timeline_component && !el.has_duration {
                missing.push(el.name);
            }
        }
    };

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let name = e.local_name().into_inner().to_string();
                open(&mut stack, name);
            }
            Ok(Event::Empty(e)) => {
                let name = e.local_name().into_inner().to_string();
                open(&mut stack, name);
                close(&mut stack, &mut missing);
            }
            Ok(Event::End(_)) => close(&mut stack, &mut missing),
            Ok(Event::Eof) | Err(_) => break,
            Ok(_) => {}
        }
    }

    missing
        .into_iter()
        .map(|component| {
            ValidationIssue::from_rule(
                &st377_1::StructuralComponentDurationMissing,
                st377_1,
                format!(
                    "{component} on a Timeline Track in MXF file at {} has no Duration; \
                     ST 377-1 Annex B.8 makes it a Best Effort property that shall be \
                     encoded (-1 when unknown)",
                    path.display(),
                ),
            )
            .with_location(Location::new().with_file(path.to_path_buf()))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The check under ST 377-1:2011.
    fn check_structural_components(regxml: &str, path: &Path) -> Vec<ValidationIssue> {
        super::check_structural_components(regxml, path, St377_1Edition::V2011)
    }

    fn track(kind: &str, clip: &str) -> String {
        format!(
            r#"<r:Preface xmlns:r="urn:x"><r:{kind}><r:TrackID>2</r:TrackID><r:TrackSegment>
<r:Sequence><r:ComponentDataDefinition>x</r:ComponentDataDefinition><r:ComponentLength>40</r:ComponentLength>
<r:ComponentObjects>{clip}</r:ComponentObjects></r:Sequence></r:TrackSegment></r:{kind}></r:Preface>"#
        )
    }

    fn check(xml: &str) -> Vec<ValidationIssue> {
        check_structural_components(xml, Path::new("t.mxf"))
    }

    #[test]
    fn duration_present_passes() {
        let xml = track(
            "TimelineTrack",
            "<r:SourceClip><r:ComponentLength>40</r:ComponentLength></r:SourceClip>",
        );
        assert!(check(&xml).is_empty());
    }

    #[test]
    fn distinguished_value_passes() {
        let xml = track(
            "TimelineTrack",
            "<r:SourceClip><r:ComponentLength>-1</r:ComponentLength></r:SourceClip>",
        );
        assert!(check(&xml).is_empty());
    }

    #[test]
    fn missing_source_clip_duration_is_an_error() {
        let xml = track(
            "TimelineTrack",
            "<r:SourceClip><r:StartPosition>0</r:StartPosition></r:SourceClip>",
        );
        let issues = check(&xml);
        assert_eq!(issues.len(), 1, "{issues:#?}");
        assert_eq!(
            issues[0].code,
            "ST377-1:2011:B.8/StructuralComponentDurationMissing"
        );
        assert!(issues[0].message.contains("SourceClip"));
    }

    #[test]
    fn missing_sequence_duration_is_an_error() {
        let xml = r#"<r:Preface xmlns:r="urn:x"><r:TimelineTrack><r:TrackSegment><r:Sequence>
<r:ComponentObjects><r:Timecode><r:ComponentLength>1</r:ComponentLength></r:Timecode></r:ComponentObjects>
</r:Sequence></r:TrackSegment></r:TimelineTrack></r:Preface>"#;
        let issues = check(xml);
        assert_eq!(issues.len(), 1, "{issues:#?}");
        assert!(issues[0].message.contains("Sequence"));
    }

    #[test]
    fn event_and_static_tracks_are_not_checked() {
        let clip = "<r:DMSegment><r:EventStartPosition>0</r:EventStartPosition></r:DMSegment>";
        assert!(check(&track("EventTrack", clip)).is_empty());
        let xml = r#"<r:Preface xmlns:r="urn:x"><r:StaticTrack><r:TrackSegment><r:Sequence>
<r:ComponentObjects><r:DMSegment/></r:ComponentObjects></r:Sequence></r:TrackSegment></r:StaticTrack></r:Preface>"#;
        assert!(check(xml).is_empty());
    }

    /// The vendored corpus MXFs encode Duration everywhere.
    #[test]
    fn corpus_timed_text_mxf_passes() {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../test-data/MERIDIAN_Netflix_Photon_161006/MERIDIAN_Netflix_Photon_161006_00_tt.mxf",
        );
        if !path.exists() {
            return;
        }
        let xml = crate::mxf::metadata::parse_mxf_to_regxml_with_partition_fallback(
            &path,
            regxml::RootMode::Preface,
        )
        .expect("RegXML");
        assert!(xml.contains("ComponentLength"));
        let issues = check_structural_components(&xml, &path);
        assert!(issues.is_empty(), "{issues:#?}");
    }
}
