use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use imferno_core::mxf as st377_1;
use std::fs;
use std::io::Cursor;

const MXF_TEST_FILES: &[(&str, &str)] = &[
    ("video", "../test-data/MERIDIAN_Netflix_Photon_161006/MERIDIAN_Netflix_Photon_161006_00.mxf"),
    ("audio", "../test-data/MERIDIAN_Netflix_Photon_161006/MERIDIAN_Netflix_Photon_161006_ENG-51_00.mxf"),
    ("subtitle", "../test-data/MERIDIAN_Netflix_Photon_161006/MERIDIAN_Netflix_Photon_161006_00_tt.mxf"),
];

/// Build a minimal valid MXF header partition pack byte stream (105 bytes).
/// Used as synthetic test data when real MXF files are not present.
fn make_synthetic_mxf() -> Vec<u8> {
    let mut data = Vec::new();
    // Key: Header Partition Pack (Closed and Complete)
    data.extend_from_slice(&[
        0x06, 0x0E, 0x2B, 0x34, 0x02, 0x05, 0x01, 0x01,
        0x0D, 0x01, 0x02, 0x01, 0x01, 0x02, 0x04, 0x00,
    ]);
    // BER length = 88 (fits in 1 byte)
    data.push(88);
    // Partition pack value (88 bytes):
    data.extend_from_slice(&[0x00, 0x01]); // MajorVersion = 1
    data.extend_from_slice(&[0x00, 0x03]); // MinorVersion = 3
    data.extend_from_slice(&[0x00, 0x00, 0x02, 0x00]); // KAGSize = 512
    data.extend_from_slice(&[0u8; 8 * 5 + 4 + 8 + 4]); // ThisPartition..BodySID
    // OP1a UL
    data.extend_from_slice(&[
        0x06, 0x0E, 0x2B, 0x34, 0x04, 0x01, 0x01, 0x02,
        0x0D, 0x01, 0x02, 0x01, 0x01, 0x01, 0x09, 0x00,
    ]);
    data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // EssenceContainers count = 0
    data.extend_from_slice(&[0x00, 0x00, 0x00, 0x10]); // element_size = 16
    data
}

fn load_mxf_test_data() -> Vec<(String, Vec<u8>)> {
    let mut test_data = Vec::new();

    for (name, path) in MXF_TEST_FILES {
        if let Ok(content) = fs::read(path) {
            // Only read first 64KB for header parsing benchmarks
            let header_size = std::cmp::min(content.len(), 65536);
            test_data.push((name.to_string(), content[..header_size].to_vec()));
        }
    }

    if test_data.is_empty() {
        test_data.push(("synthetic".to_string(), make_synthetic_mxf()));
    }

    test_data
}

fn create_invalid_headers() -> Vec<(String, Vec<u8>)> {
    vec![
        ("invalid_signature".to_string(), vec![0x00; 64]),
        ("truncated".to_string(), vec![0x06, 0x0E, 0x2B, 0x34]),
        (
            "partial_signature".to_string(),
            vec![
                0x06, 0x0E, 0x2B, 0x34, 0x02, 0x05, 0x01, 0x01,
                0x0D, 0x01, 0x02, 0x01, 0x00, 0x00, 0x00, 0x00,
            ],
        ),
    ]
}

fn bench_mxf_header_parsing_cold(c: &mut Criterion) {
    let test_data = load_mxf_test_data();
    let mut group = c.benchmark_group("mxf_header_parsing_cold");

    for (name, data) in &test_data {
        let size = data.len() as u64;
        group.throughput(Throughput::Bytes(size));
        group.bench_with_input(BenchmarkId::new("parse", name), data, |b, data| {
            b.iter(|| {
                let mut cursor = Cursor::new(black_box(data));
                let result = st377_1::parse_mxf_header_info_from_reader(&mut cursor);
                black_box(result)
            })
        });
    }

    group.finish();
}

fn bench_mxf_header_parsing_warm(c: &mut Criterion) {
    let test_data = load_mxf_test_data();
    let mut group = c.benchmark_group("mxf_header_parsing_warm");

    for (name, data) in &test_data {
        let size = data.len() as u64;
        group.throughput(Throughput::Bytes(size));

        // Warm up
        let _ = st377_1::parse_mxf_header_info_from_reader(&mut Cursor::new(data));

        group.bench_with_input(BenchmarkId::new("parse", name), data, |b, data| {
            b.iter(|| {
                let mut cursor = Cursor::new(black_box(data));
                let result = st377_1::parse_mxf_header_info_from_reader(&mut cursor);
                black_box(result)
            })
        });
    }

    group.finish();
}

fn bench_mxf_signature_validation(c: &mut Criterion) {
    let test_data = load_mxf_test_data();
    let invalid_data = create_invalid_headers();
    let mut group = c.benchmark_group("mxf_signature_validation");

    for (name, data) in &test_data {
        group.bench_with_input(BenchmarkId::new("valid", name), data, |b, data| {
            b.iter(|| {
                let mut cursor = Cursor::new(black_box(data));
                let result = st377_1::parse_mxf_header_info_from_reader(&mut cursor);
                black_box(result.is_ok())
            })
        });
    }

    for (name, data) in &invalid_data {
        group.bench_with_input(BenchmarkId::new("invalid", name), data, |b, data| {
            b.iter(|| {
                let mut cursor = Cursor::new(black_box(data));
                let result = st377_1::parse_mxf_header_info_from_reader(&mut cursor);
                black_box(result.is_err())
            })
        });
    }

    group.finish();
}

fn bench_mxf_memory_usage(c: &mut Criterion) {
    let test_data = load_mxf_test_data();
    let mut group = c.benchmark_group("mxf_memory_usage");

    for (name, data) in &test_data {
        group.bench_with_input(BenchmarkId::new("parse_and_drop", name), data, |b, data| {
            b.iter(|| {
                let mut cursor = Cursor::new(black_box(data));
                if let Ok(info) = st377_1::parse_mxf_header_info_from_reader(&mut cursor) {
                    drop(black_box(info));
                }
            })
        });
    }

    group.finish();
}

fn bench_mxf_concurrent_parsing(c: &mut Criterion) {
    let test_data = load_mxf_test_data();

    if !test_data.is_empty() {
        let data = &test_data[0].1;
        let mut group = c.benchmark_group("mxf_concurrent_parsing");

        group.bench_function("sequential_10x", |b| {
            b.iter(|| {
                for _ in 0..10 {
                    let mut cursor = Cursor::new(black_box(data));
                    let result = st377_1::parse_mxf_header_info_from_reader(&mut cursor);
                    let _ = black_box(result);
                }
            })
        });

        group.finish();
    }
}

criterion_group!(
    mxf_benches,
    bench_mxf_header_parsing_cold,
    bench_mxf_header_parsing_warm,
    bench_mxf_signature_validation,
    bench_mxf_memory_usage,
    bench_mxf_concurrent_parsing
);
criterion_main!(mxf_benches);
