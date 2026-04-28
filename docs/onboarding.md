# sec-issue-finder Onboarding Guide

This guide helps new contributors understand the `sec-issue-finder` codebase, run it locally, make safe changes, and know where tests and documentation live.

If you are new to the project, read in this order:

1. Skim this guide first.
2. Read the root [README.md](../README.md) for the user-facing product view.
3. Read [CONTRIBUTING.md](../CONTRIBUTING.md) for contribution expectations.
4. Run `cargo test` and `cargo test --features test-utils` locally to verify your checkout.
5. Before editing, inspect the tests and fixtures for the area you plan to change.

Korean version: [onboarding.ko.md](onboarding.ko.md)

## Project Summary

`sec-issue-finder` is a Rust CLI that reads dependency lockfiles, normalizes installed package data, queries public advisory databases such as OSV, and reports known open source security findings.

The current v0.1.0 scope focuses on:

- npm `package-lock.json` v2/v3
- registry npm dependencies from pnpm `pnpm-lock.yaml`
- OSV `/v1/querybatch`
- table and JSON output
- CI failure policy through `--fail-on`

Important limits:

- The tool does not guarantee complete vulnerability coverage.
- Results depend on public advisory data and package/version metadata in the lockfile.
- The tool does not run package manager commands.
- The tool does not execute dependency lifecycle scripts.
- The tool does not provide auto-fix.
- pnpm workspace, local, and path dependency handling is conservative and best effort.

## Development Setup

Required tools:

- Stable Rust toolchain
- Cargo
- `rustfmt`
- `clippy`

The minimum supported Rust version is defined by `rust-version` in [Cargo.toml](../Cargo.toml).

Baseline verification commands:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo test --features test-utils
cargo build
```

During day-to-day development, these are usually enough:

```bash
cargo fmt
cargo test
cargo clippy --all-targets --all-features
```

## Running Locally

Run without installing:

```bash
cargo run -- scan
```

Build and run the binary:

```bash
cargo build
./target/debug/sec-issue-finder scan
```

Scan an npm lockfile:

```bash
cargo run -- scan --lockfile package-lock.json
```

Scan a pnpm lockfile:

```bash
cargo run -- scan --lockfile pnpm-lock.yaml
```

Print JSON:

```bash
cargo run -- scan --lockfile pnpm-lock.yaml --format json
```

Exclude dev dependencies:

```bash
cargo run -- scan --lockfile pnpm-lock.yaml --no-dev
```

Fail like a CI policy when high or critical findings are present:

```bash
cargo run -- scan --lockfile pnpm-lock.yaml --fail-on high
```

## Common Commands

### `cargo fmt`

Formats Rust code. Run this before opening a pull request.

### `cargo fmt --check`

Checks formatting without modifying files. This is what CI runs.

### `cargo test`

Runs unit and integration tests with the default feature set.

Default tests do not call the real OSV API. OSV client tests use mocked transports, and scan tests use in-process mock advisory clients.

### `cargo test --features test-utils`

Runs tests with the `test-utils` feature enabled.

This feature enables an internal CLI integration-test hook that lets the spawned binary read an OSV mock response file.

Important details:

- `test-utils` is not needed for normal builds.
- Do not enable `test-utils` for release builds.
- `SEC_ISSUE_FINDER_OSV_MOCK_RESPONSE_FILE` only works in builds compiled with `test-utils`.

### `cargo clippy --all-targets --all-features -- -D warnings`

Runs the CI lint command and treats warnings as failures.

### `cargo build`

Builds the normal binary with default features. This command does not enable `test-utils`.

## CI

CI is defined in [.github/workflows/ci.yml](../.github/workflows/ci.yml).

Current order:

1. `cargo fmt --check`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo test`
4. `cargo test --features test-utils`
5. `cargo build`

The order is intentional:

- Formatting and linting run first.
- Default-feature tests run next.
- Feature-gated CLI integration tests run with `test-utils`.
- The final build verifies that the normal binary still builds without `test-utils`.

## Code Layout

