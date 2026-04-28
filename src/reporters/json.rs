use serde::Serialize;

use crate::error::SecFinderError;
use crate::reporters::{dependency_scope, dependency_type, severity_label, sorted_findings};
use crate::types::{AdvisorySource, Ecosystem, ScanResult, Severity};

const SCHEMA_VERSION: &str = "1.0";

pub fn render(result: &ScanResult) -> Result<String, SecFinderError> {
    let report = JsonReport::from_scan_result(result);
    serde_json::to_string_pretty(&report).map_err(SecFinderError::Reporting)
}

#[derive(Debug, Serialize)]
struct JsonReport {
    schema_version: &'static str,
    generated: GeneratedMetadata,
    summary: JsonSummary,
    findings: Vec<JsonFinding>,
}

impl JsonReport {
    fn from_scan_result(result: &ScanResult) -> Self {
        let sorted = sorted_findings(&result.findings);
        Self {
            schema_version: SCHEMA_VERSION,
            generated: GeneratedMetadata {
                tool: "sec-issue-finder",
                format: "json",
            },
            summary: JsonSummary::from_findings(&sorted),
            findings: sorted.into_iter().map(JsonFinding::from).collect(),
        }
    }
}

#[derive(Debug, Serialize)]
struct GeneratedMetadata {
    tool: &'static str,
    format: &'static str,
}

#[derive(Debug, Default, Serialize)]
struct JsonSummary {
    total: usize,
    critical: usize,
    high: usize,
    moderate: usize,
    medium: usize,
    low: usize,
    unknown: usize,
    direct: usize,
    transitive: usize,
    prod: usize,
    dev: usize,
}

impl JsonSummary {
    fn from_findings(findings: &[&crate::types::Finding]) -> Self {
        let mut summary = Self {
            total: findings.len(),
            ..Self::default()
        };

        for finding in findings {
            match finding.advisory.severity {
                Severity::Critical => summary.critical += 1,
                Severity::High => summary.high += 1,
                Severity::Moderate => summary.moderate += 1,
                Severity::Medium => summary.medium += 1,
                Severity::Low => summary.low += 1,
                Severity::Unknown => summary.unknown += 1,
            }

            if finding.dependency.direct {
                summary.direct += 1;
            } else {
                summary.transitive += 1;
            }

            if finding.dependency.dev {
                summary.dev += 1;
            } else {
                summary.prod += 1;
            }
        }

        summary
    }
}

#[derive(Debug, Serialize)]
struct JsonFinding {
    severity: &'static str,
    package: JsonPackage,
    advisory: JsonAdvisory,
    dependency_type: &'static str,
    scope: &'static str,
    fixed_versions: Vec<String>,
    references: Vec<String>,
}

impl From<&crate::types::Finding> for JsonFinding {
    fn from(finding: &crate::types::Finding) -> Self {
        Self {
            severity: severity_label(finding.advisory.severity),
            package: JsonPackage {
                name: finding.dependency.name.clone(),
                installed_version: finding.dependency.version.clone(),
                ecosystem: finding.dependency.ecosystem,
                package_url: finding.dependency.package_url.clone(),
                source_file: finding.dependency.source_file.display().to_string(),
            },
            advisory: JsonAdvisory {
                id: finding.advisory.id.clone(),
                aliases: finding.advisory.aliases.clone(),
                source: finding.advisory.source,
                summary: finding.advisory.summary.clone(),
                details: finding.advisory.details.clone(),
                url: finding.advisory.url.clone(),
            },
            dependency_type: dependency_type(finding.dependency.direct),
            scope: dependency_scope(finding.dependency.dev),
            fixed_versions: finding.advisory.fixed_versions.clone(),
            references: finding.advisory.references.clone(),
        }
    }
}

#[derive(Debug, Serialize)]
struct JsonPackage {
    name: String,
    installed_version: String,
    ecosystem: Ecosystem,
    package_url: Option<String>,
    source_file: String,
}

#[derive(Debug, Serialize)]
struct JsonAdvisory {
    id: String,
    aliases: Vec<String>,
    source: AdvisorySource,
    summary: String,
    details: Option<String>,
    url: Option<String>,
}

