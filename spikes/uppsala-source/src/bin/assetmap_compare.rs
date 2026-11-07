//! Criterion 4: the typed Asset Map built from the uppsala DOM, compared with
//! imferno's current `parse_assetmap` (quick-xml + serde) on every ASSETMAP in
//! the corpus.
//!
//! Usage: cargo run --release --bin assetmap_compare

use std::collections::BTreeMap;

use uppsala_source_spike::assetmap::{self, UserText, Value, ST429_9_2014};
use uppsala_source_spike::corpus::{imf_corpus, rel, root_local_name};
use uppsala_source_spike::lines::LineIndex;

#[derive(Default)]
struct Tally {
    /// category -> (count, first example)
    diffs: BTreeMap<String, (usize, String)>,
}

impl Tally {
    fn add(&mut self, cat: &str, example: String) {
        let e = self.diffs.entry(cat.to_string()).or_insert((0, example));
        e.0 += 1;
    }
}

fn ut(u: &Option<UserText<'_>>) -> Option<String> {
    u.as_ref().and_then(|u| u.text.text().map(str::to_string))
}

/// Compare one optional text field and classify any difference.
fn cmp_text(t: &mut Tally, file: &str, field: &str, ours: Option<String>, theirs: Option<String>) {
    match (&ours, &theirs) {
        (a, b) if a == b => {}
        (Some(a), Some(b)) if a.trim() == b => t.add(
            &format!("{field}: imferno trims whitespace"),
            format!("{file}: raw {a:?} vs imferno {b:?}"),
        ),
        (Some(a), None) if a.is_empty() => t.add(
            &format!("{field}: present-but-empty becomes None in imferno"),
            format!("{file}"),
        ),
        (Some(a), None) if a.trim().is_empty() => t.add(
            &format!("{field}: whitespace-only becomes None in imferno"),
            format!("{file}: raw {a:?}"),
        ),
        _ => t.add(
            &format!("{field}: other"),
            format!("{file}: raw {ours:?} vs imferno {theirs:?}"),
        ),
    }
}

fn v(x: &Value<'_>) -> Option<String> {
    x.text().map(str::to_string)
}

