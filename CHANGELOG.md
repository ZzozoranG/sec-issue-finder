# Changelog

[한국어](CHANGELOG.ko.md)

All notable changes to `sec-issue-finder` will be documented in this file.

The format is based on Keep a Changelog, and this project follows semantic versioning after the initial public release.

## [0.1.1] - 2026-04-30

### Changed

- Updated README installation instructions now that the npm preview packages are published.
- Synchronized Rust, main npm package, and platform npm package versions for the next patch release.
- Documented npm Trusted Publishing as the preferred automated release path after the initial manual publish.

### Verification

- Confirmed the published npm registry install path works on macOS arm64 with `npm install -D @zzozorang/sec-issue-finder` and `npx scif scan --help`.

## [0.1.0] - Preview

### Added

- Rust CLI for scanning supported dependency lockfiles.
- npm `package-lock.json` v2/v3 parsing.
- Best-effort `pnpm-lock.yaml` parsing for registry npm dependencies.
- OSV `/v1/querybatch` advisory lookup.
- Normalized internal finding model.
- Table and JSON reporters.
- `--fail-on` CI policy thresholds.
- `--no-dev` filtering.
- Source lockfile metadata in reports.
- Local npm wrapper exposing `scif`.
- Minimal Node.js `scan()` API that shells out to the Rust CLI and parses JSON output.
- Release and local validation documentation.
- CI checks for the npm wrapper, including `npm test`, lint, and `npm pack --dry-run`.
- Pull request checklist template for behavior, security, tests, documentation, and npm publish boundaries.
- Bilingual documentation coverage for release, local testing, changelog, code of conduct, and git workflow docs.
- Git workflow documentation for feature branches, version branches, commit hygiene, and release boundaries.
- Release Drafter configuration for GitHub release draft generation from merged pull requests.
- CODEOWNERS configuration for repository-wide maintainer review ownership.
- Dependabot configuration for Cargo, npm, and GitHub Actions dependency updates.
- Prebuilt npm platform packages for macOS arm64, Linux x64, and Windows x64.
- Public npm preview package `@zzozorang/sec-issue-finder`.
- npm Trusted Publishing workflow and documentation for future automated releases.

### Changed

- Repository metadata now points to the `ZzozoranG/sec-issue-finder` organization repository.

### Security

- Lockfiles are treated as untrusted input.
- Package manager commands and dependency lifecycle scripts are not executed.
- Tests use mocked OSV responses and do not call the real OSV API.

### Limitations

- The npm package is an initial preview release.
- Linux x64 and Windows x64 binaries are built in CI, but runtime smoke tests are still pending on those operating systems.
- pnpm support focuses on registry npm dependencies; local, workspace, link, file, and path-like dependencies may be skipped when no registry version is available.
- Advisory coverage depends on public OSV data and does not guarantee complete vulnerability coverage.
- Auto-fix, SARIF, CycloneDX SBOM, offline advisory cache, and additional ecosystems are not implemented yet.
