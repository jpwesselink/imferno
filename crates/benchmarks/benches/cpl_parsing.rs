use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use imferno_core::cpl as st2067_3;
use std::fs;

const TEST_FILES: &[(&str, &str)] = &[
    (
        "small",
        "../test-data/MERIDIAN_Netflix_Photon_161006/CPL_0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85.xml",
    ),
    ("medium", "../test-data/ISXD/CPL_ISXD_TEST_1.xml"),
    ("complex", "../test-data/IAB/CPL_IAB_TEST_1.xml"),
];

fn load_test_data() -> Vec<(String, String)> {
    let mut test_data = Vec::new();

    for (name, path) in TEST_FILES {
        if let Ok(content) = fs::read_to_string(path) {
            test_data.push((name.to_string(), content));
        }
    }

    // If no test files found, create synthetic test data
    if test_data.is_empty() {
        let synthetic_cpl = r#"<?xml version="1.0" encoding="UTF-8" ?>
<CompositionPlaylist xmlns="http://www.smpte-ra.org/schemas/2067-3/2013">
  <Id>urn:uuid:test-benchmark-cpl</Id>
  <Annotation>Synthetic CPL for Benchmarking</Annotation>
  <IssueDate>2023-12-10T10:00:00-00:00</IssueDate>
  <Issuer>Benchmark Suite</Issuer>
  <Creator>IMF-RS Benchmark</Creator>
  <ContentTitle>Benchmark Content</ContentTitle>
  <ContentKind>test</ContentKind>
  <SegmentList>
    <Segment>
      <Id>urn:uuid:test-segment</Id>
      <SequenceList>
      </SequenceList>
    </Segment>
  </SegmentList>
</CompositionPlaylist>"#;
        test_data.push(("synthetic".to_string(), synthetic_cpl.to_string()));
    }

    test_data
}

fn bench_cpl_parsing_cold(c: &mut Criterion) {
    let test_data = load_test_data();

    let mut group = c.benchmark_group("cpl_parsing_cold");

    for (name, content) in &test_data {
        let size = content.len() as u64;
        group.throughput(Throughput::Bytes(size));

        group.bench_with_input(BenchmarkId::new("parse", name), content, |b, content| {
            b.iter(|| {
                let result = st2067_3::parse_cpl(black_box(content));
                black_box(result)
            })
        });
    }

    group.finish();
}

fn bench_cpl_parsing_warm(c: &mut Criterion) {
    let test_data = load_test_data();

    let mut group = c.benchmark_group("cpl_parsing_warm");

    for (name, content) in &test_data {
        let size = content.len() as u64;
        group.throughput(Throughput::Bytes(size));

        // Warm up the parser
        let _ = st2067_3::parse_cpl(content);

        group.bench_with_input(BenchmarkId::new("parse", name), content, |b, content| {
            b.iter(|| {
                let result = st2067_3::parse_cpl(black_box(content));
                black_box(result)
            })
        });
    }

    group.finish();
}

fn bench_cpl_language_extraction(c: &mut Criterion) {
    let test_data = load_test_data();

    let mut group = c.benchmark_group("cpl_language_extraction");

    for (name, content) in &test_data {
        let size = content.len() as u64;
        group.throughput(Throughput::Bytes(size));

        group.bench_with_input(
            BenchmarkId::new("extract_languages", name),
            content,
            |b, content| {
                b.iter(|| {
                    let languages = st2067_3::parse_cpl(black_box(content))
                        .map(|cpl| st2067_3::extract_cpl_languages(&cpl))
                        .unwrap_or_default();
                    black_box(languages)
                })
            },
        );
    }

    group.finish();
}

fn bench_cpl_validation(c: &mut Criterion) {
    let test_data = load_test_data();

    let mut group = c.benchmark_group("cpl_validation");

    for (name, content) in &test_data {
        if let Ok(cpl) = st2067_3::parse_cpl(content) {
            group.bench_with_input(BenchmarkId::new("validate", name), &cpl, |b, cpl| {
                b.iter(|| {
                    // Basic validation operations
                    let has_title = !cpl.content_title.text.is_empty();
                    let has_segments = !cpl.segment_list.segments.is_empty();
                    let id_valid = cpl.id.to_string().starts_with("urn:uuid:");
                    black_box((has_title, has_segments, id_valid))
                })
            });
        }
    }

    group.finish();
}

criterion_group!(
    cpl_benches,
    bench_cpl_parsing_cold,
    bench_cpl_parsing_warm,
    bench_cpl_language_extraction,
    bench_cpl_validation
);
criterion_main!(cpl_benches);