Important files and directories:

```text
src/
  main.rs
  lib.rs
  cli.rs
  types.rs
  error.rs
  scan.rs
  policy.rs
  ecosystems/
    mod.rs
    npm.rs
    pnpm.rs
    dart.rs
  clients/
    mod.rs
    osv.rs
  analyzers/
    mod.rs
    osv.rs
  reporters/
    mod.rs
    table.rs
    json.rs
tests/
  cli_pnpm.rs
  fixtures/
    npm/
    pnpm/
    scan/
    osv/
docs/
  release-checklist.md
  onboarding.md
  onboarding.ko.md
```

### [src/main.rs](../src/main.rs)

The CLI entrypoint.

Responsibilities:

- Initialize tracing.
- Parse CLI arguments.
- Run the `scan` command.
- Render reports.
- Convert policy failures into process exit codes.

Keep `main.rs` thin. Real logic should live in `scan`, `policy`, `reporters`, `clients`, or `ecosystems`.

### [src/cli.rs](../src/cli.rs)

Defines the `clap` CLI.

Current options:

- `scan`
- `--lockfile <path>`
- `--format table|json`
- `--fail-on low|moderate|medium|high|critical`
- `--include-dev`
- `--no-dev`

When adding CLI options, also consider:

- CLI parsing tests
- README usage examples
- `ScanConfig` changes
- JSON output schema impact

### [src/types.rs](../src/types.rs)

Defines the core domain model.

Important types:

- `Dependency`
- `Ecosystem`
- `Severity`
- `Advisory`
- `Finding`
- `ScanConfig`
- `ScanResult`

Design principle:

- Ecosystem parsers normalize lockfile-specific data into `Dependency`.
- Advisory clients receive normalized `Dependency` values.
- Reporters render `Finding` values and should not reinterpret provider-specific response shapes.

### [src/error.rs](../src/error.rs)

Defines `SecFinderError`, the project domain error type.

Principles:

- Library/domain code uses typed errors through `thiserror`.
- The CLI entrypoint can use `anyhow` where appropriate.
- Missing files, malformed lockfiles, and OSV failures should return useful user-facing errors.

### [src/scan.rs](../src/scan.rs)

The main scan pipeline.

Current flow:

1. Resolve lockfile path.
2. Select parser by lockfile name.
3. Parse dependencies.
4. Deduplicate advisory query dependencies.
5. Query OSV.
6. Normalize OSV results into internal findings.
7. Return `ScanResult`.

Important design choices:

- Advisory queries are deduplicated before OSV.
- The dedupe key is advisory ecosystem, package name, and version.
- pnpm dependencies are queried against OSV ecosystem `"npm"`.
- Duplicate metadata is merged conservatively:
  - If any duplicate is direct, `direct = true`.
  - If any duplicate is production, `dev = false`.

Known tradeoff:

- Deduplication can collapse importer/path nuance.
- The current `Dependency` model does not include importer path metadata.
- Future workspace reporting may require extending the domain model.

### [src/ecosystems/mod.rs](../src/ecosystems/mod.rs)

Defines the lockfile parser abstraction and selects a parser for a lockfile path.

Currently supported:

- `package-lock.json` -> npm parser
- `pnpm-lock.yaml` -> pnpm parser

Future parser candidates:

- Dart `pubspec.lock`
- Rust `Cargo.lock`
- Yarn `yarn.lock`
- Bun `bun.lock`

Every ecosystem parser should normalize data into `Vec<Dependency>`.

### [src/ecosystems/npm.rs](../src/ecosystems/npm.rs)

Parser for npm `package-lock.json` v2/v3.

Responsibilities:

- Read the `packages` object.
- Determine direct dependencies from root package metadata.
- Extract package name, version, and dev/prod metadata.
- Support scoped packages.
- Normalize into `Dependency`.

It must not:

- Run `npm install`.
- Execute package scripts.
- Query OSV.
- Perform auto-fix.

### [src/ecosystems/pnpm.rs](../src/ecosystems/pnpm.rs)

