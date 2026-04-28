use crate::error::SecFinderError;
use crate::types::{Finding, ScanResult, Severity};

pub fn severity_rank(severity: &Severity) -> u8 {
    match severity {
        Severity::Unknown => 0,
        Severity::Low => 1,
        Severity::Moderate | Severity::Medium => 2,
        Severity::High => 3,
        Severity::Critical => 4,
    }
}

pub fn should_fail(findings: &[Finding], threshold: Option<Severity>) -> bool {
    let Some(threshold) = threshold else {
        return false;
    };

    findings.iter().any(|finding| {
        let severity = &finding.advisory.severity;
        *severity != Severity::Unknown && severity_rank(severity) >= severity_rank(&threshold)
    })
}

pub fn evaluate_with_threshold(
    result: &ScanResult,
    fail_on: Option<Severity>,
) -> Result<(), SecFinderError> {
    if should_fail(&result.findings, fail_on) {
        return Err(SecFinderError::PolicyFailed {
            message: match fail_on {
                Some(threshold) => format!("found vulnerabilities at or above {threshold:?}"),
                None => "found vulnerabilities that violate policy".to_string(),
            },
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::types::{
        Advisory, AdvisorySource, Dependency, Ecosystem, Finding, ScanResult, Severity,
    };

    use super::{evaluate_with_threshold, severity_rank, should_fail};

    #[test]
    fn no_findings_do_not_fail() {
        let result = result_with_findings(Vec::new());
        assert!(!should_fail(&result.findings, Some(Severity::Low)));
        assert!(evaluate_with_threshold(&result, Some(Severity::Low)).is_ok());
    }

    #[test]
    fn findings_without_fail_on_threshold_do_not_fail() {
        let result = result_with_finding(Severity::Critical);
        assert!(!should_fail(&result.findings, None));
        assert!(evaluate_with_threshold(&result, None).is_ok());
    }

    #[test]
    fn low_threshold_fails_on_low() {
        let result = result_with_finding(Severity::Low);
        assert!(should_fail(&result.findings, Some(Severity::Low)));
        assert!(evaluate_with_threshold(&result, Some(Severity::Low)).is_err());
    }

    #[test]
    fn high_threshold_ignores_moderate() {
        let result = result_with_finding(Severity::Moderate);
        assert!(!should_fail(&result.findings, Some(Severity::High)));
        assert!(evaluate_with_threshold(&result, Some(Severity::High)).is_ok());
    }

    #[test]
    fn high_threshold_fails_on_high() {
        let result = result_with_finding(Severity::High);
        assert!(should_fail(&result.findings, Some(Severity::High)));
        assert!(evaluate_with_threshold(&result, Some(Severity::High)).is_err());
    }

    #[test]
    fn critical_threshold_fails_only_on_critical() {
        let non_critical = result_with_findings(vec![
            finding(Severity::Low),
            finding(Severity::Moderate),
            finding(Severity::Medium),
            finding(Severity::High),
            finding(Severity::Unknown),
        ]);
        let critical = result_with_finding(Severity::Critical);

        assert!(!should_fail(
            &non_critical.findings,
            Some(Severity::Critical)
        ));
        assert!(should_fail(&critical.findings, Some(Severity::Critical)));
    }

    #[test]
    fn medium_and_moderate_are_equivalent() {
        let moderate = result_with_finding(Severity::Moderate);
        let medium = result_with_finding(Severity::Medium);

        assert_eq!(severity_rank(&Severity::Moderate), 2);
        assert_eq!(severity_rank(&Severity::Medium), 2);
        assert!(should_fail(&moderate.findings, Some(Severity::Medium)));
        assert!(should_fail(&medium.findings, Some(Severity::Moderate)));
    }

    #[test]
    fn unknown_severity_does_not_fail() {
        let result = result_with_finding(Severity::Unknown);

        assert!(!should_fail(&result.findings, Some(Severity::Low)));
        assert!(!should_fail(&result.findings, Some(Severity::Critical)));
        assert!(evaluate_with_threshold(&result, Some(Severity::Low)).is_ok());
    }

    fn result_with_finding(severity: Severity) -> ScanResult {
        result_with_findings(vec![finding(severity)])
    }

    fn result_with_findings(findings: Vec<Finding>) -> ScanResult {
        ScanResult {
            lockfile: PathBuf::from("package-lock.json"),
            ecosystem: Ecosystem::Npm,
            dependencies: Vec::new(),
            findings,
        }
    }

    fn finding(severity: Severity) -> Finding {
        Finding {
            dependency: Dependency {
                name: "example".to_string(),
                version: "1.0.0".to_string(),
                ecosystem: Ecosystem::Npm,
                package_url: Some("pkg:npm/example@1.0.0".to_string()),
                direct: true,
                dev: false,
                source_file: PathBuf::from("package-lock.json"),
            },
            advisory: Advisory {
                id: "OSV-000".to_string(),
                aliases: Vec::new(),
                summary: "example advisory".to_string(),
                details: None,
                severity,
                source: AdvisorySource::Osv,
                fixed_versions: Vec::new(),
                references: Vec::new(),
                url: None,
            },
        }
    }
}
