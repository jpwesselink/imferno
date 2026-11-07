//! ST 429-9:2014 Annex A (Basic Map Profile v2, normative) Path rules.
//!
//! Annex A is the Map Profile for a package on a hierarchical filesystem
//! (direct attached storage). The IMF HTTP Map Profile (ST 2067-2:2016
//! Annex L, ST 2067-2:2020 Annex K: segments up to 128 characters, Path up
//! to 1024) applies to HTTP retrieval only; on a filesystem the Annex A
//! limits apply (https://github.com/Netflix/photon/issues/192).
//! The delivery method is not known from the package, so the 429-9
//! Annex A limits are always used.
//!
//! Every rule here reads only the `<Path>` values of the Asset Map XML, so
//! it runs for every input (filesystem, in-memory, WASM). Two A.2 rules
//! need the filesystem and are not checked here: "within the Mapped File
//! Set" (resolved paths are confined to the package root when assets are
//! located on disk) and case preservation (the Path and the file name on
//! disk shall have identical case), which is not checked.

use std::collections::HashMap;

use crate::catalogue::st429_9::{self, St429_9Edition};
use crate::diagnostics::Rule;
use crate::diagnostics::ValidationIssue;

const MAX_PATH_CHARS: usize = 100;
const MAX_SEGMENT_CHARS: usize = 100;
const MAX_SEGMENTS: usize = 10;

fn allowed(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.')
}

/// Check every Chunk Path of an Asset Map against Annex A.2.
pub fn basic_map_profile_v2_issues<'a>(
    paths: impl IntoIterator<Item = &'a str>,
) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();
    let mut seen: HashMap<String, &str> = HashMap::new();

    for path in paths {
        let issue = |rule: &'static Rule<St429_9Edition>, detail: String| {
            ValidationIssue::from_rule(
                rule,
                St429_9Edition::V2014,
                format!("Asset Map Path '{path}' {detail}"),
            )
            .with_context("path", path.to_string())
        };

        // RFC 3986 relative-path reference: no scheme, does not start
        // with "/", no query or fragment component.
        let first_segment = path.split('/').next().unwrap_or("");
        if path.starts_with('/')
            || first_segment.contains(':')
            || path.contains('?')
            || path.contains('#')
        {
            issues.push(issue(
                &st429_9::PathNotRelative,
                "is not a relative-path reference without query or fragment (ST 429-9 A.2)".into(),
            ));
        }

        // Empty segments (`a//b`, a trailing `/`) are not flagged: the
        // RFC 3986 relative-path grammar allows empty segments after the
        // first (path-noscheme = segment-nz-nc *( "/" segment ), segment =
        // *pchar), and A.2 only constrains the characters a segment has.
        let segments: Vec<&str> = path.split('/').collect();
        // "/" separates segments; "?" and "#" are reported above. The
        // segment rule still covers them, so they are listed here too.
        let mut bad: Vec<char> = Vec::new();
        for c in path.chars().filter(|&c| c != '/' && !allowed(c)) {
            if !bad.contains(&c) {
                bad.push(c);
            }
        }
        if !bad.is_empty() {
            let listed: Vec<String> = bad.iter().map(|c| format!("{c:?}")).collect();
            issues.push(issue(
                &st429_9::PathCharacters,
                format!(
                    "has characters outside a-z, A-Z, 0-9, '-', '_', '.': {} (ST 429-9 A.2)",
                    listed.join(", ")
                ),
            ));
        }

        let path_len = path.chars().count();
        if path_len > MAX_PATH_CHARS {
            issues.push(issue(
                &st429_9::PathTooLong,
                format!("is {path_len} characters; the limit is {MAX_PATH_CHARS} (ST 429-9 A.2)"),
            ));
        } else if let Some(long) = segments
            .iter()
            .find(|s| s.chars().count() > MAX_SEGMENT_CHARS)
        {
            issues.push(issue(
                &st429_9::PathTooLong,
                format!(
                    "has segment '{long}' longer than {MAX_SEGMENT_CHARS} characters (ST 429-9 A.2)"
                ),
            ));
        }

        if segments.len() > MAX_SEGMENTS {
            issues.push(issue(
                &st429_9::PathTooManySegments,
                format!(
                    "has {} segments; the limit is {MAX_SEGMENTS} (ST 429-9 A.2)",
                    segments.len()
                ),
            ));
        }

        match seen.get(&path.to_lowercase()) {
            Some(other) => issues.push(issue(
                &st429_9::PathDuplicate,
                format!("is identical to '{other}' regardless of case (ST 429-9 A.2)"),
            )),
            None => {
                seen.insert(path.to_lowercase(), path);
            }
        }
    }
    issues
}