#[cfg(test)]
mod tests {
    use crate::reporters::test_support::{finding, scan_result};
    use crate::types::Severity;

    use super::render;

    #[test]
    fn empty_findings_have_stable_shape() {
        let output = render(&scan_result(Vec::new())).unwrap();

        assert_eq!(
            output,
            r#"{
  "schema_version": "1.0",
  "generated": {
    "tool": "sec-issue-finder",
    "format": "json"
  },
  "summary": {
    "total": 0,
    "critical": 0,
    "high": 0,
    "moderate": 0,
    "medium": 0,
    "low": 0,
    "unknown": 0,
    "direct": 0,
    "transitive": 0,
    "prod": 0,
    "dev": 0
  },
  "findings": []
}"#
        );
    }

    #[test]
    fn one_finding_contains_machine_readable_fields() {
        let mut finding = finding(
            "lodash",
            "4.17.20",
            "GHSA-1234",
            Severity::High,
            true,
            false,
        );
        finding.advisory.aliases = vec!["CVE-2021-0001".to_string()];
        finding.advisory.details = Some("Details".to_string());
        finding.advisory.fixed_versions = vec!["4.17.21".to_string()];
        finding.advisory.references = vec!["https://example.test/advisory".to_string()];

        let output = render(&scan_result(vec![finding])).unwrap();

        assert!(output.contains(r#""schema_version": "1.0""#));
        assert!(output.contains(r#""severity": "high""#));
        assert!(output.contains(r#""name": "lodash""#));
        assert!(output.contains(r#""installed_version": "4.17.20""#));
        assert!(output.contains(r#""source_file": "package-lock.json""#));
        assert!(output.contains(r#""id": "GHSA-1234""#));
        assert!(output.contains(r#""CVE-2021-0001""#));
        assert!(output.contains(r#""dependency_type": "direct""#));
        assert!(output.contains(r#""scope": "prod""#));
        assert!(output.contains(r#""4.17.21""#));
    }

    #[test]
    fn multiple_findings_are_sorted_deterministically() {
        let output = render(&scan_result(vec![
            finding("zeta", "1.0.0", "GHSA-low", Severity::Low, true, false),
            finding(
                "alpha",
                "1.0.0",
                "GHSA-critical",
                Severity::Critical,
                false,
                false,
            ),
            finding("beta", "1.0.0", "GHSA-high", Severity::High, false, true),
            finding("alpha", "1.0.0", "GHSA-high", Severity::High, true, false),
        ]))
        .unwrap();

        let critical = output.find("GHSA-critical").unwrap();
        let direct_high = output.find("GHSA-high").unwrap();
        let dev_high = output.rfind("GHSA-high").unwrap();
        let low = output.find("GHSA-low").unwrap();

        assert!(critical < direct_high);
        assert!(direct_high < dev_high);
        assert!(dev_high < low);
    }

    #[test]
    fn unknown_severity_is_counted_and_rendered() {
        let output = render(&scan_result(vec![finding(
            "mystery",
            "1.0.0",
            "OSV-1",
            Severity::Unknown,
            false,
            true,
        )]))
        .unwrap();

        assert!(output.contains(r#""unknown": 1"#));
        assert!(output.contains(r#""severity": "unknown""#));
        assert!(output.contains(r#""dependency_type": "transitive""#));
        assert!(output.contains(r#""scope": "dev""#));
    }

    #[test]
    fn json_output_contains_pnpm_source_file() {
        let output = render(&scan_result(vec![
            crate::reporters::test_support::finding_with_source(
                "left-pad",
                "1.3.0",
                "GHSA-pnpm",
                Severity::High,
                true,
                false,
                "fixtures/pnpm-lock.yaml",
            ),
        ]))
        .unwrap();

        assert!(output.contains(r#""source_file": "fixtures/pnpm-lock.yaml""#));
    }

    #[test]
    fn output_is_deterministic() {
        let result = scan_result(vec![
            finding("beta", "1.0.0", "GHSA-2", Severity::High, true, false),
            finding("alpha", "1.0.0", "GHSA-1", Severity::High, true, false),
        ]);

        assert_eq!(render(&result).unwrap(), render(&result).unwrap());
    }
}
