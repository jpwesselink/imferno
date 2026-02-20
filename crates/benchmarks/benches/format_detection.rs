use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::fs;

fn detect_by_filename(filename: &str) -> &'static str {
    let filename_lower = filename.to_lowercase();

    if filename_lower == "assetmap.xml" {
        "AssetMap"
    } else if filename_lower.starts_with("cpl_") && filename_lower.ends_with(".xml") {
        "CPL"
    } else if filename_lower.starts_with("pkl_") && filename_lower.ends_with(".xml") {
        "PackingList"
    } else if filename_lower == "volindex.xml" {
        "VolumeIndex"
    } else if filename_lower.ends_with(".mxf") {
        "MXF"
    } else {
        "Unknown"
    }
}

fn detect_xml_format(content: &str) -> &'static str {
    if content.contains("AssetMap") && content.contains("http://www.smpte-ra.org/schemas/2067-4") {
        "AssetMap"
    } else if content.contains("CompositionPlaylist") && content.contains("http://www.smpte-ra.org/schemas/2067-3") {
        "CPL"
    } else if content.contains("PackingList") && content.contains("http://www.smpte-ra.org/schemas/2067-4") {
        "PackingList"
    } else if content.contains("VolumeIndex") && content.contains("http://www.smpte-ra.org/schemas/2067-4") {
        "VolumeIndex"
    } else {
        "Unknown XML"
    }
}

fn is_mxf_format(data: &[u8]) -> bool {
    if data.len() < 16 {
        return false;
    }

    let mxf_signature = [0x06, 0x0E, 0x2B, 0x34, 0x02, 0x05, 0x01, 0x01, 0x0D, 0x01, 0x02, 0x01];
    data[..12] == mxf_signature
}

fn detect_by_content(data: &[u8]) -> &'static str {
    if is_mxf_format(data) {
        return "MXF";
    }

    if let Ok(content_str) = std::str::from_utf8(data) {
        if content_str.trim_start().starts_with("<?xml") {
            return detect_xml_format(content_str);
        }
    }

    "Unknown"
}

const TEST_FILES: &[(&str, &str)] = &[
    ("assetmap", "../test-data/MERIDIAN_Netflix_Photon_161006/ASSETMAP.xml"),
    ("cpl", "../test-data/MERIDIAN_Netflix_Photon_161006/CPL_0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85.xml"),
    ("pklist", "../test-data/MERIDIAN_Netflix_Photon_161006/PKL_0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85.xml"),
    ("mxf_video", "../test-data/MERIDIAN_Netflix_Photon_161006/MERIDIAN_Netflix_Photon_161006_00.mxf"),
    ("mxf_audio", "../test-data/MERIDIAN_Netflix_Photon_161006/MERIDIAN_Netflix_Photon_161006_ENG-51_00.mxf"),
    ("mxf_subtitle", "../test-data/MERIDIAN_Netflix_Photon_161006/MERIDIAN_Netflix_Photon_161006_00_tt.mxf"),
];

fn load_format_test_data() -> Vec<(String, Vec<u8>)> {
    let mut test_data = Vec::new();

    for (name, path) in TEST_FILES {
        if let Ok(content) = fs::read(path) {
            // Only read first 64KB for header-based detection
            let header_size = std::cmp::min(content.len(), 65536);
            test_data.push((name.to_string(), content[..header_size].to_vec()));
        }
    }

    // Create synthetic test data if no files found
    if test_data.is_empty() {
        // XML files
        let assetmap_sample = br#"<?xml version="1.0" encoding="UTF-8"?>
<AssetMap xmlns="http://www.smpte-ra.org/schemas/2067-4/2013">
  <Id>urn:uuid:test</Id>
</AssetMap>"#;
        test_data.push(("assetmap".to_string(), assetmap_sample.to_vec()));

        let cpl_sample = br#"<?xml version="1.0" encoding="UTF-8"?>
<CompositionPlaylist xmlns="http://www.smpte-ra.org/schemas/2067-3/2013">
  <Id>urn:uuid:test</Id>
</CompositionPlaylist>"#;
        test_data.push(("cpl".to_string(), cpl_sample.to_vec()));

        let pklist_sample = br#"<?xml version="1.0" encoding="UTF-8"?>
<PackingList xmlns="http://www.smpte-ra.org/schemas/2067-4/2013">
  <Id>urn:uuid:test</Id>
</PackingList>"#;
        test_data.push(("pklist".to_string(), pklist_sample.to_vec()));

        // MXF signature
        let mxf_header = vec![
            0x06, 0x0E, 0x2B, 0x34, 0x02, 0x05, 0x01, 0x01,
            0x0D, 0x01, 0x02, 0x01, 0x01, 0x02, 0x00, 0x00,
        ];
        let mut mxf_sample = mxf_header;
        mxf_sample.extend_from_slice(&[0u8; 1024]); // Add padding
        test_data.push(("mxf".to_string(), mxf_sample));

        // Non-IMF files for negative testing
        test_data.push(("plain_text".to_string(), b"This is just plain text content".to_vec()));
        test_data.push(("random_binary".to_string(), vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A])); // PNG signature
    }

    test_data
}

