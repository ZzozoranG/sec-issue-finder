use std::collections::BTreeMap;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::pin::Pin;

use crate::analyzers;
use crate::clients::osv::{OsvClient, OsvQueryResult};
use crate::error::SecFinderError;
use crate::types::{Dependency, Ecosystem, ScanConfig, ScanResult};

pub trait AdvisoryClient {
    fn query_batch<'a>(
        &'a self,
        dependencies: &'a [Dependency],
    ) -> Pin<Box<dyn Future<Output = Result<Vec<OsvQueryResult>, SecFinderError>> + Send + 'a>>;
}

impl AdvisoryClient for OsvClient {
    fn query_batch<'a>(
        &'a self,
        dependencies: &'a [Dependency],
    ) -> Pin<Box<dyn Future<Output = Result<Vec<OsvQueryResult>, SecFinderError>> + Send + 'a>>
    {
        Box::pin(async move { self.query_batch(dependencies).await })
    }
}

pub async fn scan(
    config: ScanConfig,
    advisory_client: &dyn AdvisoryClient,
) -> Result<ScanResult, SecFinderError> {
    let lockfile = resolve_lockfile_path(&config)?;

    let parser = crate::ecosystems::parser_for_lockfile(&lockfile)?;
    let ecosystem = parser.ecosystem();
    let dependencies = parser.parse(&lockfile, config.include_dev)?;
    let advisory_dependencies = dependencies_for_advisory_query(&dependencies);
    let osv_results = advisory_client.query_batch(&advisory_dependencies).await?;
    let findings = analyzers::osv::findings_from_osv_results(&osv_results);

    Ok(ScanResult {
        lockfile,
        ecosystem,
        dependencies,
        findings,
    })
}

fn dependencies_for_advisory_query(dependencies: &[Dependency]) -> Vec<Dependency> {
    let mut unique = BTreeMap::<AdvisoryDependencyKey, Dependency>::new();

    for dependency in dependencies {
        // Advisory lookups are per registry package identity, not per importer path.
        // pnpm dependencies still use the OSV "npm" ecosystem because pnpm installs
        // npm registry packages. This avoids duplicate OSV requests and duplicate
        // findings when one resolved package appears through multiple pnpm importers
        // or peer-suffix package keys. Importer/path details are not modeled yet, so
        // duplicates collapse to conservative direct/dev metadata below.
        let key = AdvisoryDependencyKey::from(dependency);
        unique
            .entry(key)
            .and_modify(|existing| merge_dependency_metadata(existing, dependency))
            .or_insert_with(|| dependency.clone());
    }

    unique.into_values().collect()
}

