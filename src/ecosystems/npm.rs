use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use crate::ecosystems::LockfileParser;
use crate::error::SecFinderError;
use crate::types::{Dependency, Ecosystem};

#[derive(Clone, Copy, Debug)]
pub struct NpmPackageLockParser;

impl LockfileParser for NpmPackageLockParser {
    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Npm
    }

    fn parse(&self, path: &Path, include_dev: bool) -> Result<Vec<Dependency>, SecFinderError> {
        parse_package_lock(path, include_dev)
    }
}

pub fn parse_package_lock(
    path: &Path,
    include_dev: bool,
) -> Result<Vec<Dependency>, SecFinderError> {
    if !path.exists() {
        return Err(SecFinderError::LockfileNotFound(path.to_path_buf()));
    }

    let contents = fs::read_to_string(path).map_err(|source| SecFinderError::ReadLockfile {
        path: path.to_path_buf(),
        source,
    })?;

    parse_package_lock_str(&contents, path, include_dev)
}

fn parse_package_lock_str(
    contents: &str,
    source_file: &Path,
    include_dev: bool,
) -> Result<Vec<Dependency>, SecFinderError> {
    let lockfile: PackageLock =
        serde_json::from_str(contents).map_err(|source| SecFinderError::ParseLockfileJson {
            path: source_file.to_path_buf(),
            source,
        })?;

    let packages = lockfile
        .packages
        .ok_or_else(|| SecFinderError::UnsupportedNpmLockfile(source_file.to_path_buf()))?;
    let root = packages.get("");
    let direct_dependencies = direct_dependency_names(root);
    let mut dependencies = Vec::new();

    for (entry_path, package) in packages.iter() {
        let Some(name) = package_name_from_entry_path(entry_path) else {
            continue;
        };

        let Some(version) = package.version.clone() else {
            // npm can retain placeholder entries for optional dependencies that
            // were not installed on the current platform. They do not identify a
            // registry package version and cannot be queried against OSV.
            if package.optional {
                continue;
            }

            return Err(SecFinderError::MissingDependencyVersion {
                path: source_file.to_path_buf(),
                entry: entry_path.clone(),
            });
        };

        let dev = package.dev || package.dev_optional;
        if dev && !include_dev {
            continue;
        }

        dependencies.push(Dependency {
            package_url: Some(package_url(&name, &version)),
            direct: direct_dependencies.contains(&name),
            dev,
            ecosystem: Ecosystem::Npm,
            name,
            version,
            source_file: source_file.to_path_buf(),
        });
    }

    dependencies.sort_by(|left, right| {
        left.name
            .cmp(&right.name)
            .then(left.version.cmp(&right.version))
    });
    Ok(dependencies)
}

fn direct_dependency_names(root: Option<&PackageEntry>) -> HashSet<String> {
    let mut names = HashSet::new();

    let Some(root) = root else {
        return names;
    };

    for dependency_map in [
        root.dependencies.as_ref(),
        root.dev_dependencies.as_ref(),
        root.optional_dependencies.as_ref(),
        root.peer_dependencies.as_ref(),
    ]
    .into_iter()
    .flatten()
    {
        names.extend(dependency_map.keys().cloned());
    }

    names
}

fn package_name_from_entry_path(entry_path: &str) -> Option<String> {
    let mut segments = entry_path.split('/').peekable();
    let mut package_name = None;

    while let Some(segment) = segments.next() {
        if segment != "node_modules" {
            continue;
        }

        let first = segments.next()?;
        if first.starts_with('@') {
            let second = segments.next()?;
            package_name = Some(format!("{first}/{second}"));
        } else {
            package_name = Some(first.to_string());
        }
    }

    package_name
}

