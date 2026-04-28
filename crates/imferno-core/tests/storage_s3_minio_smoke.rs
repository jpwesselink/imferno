//! Live S3 smoke test against a local MinIO container.
//!
//! Disabled by default. To run, start MinIO + populate a bucket, then:
//!
//! ```sh
//! docker run --rm -d --name imferno-minio -p 9000:9000 \
//!     -e MINIO_ROOT_USER=minioadmin -e MINIO_ROOT_PASSWORD=minioadmin \
//!     minio/minio server /data
//!
//! AWS_ACCESS_KEY_ID=minioadmin AWS_SECRET_ACCESS_KEY=minioadmin \
//!     aws --endpoint-url http://127.0.0.1:9000 s3 mb s3://imferno-test
//! AWS_ACCESS_KEY_ID=minioadmin AWS_SECRET_ACCESS_KEY=minioadmin \
//!     aws --endpoint-url http://127.0.0.1:9000 s3 cp test-data/Application2/ \
//!         s3://imferno-test/Application2/ --recursive --include '*.xml'
//!
//! RUN_MINIO=1 cargo test -p imferno-core --features aws-s3 \
//!     --test storage_s3_minio_smoke
//! ```

#![cfg(feature = "aws-s3")]

use imferno_core::storage::{s3::S3Storage, Storage, StorageUri};

fn build_minio_client() -> aws_sdk_s3::Client {
    use aws_sdk_s3::config::{Builder, Credentials, Region};

    let creds = Credentials::new("minioadmin", "minioadmin", None, None, "minio");
    let cfg = Builder::new()
        .region(Region::new("us-east-1"))
        .endpoint_url("http://127.0.0.1:9000")
        .credentials_provider(creds)
        .force_path_style(true)
        .behavior_version(aws_sdk_s3::config::BehaviorVersion::latest())
        .build();
    aws_sdk_s3::Client::from_conf(cfg)
}

#[test]
fn s3_storage_lists_and_reads_against_minio() {
    if std::env::var("RUN_MINIO").is_err() {
        eprintln!("skipping (set RUN_MINIO=1 to run)");
        return;
    }

    let storage = S3Storage::from_client(build_minio_client()).unwrap();
    let uri = StorageUri::parse("s3://imferno-test/Application2/").unwrap();

    let entries = storage.list(&uri).expect("list");
    assert!(!entries.is_empty(), "bucket should contain XML fixtures");
    let any_cpl = entries
        .iter()
        .find(|e| e.uri.contains("CPL_") && e.uri.ends_with(".xml"))
        .expect("at least one CPL XML present");

    let cpl_uri = StorageUri::parse(&any_cpl.uri).unwrap();
    let body = storage.read_to_string(&cpl_uri).expect("read");
    assert!(body.contains("CompositionPlaylist"), "CPL body should be valid XML");
}

#[test]
fn s3_read_xml_files_round_trips_against_minio() {
    if std::env::var("RUN_MINIO").is_err() {
        eprintln!("skipping (set RUN_MINIO=1 to run)");
        return;
    }

    use imferno_core::package::read_xml_files;

    let storage = S3Storage::from_client(build_minio_client()).unwrap();
    let uri = StorageUri::parse("s3://imferno-test/Application2/").unwrap();

    let files = read_xml_files(&uri, &storage).expect("read_xml_files");
    assert!(!files.is_empty(), "should return at least one XML file");
    assert!(
        files.keys().all(|k| k.starts_with("s3://imferno-test/")),
        "keys should be s3:// URIs",
    );
    assert!(
        files.values().all(|v| v.contains("xml")),
        "every value should look like an XML file",
    );
}