fn merge_dependency_metadata(existing: &mut Dependency, duplicate: &Dependency) {
    existing.direct |= duplicate.direct;
    existing.dev &= duplicate.dev;

    if duplicate.source_file < existing.source_file {
        existing.source_file = duplicate.source_file.clone();
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct AdvisoryDependencyKey {
    osv_ecosystem: &'static str,
    name: String,
    version: String,
}

impl From<&Dependency> for AdvisoryDependencyKey {
    fn from(dependency: &Dependency) -> Self {
        Self {
            osv_ecosystem: advisory_ecosystem(dependency.ecosystem),
            name: dependency.name.clone(),
            version: dependency.version.clone(),
        }
    }
}

fn advisory_ecosystem(ecosystem: Ecosystem) -> &'static str {
    match ecosystem {
        Ecosystem::Npm => "npm",
        Ecosystem::Dart => "Pub",
    }
}

pub fn resolve_lockfile_path(config: &ScanConfig) -> Result<PathBuf, SecFinderError> {
    resolve_lockfile_path_in(config, Path::new("."))
}

fn resolve_lockfile_path_in(config: &ScanConfig, cwd: &Path) -> Result<PathBuf, SecFinderError> {
    if let Some(lockfile) = &config.lockfile {
        return Ok(lockfile.clone());
    }

    let package_lock = cwd.join("package-lock.json");
    let pnpm_lock = cwd.join("pnpm-lock.yaml");
    let package_lock_exists = package_lock.exists();
    let pnpm_lock_exists = pnpm_lock.exists();

    match (package_lock_exists, pnpm_lock_exists) {
        (true, true) => Err(SecFinderError::AmbiguousLockfiles {
            paths: format!("{}, {}", package_lock.display(), pnpm_lock.display()),
        }),
        (true, false) => Ok(package_lock),
        (false, true) => Ok(pnpm_lock),
        (false, false) => Ok(package_lock),
    }
}

#[cfg(test)]
mod tests {
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::{Arc, Mutex};

    use reqwest::StatusCode;

    use crate::clients::osv::{OsvQueryResult, OsvSeverity, OsvVulnerability};
    use crate::error::SecFinderError;
    use crate::policy;
    use crate::reporters;
    use crate::types::{Dependency, Ecosystem, ReportFormat, ScanConfig, Severity};

    use super::{dependencies_for_advisory_query, scan, AdvisoryClient};

    #[tokio::test]
    async fn no_vulnerabilities() {
        let client = MockAdvisoryClient::no_vulnerabilities();
        let result = scan(
            config("tests/fixtures/npm/direct-package-lock/package-lock.json"),
            &client,
        )
        .await
        .expect("scan should succeed");

        assert_eq!(result.dependencies.len(), 1);
        assert!(result.findings.is_empty());
    }

    #[tokio::test]
    async fn one_high_vulnerability() {
        let client = MockAdvisoryClient::one_high_vulnerability();
        let result = scan(
            config("tests/fixtures/npm/direct-package-lock/package-lock.json"),
            &client,
        )
        .await
        .expect("scan should succeed");

        assert_eq!(result.findings.len(), 1);
        assert_eq!(result.findings[0].dependency.name, "left-pad");
        assert_eq!(result.findings[0].advisory.id, "GHSA-high");
        assert_eq!(result.findings[0].advisory.severity, Severity::High);
    }

    #[tokio::test]
    async fn format_json_renders_json_report() {
        let client = MockAdvisoryClient::one_high_vulnerability();
        let result = scan(
            config("tests/fixtures/npm/direct-package-lock/package-lock.json"),
            &client,
        )
        .await
        .expect("scan should succeed");
        let output = reporters::render_with_format(&result, ReportFormat::Json).unwrap();

        assert!(output.starts_with("{"));
        assert!(output.contains(r#""schema_version": "1.0""#));
        assert!(output.contains(r#""id": "GHSA-high""#));
    }

    #[tokio::test]
    async fn fail_on_high_exits_with_failure_policy() {
        let client = MockAdvisoryClient::one_high_vulnerability();
        let result = scan(
            config("tests/fixtures/npm/direct-package-lock/package-lock.json"),
            &client,
        )
        .await
        .expect("scan should succeed");

        assert!(policy::evaluate_with_threshold(&result, Some(Severity::High)).is_err());
    }

    #[tokio::test]
    async fn fail_on_critical_does_not_fail_on_high() {
        let client = MockAdvisoryClient::one_high_vulnerability();
        let result = scan(
            config("tests/fixtures/npm/direct-package-lock/package-lock.json"),
            &client,
        )
        .await
        .expect("scan should succeed");

        assert!(policy::evaluate_with_threshold(&result, Some(Severity::Critical)).is_ok());
    }

    #[tokio::test]
    async fn missing_lockfile_returns_helpful_error() {
        let client = MockAdvisoryClient::no_vulnerabilities();
        let error = scan(
            config("tests/fixtures/npm/missing/package-lock.json"),
            &client,
        )
        .await
        .unwrap_err();

        assert!(matches!(error, SecFinderError::LockfileNotFound(_)));
    }

    #[tokio::test]
    async fn malformed_lockfile_returns_helpful_error() {
        let client = MockAdvisoryClient::no_vulnerabilities();
        let error = scan(
            config("tests/fixtures/npm/malformed-package-lock/package-lock.json"),
            &client,
        )
        .await
        .unwrap_err();

        assert!(matches!(error, SecFinderError::ParseLockfileJson { .. }));
    }

    #[tokio::test]
    async fn osv_client_errors_are_returned() {
        let client = MockAdvisoryClient::error();
        let error = scan(
            config("tests/fixtures/npm/direct-package-lock/package-lock.json"),
            &client,
        )
        .await
        .unwrap_err();

        assert!(matches!(
            error,
            SecFinderError::OsvStatus {
                status: StatusCode::BAD_GATEWAY,
                ..
            }
        ));
    }

    #[test]
    fn default_lockfile_path_is_package_lock_json() {
        let lockfile = super::resolve_lockfile_path(&ScanConfig {
            lockfile: None,
            format: ReportFormat::Table,
            fail_on: None,
            include_dev: true,
        })
        .unwrap();

        assert_eq!(lockfile, std::path::PathBuf::from("./package-lock.json"));
    }

    #[test]
    fn auto_detects_pnpm_lock_when_package_lock_is_absent() {
        let lockfile = super::resolve_lockfile_path_in(
            &ScanConfig {
                lockfile: None,
                format: ReportFormat::Table,
                fail_on: None,
                include_dev: true,
            },
            std::path::Path::new("tests/fixtures/scan/pnpm-only"),
        )
        .unwrap();

        assert_eq!(
            lockfile,
            std::path::PathBuf::from("tests/fixtures/scan/pnpm-only/pnpm-lock.yaml")
        );
    }

    #[test]
    fn package_lock_and_pnpm_lock_are_ambiguous_by_default() {
        let error = super::resolve_lockfile_path_in(
            &ScanConfig {
                lockfile: None,
                format: ReportFormat::Table,
                fail_on: None,
                include_dev: true,
            },
            std::path::Path::new("tests/fixtures/scan/ambiguous"),
        )
        .unwrap_err();

        assert!(matches!(error, SecFinderError::AmbiguousLockfiles { .. }));
    }

    #[tokio::test]
    async fn scans_explicit_pnpm_lockfile_with_mocked_osv() {
        let client = MockAdvisoryClient::one_high_vulnerability();
        let result = scan(
            config("tests/fixtures/scan/pnpm-only/pnpm-lock.yaml"),
            &client,
        )
        .await
        .expect("scan should succeed");

        assert_eq!(result.dependencies.len(), 1);
        assert_eq!(result.dependencies[0].name, "left-pad");
        assert_eq!(
            result.dependencies[0].ecosystem,
            crate::types::Ecosystem::Npm
        );
        assert_eq!(result.findings.len(), 1);
        assert_eq!(result.findings[0].advisory.id, "GHSA-high");
    }

    #[tokio::test]
    async fn pnpm_json_report_uses_existing_schema() {
        let client = MockAdvisoryClient::one_high_vulnerability();
        let result = scan(
            config("tests/fixtures/scan/pnpm-only/pnpm-lock.yaml"),
            &client,
        )
        .await
        .expect("scan should succeed");
        let output = reporters::render_with_format(&result, ReportFormat::Json).unwrap();

        assert!(output.contains(r#""schema_version": "1.0""#));
        assert!(output.contains(r#""ecosystem": "npm""#));
        assert!(output.contains(r#""name": "left-pad""#));
    }

    #[tokio::test]
    async fn pnpm_lodash_is_queried_as_osv_npm_dependency() {
        let client = RecordingAdvisoryClient::no_vulnerabilities();
        let result = scan(
            config("tests/fixtures/scan/pnpm-only/pnpm-lock.yaml"),
            &client,
        )
        .await
        .expect("scan should succeed");
        let queried = client.queried_dependencies();

        assert_eq!(result.dependencies.len(), 1);
        assert_eq!(queried.len(), 1);
        assert_eq!(queried[0].ecosystem, Ecosystem::Npm);
        assert_eq!(queried[0].name, "left-pad");
        assert_eq!(queried[0].version, "1.3.0");
    }

    #[tokio::test]
    async fn pnpm_scoped_package_name_is_preserved_for_osv_query() {
        let client = RecordingAdvisoryClient::no_vulnerabilities();
        scan(
            config("tests/fixtures/scan/pnpm-scoped/pnpm-lock.yaml"),
            &client,
        )
        .await
        .expect("scan should succeed");
        let queried = client.queried_dependencies();

        assert_eq!(queried.len(), 1);
        assert_eq!(queried[0].name, "@scope/pkg");
        assert_eq!(queried[0].version, "2.0.0");
        assert_eq!(queried[0].ecosystem, Ecosystem::Npm);
    }

    #[tokio::test]
    async fn pnpm_peer_suffix_key_queries_base_package_name_and_version() {
        let client = RecordingAdvisoryClient::no_vulnerabilities();
        scan(
            config("tests/fixtures/scan/pnpm-peer-suffix/pnpm-lock.yaml"),
            &client,
        )
        .await
        .expect("scan should succeed");
        let queried = client.queried_dependencies();

        assert_eq!(queried.len(), 1);
        assert_eq!(queried[0].name, "react-dom");
        assert_eq!(queried[0].version, "18.2.0");
    }

    #[tokio::test]
    async fn pnpm_local_workspace_dependency_is_not_queried_against_osv() {
        let client = RecordingAdvisoryClient::no_vulnerabilities();
        let result = scan(
            config("tests/fixtures/scan/pnpm-local/pnpm-lock.yaml"),
            &client,
        )
        .await
        .expect("scan should succeed");
        let queried = client.queried_dependencies();

        assert_eq!(result.dependencies.len(), 1);
        assert_eq!(result.dependencies[0].name, "lodash");
        assert_eq!(queried.len(), 1);
        assert_eq!(queried[0].name, "lodash");
    }

    #[tokio::test]
    async fn pnpm_local_path_like_keys_with_at_are_not_queried_against_osv() {
        let client = RecordingAdvisoryClient::no_vulnerabilities();
        let result = scan(
            config("tests/fixtures/scan/pnpm-path-like/pnpm-lock.yaml"),
            &client,
        )
        .await
        .expect("scan should succeed");
        let queried = client.queried_dependencies();
        let queried_names: Vec<_> = queried
            .iter()
            .map(|dependency| dependency.name.as_str())
            .collect();

        assert_eq!(result.dependencies.len(), 2);
        assert_eq!(queried.len(), 2);
        assert_eq!(queried_names, vec!["@scope/pkg", "react-dom"]);
        assert!(queried
            .iter()
            .all(|dependency| dependency.ecosystem == Ecosystem::Npm));
    }

    #[tokio::test]
    async fn duplicate_pnpm_name_version_pairs_are_queried_once() {
        let client = RecordingAdvisoryClient::one_high_vulnerability();
        let result = scan(
            config("tests/fixtures/scan/pnpm-duplicate/pnpm-lock.yaml"),
            &client,
        )
        .await
        .expect("scan should succeed");
        let queried = client.queried_dependencies();

        assert_eq!(result.dependencies.len(), 2);
        assert_eq!(queried.len(), 1);
        assert_eq!(queried[0].name, "lodash");
        assert_eq!(queried[0].version, "4.17.20");
        assert_eq!(result.findings.len(), 1);
    }

    #[tokio::test]
    async fn duplicate_pnpm_name_version_pairs_across_importers_are_queried_once() {
        let client = RecordingAdvisoryClient::no_vulnerabilities();
        let result = scan(
            config("tests/fixtures/scan/pnpm-duplicate-importers/pnpm-lock.yaml"),
            &client,
        )
        .await
        .expect("scan should succeed");
        let queried = client.queried_dependencies();

        assert_eq!(result.dependencies.len(), 2);
        assert_eq!(queried.len(), 1);
        assert_eq!(queried[0].name, "lodash");
        assert_eq!(queried[0].version, "4.17.20");
        assert_eq!(queried[0].ecosystem, Ecosystem::Npm);
    }

    #[test]
    fn duplicate_advisory_dependency_prefers_production_metadata() {
        let dependencies = vec![
            dependency("shared-pkg", "1.0.0", true, true, "b/pnpm-lock.yaml"),
            dependency("shared-pkg", "1.0.0", true, false, "a/pnpm-lock.yaml"),
        ];

        let queried = dependencies_for_advisory_query(&dependencies);

        assert_eq!(queried.len(), 1);
        assert!(!queried[0].dev);
        assert_eq!(
            queried[0].source_file,
            std::path::PathBuf::from("a/pnpm-lock.yaml")
        );
    }

    #[test]
    fn duplicate_advisory_dependency_prefers_direct_metadata() {
        let dependencies = vec![
            dependency("shared-pkg", "1.0.0", false, false, "pnpm-lock.yaml"),
            dependency("shared-pkg", "1.0.0", true, false, "pnpm-lock.yaml"),
        ];

        let queried = dependencies_for_advisory_query(&dependencies);

        assert_eq!(queried.len(), 1);
        assert!(queried[0].direct);
    }

    #[test]
    fn duplicate_advisory_dependencies_have_deterministic_output() {
        let dependencies = vec![
            dependency("zeta", "1.0.0", false, true, "pnpm-lock.yaml"),
            dependency("alpha", "2.0.0", false, true, "pnpm-lock.yaml"),
            dependency("alpha", "1.0.0", true, false, "pnpm-lock.yaml"),
            dependency("alpha", "1.0.0", false, true, "pnpm-lock.yaml"),
        ];

        let queried = dependencies_for_advisory_query(&dependencies);
        let identities: Vec<_> = queried
            .iter()
            .map(|dependency| {
                (
                    dependency.name.as_str(),
                    dependency.version.as_str(),
                    dependency.direct,
                    dependency.dev,
                )
            })
            .collect();

        assert_eq!(
            identities,
            vec![
                ("alpha", "1.0.0", true, false),
                ("alpha", "2.0.0", false, true),
                ("zeta", "1.0.0", false, true),
            ]
        );
    }

    #[test]
    fn advisory_deduplication_keeps_distinct_npm_versions() {
        let dependencies = vec![
            dependency("left-pad", "1.3.0", true, false, "package-lock.json"),
            dependency("left-pad", "1.2.0", false, false, "package-lock.json"),
            dependency("left-pad", "1.3.0", false, true, "package-lock.json"),
        ];

        let queried = dependencies_for_advisory_query(&dependencies);

        assert_eq!(queried.len(), 2);
        assert_eq!(queried[0].name, "left-pad");
        assert_eq!(queried[0].version, "1.2.0");
        assert_eq!(queried[1].name, "left-pad");
        assert_eq!(queried[1].version, "1.3.0");
        assert!(queried[1].direct);
        assert!(!queried[1].dev);
    }

    fn config(lockfile: &str) -> ScanConfig {
        ScanConfig {
            lockfile: Some(lockfile.into()),
            format: ReportFormat::Table,
            fail_on: None,
            include_dev: true,
        }
    }

    fn dependency(
        name: &str,
        version: &str,
        direct: bool,
        dev: bool,
        source_file: &str,
    ) -> Dependency {
        Dependency {
            name: name.to_string(),
            version: version.to_string(),
            ecosystem: Ecosystem::Npm,
            package_url: Some(format!("pkg:npm/{name}@{version}")),
            direct,
            dev,
            source_file: source_file.into(),
        }
    }

    struct MockAdvisoryClient {
        response: MockResponse,
    }

    #[derive(Clone, Copy)]
    enum MockResponse {
        NoVulnerabilities,
        OneHighVulnerability,
        Error,
    }

    impl MockAdvisoryClient {
        fn no_vulnerabilities() -> Self {
            Self {
                response: MockResponse::NoVulnerabilities,
            }
        }

        fn one_high_vulnerability() -> Self {
            Self {
                response: MockResponse::OneHighVulnerability,
            }
        }

        fn error() -> Self {
            Self {
                response: MockResponse::Error,
            }
        }
    }

    impl AdvisoryClient for MockAdvisoryClient {
        fn query_batch<'a>(
            &'a self,
            dependencies: &'a [crate::types::Dependency],
        ) -> Pin<Box<dyn Future<Output = Result<Vec<OsvQueryResult>, SecFinderError>> + Send + 'a>>
        {
            Box::pin(async move {
                match self.response {
                    MockResponse::NoVulnerabilities => Ok(dependencies
                        .iter()
                        .cloned()
                        .map(|dependency| OsvQueryResult {
                            dependency,
                            vulnerabilities: Vec::new(),
                        })
                        .collect()),
                    MockResponse::OneHighVulnerability => Ok(dependencies
                        .iter()
                        .cloned()
                        .map(|dependency| OsvQueryResult {
                            dependency,
                            vulnerabilities: vec![OsvVulnerability {
                                id: "GHSA-high".to_string(),
                                summary: Some("high vulnerability".to_string()),
                                details: None,
                                aliases: Vec::new(),
                                severity: vec![OsvSeverity {
                                    severity_type: "GHSA".to_string(),
                                    score: "HIGH".to_string(),
                                }],
                                affected: Vec::new(),
                                references: Vec::new(),
                                modified: None,
                                published: None,
                            }],
                        })
                        .collect()),
                    MockResponse::Error => Err(SecFinderError::OsvStatus {
                        status: StatusCode::BAD_GATEWAY,
                        body: "mock OSV failure".to_string(),
                    }),
                }
            })
        }
    }

    #[derive(Clone)]
    struct RecordingAdvisoryClient {
        response: MockResponse,
        queried: Arc<Mutex<Vec<Dependency>>>,
    }

    impl RecordingAdvisoryClient {
        fn no_vulnerabilities() -> Self {
            Self {
                response: MockResponse::NoVulnerabilities,
                queried: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn one_high_vulnerability() -> Self {
            Self {
                response: MockResponse::OneHighVulnerability,
                queried: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn queried_dependencies(&self) -> Vec<Dependency> {
            self.queried.lock().unwrap().clone()
        }
    }

    impl AdvisoryClient for RecordingAdvisoryClient {
        fn query_batch<'a>(
            &'a self,
            dependencies: &'a [Dependency],
        ) -> Pin<Box<dyn Future<Output = Result<Vec<OsvQueryResult>, SecFinderError>> + Send + 'a>>
        {
            Box::pin(async move {
                self.queried.lock().unwrap().extend_from_slice(dependencies);
                match self.response {
                    MockResponse::NoVulnerabilities => Ok(dependencies
                        .iter()
                        .cloned()
                        .map(|dependency| OsvQueryResult {
                            dependency,
                            vulnerabilities: Vec::new(),
                        })
                        .collect()),
                    MockResponse::OneHighVulnerability => Ok(dependencies
                        .iter()
                        .cloned()
                        .map(|dependency| OsvQueryResult {
                            dependency,
                            vulnerabilities: vec![OsvVulnerability {
                                id: "GHSA-high".to_string(),
                                summary: Some("high vulnerability".to_string()),
                                details: None,
                                aliases: Vec::new(),
                                severity: vec![OsvSeverity {
                                    severity_type: "GHSA".to_string(),
                                    score: "HIGH".to_string(),
                                }],
                                affected: Vec::new(),
                                references: Vec::new(),
                                modified: None,
                                published: None,
                            }],
                        })
                        .collect()),
                    MockResponse::Error => Err(SecFinderError::OsvStatus {
                        status: StatusCode::BAD_GATEWAY,
                        body: "mock OSV failure".to_string(),
                    }),
                }
            })
        }
    }
}
