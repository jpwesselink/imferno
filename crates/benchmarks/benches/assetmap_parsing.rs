use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use imferno_core::assetmap as st2067_2;
use std::fs;

const TEST_FILES: &[(&str, &str)] = &[
    ("netflix", "../test-data/MERIDIAN_Netflix_Photon_161006/ASSETMAP.xml"),
    ("isxd", "../test-data/ISXD/CompleteIMP/ASSETMAP.xml"),
    ("iab", "../test-data/IAB/CompleteIMP/ASSETMAP.xml"),
];

fn load_assetmap_test_data() -> Vec<(String, String)> {
    let mut test_data = Vec::new();

    for (name, path) in TEST_FILES {
        if let Ok(content) = fs::read_to_string(path) {
            test_data.push((name.to_string(), content));
        }
    }

    // If no test files found, create synthetic test data
    if test_data.is_empty() {
        let synthetic_assetmap = r#"<?xml version="1.0" encoding="UTF-8"?>
<AssetMap xmlns="http://www.smpte-ra.org/schemas/2067-4/2013">
  <Id>urn:uuid:test-assetmap</Id>
  <AnnotationText>Synthetic AssetMap for Benchmarking</AnnotationText>
  <VolumeCount>1</VolumeCount>
  <IssueDate>2023-12-10T10:00:00-00:00</IssueDate>
  <Issuer>Benchmark Suite</Issuer>
  <Creator>IMF-RS Benchmark</Creator>
  <AssetList>
    <Asset>
      <Id>urn:uuid:test-asset-1</Id>
      <ChunkList>
        <Chunk>
          <Path>test_video.mxf</Path>
          <VolumeIndex>1</VolumeIndex>
          <Offset>0</Offset>
          <Length>1000000</Length>
        </Chunk>
      </ChunkList>
    </Asset>
    <Asset>
      <Id>urn:uuid:test-asset-2</Id>
      <ChunkList>
        <Chunk>
          <Path>test_audio.mxf</Path>
          <VolumeIndex>1</VolumeIndex>
          <Offset>0</Offset>
          <Length>500000</Length>
        </Chunk>
      </ChunkList>
    </Asset>
  </AssetList>
</AssetMap>"#;
        test_data.push(("synthetic".to_string(), synthetic_assetmap.to_string()));
    }

    test_data
}

fn load_volindex_test_data() -> String {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<VolumeIndex xmlns="http://www.smpte-ra.org/schemas/2067-4/2013">
  <Index>1</Index>
</VolumeIndex>"#.to_string()
}

fn bench_assetmap_parsing_cold(c: &mut Criterion) {
    let test_data = load_assetmap_test_data();

    let mut group = c.benchmark_group("assetmap_parsing_cold");

    for (name, content) in &test_data {
        let size = content.len() as u64;
        group.throughput(Throughput::Bytes(size));

        group.bench_with_input(
            BenchmarkId::new("parse", name),
            content,
            |b, content| {
                b.iter(|| {
                    let result = st2067_2::parse_assetmap(black_box(content));
                    black_box(result)
                })
            }
        );
    }

    group.finish();
}

fn bench_assetmap_parsing_warm(c: &mut Criterion) {
    let test_data = load_assetmap_test_data();

    let mut group = c.benchmark_group("assetmap_parsing_warm");

    for (name, content) in &test_data {
        let size = content.len() as u64;
        group.throughput(Throughput::Bytes(size));

        // Warm up the parser
        let _ = st2067_2::parse_assetmap(content);

        group.bench_with_input(
            BenchmarkId::new("parse", name),
            content,
            |b, content| {
                b.iter(|| {
                    let result = st2067_2::parse_assetmap(black_box(content));
                    black_box(result)
                })
            }
        );
    }

    group.finish();
}

fn bench_volindex_parsing(c: &mut Criterion) {
    let volindex_xml = load_volindex_test_data();

    let mut group = c.benchmark_group("volindex_parsing");
    group.throughput(Throughput::Bytes(volindex_xml.len() as u64));

    group.bench_function("parse", |b| {
        b.iter(|| {
            let result = st2067_2::parse_volindex(black_box(&volindex_xml));
            black_box(result)
        })
    });

    group.finish();
}


fn bench_asset_lookup(c: &mut Criterion) {
    let test_data = load_assetmap_test_data();

    let mut group = c.benchmark_group("asset_lookup");

    for (name, content) in &test_data {
        if let Ok(assetmap) = st2067_2::parse_assetmap(content) {
            group.bench_with_input(
                BenchmarkId::new("lookup_by_id", name),
                &assetmap,
                |b, assetmap| {
                    b.iter(|| {
                        // Find assets by ID (simulating lookup operations)
                        let mut found = 0;
                        for asset in &assetmap.asset_list.assets {
                            if !asset.id.to_string().is_empty() {
                                found += 1;
                            }
                        }
                        black_box(found)
                    })
                }
            );

            group.bench_with_input(
                BenchmarkId::new("calculate_total_chunks", name),
                &assetmap,
                |b, assetmap| {
                    b.iter(|| {
                        let mut total_chunks = 0u64;
                        for asset in &assetmap.asset_list.assets {
                            total_chunks += asset.chunk_list.chunks.len() as u64;
                        }
                        black_box(total_chunks)
                    })
                }
            );
        }
    }

    group.finish();
}

criterion_group!(
    assetmap_benches,
    bench_assetmap_parsing_cold,
    bench_assetmap_parsing_warm,
    bench_volindex_parsing,
    bench_asset_lookup
);
criterion_main!(assetmap_benches);