Parser for pnpm `pnpm-lock.yaml`.

Current scope:

- Best-effort parsing of registry npm dependencies.
- `importers`, `packages`, and `snapshots`.
- Scoped packages.
- Peer suffix normalization.
- Direct/dev classification from `dependencies`, `devDependencies`, and `optionalDependencies`.
- Skipping local, workspace, and path-like dependencies when no registry version is available.

Important pnpm policy:

- pnpm installs npm packages, so `Dependency.ecosystem` is `Ecosystem::Npm`.
- OSV queries use ecosystem `"npm"`.
- The scanner does not create a separate `"pnpm"` OSV ecosystem.
- Local dependencies are skipped when a registry version cannot be determined.

Common skipped forms:

- `file:`
- `link:`
- `workspace:`
- `portal:`
- `path:`
- `../foo`
- `./foo`
- absolute/path-like keys
- versions that cannot be treated as registry package versions

Limitations:

- pnpm workspace and multi-importer classification is best effort.
- Duplicate package name/version entries are deduplicated in the scan pipeline.
- Importer-level reporting is not implemented yet.

### [src/clients/osv.rs](../src/clients/osv.rs)

OSV `/v1/querybatch` client.

Responsibilities:

- Convert `Dependency` values into OSV querybatch requests.
- Send HTTP requests.
- Parse OSV responses into typed structs.
- Preserve dependency/result order mapping.

It must not:

- Parse lockfiles.
- Normalize findings for reporters.
- Expose raw `serde_json::Value` as a public API.

Testing strategy:

- Unit tests use mocked transports.
- CLI integration tests use the file-based mock response hook only with `test-utils`.
- Tests do not call the real OSV API.

### [src/analyzers/osv.rs](../src/analyzers/osv.rs)

Converts OSV response data into internal `Finding` values.

Responsibilities:

- Extract vulnerability ID, aliases, summary, and details.
- Normalize severity.
- Extract fixed versions.
- Extract references.
- Preserve dependency metadata such as direct/dev/source file.

Policies:

- Missing severity becomes `Severity::Unknown`.
- Missing fixed versions become an empty list.
- Missing references become an empty list.

### [src/reporters](../src/reporters)

Output modules.

Current reporters:

- table
- JSON

Principles:

- Reporters sort and render findings.
- Reporters should not parse OSV-specific response shapes.
- Reporters should not mutate finding meaning.
- Output should be deterministic for tests.

### [src/policy.rs](../src/policy.rs)

CI failure policy.

Current behavior:

- Without `--fail-on`, findings do not fail the command.
- With `--fail-on`, findings at or above the threshold fail the command.
- `moderate` and `medium` are equivalent.
- `unknown` severity does not fail by default.

## Test Structure

### 1. Parser Unit Tests

Locations:

- [src/ecosystems/npm.rs](../src/ecosystems/npm.rs)
- [src/ecosystems/pnpm.rs](../src/ecosystems/pnpm.rs)

Fixtures:

- `tests/fixtures/npm`
- `tests/fixtures/pnpm`

Examples:

- direct dependency
- dev dependency
- transitive dependency
- scoped package
- malformed lockfile
- missing version
- local/path-like dependency skipping
- pnpm peer suffix normalization
- pnpm multi-importer behavior

### 2. OSV Client Tests

Location:

- [src/clients/osv.rs](../src/clients/osv.rs)

Examples:

- request body construction
- empty dependency list
- non-2xx response
- malformed response
- vulnerability response
- response order mapping

### 3. Scan Pipeline Tests

Location:

- [src/scan.rs](../src/scan.rs)

Examples:

- lockfile auto-detection
- ambiguous lockfile error
- pnpm dependency OSV mapping
- advisory query deduplication
- policy interaction
- mocked advisory client behavior

### 4. CLI Integration Tests

Location:

- [tests/cli_pnpm.rs](../tests/cli_pnpm.rs)

Characteristics:

