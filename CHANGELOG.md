# Changelog

[한국어](CHANGELOG.ko.md)

All notable changes to `sec-issue-finder` will be documented in this file.

The format is based on Keep a Changelog, and this project follows semantic versioning after the initial public release.

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

### Changed

- Repository metadata now points to the `ZzozoranG/sec-issue-finder` organization repository.

### Security

- Lockfiles are treated as untrusted input.
- Package manager commands and dependency lifecycle scripts are not executed.
- Tests use mocked OSV responses and do not call the real OSV API.

### Limitations

- The npm wrapper is preview/local-validation focused.
- The npm package does not include prebuilt Rust binaries yet.
- Public npm install without Rust or an existing `sec-issue-finder` binary on `PATH` is not the intended distribution mode yet.
- pnpm support focuses on registry npm dependencies; local, workspace, link, file, and path-like dependencies may be skipped when no registry version is available.
- Advisory coverage depends on public OSV data and does not guarantee complete vulnerability coverage.
- Auto-fix, SARIF, CycloneDX SBOM, offline advisory cache, and additional ecosystems are not implemented yet.