fn main() {
    let mut files = 0;
    let mut both_ok = 0;
    let mut imferno_err = Vec::new();
    let mut findings: BTreeMap<&str, usize> = BTreeMap::new();
    let mut finding_examples = Vec::new();
    let mut t = Tally::default();
    let mut extra: BTreeMap<&str, usize> = BTreeMap::new();

    for path in imf_corpus() {
        let src = std::fs::read_to_string(&path).unwrap();
        if root_local_name(&src).as_deref() != Some("AssetMap") {
            continue;
        }
        files += 1;
        let file = rel(&path);
        let doc = uppsala::parse(&src).expect("uppsala parses every corpus file");
        let lines = LineIndex::new(&src);
        let parsed = assetmap::from_document(&doc, &lines, &ST429_9_2014);
        for f in &parsed.findings {
            *findings.entry(f.code).or_default() += 1;
            if finding_examples.len() < 12 {
                finding_examples.push(format!(
                    "{file}:{}:{} {} {}",
                    f.at.line, f.at.col, f.code, f.message
                ));
            }
        }
        let Some(ours) = parsed.asset_map else {
            println!("{file}: no typed model (root not an AM-namespace AssetMap)");
            continue;
        };
        let theirs = match imferno_core::assetmap::parse_assetmap(&src) {
            Ok(a) => a,
            Err(e) => {
                imferno_err.push(format!("{file}: {e}"));
                continue;
            }
        };
        both_ok += 1;

        cmp_text(&mut t, &file, "Id", v(&ours.id), Some(theirs.id.to_urn()));
        cmp_text(
            &mut t,
            &file,
            "AnnotationText",
            ut(&ours.annotation_text),
            theirs.annotation_text.clone(),
        );
        cmp_text(
            &mut t,
            &file,
            "Creator",
            ut(&ours.creator),
            theirs.creator.clone(),
        );
        cmp_text(
            &mut t,
            &file,
            "VolumeCount",
            v(&ours.volume_count),
            Some(theirs.volume_count.to_string()),
        );
        cmp_text(
            &mut t,
            &file,
            "IssueDate",
            v(&ours.issue_date),
            Some(theirs.issue_date.clone()),
        );
        cmp_text(
            &mut t,
            &file,
            "Issuer",
            ut(&ours.issuer),
            theirs.issuer.clone(),
        );
        for u in [&ours.annotation_text, &ours.creator, &ours.issuer]
            .into_iter()
            .flatten()
        {
            if !u.language.is_absent() {
                *extra
                    .entry("UserText@language (dropped by imferno)")
                    .or_default() += 1;
            }
        }

        let our_assets = ours.asset_list.clone().unwrap_or_default();
        let their_assets = &theirs.asset_list.assets;
        if our_assets.len() != their_assets.len() {
            t.add(
                "Asset count",
                format!("{file}: {} vs {}", our_assets.len(), their_assets.len()),
            );
        }
        for (a, b) in our_assets.iter().zip(their_assets) {
            cmp_text(&mut t, &file, "Asset/Id", v(&a.id), Some(b.id.to_urn()));
            if a.annotation_text.is_some() {
                *extra
                    .entry("Asset/AnnotationText (not modelled by imferno)")
                    .or_default() += 1;
            }
            cmp_text(
                &mut t,
                &file,
                "Asset/PackingList",
                v(&a.packing_list),
                b.packing_list.map(|x| x.to_string()),
            );
            let our_chunks = a.chunk_list.clone().unwrap_or_default();
            if our_chunks.len() != b.chunk_list.chunks.len() {
                t.add("Chunk count", format!("{file}"));
            }
            for (c, d) in our_chunks.iter().zip(&b.chunk_list.chunks) {
                cmp_text(
                    &mut t,
                    &file,
                    "Chunk/Path",
                    v(&c.path),
                    Some(d.path.clone()),
                );
                if c.volume_index.is_absent() {
                    t.add(
                        "Chunk/VolumeIndex: absent becomes 1 in imferno (raw: Absent)",
                        format!("{file}: imferno {}", d.volume_index),
                    );
                } else {
                    cmp_text(
                        &mut t,
                        &file,
                        "Chunk/VolumeIndex",
                        v(&c.volume_index),
                        Some(d.volume_index.to_string()),
                    );
                }
                if !c.offset.is_absent() {
                    *extra
                        .entry("Chunk/Offset (dropped by imferno)")
                        .or_default() += 1;
                }
                if !c.length.is_absent() {
                    *extra
                        .entry("Chunk/Length (dropped by imferno)")
                        .or_default() += 1;
                }
            }
        }
    }

    println!("ASSETMAP files: {files}; both parsers produced a model: {both_ok}");
    println!("imferno parse_assetmap errors ({}):", imferno_err.len());
    for e in &imferno_err {
        println!("  {e}");
    }
    println!("typed-model findings: {findings:?}");
    for e in &finding_examples {
        println!("  {e}");
    }
    println!("content differences (category: count, first example):");
    for (cat, (n, ex)) in &t.diffs {
        println!("  {cat}: {n}  e.g. {ex}");
    }
    println!("fields the typed model keeps that imferno drops: {extra:?}");
    synthetic();
}

