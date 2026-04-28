# sec-issue-finder

[한국어 문서](README.ko.md) | [Onboarding guide](docs/onboarding.md)

`sec-issue-finder` is a Rust CLI for scanning dependency lockfiles for known open source security advisories.

It currently reads npm `package-lock.json` files and best-effort registry npm dependencies from `pnpm-lock.yaml`, normalizes installed dependencies, queries OSV, and reports findings in table or JSON format. It is intended for local development checks and CI policy gates.

Results depend on public advisory databases and the package/version data present in the lockfile. This tool does not claim complete vulnerability coverage.

## Current Support

- Supported lockfiles:
  - npm `package-lock.json` v2/v3
  - pnpm `pnpm-lock.yaml` for registry npm dependencies
- OSV advisory lookup through `/v1/querybatch`
- Table output for humans
- JSON output for automation
- `--fail-on` CI policy thresholds: `low`, `moderate`, `medium`, `high`, `critical`
- Dev dependency inclusion by default, with `--no-dev` to exclude dev dependencies

## What It Does

- Detects and parses supported npm ecosystem lockfiles.
- Extracts installed package names, versions, direct/transitive status, and dev/prod scope.
- Queries OSV for known vulnerabilities affecting those package versions.
- Normalizes OSV responses into internal findings.
- Sorts output deterministically.
- Exits non-zero when `--fail-on` is set and matching findings are present.

## What It Does Not Do

- It does not install packages or run package manager commands.
- It does not execute dependency lifecycle scripts.
- It does not auto-fix vulnerable dependencies.
- It does not create pull requests.
- It does not perform malware, typosquatting, or install-script behavior analysis.
- It does not generate SBOMs yet.
- It does not maintain a local offline vulnerability database yet.
- It does not currently support Dart, Rust, Yarn, Bun, or Python lockfiles.

## Installation

Prerequisites:

- Rust 2021-compatible toolchain
- Cargo

### Install From Source

Install the Rust CLI from this repository:

```bash
cargo install --path .
```

Or run without installing:

```bash
cargo run -- scan
```

### npm Global Install

Planned npm install command after package publication:

```bash
npm install -g sec-issue-finder
scif scan
```

Preview limitation: the npm package currently does not include prebuilt Rust binaries. Until binary distribution is implemented, users must build the Rust CLI first or have `sec-issue-finder` available on `PATH`.

### npm Project Install

Planned project-local npm install command after package publication:

```bash
npm install -D sec-issue-finder
npx scif scan
```

For pre-publish validation, use the local file dependency or tarball workflows in [docs/scif-local-testing.md](docs/scif-local-testing.md).

### pnpm Project Install

Planned project-local pnpm install command after package publication:

```bash
pnpm add -D sec-issue-finder
pnpm exec scif scan
```

This is also subject to the current preview limitation: the npm wrapper does not yet ship prebuilt binaries.

## Local Usage

Auto-detect and scan a supported lockfile in the current directory:

```bash
sec-issue-finder scan
```

Scan an npm lockfile:

```bash
sec-issue-finder scan --lockfile package-lock.json
```

Scan a pnpm lockfile:

```bash
sec-issue-finder scan --lockfile pnpm-lock.yaml
```

Scan a pnpm lockfile and print JSON:

```bash
sec-issue-finder scan --lockfile pnpm-lock.yaml --format json
```

Exclude dev dependencies:

```bash
sec-issue-finder scan --no-dev
```

Exclude dev dependencies from a pnpm lockfile:

```bash
sec-issue-finder scan --lockfile pnpm-lock.yaml --no-dev
```

Fail when high or critical findings are present:

```bash
sec-issue-finder scan --lockfile pnpm-lock.yaml --fail-on high
```

Fail on any known severity at low or above:

```bash
sec-issue-finder scan --fail-on low
```

Unknown severity findings do not fail policy by default.

## Local scif Wrapper Testing

The repository includes a local npm wrapper that exposes the short `scif` command for development testing. This is intended for validating real npm and pnpm projects before any npm publish or prebuilt binary work.

The npm wrapper is still preview/local-validation focused. It does not include prebuilt binaries yet, so local testing expects you to build the Rust CLI from this checkout first.

See [docs/scif-local-testing.md](docs/scif-local-testing.md) for:

- `npm link` smoke testing
- `npm install -D ../sec-finder` and `npx scif ...`
- `pnpm add -D ../sec-finder` and `pnpm exec scif ...`
- `npm pack` tarball installation testing without publishing

## Lockfile Auto-Detection

When `--lockfile` is not provided:

- If only `package-lock.json` exists, it is scanned.
- If only `pnpm-lock.yaml` exists, it is scanned.
- If both `package-lock.json` and `pnpm-lock.yaml` exist, the command returns an ambiguity error and asks you to pass `--lockfile`.