fn package_url(name: &str, version: &str) -> String {
    format!("pkg:npm/{}@{}", name.replace('@', "%40"), version)
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct PackageLock {
    packages: Option<HashMap<String, PackageEntry>>,
}

#[derive(Debug, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct PackageEntry {
    version: Option<String>,
    #[serde(default)]
    dev: bool,
    #[serde(default)]
    dev_optional: bool,
    #[serde(default)]
    optional: bool,
    dependencies: Option<HashMap<String, String>>,
    dev_dependencies: Option<HashMap<String, String>>,
    optional_dependencies: Option<HashMap<String, String>>,
    peer_dependencies: Option<HashMap<String, String>>,
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::ecosystems::{parser_for_lockfile, LockfileParser};
    use crate::error::SecFinderError;
    use crate::types::Ecosystem;

    use super::{parse_package_lock, NpmPackageLockParser};

    fn fixture(name: &str) -> String {
        format!("tests/fixtures/npm/{name}")
    }

    #[test]
    fn parses_empty_packages_object() {
        let dependencies =
            parse_package_lock(Path::new(&fixture("empty-packages.json")), true).unwrap();

        assert!(dependencies.is_empty());
    }

    #[test]
    fn parses_through_lockfile_parser_trait() {
        let parser = NpmPackageLockParser;
        let dependencies = parser
            .parse(Path::new(&fixture("direct-production.json")), true)
            .unwrap();

        assert_eq!(parser.ecosystem(), Ecosystem::Npm);
        assert_eq!(dependencies.len(), 1);
        assert_eq!(dependencies[0].name, "left-pad");
    }

    #[test]
    fn selects_npm_parser_for_package_lock() {
        let parser = parser_for_lockfile(Path::new("package-lock.json")).unwrap();
        let dependencies = parser
            .parse(Path::new(&fixture("direct-production.json")), true)
            .unwrap();

        assert_eq!(parser.ecosystem(), Ecosystem::Npm);
        assert_eq!(dependencies[0].ecosystem, Ecosystem::Npm);
    }

    #[test]
    fn parses_direct_production_dependency() {
        let dependencies =
            parse_package_lock(Path::new(&fixture("direct-production.json")), true).unwrap();

        assert_eq!(dependencies.len(), 1);
        assert_eq!(dependencies[0].name, "left-pad");
        assert_eq!(dependencies[0].version, "1.3.0");
        assert_eq!(
            dependencies[0].package_url.as_deref(),
            Some("pkg:npm/left-pad@1.3.0")
        );
        assert_eq!(
            dependencies[0].source_file,
            Path::new(&fixture("direct-production.json"))
        );
        assert!(dependencies[0].direct);
        assert!(!dependencies[0].dev);
    }

    #[test]
    fn parses_direct_dev_dependency() {
        let dependencies =
            parse_package_lock(Path::new(&fixture("direct-dev.json")), true).unwrap();

        assert_eq!(dependencies.len(), 1);
        assert_eq!(dependencies[0].name, "eslint");
        assert!(dependencies[0].direct);
        assert!(dependencies[0].dev);
    }

    #[test]
    fn excludes_dev_dependency_when_requested() {
        let dependencies =
            parse_package_lock(Path::new(&fixture("direct-dev.json")), false).unwrap();

        assert!(dependencies.is_empty());
    }

    #[test]
    fn parses_transitive_dependency() {
        let dependencies =
            parse_package_lock(Path::new(&fixture("transitive.json")), true).unwrap();

        let transitive = dependencies
            .iter()
            .find(|dependency| dependency.name == "loose-envify")
            .unwrap();

        assert!(!transitive.direct);
        assert!(!transitive.dev);
    }

    #[test]
    fn parses_scoped_package() {
        let dependencies = parse_package_lock(Path::new(&fixture("scoped.json")), true).unwrap();

        assert_eq!(dependencies.len(), 1);
        assert_eq!(dependencies[0].name, "@scope/pkg");
        assert_eq!(
            dependencies[0].package_url.as_deref(),
            Some("pkg:npm/%40scope/pkg@2.0.0")
        );
        assert!(dependencies[0].direct);
    }

    #[test]
    fn malformed_json_returns_parser_error() {
        let error = parse_package_lock(Path::new(&fixture("malformed.json")), true).unwrap_err();

        assert!(matches!(error, SecFinderError::ParseLockfileJson { .. }));
    }

    #[test]
    fn missing_version_returns_useful_error() {
        let error =
            parse_package_lock(Path::new(&fixture("missing-version.json")), true).unwrap_err();

        assert!(matches!(
            error,
            SecFinderError::MissingDependencyVersion { .. }
        ));
    }

    #[test]
    fn missing_version_optional_entry_is_skipped() {
        let dependencies =
            parse_package_lock(Path::new(&fixture("missing-version-optional.json")), true).unwrap();

        assert_eq!(dependencies.len(), 1);
        assert_eq!(dependencies[0].name, "sec-issue-finder");
        assert_eq!(dependencies[0].version, "0.1.0");
    }

    #[test]
    fn missing_file_returns_useful_error() {
        let error = parse_package_lock(Path::new("tests/fixtures/npm/does-not-exist.json"), true)
            .unwrap_err();

        assert!(matches!(error, SecFinderError::LockfileNotFound(_)));
    }
}
