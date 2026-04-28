# Release Checklist

[한국어](release.ko.md)

This document is for checking `sec-issue-finder` immediately before a public npm release. It is a checklist only. Do not run `npm publish` until every blocking item is resolved and the release owner has explicitly approved publication.

## Current Release Scope

Target version: `0.1.0` preview.

Included:

- Rust CLI named `sec-issue-finder`.
- npm wrapper command named `scif`.
- npm `package-lock.json` scanning.
- Best-effort `pnpm-lock.yaml` scanning for registry npm dependencies.
- OSV advisory lookup.
- Table and JSON output.
- `--fail-on` CI policy.
- Local Node.js `scan()` API that executes the Rust CLI.

Not included:

- Prebuilt binary distribution.
- Auto-fix.
- GitHub release automation.
- npm provenance automation.
- SARIF.
- CycloneDX SBOM.
- Offline advisory cache.
- Complete pnpm workspace coverage.
- Complete vulnerability coverage guarantees.

## Preview Binary Distribution Warning

The npm package currently includes the JavaScript wrapper, but it does not include prebuilt Rust binaries.

Before a broad public npm release, decide one of the following:

- Keep the release clearly marked as preview and require users to build the Rust CLI or provide `sec-issue-finder` on `PATH`.
- Implement prebuilt binary distribution for supported platforms.
- Provide a documented installer strategy that downloads or builds the Rust binary safely.

Long term, prebuilt binaries are recommended for a normal npm CLI user experience. This checklist does not implement that strategy.

## Required Local Checks

Run all checks from the repository root:

```bash
npm test
npm run build --if-present
npm run lint --if-present
npm pack --dry-run
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo test --features test-utils
cargo build
```

Confirm:

- [ ] All commands pass locally.
- [ ] CI runs the same Rust and npm wrapper checks.
- [ ] Tests do not call the real OSV API.
- [ ] Feature-gated CLI tests run through `cargo test --features test-utils`.

## Tarball Install Tests

Create the tarball:

```bash
cargo build
npm pack
```

### npm Tarball Test

```bash
mkdir /tmp/scif-test
cd /tmp/scif-test
npm init -y
npm install /path/to/sec-issue-finder-0.1.0.tgz
npx scif scan --help
```

Then test against a real npm project with `package-lock.json`:

```bash
npx scif scan --lockfile package-lock.json
npx scif scan --lockfile package-lock.json --format json
npx scif scan --lockfile package-lock.json --fail-on high
```

### pnpm Tarball Test

```bash
mkdir /tmp/scif-pnpm-test
cd /tmp/scif-pnpm-test
pnpm init
pnpm add -D /path/to/sec-issue-finder-0.1.0.tgz
pnpm exec scif scan --help
```

Then test against a real pnpm project with `pnpm-lock.yaml`:

```bash
pnpm exec scif scan --lockfile pnpm-lock.yaml
pnpm exec scif scan --lockfile pnpm-lock.yaml --format json
pnpm exec scif scan --lockfile pnpm-lock.yaml --fail-on high
```

Confirm:

- [ ] npm tarball install works without publishing.
- [ ] pnpm tarball install works without publishing.
- [ ] `scif scan --help` works through npm and pnpm.
- [ ] Lockfile scanning works for npm and pnpm fixtures or real test projects.
- [ ] JSON output is valid JSON on stdout.
- [ ] `--fail-on` returns the expected exit code.
- [ ] Missing binary errors are understandable.

## Package Contents Check

Run:

```bash
npm pack --dry-run
```

Confirm:

- [ ] `LICENSE` is included.
- [ ] `README.md` is included.
- [ ] `package.json` is included.
- [ ] Runtime files under `npm/` are included.
- [ ] `.env` files are not included.
- [ ] `target/` is not included.
- [ ] `.github/` is not included.
- [ ] `tests/fixtures/` is not included.
- [ ] Rust source files are not included unless the packaging strategy intentionally changes.
- [ ] Local test artifacts are not included.

## Metadata Check

Confirm `package.json` has correct final values:

- [ ] `name`
- [ ] `version`
- [ ] `description`
- [ ] `license`
- [ ] `repository`
- [ ] `bugs`
- [ ] `homepage`
- [ ] `bin`
- [ ] `main`
- [ ] `types`
- [ ] `files`

Confirm repository URLs match the public GitHub repository before release.

## README Check

Confirm:

- [ ] README install instructions match the actual distribution mode.
- [ ] README states the current preview binary limitation.
- [ ] README documents npm usage:

```bash
npm install -g sec-issue-finder
scif scan
npm install -D sec-issue-finder
npx scif scan
```

- [ ] README documents pnpm usage:

```bash
pnpm add -D sec-issue-finder
pnpm exec scif scan
```

- [ ] README does not claim unsupported ecosystems are implemented.
- [ ] README does not claim complete vulnerability coverage.
- [ ] README does not imply auto-fix exists.

## npm Account And Publishing Policy

Before publishing:

- [ ] Confirm npm account ownership.
- [ ] Confirm npm account 2FA is enabled.
- [ ] Confirm organization/package access policy.
- [ ] Confirm npm provenance policy.
- [ ] Confirm whether provenance is required for this release.
- [ ] Confirm package name availability.
- [ ] Confirm final version number.

Publish command, for release-owner use only after approval:

```bash
npm publish
```

If provenance is required and the release is run from a supported CI environment, use the approved project policy for provenance. Do not improvise provenance settings during release.

## Post-Publish Checks

After publication, verify from a clean temporary project:

```bash
npm install -g sec-issue-finder
scif scan --help
```

```bash
npm install -D sec-issue-finder
npx scif scan --help
```

```bash
pnpm add -D sec-issue-finder
pnpm exec scif scan --help
```

If prebuilt binaries are still not implemented, these commands should only be used for a clearly marked preview release where users understand the binary requirement.