fn bench_format_detection_by_extension(c: &mut Criterion) {
    let test_files = &[
        ("assetmap.xml", "AssetMap"),
        ("ASSETMAP.xml", "AssetMap"),
        ("CPL_test.xml", "CPL"),
        ("PKL_test.xml", "PackingList"),
        ("video.mxf", "MXF"),
        ("audio.mxf", "MXF"),
        ("unknown.txt", "Unknown"),
    ];

    let mut group = c.benchmark_group("format_detection_by_extension");

    for (filename, _expected_format) in test_files {
        group.bench_with_input(
            BenchmarkId::new("detect", filename),
            filename,
            |b, fname| {
                b.iter(|| {
                    let detected = detect_by_filename(black_box(fname));
                    black_box(detected)
                })
            }
        );
    }

    group.finish();
}

fn bench_format_detection_by_content(c: &mut Criterion) {
    let test_data = load_format_test_data();

    let mut group = c.benchmark_group("format_detection_by_content");

    for (name, content) in &test_data {
        let size = content.len() as u64;
        group.throughput(Throughput::Bytes(size));

        group.bench_with_input(
            BenchmarkId::new("detect", name),
            content,
            |b, data| {
                b.iter(|| {
                    let detected = detect_by_content(black_box(data));
                    black_box(detected)
                })
            }
        );
    }

    group.finish();
}

fn bench_xml_namespace_detection(c: &mut Criterion) {
    let xml_samples = vec![
        ("assetmap", r#"<?xml version="1.0"?><AssetMap xmlns="http://www.smpte-ra.org/schemas/2067-4/2013"><Id>test</Id></AssetMap>"#),
        ("cpl", r#"<?xml version="1.0"?><CompositionPlaylist xmlns="http://www.smpte-ra.org/schemas/2067-3/2013"><Id>test</Id></CompositionPlaylist>"#),
        ("pklist", r#"<?xml version="1.0"?><PackingList xmlns="http://www.smpte-ra.org/schemas/2067-4/2013"><Id>test</Id></PackingList>"#),
        ("volindex", r#"<?xml version="1.0"?><VolumeIndex xmlns="http://www.smpte-ra.org/schemas/2067-4/2013"><Index>1</Index></VolumeIndex>"#),
        ("unknown_xml", r#"<?xml version="1.0"?><SomeRoot xmlns="http://example.com/unknown"><Element>data</Element></SomeRoot>"#),
    ];

    let mut group = c.benchmark_group("xml_namespace_detection");

    for (name, xml_content) in &xml_samples {
        let size = xml_content.len() as u64;
        group.throughput(Throughput::Bytes(size));

        group.bench_with_input(
            BenchmarkId::new("detect", name),
            xml_content,
            |b, content| {
                b.iter(|| {
                    let detected = detect_xml_format(black_box(content));
                    black_box(detected)
                })
            }
        );
    }

    group.finish();
}

fn bench_mxf_signature_detection(c: &mut Criterion) {
    let mxf_samples = vec![
        ("valid_mxf", vec![
            0x06, 0x0E, 0x2B, 0x34, 0x02, 0x05, 0x01, 0x01,
            0x0D, 0x01, 0x02, 0x01, 0x01, 0x02, 0x00, 0x00,
        ]),
        ("invalid_signature", vec![0x00; 16]),
        ("partial_signature", vec![0x06, 0x0E, 0x2B, 0x34, 0x02]),
        ("wrong_header", vec![
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A,
            0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
        ]),
    ];

    let mut group = c.benchmark_group("mxf_signature_detection");

    for (name, header) in &mxf_samples {
        let mut extended_header = header.clone();
        extended_header.extend_from_slice(&[0u8; 1024]); // Add padding for realistic size

        let size = extended_header.len() as u64;
        group.throughput(Throughput::Bytes(size));

        group.bench_with_input(
            BenchmarkId::new("detect", name),
            &extended_header,
            |b, data| {
                b.iter(|| {
                    let detected = is_mxf_format(black_box(data));
                    black_box(detected)
                })
            }
        );
    }

    group.finish();
}

fn bench_comprehensive_format_detection(c: &mut Criterion) {
    let test_data = load_format_test_data();
    let filenames = vec![
        "ASSETMAP.xml",
        "CPL_test.xml",
        "PKL_test.xml",
        "video.mxf",
        "unknown.dat",
    ];

    let mut group = c.benchmark_group("comprehensive_format_detection");

    for ((name, content), filename) in test_data.iter().zip(filenames.iter().cycle()) {
        let size = content.len() as u64;
        group.throughput(Throughput::Bytes(size));

        group.bench_with_input(
            BenchmarkId::new("detect_comprehensive", name),
            &(filename, content),
            |b, (fname, data)| {
                b.iter(|| {
                    // First try filename detection
                    let by_filename = detect_by_filename(black_box(fname));

                    // If uncertain, use content detection
                    let final_format = if by_filename == "Unknown" {
                        detect_by_content(black_box(data))
                    } else {
                        by_filename
                    };

                    black_box(final_format)
                })
            }
        );
    }

    group.finish();
}

criterion_group!(
    format_detection_benches,
    bench_format_detection_by_extension,
    bench_format_detection_by_content,
    bench_xml_namespace_detection,
    bench_mxf_signature_detection,
    bench_comprehensive_format_detection
);
criterion_main!(format_detection_benches);