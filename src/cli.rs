use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

use crate::types::{ReportFormat, ScanConfig, Severity};

#[derive(Debug, Parser)]
#[command(name = "sec-issue-finder")]
#[command(about = "Scan dependency lockfiles for known security advisories.")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Scan a dependency lockfile.
    Scan(ScanArgs),
}

#[derive(Debug, Args)]
pub struct ScanArgs {
    /// Path to a lockfile. Defaults to package-lock.json in the current directory.
    #[arg(long)]
    pub lockfile: Option<PathBuf>,

    /// Output format.
    #[arg(long, value_enum, default_value_t = CliReportFormat::Table)]
    pub format: CliReportFormat,

    /// Exit with code 1 when findings are at or above this severity.
    #[arg(long, value_enum)]
    pub fail_on: Option<CliSeverity>,

    /// Include development dependencies.
    #[arg(long, conflicts_with = "no_dev")]
    pub include_dev: bool,

    /// Exclude development dependencies.
    #[arg(long)]
    pub no_dev: bool,
}

impl ScanArgs {
    pub fn into_scan_config(self) -> ScanConfig {
        ScanConfig {
            lockfile: self.lockfile,
            format: self.format.into(),
            fail_on: self.fail_on.map(Into::into),
            include_dev: !self.no_dev,
        }
    }
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum CliReportFormat {
    Table,
    Json,
}

impl From<CliReportFormat> for ReportFormat {
    fn from(value: CliReportFormat) -> Self {
        match value {
            CliReportFormat::Table => Self::Table,
            CliReportFormat::Json => Self::Json,
        }
    }
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum CliSeverity {
    Low,
    Moderate,
    Medium,
    High,
    Critical,
}

impl From<CliSeverity> for Severity {
    fn from(value: CliSeverity) -> Self {
        match value {
            CliSeverity::Low => Self::Low,
            CliSeverity::Moderate => Self::Moderate,
            CliSeverity::Medium => Self::Medium,
            CliSeverity::High => Self::High,
            CliSeverity::Critical => Self::Critical,
        }
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;

    use super::{Cli, Command};
    use crate::types::{ReportFormat, Severity};

    #[test]
    fn parses_default_scan_args() {
        let cli = Cli::parse_from(["sec-issue-finder", "scan"]);
        let Command::Scan(args) = cli.command;
        let config = args.into_scan_config();

        assert!(config.lockfile.is_none());
        assert_eq!(config.format, ReportFormat::Table);
        assert!(config.include_dev);
        assert_eq!(config.fail_on, None);
    }

    #[test]
    fn parses_scan_options() {
        let cli = Cli::parse_from([
            "sec-issue-finder",
            "scan",
            "--lockfile",
            "package-lock.json",
            "--format",
            "json",
            "--fail-on",
            "high",
            "--no-dev",
        ]);
        let Command::Scan(args) = cli.command;
        let config = args.into_scan_config();

        assert_eq!(
            config.lockfile.unwrap().to_string_lossy(),
            "package-lock.json"
        );
        assert_eq!(config.format, ReportFormat::Json);
        assert!(!config.include_dev);
        assert_eq!(config.fail_on, Some(Severity::High));
    }
}
