use comfy_table::{presets::UTF8_FULL, Table};

use crate::reporters::{dependency_scope, dependency_type, severity_label, sorted_findings};
use crate::types::{Dependency, ScanResult};

pub fn render(result: &ScanResult) -> String {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header([
        "Severity",
        "Package",
        "Installed",
        "Advisory",
        "Summary",
        "Dependency",
        "Scope",
        "Source",
        "Fixed Versions",
    ]);

    for finding in sorted_findings(&result.findings) {
        table.add_row([
            severity_label(finding.advisory.severity).to_string(),
            finding.dependency.name.clone(),
            finding.dependency.version.clone(),
            finding.advisory.id.clone(),
            finding.advisory.summary.clone(),
            dependency_type(finding.dependency.direct).to_string(),
            dependency_scope(finding.dependency.dev).to_string(),
            source_label(&finding.dependency),
            fixed_versions(&finding.advisory.fixed_versions),
        ]);
    }

    table.to_string()
}

fn source_label(dependency: &Dependency) -> String {
    dependency
        .source_file
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .map(str::to_string)
        .unwrap_or_else(|| dependency.source_file.display().to_string())
}

fn fixed_versions(versions: &[String]) -> String {
    if versions.is_empty() {
        "unknown".to_string()
    } else {
        versions.join(", ")
    }
}

#[cfg(test)]
mod tests {
    use crate::reporters::test_support::{finding, scan_result};
    use crate::types::Severity;

    use super::render;

    #[test]
    fn empty_findings_render_header() {
        let output = render(&scan_result(Vec::new()));

        assert!(output.contains("Severity"));
        assert!(output.contains("Package"));
        assert!(output.contains("Source"));
        assert!(output.contains("Fixed Versions"));
    }

    #[test]
    fn one_finding_contains_expected_fields() {
        let mut finding = finding(
            "lodash",
            "4.17.20",
            "GHSA-1234",
            Severity::High,
            true,
            false,
        );
        finding.advisory.summary = "Prototype pollution".to_string();
        finding.advisory.fixed_versions = vec!["4.17.21".to_string()];

        let output = render(&scan_result(vec![finding]));

        assert!(output.contains("high"));
        assert!(output.contains("lodash"));
        assert!(output.contains("4.17.20"));
        assert!(output.contains("GHSA-1234"));
        assert!(output.contains("Prototype pollution"));
        assert!(output.contains("direct"));
        assert!(output.contains("prod"));
        assert!(output.contains("package-lock.json"));
        assert!(output.contains("4.17.21"));
    }

    #[test]
    fn pnpm_finding_contains_pnpm_lock_source() {
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
        ]));

        assert!(output.contains("pnpm-lock.yaml"));
    }

    #[test]
    fn unknown_severity_and_missing_fixed_version_are_explicit() {
        let output = render(&scan_result(vec![finding(
            "mystery",
            "1.0.0",
            "OSV-1",
            Severity::Unknown,
            false,
            true,
        )]));

        assert!(output.contains("unknown"));
        assert!(output.contains("transitive"));
        assert!(output.contains("dev"));
    }
}
