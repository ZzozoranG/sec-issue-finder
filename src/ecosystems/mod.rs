use std::path::Path;

use crate::error::SecFinderError;
use crate::types::{Dependency, Ecosystem};

pub mod dart;
pub mod npm;
pub mod pnpm;

pub trait LockfileParser {
    fn ecosystem(&self) -> Ecosystem;
    fn parse(&self, path: &Path, include_dev: bool) -> Result<Vec<Dependency>, SecFinderError>;
}

pub fn parser_for_lockfile(path: &Path) -> Result<Box<dyn LockfileParser>, SecFinderError> {
    match path.file_name().and_then(|name| name.to_str()) {
        Some("package-lock.json") => Ok(Box::new(npm::NpmPackageLockParser)),
        Some("pnpm-lock.yaml") => Ok(Box::new(pnpm::PnpmLockParser)),
        _ => Err(SecFinderError::UnsupportedLockfile(path.to_path_buf())),
    }
}