- They run the actual binary.
- Some tests require `test-utils`.
- `SEC_ISSUE_FINDER_OSV_MOCK_RESPONSE_FILE` is a feature-gated test hook.
- They do not call the real OSV API.

## Fixture Guidelines

Fixtures should be small and hand-written when possible.

Good fixtures:

- Test one behavior.
- Keep lockfile structure minimal.
- Use only a few dependencies.
- Make malformed cases obvious.

Avoid:

- Large generated lockfiles from `npm install` or `pnpm install`.
- `node_modules`.
- Fixtures requiring network access.
- Lifecycle script execution.
- Metadata unrelated to the test.

Common fixture locations:

```text
tests/fixtures/npm/
tests/fixtures/pnpm/
tests/fixtures/scan/
tests/fixtures/osv/
```

## Security Principles

Treat lockfiles as untrusted input.

Rules:

- Do not trust lockfile contents.
- Avoid `unwrap` or `expect` on lockfile-derived values.
- Do not run package managers.
- Do not pass dependency names or versions to shell commands.
- Do not execute dependency lifecycle scripts.
- Malformed JSON/YAML should return typed errors or be skipped safely, not panic.
- Skip local/path-like dependencies when no registry version can be determined.
- Tests must not call the real OSV API.

## Working On pnpm Support

pnpm support is one of the most sensitive areas in the current release scope.

Check these files before making changes:

- [src/ecosystems/pnpm.rs](../src/ecosystems/pnpm.rs)
- [src/scan.rs](../src/scan.rs)
- [tests/fixtures/pnpm](../tests/fixtures/pnpm)
- [tests/fixtures/scan](../tests/fixtures/scan)
- [tests/cli_pnpm.rs](../tests/cli_pnpm.rs)
- [README.md](../README.md)

Expected behavior:

- Scoped package `@scope/pkg` parses correctly.
- Peer suffixes normalize to the base package/version where possible.
- `file:`, `link:`, `workspace:`, `portal:`, and `path:` entries are skipped when no registry version is available.
- Relative and absolute path-like keys should not create bogus dependencies.
- Duplicate package name/version entries are deduplicated before OSV queries.
- pnpm dependencies query OSV as ecosystem `"npm"`.
- `--no-dev` excludes dev dependencies but still queries OSV when production dependencies remain.

Recommended tests after pnpm changes:

```bash
cargo test pnpm
cargo test --features test-utils --test cli_pnpm
cargo test
cargo test --features test-utils
```

## Working On The OSV Client

OSV code touches an external API, but tests must not use the real network.

Check:

- Request body matches OSV expectations.
- Dependency/result order is preserved.
- Empty dependency lists do not send HTTP requests.
- Non-2xx status returns a typed error.
- Malformed responses return typed errors.
- Public APIs do not expose raw `serde_json::Value`.

About `SEC_ISSUE_FINDER_OSV_MOCK_RESPONSE_FILE`:

- It is not a user-facing feature.
- It is compiled only with the `test-utils` feature.
- It exists to mock spawned-binary CLI integration tests.
- Normal and release builds must not depend on it.

## Working On Reporters

Reporter changes affect user-visible output.

Check these files:

- [src/reporters/mod.rs](../src/reporters/mod.rs)
- [src/reporters/table.rs](../src/reporters/table.rs)
- [src/reporters/json.rs](../src/reporters/json.rs)

Rules:

- Output ordering must be deterministic.
- If JSON schema changes, update README examples.
- Do not remove source metadata unless there is a replacement.
- Do not parse provider-specific response shapes in reporters.

Current sorting uses severity, direct/transitive, prod/dev, package name, and advisory ID.

## Working On Policy

Policy changes affect CI exit codes.

Check these files:

- [src/policy.rs](../src/policy.rs)
- [src/main.rs](../src/main.rs)
- [src/cli.rs](../src/cli.rs)

Current policy:

- No threshold means no failure from findings.
- `low` fails on low and above.
- `high` fails on high and critical.
- `critical` fails only on critical.
- `moderate` and `medium` are equivalent.
- `unknown` does not fail by default.

If policy behavior changes, update CLI tests and README.

