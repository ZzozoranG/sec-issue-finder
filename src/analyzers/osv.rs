use crate::clients::osv::{OsvQueryResult, OsvVulnerability};
use crate::types::{Advisory, AdvisorySource, Finding, Severity};

pub fn findings_from_osv_results(results: &[OsvQueryResult]) -> Vec<Finding> {
    results
        .iter()
        .flat_map(|result| {
            result.vulnerabilities.iter().map(|vulnerability| Finding {
                dependency: result.dependency.clone(),
                advisory: advisory_from_vulnerability(vulnerability),
            })
        })
        .collect()
}

fn advisory_from_vulnerability(vulnerability: &OsvVulnerability) -> Advisory {
    let references = references(vulnerability);

    Advisory {
        id: vulnerability.id.clone(),
        aliases: vulnerability.aliases.clone(),
        summary: vulnerability
            .summary
            .clone()
            .unwrap_or_else(|| vulnerability.id.clone()),
        details: vulnerability.details.clone(),
        severity: severity(vulnerability),
        source: AdvisorySource::Osv,
        fixed_versions: fixed_versions(vulnerability),
        url: references.first().cloned(),
        references,
    }
}

fn severity(vulnerability: &OsvVulnerability) -> Severity {
    vulnerability
        .severity
        .iter()
        .find_map(|severity| severity_from_score(&severity.score))
        .unwrap_or(Severity::Unknown)
}

fn severity_from_score(score: &str) -> Option<Severity> {
    match score.to_ascii_lowercase().as_str() {
        "low" => Some(Severity::Low),
        "moderate" => Some(Severity::Moderate),
        "medium" => Some(Severity::Medium),
        "high" => Some(Severity::High),
        "critical" => Some(Severity::Critical),
        _ => score.parse::<f32>().ok().map(severity_from_cvss_score),
    }
}

fn severity_from_cvss_score(score: f32) -> Severity {
    if score >= 9.0 {
        Severity::Critical
    } else if score >= 7.0 {
        Severity::High
    } else if score >= 4.0 {
        Severity::Medium
    } else if score > 0.0 {
        Severity::Low
    } else {
        Severity::Unknown
    }
}

fn fixed_versions(vulnerability: &OsvVulnerability) -> Vec<String> {
    let mut versions = Vec::new();

    for affected in &vulnerability.affected {
        for range in &affected.ranges {
            for event in &range.events {
                if let Some(fixed) = &event.fixed {
                    push_unique(&mut versions, fixed.clone());
                }
            }
        }
    }

    versions
}

fn references(vulnerability: &OsvVulnerability) -> Vec<String> {
    let mut references = Vec::new();

    for reference in &vulnerability.references {
        push_unique(&mut references, reference.url.clone());
    }

    references
}

