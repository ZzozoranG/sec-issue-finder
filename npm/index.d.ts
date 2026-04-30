export interface ScanOptions {
  lockfile?: string;
  noDev?: boolean;
  failOn?: "low" | "moderate" | "medium" | "high" | "critical";
  cwd?: string;
  /**
   * Explicit Rust CLI binary path. This has priority over
   * SEC_ISSUE_FINDER_BINARY_PATH, optional platform packages, local build
   * fallbacks, and PATH lookup.
   */
  binaryPath?: string;
}

export interface ScanSummary {
  total: number;
  critical: number;
  high: number;
  moderate: number;
  medium: number;
  low: number;
  unknown: number;
  direct: number;
  transitive: number;
  prod: number;
  dev: number;
}

export interface ScanFinding {
  severity: string;
  package: {
    name: string;
    installed_version: string;
    ecosystem: string;
    package_url: string;
    source_file?: string;
  };
  advisory: {
    id: string;
    aliases: string[];
    source: string;
    summary: string;
    details: string | null;
    url: string | null;
  };
  dependency_type: "direct" | "transitive";
  scope: "prod" | "dev";
  fixed_versions: string[];
  references: string[];
}

export interface ScanResult {
  schema_version: string;
  generated: {
    tool: string;
    format: string;
  };
  summary: ScanSummary;
  findings: ScanFinding[];
}

export class ScifScanError extends Error {
  exitCode: number | null;
  stdout: string;
  stderr: string;
}

export function scan(options?: ScanOptions): Promise<ScanResult>;