/// Variants of a real ASSETMAP covering what the corpus does not exercise.
fn synthetic() {
    let base_path = uppsala_source_spike::corpus::repo_root()
        .join("test-data/Application5/MultiAppIMP/ASSETMAP.xml");
    let base = std::fs::read_to_string(base_path).unwrap();
    let id = "<Id>urn:uuid:5671d298-ce6e-4a28-95d9-83778b52104b</Id>";
    let creator = r#"<Creator language="en">IMF-Tool 1.5.293</Creator>"#;
    let issuer = r#"<Issuer language="en">WR</Issuer>"#;
    let first_asset_id = "<Id>urn:uuid:eaf76289-fd79-477f-9526-e34d69a8f57a</Id>";
    let variants: Vec<(&str, String)> = vec![
        (
            "padded Creator",
            base.replace(creator, r#"<Creator language="en">  IMF-Tool  </Creator>"#),
        ),
        ("empty Creator", base.replace(creator, "<Creator/>")),
        (
            "whitespace-only Creator",
            base.replace(creator, "<Creator>   </Creator>"),
        ),
        ("absent Creator", base.replace(creator, "")),
        (
            "empty language attr",
            base.replace(creator, r#"<Creator language="">x</Creator>"#),
        ),
        (
            "upper-case UUID",
            base.replace(id, "<Id>urn:uuid:5671D298-CE6E-4A28-95D9-83778B52104B</Id>"),
        ),
        (
            "bare UUID (no urn:uuid:)",
            base.replace(id, "<Id>5671d298-ce6e-4a28-95d9-83778b52104b</Id>"),
        ),
        (
            "entity + CDATA in Creator",
            base.replace(creator, "<Creator>A &amp; B<![CDATA[ <c> ]]></Creator>"),
        ),
        (
            "unknown AM element",
            base.replace(creator, &format!("{creator}<Bogus>1</Bogus>")),
        ),
        (
            "foreign-namespace element",
            base.replace(
                creator,
                &format!(r#"{creator}<v:X xmlns:v="urn:v">1</v:X>"#),
            ),
        ),
        (
            "duplicate Issuer",
            base.replace(issuer, "<Issuer>WR</Issuer><Issuer>WR2</Issuer>"),
        ),
        (
            "prefixed AM namespace",
            prefix_all(&base.replace("<AssetMap xmlns=", "<AssetMap xmlns:am=")),
        ),
        (
            "default ns dropped on children (prefixed root only)",
            base.replace("<AssetMap xmlns=", "<am:AssetMap xmlns:am=")
                .replace("</AssetMap>", "</am:AssetMap>"),
        ),
        (
            "PackingList = 1",
            base.replace(
                first_asset_id,
                &format!("{first_asset_id}<PackingList>1</PackingList>"),
            ),
        ),
        (
            "VolumeCount padded",
            base.replace(
                "<VolumeCount>1</VolumeCount>",
                "<VolumeCount> 1 </VolumeCount>",
            ),
        ),
        (
            "wrong root namespace",
            base.replace("2007/AM\"", "2007/XX\""),
        ),
    ];
    println!("\nsynthetic variants of test-data/Application5/MultiAppIMP/ASSETMAP.xml:");
    for (label, src) in variants {
        let doc = match uppsala::parse(&src) {
            Ok(d) => d,
            Err(e) => {
                println!("  [{label}] uppsala: {e}");
                continue;
            }
        };
        let lines = LineIndex::new(&src);
        let p = assetmap::from_document(&doc, &lines, &ST429_9_2014);
        let ours = match &p.asset_map {
            None => "no model".to_string(),
            Some(am) => format!(
                "Id={:?} Creator={:?} Creator@language={:?} Issuer={:?} VolumeCount={:?} Asset[0]/PackingList={:?}",
                am.id.text(),
                am.creator.as_ref().map(|c| short(&c.text)),
                am.creator.as_ref().map(|c| short(&c.language)),
                am.issuer.as_ref().map(|c| short(&c.text)),
                am.volume_count.text(),
                am.asset_list.as_ref().and_then(|l| l.first()).map(|a| short(&a.packing_list)),
            ),
        };
        let theirs = match imferno_core::assetmap::parse_assetmap(&src) {
            Err(e) => format!("ERROR {e}"),
            Ok(am) => format!(
                "Id={} Creator={:?} Issuer={:?} VolumeCount={} Asset[0]/PackingList={:?} namespace={:?}",
                am.id.to_urn(),
                am.creator,
                am.issuer,
                am.volume_count,
                am.asset_list.assets.first().and_then(|a| a.packing_list),
                am.namespace
            ),
        };
        let f: Vec<String> = p
            .findings
            .iter()
            .map(|f| format!("{}@{}:{}", f.code, f.at.line, f.at.col))
            .collect();
        println!(
            "  [{label}]\n    typed  : {ours}\n    finds  : {f:?} extensions={}\n    imferno: {theirs}",
            p.extensions.len()
        );
    }
}

/// Put every element name that starts with an upper-case letter in `am:`.
fn prefix_all(src: &str) -> String {
    let mut out = String::with_capacity(src.len() + 1024);
    let mut chars = src.chars().peekable();
    while let Some(c) = chars.next() {
        out.push(c);
        if c == '<' {
            if chars.peek() == Some(&'/') {
                out.push(chars.next().unwrap());
            }
            if chars.peek().is_some_and(|n| n.is_ascii_uppercase()) {
                out.push_str("am:");
            }
        }
    }
    out
}

fn short(v: &Value<'_>) -> String {
    match v {
        Value::Absent => "Absent".into(),
        Value::Empty { .. } => "Empty".into(),
        Value::Present { text, .. } => format!("Present({text:?})"),
    }
}