fn push_unique(values: &mut Vec<String>, value: String) {
    if !values.contains(&value) {
        values.push(value);
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::clients::osv::{
        OsvAffected, OsvQueryResult, OsvRange, OsvRangeEvent, OsvReference, OsvSeverity,
        OsvVulnerability,
    };
    use crate::types::{Dependency, Ecosystem, Severity};

    use super::findings_from_osv_results;

    #[test]
    fn normalizes_one_ghsa() {
        let results = vec![query_result(
            dependency("lodash", "4.17.20", true, false),
            vec![vulnerability("GHSA-1234-5678-90ab")
                .summary("Prototype pollution")
                .details("Details about the advisory")
                .severity("GHSA", "HIGH")
                .reference(
                    "ADVISORY",
                    "https://github.com/advisories/GHSA-1234-5678-90ab",
                )
                .build()],
        )];

        let findings = findings_from_osv_results(&results);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].dependency.name, "lodash");
        assert_eq!(findings[0].dependency.version, "4.17.20");
        assert_eq!(findings[0].dependency.ecosystem, Ecosystem::Npm);
        assert!(findings[0].dependency.direct);
        assert!(!findings[0].dependency.dev);
        assert_eq!(findings[0].advisory.id, "GHSA-1234-5678-90ab");
        assert_eq!(findings[0].advisory.severity, Severity::High);
        assert_eq!(findings[0].advisory.summary, "Prototype pollution");
        assert_eq!(
            findings[0].advisory.details.as_deref(),
            Some("Details about the advisory")
        );
        assert_eq!(
            findings[0].advisory.references,
            vec!["https://github.com/advisories/GHSA-1234-5678-90ab"]
        );
    }

    #[test]
    fn normalizes_cve_alias() {
        let results = vec![query_result(
            dependency("minimist", "1.2.5", false, false),
            vec![vulnerability("GHSA-xvch-5gv4-984h")
                .alias("CVE-2021-44906")
                .build()],
        )];

        let findings = findings_from_osv_results(&results);

        assert_eq!(findings[0].advisory.aliases, vec!["CVE-2021-44906"]);
        assert!(!findings[0].dependency.direct);
    }

    #[test]
    fn missing_severity_becomes_unknown() {
        let results = vec![query_result(
            dependency("package", "1.0.0", true, true),
            vec![vulnerability("OSV-UNKNOWN").build()],
        )];

        let findings = findings_from_osv_results(&results);

        assert_eq!(findings[0].advisory.severity, Severity::Unknown);
        assert!(findings[0].dependency.dev);
    }

    #[test]
    fn normalizes_fixed_version() {
        let results = vec![query_result(
            dependency("package", "1.0.0", true, false),
            vec![vulnerability("GHSA-fixed").fixed("1.0.1").build()],
        )];

        let findings = findings_from_osv_results(&results);

        assert_eq!(findings[0].advisory.fixed_versions, vec!["1.0.1"]);
    }

    #[test]
    fn missing_fixed_version_is_empty() {
        let results = vec![query_result(
            dependency("package", "1.0.0", true, false),
            vec![vulnerability("GHSA-no-fixed").build()],
        )];

        let findings = findings_from_osv_results(&results);

        assert!(findings[0].advisory.fixed_versions.is_empty());
    }

    #[test]
    fn multiple_vulnerabilities_for_one_dependency_create_multiple_findings() {
        let results = vec![query_result(
            dependency("package", "1.0.0", true, false),
            vec![
                vulnerability("GHSA-one").severity("CVSS_V3", "9.8").build(),
                vulnerability("GHSA-two").severity("CVSS_V3", "5.5").build(),
            ],
        )];

        let findings = findings_from_osv_results(&results);

        assert_eq!(findings.len(), 2);
        assert_eq!(findings[0].advisory.id, "GHSA-one");
        assert_eq!(findings[0].advisory.severity, Severity::Critical);
        assert_eq!(findings[1].advisory.id, "GHSA-two");
        assert_eq!(findings[1].advisory.severity, Severity::Medium);
    }

    #[test]
    fn no_vulnerabilities_for_dependency_creates_no_findings() {
        let results = vec![query_result(
            dependency("package", "1.0.0", true, false),
            Vec::new(),
        )];

        let findings = findings_from_osv_results(&results);

        assert!(findings.is_empty());
    }

    fn dependency(name: &str, version: &str, direct: bool, dev: bool) -> Dependency {
        Dependency {
            name: name.to_string(),
            version: version.to_string(),
            ecosystem: Ecosystem::Npm,
            package_url: Some(format!("pkg:npm/{name}@{version}")),
            direct,
            dev,
            source_file: PathBuf::from("package-lock.json"),
        }
    }

    fn query_result(
        dependency: Dependency,
        vulnerabilities: Vec<OsvVulnerability>,
    ) -> OsvQueryResult {
        OsvQueryResult {
            dependency,
            vulnerabilities,
        }
    }

    fn vulnerability(id: &str) -> VulnerabilityBuilder {
        VulnerabilityBuilder {
            vulnerability: OsvVulnerability {
                id: id.to_string(),
                summary: None,
                details: None,
                aliases: Vec::new(),
                severity: Vec::new(),
                affected: Vec::new(),
                references: Vec::new(),
                modified: None,
                published: None,
            },
        }
    }

    struct VulnerabilityBuilder {
        vulnerability: OsvVulnerability,
    }

    impl VulnerabilityBuilder {
        fn summary(mut self, summary: &str) -> Self {
            self.vulnerability.summary = Some(summary.to_string());
            self
        }

        fn details(mut self, details: &str) -> Self {
            self.vulnerability.details = Some(details.to_string());
            self
        }

        fn alias(mut self, alias: &str) -> Self {
            self.vulnerability.aliases.push(alias.to_string());
            self
        }

        fn severity(mut self, severity_type: &str, score: &str) -> Self {
            self.vulnerability.severity.push(OsvSeverity {
                severity_type: severity_type.to_string(),
                score: score.to_string(),
            });
            self
        }

        fn fixed(mut self, version: &str) -> Self {
            self.vulnerability.affected.push(OsvAffected {
                ranges: vec![OsvRange {
                    events: vec![OsvRangeEvent {
                        fixed: Some(version.to_string()),
                    }],
                }],
            });
            self
        }

        fn reference(mut self, reference_type: &str, url: &str) -> Self {
            self.vulnerability.references.push(OsvReference {
                reference_type: reference_type.to_string(),
                url: url.to_string(),
            });
            self
        }
    }

    impl VulnerabilityBuilder {
        fn build(self) -> OsvVulnerability {
            self.vulnerability
        }
    }
}
