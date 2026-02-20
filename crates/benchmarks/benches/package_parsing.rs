use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use imferno_core::assetmap as st2067_2;
use imferno_core::cpl as st2067_3;
use imferno_core::package::{Imferno, read_dir};
use std::fs;
use std::path::Path;

const TEST_PACKAGE_DIRS: &[(&str, &str)] = &[
    ("netflix", "../test-data/MERIDIAN_Netflix_Photon_161006"),
    ("isxd", "../test-data/ISXD/CompleteIMP"),
    ("iab", "../test-data/IAB/CompleteIMP"),
];

fn load_package_test_data() -> Vec<(String, String)> {
    let mut test_data = Vec::new();

    for (name, path) in TEST_PACKAGE_DIRS {
        if Path::new(path).is_dir() {
            test_data.push((name.to_string(), path.to_string()));
        }
    }

    // If no test packages found, create synthetic package structure
    if test_data.is_empty() {
        test_data.push(("synthetic".to_string(), "synthetic".to_string()));
    }

    test_data
}

fn create_synthetic_package() -> (String, String, String) {
    let assetmap = r#"<?xml version="1.0" encoding="UTF-8"?>
<AssetMap xmlns="http://www.smpte-ra.org/schemas/2067-4/2013">
  <Id>urn:uuid:synthetic-assetmap</Id>
  <AnnotationText>Synthetic Package for Benchmarking</AnnotationText>
  <VolumeCount>1</VolumeCount>
  <IssueDate>2023-12-10T10:00:00-00:00</IssueDate>
  <Issuer>Benchmark Suite</Issuer>
  <Creator>IMF-RS Benchmark</Creator>
  <AssetList>
    <Asset>
      <Id>urn:uuid:cpl-asset</Id>
      <ChunkList>
        <Chunk>
          <Path>CPL_synthetic.xml</Path>
          <VolumeIndex>1</VolumeIndex>
          <Offset>0</Offset>
          <Length>5000</Length>
        </Chunk>
      </ChunkList>
    </Asset>
    <Asset>
      <Id>urn:uuid:video-asset</Id>
      <ChunkList>
        <Chunk>
          <Path>video_track.mxf</Path>
          <VolumeIndex>1</VolumeIndex>
          <Offset>0</Offset>
          <Length>1000000</Length>
        </Chunk>
      </ChunkList>
    </Asset>
  </AssetList>
</AssetMap>"#;

    let cpl = r#"<?xml version="1.0" encoding="UTF-8" ?>
<CompositionPlaylist xmlns="http://www.smpte-ra.org/schemas/2067-3/2013">
  <Id>urn:uuid:synthetic-cpl</Id>
  <Annotation>Synthetic CPL for Benchmarking</Annotation>
  <IssueDate>2023-12-10T10:00:00-00:00</IssueDate>
  <Issuer>Benchmark Suite</Issuer>
  <Creator>IMF-RS Benchmark</Creator>
  <ContentTitle>Benchmark Package Content</ContentTitle>
  <ContentKind>test</ContentKind>
  <SegmentList>
    <Segment>
      <Id>urn:uuid:synthetic-segment</Id>
      <SequenceList>
        <Sequence>
          <Id>urn:uuid:video-sequence</Id>
          <TrackId>urn:uuid:video-track</TrackId>
          <ResourceList>
            <Resource>
              <Id>urn:uuid:video-resource</Id>
              <SourceEncoding>urn:uuid:video-asset</SourceEncoding>
              <EntryPoint>0</EntryPoint>
              <IntrinsicDuration>24000</IntrinsicDuration>
              <Duration>24000</Duration>
            </Resource>
          </ResourceList>
        </Sequence>
      </SequenceList>
    </Segment>
  </SegmentList>
</CompositionPlaylist>"#;

    let pklist = r#"<?xml version="1.0" encoding="UTF-8"?>
<PackingList xmlns="http://www.smpte-ra.org/schemas/2067-4/2013">
  <Id>urn:uuid:synthetic-pklist</Id>
  <AnnotationText>Synthetic Packing List for Benchmarking</AnnotationText>
  <IssueDate>2023-12-10T10:00:00-00:00</IssueDate>
  <Issuer>Benchmark Suite</Issuer>
  <Creator>IMF-RS Benchmark</Creator>
  <AssetList>
    <Asset>
      <Id>urn:uuid:cpl-asset</Id>
      <AnnotationText>Composition Playlist</AnnotationText>
      <Hash>abcdef1234567890abcdef1234567890</Hash>
      <Size>5000</Size>
      <Type>text/xml</Type>
    </Asset>
    <Asset>
      <Id>urn:uuid:video-asset</Id>
      <AnnotationText>Video Track</AnnotationText>
      <Hash>1234567890abcdef1234567890abcdef</Hash>
      <Size>1000000</Size>
      <Type>application/mxf</Type>
    </Asset>
  </AssetList>
</PackingList>"#;

    (assetmap.to_string(), cpl.to_string(), pklist.to_string())
}