This avoids silently scanning the wrong package manager lockfile in mixed projects.

## OSV Ecosystem Mapping

pnpm installs npm packages. For advisory lookup, registry dependencies parsed from `pnpm-lock.yaml` are queried against OSV using ecosystem `"npm"`.

There is no separate OSV ecosystem named `"pnpm"` in this scanner. The source lockfile is tracked separately from the advisory ecosystem.

Reports include the source lockfile so results can still show whether a dependency came from `package-lock.json` or `pnpm-lock.yaml`.

## CI Usage

Example shell step:

```bash
sec-issue-finder scan --format table --fail-on high
```

Example JSON artifact step:

```bash
sec-issue-finder scan --format json --fail-on high > sec-issue-finder-report.json
```

Example GitHub Actions step after installing the tool:

```yaml
- name: Scan npm dependencies
  run: sec-issue-finder scan --lockfile package-lock.json --format table --fail-on high
```

With `--fail-on`, the command exits with a failing status when any finding is at or above the configured threshold. Without `--fail-on`, findings are reported but do not cause exit code 1.

## JSON Output

Example shape:

```json
{
  "schema_version": "1.0",
  "generated": {
    "tool": "sec-issue-finder",
    "format": "json"
  },
  "summary": {
    "total": 1,
    "critical": 0,
    "high": 1,
    "moderate": 0,
    "medium": 0,
    "low": 0,
    "unknown": 0,
    "direct": 1,
    "transitive": 0,
    "prod": 1,
    "dev": 0
  },
  "findings": [
    {
      "severity": "high",
      "package": {
        "name": "example-package",
        "installed_version": "1.0.0",
        "ecosystem": "npm",
        "package_url": "pkg:npm/example-package@1.0.0",
        "source_file": "pnpm-lock.yaml"
      },
      "advisory": {
        "id": "GHSA-example",
        "aliases": ["CVE-0000-0000"],
        "source": "osv",
        "summary": "Example advisory summary",
        "details": null,
        "url": "https://example.test/advisory"
      },
      "dependency_type": "direct",
      "scope": "prod",
      "fixed_versions": ["1.0.1"],
      "references": ["https://example.test/advisory"]
    }
  ]
}
```

The exact findings depend on OSV data available at scan time.

## Security Model

- Lockfile contents are treated as untrusted input.
- The parser reads lockfiles as data and does not execute scripts.
- The scanner does not shell out using package names.
- Tests mock advisory responses and do not call the real OSV API.
- Runtime scans query public advisory services unless future offline cache support is added.
- No secrets or tokens are required for current OSV usage.

## Limitations

- Advisory coverage depends on OSV and upstream public advisory data.
- OSV availability, rate limits, and response quality can affect scan results.
- Severity normalization is conservative and may report `unknown` when severity data is missing or not recognized.
- Only npm ecosystem lockfiles `package-lock.json` and `pnpm-lock.yaml` are supported today.
- pnpm support currently focuses on registry npm dependencies.
- pnpm local `workspace:`, `link:`, `file:`, and path-like dependencies may be skipped when no registry package version is available.
- pnpm peer dependency suffixes are normalized where possible, for example `react-dom@18.2.0(react@18.2.0)` becomes `react-dom@18.2.0`.
- pnpm multi-importer and workspace direct/dev classification is best effort.
- The scanner does not prove that a vulnerability is reachable or exploitable in your application.
- The scanner does not replace dependency review, patch testing, or broader supply chain controls.

## Roadmap

- Dart `pubspec.lock`
- Rust `Cargo.lock`
- `yarn.lock`
- `bun.lock`
- SARIF output
- CycloneDX SBOM output
- GitHub Actions integration
- Offline advisory cache

Roadmap items are planned work and are not supported unless documented in the current support section.

## Contributing

Contributions are welcome. Useful areas include parser fixtures, OSV response edge cases, reporter output tests, and CI integration examples.

New to the project? Start with the [onboarding guide](docs/onboarding.md). A Korean version is also available at [docs/onboarding.ko.md](docs/onboarding.ko.md).

For release preparation, see [docs/release.md](docs/release.md). Changes are tracked in [CHANGELOG.md](CHANGELOG.md).

For branch and commit strategy, see [docs/git-workflow.md](docs/git-workflow.md).

Read [CONTRIBUTING.md](CONTRIBUTING.md) before opening a pull request. For vulnerability reports, use the private process in [SECURITY.md](SECURITY.md).

Before submitting changes:

```bash
cargo fmt
cargo test
cargo clippy --all-targets --all-features
```

Keep changes focused, add tests for behavior changes, and avoid real network calls in tests.
