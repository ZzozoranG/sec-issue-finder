use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use serde::Deserialize;
use tracing::debug;

use crate::ecosystems::LockfileParser;
use crate::error::SecFinderError;
use crate::types::{Dependency, Ecosystem};

#[derive(Clone, Copy, Debug)]
pub struct PnpmLockParser;

impl LockfileParser for PnpmLockParser {
    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Npm
    }

    fn parse(&self, path: &Path, include_dev: bool) -> Result<Vec<Dependency>, SecFinderError> {
        parse_pnpm_lock(path, include_dev)
    }
}

pub fn parse_pnpm_lock(path: &Path, include_dev: bool) -> Result<Vec<Dependency>, SecFinderError> {
    if !path.exists() {
        return Err(SecFinderError::LockfileNotFound(path.to_path_buf()));
    }

    let contents = fs::read_to_string(path).map_err(|source| SecFinderError::ReadLockfile {
        path: path.to_path_buf(),
        source,
    })?;

    parse_pnpm_lock_str(&contents, path, include_dev)
}

fn parse_pnpm_lock_str(
    contents: &str,
    source_file: &Path,
    include_dev: bool,
) -> Result<Vec<Dependency>, SecFinderError> {
    let lockfile: PnpmLock =
        serde_yaml::from_str(contents).map_err(|source| SecFinderError::ParseLockfileYaml {
            path: source_file.to_path_buf(),
            source,
        })?;

    if lockfile.importers.is_none() && lockfile.packages.is_none() && lockfile.snapshots.is_none() {
        return Err(SecFinderError::UnsupportedPnpmLockfile(
            source_file.to_path_buf(),
        ));
    }

    let direct = direct_dependencies(lockfile.importers.as_ref());
    let mut entries = HashMap::<String, PnpmPackageMetadata>::new();

    if let Some(packages) = lockfile.packages {
        for (key, package) in packages {
            entries.insert(key, package);
        }
    }

    if let Some(snapshots) = lockfile.snapshots {
        for (key, snapshot) in snapshots {
            entries.entry(key).or_insert_with(|| snapshot.into());
        }
    }

    let mut dependencies = Vec::new();

    for (key, metadata) in entries {
        let Some((name, version)) = package_key_name_and_version(&key) else {
            debug!(package_key = %key, "skipping unsupported pnpm package key");
            continue;
        };
        let key = DependencyKey::new(name.clone(), version.clone());
        let direct_context = direct.by_key.get(&key);
        let direct = direct_context.is_some();
        let dev = direct_context
            .map(|context| !context.prod)
            .unwrap_or_else(|| metadata.dev.unwrap_or(false));

        if dev && !include_dev {
            continue;
        }

        dependencies.push(Dependency {
            name: name.clone(),
            version: version.clone(),
            ecosystem: Ecosystem::Npm,
            package_url: Some(package_url(&name, &version)),
            direct,
            dev,
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

fn is_local_package_key(package_key: &str) -> bool {
    let key = package_key.trim();

    has_local_protocol(key)
        || is_relative_path_like(key)
        || key.contains("@file:")
        || key.contains("@link:")
        || key.contains("@workspace:")
        || key.contains("@portal:")
        || key.contains("@path:")
}

fn direct_dependencies(importers: Option<&HashMap<String, PnpmImporter>>) -> DirectDependencies {
    let mut direct = DirectDependencies::default();

    let Some(importers) = importers else {
        return direct;
    };

    for (importer_path, importer) in importers {
        direct.record_section(
            importer_path,
            &importer.dependencies,
            DependencySection::Production,
        );
        direct.record_section(
            importer_path,
            &importer.optional_dependencies,
            DependencySection::Optional,
        );
        direct.record_section(
            importer_path,
            &importer.dev_dependencies,
            DependencySection::Development,
        );
    }

    direct
}

fn dependency_spec_version(spec: &serde_yaml::Value) -> Option<String> {
    match spec {
        serde_yaml::Value::String(version) => normalize_resolved_version(version),
        serde_yaml::Value::Mapping(mapping) => mapping
            .get(serde_yaml::Value::String("version".to_string()))
            .and_then(|value| match value {
                serde_yaml::Value::String(version) => normalize_resolved_version(version),
                _ => None,
            }),
        _ => None,
    }
}

fn normalize_resolved_version(version: &str) -> Option<String> {
    let version = version.split('(').next().unwrap_or(version).trim();

    if !is_registry_version(version) {
        debug!(version, "skipping unsupported pnpm dependency version");
        return None;
    }

    Some(version.to_string())
}

fn package_key_name_and_version(package_key: &str) -> Option<(String, String)> {
    if is_local_package_key(package_key) {
        return None;
    }

    let key = package_key.trim().trim_start_matches('/');
    let key = key.split('(').next().unwrap_or(key).trim();
    let at_index = key.rfind('@')?;

    if at_index == 0 {
        return None;
    }

    let name = &key[..at_index];
    let version = &key[at_index + 1..];

    if !is_registry_package_name(name) || !is_registry_version(version) {
        return None;
    }

    Some((name.to_string(), version.to_string()))
}

fn is_registry_package_name(name: &str) -> bool {
    let name = name.trim();
    if name.is_empty()
        || has_local_protocol(name)
        || is_relative_path_like(name)
        || name.starts_with('/')
        || name.contains('\\')
    {
        return false;
    }

    if let Some(scoped_name) = name.strip_prefix('@') {
        let mut parts = scoped_name.split('/');
        let Some(scope) = parts.next() else {
            return false;
        };
        let Some(package) = parts.next() else {
            return false;
        };

        return !scope.is_empty() && !package.is_empty() && parts.next().is_none();
    }

    !name.contains('/')
}

fn is_registry_version(version: &str) -> bool {
    let version = version.trim();
    !version.is_empty()
        && !has_local_protocol(version)
        && !is_relative_path_like(version)
        && !version.starts_with('/')
        && !version.contains('/')
        && !version.contains('\\')
}

fn has_local_protocol(value: &str) -> bool {
    value.starts_with("file:")
        || value.starts_with("link:")
        || value.starts_with("workspace:")
        || value.starts_with("portal:")
        || value.starts_with("path:")
}

fn is_relative_path_like(value: &str) -> bool {
    value.starts_with("../")
        || value.starts_with("./")
        || value.starts_with("..\\")
        || value.starts_with(".\\")
        || value.contains("/../")
        || value.contains("\\..\\")
}

fn package_url(name: &str, version: &str) -> String {
    format!("pkg:npm/{}@{}", name.replace('@', "%40"), version)
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct DependencyKey {
    name: String,
    version: String,
}

impl DependencyKey {
    fn new(name: String, version: String) -> Self {
        Self { name, version }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DependencySection {
    Production,
    Development,
    Optional,
}

#[derive(Clone, Debug, Default)]
struct DirectDependencyContext {
    prod: bool,
    dev: bool,
    importers: HashSet<String>,
}

#[derive(Default)]
struct DirectDependencies {
    by_key: HashMap<DependencyKey, DirectDependencyContext>,
}

impl DirectDependencies {
    fn record_section(
        &mut self,
        importer_path: &str,
        dependencies: &HashMap<String, serde_yaml::Value>,
        section: DependencySection,
    ) {
        for (name, spec) in dependencies {
            let Some(version) = dependency_spec_version(spec) else {
                continue;
            };
            let context = self
                .by_key
                .entry(DependencyKey::new(name.clone(), version))
                .or_default();

            match section {
                DependencySection::Production | DependencySection::Optional => context.prod = true,
                DependencySection::Development => context.dev = true,
            }
            context.importers.insert(importer_path.to_string());
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PnpmLock {
    importers: Option<HashMap<String, PnpmImporter>>,
    packages: Option<HashMap<String, PnpmPackageMetadata>>,
    snapshots: Option<HashMap<String, PnpmSnapshot>>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PnpmImporter {
    #[serde(default)]
    dependencies: HashMap<String, serde_yaml::Value>,
    #[serde(default)]
    dev_dependencies: HashMap<String, serde_yaml::Value>,
    #[serde(default)]
    optional_dependencies: HashMap<String, serde_yaml::Value>,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct PnpmPackageMetadata {
    dev: Option<bool>,
}

#[derive(Debug, Default, Deserialize)]
struct PnpmSnapshot {
    dev: Option<bool>,
}

impl From<PnpmSnapshot> for PnpmPackageMetadata {
    fn from(snapshot: PnpmSnapshot) -> Self {
        Self { dev: snapshot.dev }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::error::SecFinderError;
    use crate::types::Ecosystem;

    use super::{package_key_name_and_version, parse_pnpm_lock};

    fn fixture(name: &str) -> String {
        format!("tests/fixtures/pnpm/{name}")
    }

    #[test]
    fn parses_importers_and_packages() {
        let dependencies = parse_pnpm_lock(Path::new(&fixture("basic.yaml")), true).unwrap();

        assert_eq!(dependencies.len(), 2);
        assert_eq!(dependencies[0].name, "left-pad");
        assert_eq!(dependencies[0].version, "1.3.0");
        assert_eq!(dependencies[0].ecosystem, Ecosystem::Npm);
        assert_eq!(
            dependencies[0].package_url.as_deref(),
            Some("pkg:npm/left-pad@1.3.0")
        );
        assert!(dependencies[0].direct);
        assert!(!dependencies[0].dev);

        assert_eq!(dependencies[1].name, "loose-envify");
        assert!(!dependencies[1].direct);
    }

    #[test]
    fn excludes_dev_dependencies_when_requested() {
        let dependencies = parse_pnpm_lock(Path::new(&fixture("dev.yaml")), false).unwrap();

        assert!(dependencies.is_empty());
    }

    #[test]
    fn parses_dev_dependency_when_included() {
        let dependencies = parse_pnpm_lock(Path::new(&fixture("dev.yaml")), true).unwrap();

        assert_eq!(dependencies.len(), 1);
        assert_eq!(dependencies[0].name, "eslint");
        assert!(dependencies[0].direct);
        assert!(dependencies[0].dev);
    }

    #[test]
    fn parses_scoped_package() {
        let dependencies = parse_pnpm_lock(Path::new(&fixture("scoped.yaml")), true).unwrap();

        assert_eq!(dependencies.len(), 1);
        assert_eq!(dependencies[0].name, "@scope/pkg");
        assert_eq!(dependencies[0].version, "2.0.0");
        assert_eq!(
            dependencies[0].package_url.as_deref(),
            Some("pkg:npm/%40scope/pkg@2.0.0")
        );
    }

    #[test]
    fn parses_peer_suffix_package_key() {
        assert_eq!(
            package_key_name_and_version("/react-dom@18.2.0(react@18.2.0)").unwrap(),
            ("react-dom".to_string(), "18.2.0".to_string())
        );
    }

    #[test]
    fn parses_snapshots_without_packages() {
        let dependencies = parse_pnpm_lock(Path::new(&fixture("snapshots.yaml")), true).unwrap();

        assert_eq!(dependencies.len(), 1);
        assert_eq!(dependencies[0].name, "is-number");
        assert_eq!(dependencies[0].version, "7.0.0");
    }

    #[test]
    fn parses_workspace_with_multiple_importers() {
        let dependencies = parse_pnpm_lock(Path::new(&fixture("workspace.yaml")), true).unwrap();
        let names: Vec<_> = dependencies
            .iter()
            .map(|dependency| dependency.name.as_str())
            .collect();

        assert_eq!(
            names,
            vec![
                "@scope/pkg",
                "eslint",
                "left-pad",
                "loose-envify",
                "react-dom"
            ]
        );

        let left_pad = dependency(&dependencies, "left-pad");
        assert!(left_pad.direct);
        assert!(!left_pad.dev);

        let eslint = dependency(&dependencies, "eslint");
        assert!(eslint.direct);
        assert!(eslint.dev);

        let loose_envify = dependency(&dependencies, "loose-envify");
        assert!(!loose_envify.direct);
        assert!(!loose_envify.dev);

        let scoped = dependency(&dependencies, "@scope/pkg");
        assert!(scoped.direct);
        assert_eq!(scoped.version, "2.0.0");

        let react_dom = dependency(&dependencies, "react-dom");
        assert_eq!(react_dom.version, "18.2.0");
    }

    #[test]
    fn classifies_same_name_different_versions_by_resolved_version() {
        let dependencies = parse_pnpm_lock(
            Path::new(&fixture("same-name-different-versions.yaml")),
            true,
        )
        .unwrap();
        let old_version = dependency_with_version(&dependencies, "shared-pkg", "1.0.0");
        let new_version = dependency_with_version(&dependencies, "shared-pkg", "2.0.0");

        assert!(old_version.direct);
        assert!(!old_version.dev);
        assert!(new_version.direct);
        assert!(new_version.dev);
    }

    #[test]
    fn prod_wins_when_same_name_version_is_prod_and_dev() {
        let dependencies =
            parse_pnpm_lock(Path::new(&fixture("prod-dev-same-version.yaml")), true).unwrap();
        let dependency = dependency_with_version(&dependencies, "shared-pkg", "1.0.0");

        assert!(dependency.direct);
        assert!(!dependency.dev);
    }

    #[test]
    fn scoped_package_can_be_direct_dev() {
        let dependencies = parse_pnpm_lock(Path::new(&fixture("scoped-dev.yaml")), true).unwrap();
        let dependency = dependency(&dependencies, "@scope/tool");

        assert_eq!(dependency.version, "3.0.0");
        assert!(dependency.direct);
        assert!(dependency.dev);
    }

    #[test]
    fn optional_dependency_is_direct_prod() {
        let dependencies =
            parse_pnpm_lock(Path::new(&fixture("optional-multi-importer.yaml")), true).unwrap();
        let dependency = dependency(&dependencies, "optional-pkg");

        assert!(dependency.direct);
        assert!(!dependency.dev);
    }

    #[test]
    fn transitive_same_name_different_version_stays_transitive() {
        let dependencies =
            parse_pnpm_lock(Path::new(&fixture("transitive-name-collision.yaml")), true).unwrap();
        let direct_version = dependency_with_version(&dependencies, "shared-pkg", "1.0.0");
        let transitive_version = dependency_with_version(&dependencies, "shared-pkg", "2.0.0");

        assert!(direct_version.direct);
        assert!(!direct_version.dev);
        assert!(!transitive_version.direct);
        assert!(!transitive_version.dev);
    }

    #[test]
    fn workspace_dev_dependency_is_filtered_when_dev_is_excluded() {
        let dependencies = parse_pnpm_lock(Path::new(&fixture("workspace.yaml")), false).unwrap();

        assert!(dependencies
            .iter()
            .all(|dependency| dependency.name != "eslint"));
    }

    #[test]
    fn ignores_local_workspace_link_and_file_package_keys() {
        let dependencies =
            parse_pnpm_lock(Path::new(&fixture("local-dependencies.yaml")), true).unwrap();

        assert_eq!(dependencies.len(), 1);
        assert_eq!(dependencies[0].name, "lodash");
        assert_eq!(dependencies[0].version, "4.17.20");
    }

    #[test]
    fn skips_local_and_path_like_package_keys() {
        let dependencies =
            parse_pnpm_lock(Path::new(&fixture("path-like-keys.yaml")), true).unwrap();

        assert_eq!(dependencies.len(), 2);
        assert_eq!(dependencies[0].name, "@scope/pkg");
        assert_eq!(dependencies[0].version, "2.0.0");
        assert_eq!(dependencies[1].name, "react-dom");
        assert_eq!(dependencies[1].version, "18.2.0");
    }

    #[test]
    fn malformed_package_keys_are_skipped_without_error() {
        let dependencies =
            parse_pnpm_lock(Path::new(&fixture("malformed-package-key.yaml")), true).unwrap();

        assert_eq!(dependencies.len(), 1);
        assert_eq!(dependencies[0].name, "left-pad");
    }

    fn dependency<'a>(
        dependencies: &'a [crate::types::Dependency],
        name: &str,
    ) -> &'a crate::types::Dependency {
        dependencies
            .iter()
            .find(|dependency| dependency.name == name)
            .unwrap()
    }

    fn dependency_with_version<'a>(
        dependencies: &'a [crate::types::Dependency],
        name: &str,
        version: &str,
    ) -> &'a crate::types::Dependency {
        dependencies
            .iter()
            .find(|dependency| dependency.name == name && dependency.version == version)
            .unwrap()
    }

    #[test]
    fn malformed_yaml_returns_parser_error() {
        let error = parse_pnpm_lock(Path::new(&fixture("malformed.yaml")), true).unwrap_err();

        assert!(matches!(error, SecFinderError::ParseLockfileYaml { .. }));
    }

    #[test]
    fn missing_version_package_key_is_skipped() {
        let dependencies =
            parse_pnpm_lock(Path::new(&fixture("missing-version.yaml")), true).unwrap();

        assert!(dependencies.is_empty());
    }

    #[test]
    fn missing_file_returns_useful_error() {
        let error = parse_pnpm_lock(Path::new("tests/fixtures/pnpm/does-not-exist.yaml"), true)
            .unwrap_err();

        assert!(matches!(error, SecFinderError::LockfileNotFound(_)));
    }
}