fn bench_full_package_loading(c: &mut Criterion) {
    let test_data = load_package_test_data();

    let mut group = c.benchmark_group("full_package_loading");

    for (name, path) in &test_data {
        if name == "synthetic" {
            let (assetmap_content, _, _) = create_synthetic_package();
            let size = assetmap_content.len() as u64;
            group.throughput(Throughput::Bytes(size));

            group.bench_with_input(
                BenchmarkId::new("load", name),
                &assetmap_content,
                |b, content| {
                    b.iter(|| {
                        let result = st2067_2::parse_assetmap(black_box(content));
                        black_box(result)
                    })
                }
            );
        } else {
            let assetmap_path = format!("{}/ASSETMAP.xml", path);
            if let Ok(assetmap_content) = fs::read_to_string(&assetmap_path) {
                let size = assetmap_content.len() as u64;
                group.throughput(Throughput::Bytes(size));

                group.bench_with_input(
                    BenchmarkId::new("load", name),
                    path,
                    |b, pkg_path| {
                        b.iter(|| {
                            let result = read_dir(black_box(pkg_path)).and_then(|f| Imferno::parse(f));
                            black_box(result)
                        })
                    }
                );
            }
        }
    }

    group.finish();
}

fn bench_package_validation(c: &mut Criterion) {
    let test_data = load_package_test_data();

    let mut group = c.benchmark_group("package_validation");

    for (name, path) in &test_data {
        if name == "synthetic" {
            let (assetmap_content, cpl_content, pklist_content) = create_synthetic_package();

            group.bench_with_input(
                BenchmarkId::new("validate", name),
                &(assetmap_content, cpl_content, pklist_content),
                |b, (assetmap, cpl, pklist)| {
                    b.iter(|| {
                        // Parse all components
                        let assetmap_result = st2067_2::parse_assetmap(black_box(assetmap));
                        let cpl_result = st2067_3::parse_cpl(black_box(cpl));
                        let pklist_result = st2067_2::parse_assetmap(black_box(pklist));

                        // Basic validation
                        let valid = assetmap_result.is_ok() &&
                                   cpl_result.is_ok() &&
                                   pklist_result.is_ok();
                        black_box(valid)
                    })
                }
            );
        } else {
            group.bench_with_input(
                BenchmarkId::new("validate", name),
                path,
                |b, pkg_path| {
                    b.iter(|| {
                        if let Ok(package) = read_dir(black_box(pkg_path)).and_then(|f| Imferno::parse(f)) {
                            let validation_result = package.validate();
                            black_box(validation_result.is_ok())
                        } else {
                            black_box(false)
                        }
                    })
                }
            );
        }
    }

    group.finish();
}

fn bench_cpl_access(c: &mut Criterion) {
    let test_data = load_package_test_data();

    let mut group = c.benchmark_group("cpl_access");

    for (name, path) in &test_data {
        if name != "synthetic" {
            if let Ok(package) = read_dir(path).and_then(|f| Imferno::parse(f)) {
                group.bench_with_input(
                    BenchmarkId::new("access", name),
                    &package,
                    |b, pkg| {
                        b.iter(|| {
                            let main_cpl = pkg.get_main_cpl();
                            let cpl_count = pkg.composition_playlists.len();
                            black_box((main_cpl.is_some(), cpl_count))
                        })
                    }
                );
            }
        }
    }

    group.finish();
}

fn bench_package_component_access(c: &mut Criterion) {
    let test_data = load_package_test_data();

    let mut group = c.benchmark_group("package_component_access");

    for (name, path) in &test_data {
        if name != "synthetic" {
            if let Ok(package) = read_dir(path).and_then(|f| Imferno::parse(f)) {
                group.bench_with_input(
                    BenchmarkId::new("access_cpls", name),
                    &package,
                    |b, pkg| {
                        b.iter(|| {
                            let count = pkg.composition_playlists.len();
                            black_box(count)
                        })
                    }
                );

                group.bench_with_input(
                    BenchmarkId::new("access_assets", name),
                    &package,
                    |b, pkg| {
                        b.iter(|| {
                            let asset_count = pkg.asset_map.asset_list.assets.len();
                            black_box(asset_count)
                        })
                    }
                );
            }
        }
    }

    group.finish();
}

fn bench_package_cross_reference(c: &mut Criterion) {
    let test_data = load_package_test_data();

    let mut group = c.benchmark_group("package_cross_reference");

    for (name, path) in &test_data {
        if name != "synthetic" {
            if let Ok(package) = read_dir(path).and_then(|f| Imferno::parse(f)) {
                group.bench_with_input(
                    BenchmarkId::new("cross_ref", name),
                    &package,
                    |b, pkg| {
                        b.iter(|| {
                            // Cross-reference CPLs with assets
                            let mut ref_count = 0;
                            for cpl in pkg.composition_playlists.values() {
                                for segment in &cpl.segment_list.segments {
                                    ref_count += segment.sequence_list.main_image_sequences.len();
                                    ref_count += segment.sequence_list.main_audio_sequences.len();
                                    ref_count += segment.sequence_list.subtitles_sequences.len();
                                }
                            }
                            black_box(ref_count)
                        })
                    }
                );
            }
        }
    }

    group.finish();
}

criterion_group!(
    package_benches,
    bench_full_package_loading,
    bench_package_validation,
    bench_cpl_access,
    bench_package_component_access,
    bench_package_cross_reference
);
criterion_main!(package_benches);