## Adding A New Ecosystem Parser

For example, to add Dart `pubspec.lock` support:

1. Check whether the domain model already has the ecosystem.
2. Add `src/ecosystems/<name>.rs`.
3. Implement the `LockfileParser` trait.
4. Normalize parser output into `Vec<Dependency>`.
5. Add parser selection in `src/ecosystems/mod.rs`.
6. Confirm advisory ecosystem mapping.
7. Add small fixtures.
8. Add parser unit tests.
9. Add scan pipeline tests.
10. If CLI behavior changes, add integration tests and update README.

Rules:

- Parsers must not call advisory APIs.
- Advisory clients must not parse lockfiles.
- Reporters must not perform ecosystem-specific parsing.

## Common Checklists

### Lockfile Parser Changes

- [ ] No panic on malformed input.
- [ ] Missing file errors are useful.
- [ ] Missing fields are handled safely.
- [ ] Scoped packages still work.
- [ ] Local/path-like dependencies are safe.
- [ ] Fixtures are small and clear.
- [ ] Relevant parser tests were run.
- [ ] User-facing behavior is documented.

### OSV Client Changes

- [ ] Tests do not call the real OSV API.
- [ ] Request body tests exist.
- [ ] Non-2xx tests exist.
- [ ] Malformed response tests exist.
- [ ] Response order mapping is preserved.
- [ ] Typed response structs are preserved.

### Reporter Changes

- [ ] Output is deterministic.
- [ ] Empty findings are tested.
- [ ] One finding is tested.
- [ ] Multiple sorted findings are tested.
- [ ] README JSON examples are updated if schema changes.

### CLI Changes

- [ ] `clap` parsing tests exist.
- [ ] CLI logic remains thin.
- [ ] Exit-code behavior is clear.
- [ ] Integration tests are added if needed.
- [ ] README usage is updated.

## Before Opening A PR

Run at least:

```bash
cargo fmt
cargo test
cargo test --features test-utils
cargo clippy --all-targets --all-features
cargo build
```

For release-adjacent changes, run the stricter CI-equivalent checks:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo test --features test-utils
cargo build
```

## Documentation Update Rules

Update docs when you change:

- CLI options
- supported lockfiles
- output schema
- policy behavior
- security model or limitations
- release scope

Likely files:

- [README.md](../README.md)
- [README.ko.md](../README.ko.md)
- [CONTRIBUTING.md](../CONTRIBUTING.md)
- [CONTRIBUTING.ko.md](../CONTRIBUTING.ko.md)
- [docs/release-checklist.md](release-checklist.md)
- this guide

## Current Release Readiness

For v0.1.0, the project has:

- npm `package-lock.json` parser
- pnpm `pnpm-lock.yaml` best-effort parser for registry npm dependencies
- OSV querybatch client
- OSV result normalization
- table reporter
- JSON reporter
- `--fail-on` policy
- default-feature tests
- `test-utils` feature-gated CLI integration tests
- CI coverage for both test modes
- conservative pnpm documentation

Remaining release checks:

- Replace the repository placeholder in `Cargo.toml` with the real GitHub URL.
- Run `cargo publish --dry-run`.
- Prepare GitHub release notes.
- Confirm the release checklist before tagging.

## Asking For Help

When asking for help in an issue or pull request, include:

- command run
- expected result
- actual result
- lockfile type
- related fixture path
- failing test name
- error message
- changed files

Example:

```text
Command: cargo test pnpm_peer_suffix
Expected: normalize to react-dom@18.2.0
Actual: no dependency was produced
Fixture: tests/fixtures/pnpm/peer-suffix.yaml
Changed files: src/ecosystems/pnpm.rs
```

## Core Principle

The project is built around a simple separation of responsibilities:

- Parsers read lockfiles.
- Clients talk to advisory providers.
- Analyzers convert provider responses into internal findings.
- Reporters render findings.
- Policy decides whether findings should fail the command.
- Tests do not depend on external network calls.
- Documentation does not overstate supported coverage.
