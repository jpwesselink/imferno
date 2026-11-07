//! ST 2067-2:2013 IMPs: Packing List Size and Hash are checked under
//! ST 429-8 (the PKL spec of a 2013 IMP, 2013 §7.1), and ST 429-9 findings
//! name the ST 429-9 edition assumed.

use std::path::PathBuf;

use imferno_core::catalogue::{imferno, st2067_2::St2067_2Edition};
use imferno_core::package::{read_dir, Imferno, ValidationOptions};
use imferno_core::{ValidationIssue, ValidationReport};

fn all(report: &ValidationReport) -> Vec<&ValidationIssue> {
    report
        .critical
        .iter()
        .chain(&report.errors)
        .chain(&report.warnings)
        .chain(&report.info)
        .collect()
}

const PKL_429_8: &str = r#"<?xml version="1.0"?>
<PackingList xmlns="http://www.smpte-ra.org/schemas/429-8/2007/PKL">
<Id>urn:uuid:aaaaaaaa-0000-0000-0000-000000000001</Id>
<IssueDate>2024-01-01T00:00:00Z</IssueDate>
<Issuer>t</Issuer><Creator>t</Creator>
<AssetList>
  <Asset>
    <Id>urn:uuid:cccccccc-0000-0000-0000-000000000099</Id>
    <Hash>2jmj7l5rSw0yVb/vlWAYkK/YBwk=</Hash>
    <Size>100</Size>
    <Type>application/mxf</Type>
  </Asset>
</AssetList>
</PackingList>"#;

const ASSETMAP: &str = r#"<?xml version="1.0"?>
<AssetMap xmlns="http://www.smpte-ra.org/schemas/429-9/2007/AM">
<Id>urn:uuid:dddddddd-0000-0000-0000-000000000004</Id>
<Creator>t</Creator><VolumeCount>1</VolumeCount>
<IssueDate>2024-01-01T00:00:00Z</IssueDate><Issuer>t</Issuer>
<AssetList>
  <Asset><Id>urn:uuid:aaaaaaaa-0000-0000-0000-000000000001</Id><PackingList>true</PackingList>
    <ChunkList><Chunk><Path>PKL.xml</Path></Chunk></ChunkList></Asset>
  <Asset><Id>urn:uuid:cccccccc-0000-0000-0000-000000000099</Id>
    <ChunkList><Chunk><Path>track.mxf</Path></Chunk></ChunkList></Asset>
</AssetList>
</AssetMap>"#;

/// A 2013 IMP whose asset has the wrong size and hash still gets findings,
/// under ST 429-8:2007 §6.4 / §6.3 (not dropped as delivery rules).
#[test]
fn size_and_hash_mismatch_of_a_2013_imp_are_st429_8_findings() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    std::fs::write(root.join("track.mxf"), b"short").unwrap();
    std::fs::write(root.join("PKL.xml"), PKL_429_8).unwrap();
    std::fs::write(root.join("ASSETMAP.xml"), ASSETMAP).unwrap();
    let pkg = Imferno::parse(read_dir(root).unwrap()).expect("parse");
    assert_eq!(pkg.package_edition(None).edition, St2067_2Edition::V2013);

    let report = pkg.validate(&ValidationOptions::default());
    let size: Vec<_> = all(&report)
        .into_iter()
        .filter(|i| i.rule_name() == "SizeMismatch")
        .collect();
    assert_eq!(size.len(), 1, "{report}");
    assert_eq!(size[0].code, "ST429-8:2007:6.4/SizeMismatch");

    // Right size, wrong hash.
    std::fs::write(
        root.join("PKL.xml"),
        PKL_429_8.replace("<Size>100</Size>", "<Size>5</Size>"),
    )
    .unwrap();
    let pkg = Imferno::parse(read_dir(root).unwrap()).expect("parse");
    let hashed = pkg.validate_hashes(&ValidationOptions::default());
    assert!(
        all(&hashed)
            .iter()
            .any(|i| i.code == "ST429-8:2007:6.3/ChecksumMismatch"),
        "{hashed}"
    );
}

/// An ST 429-9 finding comes with a notice naming the assumed 429-9 edition.
#[test]
fn st429_9_findings_announce_the_assumed_edition() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../test-data/ISXD/CompleteIMP");
    let mut pkg = Imferno::parse(read_dir(&dir).unwrap()).expect("parse");
    pkg.asset_map.asset_list.assets.clear();
    let report = pkg.validate(&ValidationOptions::default());
    assert!(all(&report)
        .iter()
        .any(|i| i.code == "ST429-9:2014:11/AssetListEmpty"));
    let notices: Vec<_> = all(&report)
        .into_iter()
        .filter(|i| i.is(&imferno::EditionAssumed) && i.message.contains("ST 429-9:2014"))
        .collect();
    assert_eq!(notices.len(), 1, "{report}");
}
