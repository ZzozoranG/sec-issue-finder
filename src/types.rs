use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub version: String,
    pub ecosystem: Ecosystem,
    pub package_url: Option<String>,
    pub direct: bool,
    pub dev: bool,
    pub source_file: PathBuf,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Ecosystem {
    Npm,
    Dart,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Low,
    Moderate,
    Medium,
    High,
    Critical,
    Unknown,
}

impl Severity {
    pub fn rank(self) -> u8 {
        match self {
            Self::Unknown => 0,
            Self::Low => 1,
            Self::Moderate | Self::Medium => 2,
            Self::High => 3,
            Self::Critical => 4,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Advisory {
    pub id: String,
    pub aliases: Vec<String>,
    pub summary: String,
    pub details: Option<String>,
    pub severity: Severity,
    pub source: AdvisorySource,
    pub fixed_versions: Vec<String>,
    pub references: Vec<String>,
    pub url: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AdvisorySource {
    Osv,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Finding {
    pub dependency: Dependency,
    pub advisory: Advisory,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReportFormat {
    Table,
    Json,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScanConfig {
    pub lockfile: Option<PathBuf>,
    pub format: ReportFormat,
    pub fail_on: Option<Severity>,
    pub include_dev: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ScanResult {
    pub lockfile: PathBuf,
    pub ecosystem: Ecosystem,
    pub dependencies: Vec<Dependency>,
    pub findings: Vec<Finding>,
}
