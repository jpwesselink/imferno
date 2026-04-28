//! Storage abstraction for IMF package I/O.
//!
//! Provides the [`Storage`] trait and built-in implementations for
//! local filesystem ([`fs::FsStorage`]) and S3 ([`s3::S3Storage`],
//! behind the `aws-s3` feature flag).

pub mod fs;

#[cfg(feature = "aws-s3")]
pub mod s3;
