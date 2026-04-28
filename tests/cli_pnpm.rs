use std::path::{Path, PathBuf};
use std::process::{Command, Output};

#[cfg(feature = "test-utils")]
const OSV_MOCK_RESPONSE_FILE_ENV: &str = "SEC_ISSUE_FINDER_OSV_MOCK_RESPONSE_FILE";

#[cfg(feature = "test-utils")]
#[test]
fn explicit_pnpm_lockfile_scan_succeeds() {
    let output = command()
        .args([
            "scan",
            "--lockfile",
            fixture("pnpm/simple_prod_dependency/pnpm-lock.yaml")
                .to_str()
                .unwrap(),
        ])
        .env(
            OSV_MOCK_RESPONSE_FILE_ENV,
            fixture("osv/no-vulnerabilities-one.json"),
        )
        .output()
        .unwrap();

    assert_success(&output);
    let stdout = stdout(&output);
    assert!(stdout.contains("Severity"));
    assert!(stdout.contains("Package"));
}

#[cfg(feature = "test-utils")]
#[test]
fn pnpm_json_output_succeeds() {
    let output = command()
        .args([
            "scan",
            "--lockfile",
            fixture("pnpm/simple_prod_dependency/pnpm-lock.yaml")
                .to_str()
                .unwrap(),
            "--format",
            "json",
        ])
        .env(
            OSV_MOCK_RESPONSE_FILE_ENV,
            fixture("osv/no-vulnerabilities-one.json"),
        )
        .output()
        .unwrap();

    assert_success(&output);
    let stdout = stdout(&output);
    assert!(stdout.trim_start().starts_with('{'));
    assert!(stdout.contains(r#""schema_version": "1.0""#));
    assert!(stdout.contains(r#""findings": []"#));
}

#[test]
fn pnpm_dev_dependency_filtering_succeeds_without_osv_call() {
    let output = command()
        .args([
            "scan",
            "--lockfile",
            fixture("pnpm/simple_dev_dependency/pnpm-lock.yaml")
                .to_str()
                .unwrap(),
            "--no-dev",
        ])
        .output()
        .unwrap();

    assert_success(&output);
    let stdout = stdout(&output);
    assert!(stdout.contains("Severity"));
    assert!(!stdout.contains("eslint"));
}

#[cfg(feature = "test-utils")]
#[test]
fn pnpm_no_dev_still_queries_osv_when_production_dependencies_remain() {
    let output = command()
        .args([
            "scan",
            "--lockfile",
            fixture("pnpm/prod_and_dev_cli/pnpm-lock.yaml")
                .to_str()
                .unwrap(),
            "--no-dev",
        ])
        .env(
            OSV_MOCK_RESPONSE_FILE_ENV,
            fixture("osv/high-vulnerability-one.json"),
        )
        .output()
        .unwrap();

    assert_success(&output);
    let stdout = stdout(&output);
    assert!(stdout.contains("Severity"));
    assert!(stdout.contains("GHSA-high"));
    assert!(!stdout.contains("eslint"));
}

#[cfg(feature = "test-utils")]
#[test]
fn pnpm_json_finding_includes_pnpm_lock_source_file() {
    let output = command()
        .args([
            "scan",
            "--lockfile",
            fixture("pnpm/vulnerable_dependency/pnpm-lock.yaml")
                .to_str()
                .unwrap(),
            "--format",
            "json",
        ])
        .env(
            OSV_MOCK_RESPONSE_FILE_ENV,
            fixture("osv/high-vulnerability-one.json"),
        )
        .output()
        .unwrap();

    assert_success(&output);
    let stdout = stdout(&output);
    assert!(stdout.trim_start().starts_with('{'));
    assert!(stdout.contains(r#""source_file":"#));
    assert!(stdout.contains("pnpm-lock.yaml"));
    assert!(stdout.contains("GHSA-high"));
}

#[cfg(feature = "test-utils")]
#[test]
fn pnpm_fail_on_high_exits_with_failure() {
    let output = command()
        .args([
            "scan",
            "--lockfile",
            fixture("pnpm/vulnerable_dependency/pnpm-lock.yaml")
                .to_str()
                .unwrap(),
            "--fail-on",
            "high",
        ])
        .env(
            OSV_MOCK_RESPONSE_FILE_ENV,
            fixture("osv/high-vulnerability-one.json"),
        )
        .output()
        .unwrap();

    assert_failure(&output);
    let stdout = stdout(&output);
    let stderr = stderr(&output);
    assert!(stdout.contains("GHSA-high"));
    assert!(stderr.contains("policy failed"));
}

#[test]
fn ambiguous_lockfiles_return_helpful_error() {
    let output = command()
        .arg("scan")
        .current_dir(fixture("pnpm/ambiguous_cli"))
        .output()
        .unwrap();

    assert_failure(&output);
    let stderr = stderr(&output);
    assert!(stderr.contains("multiple supported lockfiles found"));
    assert!(stderr.contains("--lockfile"));
}

#[test]
fn missing_pnpm_lockfile_returns_helpful_error() {
    let output = command()
        .args([
            "scan",
            "--lockfile",
            fixture("pnpm/does_not_exist/pnpm-lock.yaml")
                .to_str()
                .unwrap(),
        ])
        .output()
        .unwrap();

    assert_failure(&output);
    assert!(stderr(&output).contains("lockfile not found"));
}

#[test]
fn malformed_pnpm_lockfile_returns_helpful_error() {
    let output = command()
        .args([
            "scan",
            "--lockfile",
            fixture("pnpm/malformed_cli/pnpm-lock.yaml")
                .to_str()
                .unwrap(),
        ])
        .output()
        .unwrap();

    assert_failure(&output);
    assert!(stderr(&output).contains("failed to parse YAML lockfile"));
}

fn command() -> Command {
    Command::new(env!("CARGO_BIN_EXE_sec-issue-finder"))
}

fn fixture(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(relative)
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "expected success\nstdout:\n{}\nstderr:\n{}",
        stdout(output),
        stderr(output)
    );
}

fn assert_failure(output: &Output) {
    assert!(
        !output.status.success(),
        "expected failure\nstdout:\n{}\nstderr:\n{}",
        stdout(output),
        stderr(output)
    );
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}