#[cfg(test)]
mod tests {
    use super::*;

    fn codes(paths: &[&str]) -> Vec<String> {
        basic_map_profile_v2_issues(paths.iter().copied())
            .into_iter()
            .map(|i| i.code)
            .collect()
    }

    #[test]
    fn conforming_paths_pass() {
        assert!(codes(&[
            "CPL_0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85.xml",
            "video/MERIDIAN_00.mxf",
            "./a/../b.mxf",
        ])
        .is_empty());
    }

    /// Neither A.2 nor the RFC 3986 relative-path grammar forbids empty
    /// segments after the first, so they are not flagged.
    #[test]
    fn empty_segments_are_not_flagged() {
        assert!(codes(&["a//b.mxf", "dir/"]).is_empty());
    }

    #[test]
    fn absolute_scheme_query_and_fragment_are_not_relative() {
        for p in ["/mnt/a.mxf", "file:a.mxf", "a.mxf?x", "a.mxf#f"] {
            assert!(
                codes(&[p]).contains(&"ST429-9:2014:A.2/PathNotRelative".to_string()),
                "{p}"
            );
        }
    }

    #[test]
    fn characters_outside_the_set_are_flagged() {
        for p in ["my video.mxf", "vid%20eo.mxf", "vidéo.mxf"] {
            assert!(
                codes(&[p]).contains(&"ST429-9:2014:A.2/PathCharacters".to_string()),
                "{p}"
            );
        }
    }

    /// Paths seen in the wild (imferno issue 79): one finding per path,
    /// naming every offending character.
    #[test]
    fn offending_characters_are_named_once_per_path() {
        let issues = basic_map_profile_v2_issues(["FR Dubcard.mxf", "A&B (it's)+new!.mxf"]);
        let msgs: Vec<_> = issues
            .iter()
            .filter(|i| i.code == "ST429-9:2014:A.2/PathCharacters")
            .map(|i| i.message.clone())
            .collect();
        assert_eq!(msgs.len(), 2, "{issues:#?}");
        assert!(msgs[0].contains("' '"), "{}", msgs[0]);
        for c in ["'&'", "' '", "'('", "'\\''", "')'", "'+'", "'!'"] {
            assert!(msgs[1].contains(c), "{c} in {}", msgs[1]);
        }
    }

    /// The file name from photon#192 (111 characters) is over the Annex A
    /// limit (it is under the 128 of the HTTP profile, which does not
    /// apply to a filesystem).
    #[test]
    fn segment_and_path_over_100_characters_are_flagged() {
        let long = "418519_TerraX_MythosWolfskindMogliUndDieWildenKinder_InternationaleFassung_deu_clean_stereo_51_MundE_stereo.mxf";
        assert_eq!(long.len(), 111);
        assert_eq!(codes(&[long]), vec!["ST429-9:2014:A.2/PathTooLong"]);
        let exactly_100 = format!("{}.mxf", "a".repeat(96));
        assert!(codes(&[&exactly_100]).is_empty());
        let nested = format!("{}/{}", "a".repeat(60), "b".repeat(60));
        assert_eq!(codes(&[&nested]), vec!["ST429-9:2014:A.2/PathTooLong"]);
    }

    #[test]
    fn more_than_ten_segments_are_flagged() {
        assert!(codes(&["a/b/c/d/e/f/g/h/i/j.mxf"]).is_empty());
        assert_eq!(
            codes(&["a/b/c/d/e/f/g/h/i/j/k.mxf"]),
            vec!["ST429-9:2014:A.2/PathTooManySegments"]
        );
    }

    #[test]
    fn case_insensitive_duplicates_are_flagged() {
        assert_eq!(
            codes(&["Video.mxf", "video.MXF"]),
            vec!["ST429-9:2014:A.2/PathDuplicate"]
        );
    }
}
