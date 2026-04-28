pub mod json;
pub mod table;

use std::cmp::Ordering;

use crate::error::SecFinderError;
use crate::types::{Finding, ReportFormat, ScanResult, Severity};

pub fn print_with_format(result: &ScanResult, format: ReportFormat) -> Result<(), SecFinderError> {
    let output = render_with_format(result, format)?;
    println!("{output}");
    Ok(())
}

pub fn render_with_format(
    result: &ScanResult,
    format: ReportFormat,
) -> Result<String, SecFinderError> {
    match format {
        ReportFormat::Table => Ok(table::render(result)),
        ReportFormat::Json => json::render(result),
    }
}

pub(crate) fn sorted_findings(findings: &[Finding]) -> Vec<&Finding> {
    let mut sorted: Vec<_> = findings.iter().collect();
    sorted.sort_by(compare_findings);
    sorted
}

fn compare_findings(left: &&Finding, right: &&Finding) -> Ordering {
    severity_sort_key(left.advisory.severity)
        .cmp(&severity_sort_key(right.advisory.severity))
        .then_with(|| {
            left.dependency
                .direct
                .cmp(&right.dependency.direct)
                .reverse()
        })
        .then_with(|| left.dependency.dev.cmp(&right.dependency.dev))
        .then_with(|| left.dependency.name.cmp(&right.dependency.name))
        .then_with(|| left.advisory.id.cmp(&right.advisory.id))
}

fn severity_sort_key(severity: Severity) -> u8 {
    match severity {
        Severity::Critical => 0,
        Severity::High => 1,
        Severity::Moderate | Severity::Medium => 2,
        Severity::Low => 3,
        Severity::Unknown => 4,
    }
}

pub(crate) fn severity_label(severity: Severity) -> &'static str {
    match severity {
        Severity::Critical => "critical",
        Severity::High => "high",
        Severity::Moderate => "moderate",
        Severity::Medium => "medium",
        Severity::Low => "low",
        Severity::Unknown => "unknown",
    }
}

pub(crate) fn dependency_type(direct: bool) -> &'static str {
    if direct {
        "direct"
    } else {
        "transitive"
    }
}

pub(crate) fn dependency_scope(dev: bool) -> &'static str {
    if dev {
        "dev"
    } else {
        "prod"
    }
}

#[cfg(test)]
pub(crate) mod test_support {
    use std::path::PathBuf;

    use crate::types::{
        Advisory, AdvisorySource, Dependency, Ecosystem, Finding, ScanResult, Severity,
    };

    pub(crate) fn scan_result(findings: Vec<Finding>) -> ScanResult {
        ScanResult {
            lockfile: PathBuf::from("package-lock.json"),
            ecosystem: Ecosystem::Npm,
            dependencies: Vec::new(),
            findings,
        }
    }

    pub(crate) fn finding(
        package: &str,
        version: &str,
        advisory_id: &str,
        severity: Severity,
        direct: bool,
        dev: bool,
    ) -> Finding {
        finding_with_source(
            package,
            version,
            advisory_id,
            severity,
            direct,
            dev,
            "package-lock.json",
        )
    }

    pub(crate) fn finding_with_source(
        package: &str,
        version: &str,
        advisory_id: &str,
        severity: Severity,
        direct: bool,
        dev: bool,
        source_file: &str,
    ) -> Finding {
        Finding {
            dependency: Dependency {
                name: package.to_string(),
                version: version.to_string(),
                ecosystem: Ecosystem::Npm,
                package_url: Some(format!("pkg:npm/{package}@{version}")),
                direct,
                dev,
                source_file: PathBuf::from(source_file),
            },
            advisory: Advisory {
                id: advisory_id.to_string(),
                aliases: Vec::new(),
                summary: format!("{package} vulnerability"),
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
