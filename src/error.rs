use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum SecFinderError {
    #[error("unsupported lockfile: {0}")]
    UnsupportedLockfile(PathBuf),

    #[error("multiple supported lockfiles found; pass --lockfile to choose one: {paths}")]
    AmbiguousLockfiles { paths: String },

    #[error("lockfile not found: {0}")]
    LockfileNotFound(PathBuf),

    #[error("failed to read lockfile {path}: {source}")]
    ReadLockfile {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to parse JSON lockfile {path}: {source}")]
    ParseLockfileJson {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },

    #[error("unsupported npm lockfile structure in {0}")]
    UnsupportedNpmLockfile(PathBuf),

    #[error("dependency entry {entry} in {path} is missing a version")]
    MissingDependencyVersion { path: PathBuf, entry: String },

    #[error("failed to parse YAML lockfile {path}: {source}")]
    ParseLockfileYaml {
        path: PathBuf,
        #[source]
        source: serde_yaml::Error,
    },

    #[error("unsupported pnpm lockfile structure in {0}")]
    UnsupportedPnpmLockfile(PathBuf),

    #[error("OSV request failed: {0}")]
    OsvRequest(#[from] reqwest::Error),

    #[error("failed to read OSV mock response {path}: {source}")]
    ReadOsvMockResponse {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("OSV returned non-success status {status}: {body}")]
    OsvStatus {
        status: reqwest::StatusCode,
        body: String,
    },

    #[error("failed to parse OSV response: {0}")]
    OsvMalformedResponse(serde_json::Error),

    #[error("OSV response result count {actual} did not match dependency count {expected}")]
    OsvResponseLengthMismatch { expected: usize, actual: usize },

    #[error("policy failed: {message}")]
    PolicyFailed { message: String },

    #[error("reporting failed: {0}")]
    Reporting(#[from] serde_json::Error),
}
