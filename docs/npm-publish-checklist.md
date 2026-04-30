# npm Publish Checklist

This checklist is for publishing `sec-issue-finder` with prebuilt binary platform packages.

It is a release checklist only. Do not run `npm publish` until every blocking item is complete and the maintainer explicitly approves publication.

## 1. Pre-Publish Checks

Confirm:

- [ ] The npm scope `@sec-issue-finder` is secured by the project owner or organization.
- [ ] Maintainers have publish access for all platform packages.
- [ ] Maintainers have publish access for the main `sec-issue-finder` package.
- [ ] npm account 2FA is enabled and usable.
- [ ] npm provenance policy is decided.
- [ ] All package versions are synchronized:
  - [ ] Root `package.json`
  - [ ] `packages/darwin-arm64/package.json`
  - [ ] `packages/darwin-x64/package.json`
  - [ ] `packages/linux-x64/package.json`
  - [ ] `packages/linux-arm64/package.json`
  - [ ] `packages/win32-x64/package.json`
- [ ] `CHANGELOG.md` is updated.
- [ ] `CHANGELOG.ko.md` is updated if Korean release notes are maintained for this release.
- [ ] `LICENSE` is included in the main package.
- [ ] README install instructions match the actual release behavior.
- [ ] README does not claim unsupported ecosystems or complete vulnerability coverage.
- [ ] `docs/npm-prebuilt-binaries.md` reflects the current platform support.
- [ ] `docs/npm-prebuilt-smoke-test.md` has been followed with local tarballs.
- [ ] Local tarball smoke tests install the main tarball with `--omit=optional` so npm does not wait on unpublished registry platform packages.

## 2. Build Verification

Run from the repository root:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo test --features test-utils
cargo build --release
npm test
npm run lint --if-present
npm pack --dry-run
```

Confirm:

- [ ] All commands pass.
- [ ] Tests do not call the real OSV API.
- [ ] `npm pack --dry-run` for the main package does not include `target/`, `.github/`, fixtures, `.env`, or local artifacts.
- [ ] JSON output remains clean on stdout.

## 3. Platform Package Verification

For each platform package:

```bash
cd packages/<platform-arch>
npm pack --dry-run
```

Confirm:

- [ ] The package contains the expected binary under `bin/`.
- [ ] The package does not include `target/`, source trees, fixtures, `.env`, logs, or local artifacts.
- [ ] `package.json` has the correct `name`.
- [ ] `package.json` has the same `version` as the main package.
- [ ] `package.json` has the correct `os` value.
- [ ] `package.json` has the correct `cpu` value.
- [ ] Unix binaries are executable.
- [ ] Windows package contains `bin/sec-issue-finder.exe`.
- [ ] Non-Windows packages contain `bin/sec-issue-finder`.

Expected package metadata:

| Publish order | Package | `os` | `cpu` | binary |
|---:|---|---|---|---|
| 1 | `@sec-issue-finder/darwin-arm64` | `darwin` | `arm64` | `bin/sec-issue-finder` |
| 2 | `@sec-issue-finder/darwin-x64` | `darwin` | `x64` | `bin/sec-issue-finder` |
| 3 | `@sec-issue-finder/linux-x64` | `linux` | `x64` | `bin/sec-issue-finder` |
| 4 | `@sec-issue-finder/linux-arm64` | `linux` | `arm64` | `bin/sec-issue-finder` |
| 5 | `@sec-issue-finder/win32-x64` | `win32` | `x64` | `bin/sec-issue-finder.exe` |
| 6 | `sec-issue-finder` | platform-independent wrapper | platform-independent wrapper | `npm/bin.js` |

## 4. Publish Order

Publish platform packages first. Publish the main package last.

Reason:

- The main package uses `optionalDependencies` to reference platform packages.
- If the main package is published first, users may install it before the referenced platform packages exist.
- Publishing platform packages first gives the first public install path the best chance of working.

Publish order:

1. `@sec-issue-finder/darwin-arm64`
2. `@sec-issue-finder/darwin-x64`
3. `@sec-issue-finder/linux-x64`
4. `@sec-issue-finder/linux-arm64`
5. `@sec-issue-finder/win32-x64`
6. `sec-issue-finder`

## 5. Publish Command Examples

These are examples only. Do not run them until release approval is explicit.

Platform package example:

```bash
cd packages/linux-x64
npm publish --access public
```

With provenance, if the release policy requires it:

```bash
cd packages/linux-x64
npm publish --access public --provenance
```

Main package example:

```bash
cd /path/to/sec-finder
npm publish --access public
```

Main package with provenance:

```bash
cd /path/to/sec-finder
npm publish --access public --provenance
```

Before using `--provenance`, confirm the package is being published from a supported CI environment and the repository policy is ready.

## 6. Post-Publish Smoke Tests

Run on a new machine, clean container, or clean environment where Rust is not installed or not on `PATH`.

Global npm install:

```bash
npm install -g sec-issue-finder
scif scan --help
```

Local npm install:

```bash
mkdir /tmp/scif-npm-published-test
cd /tmp/scif-npm-published-test
npm init -y
npm install -D sec-issue-finder
npx scif scan --help
```

pnpm install:

```bash
mkdir /tmp/scif-pnpm-published-test
cd /tmp/scif-pnpm-published-test
pnpm init
pnpm add -D sec-issue-finder
pnpm exec scif scan --help
```

Additional checks:

```bash
npx scif scan --help
npx scif scan --format json
```

Confirm:

- [ ] `scif scan --help` works without Rust.
- [ ] The wrapper resolves the platform package binary, not `target/debug`.
- [ ] The wrapper does not depend on a local development binary on `PATH`.
- [ ] JSON output is not polluted by wrapper logs.
- [ ] Unsupported platform and missing optional package errors are clear.

## 7. Failure and Rollback Strategy

If a published version is broken:

- Deprecate the bad version with a clear message.
- Publish a fixed patch version.
- Do not overwrite or reuse the same published version.
- If a platform package is broken, publish a fixed patch version for that platform and then publish a matching main package patch if needed.
- If the main package references the wrong `optionalDependencies`, publish a main package patch release.
- Update release notes and known issues.

Example deprecation command:

```bash
npm deprecate sec-issue-finder@0.1.0 "Broken prebuilt binary resolution; please upgrade to 0.1.1."
```

For a platform package:

```bash
npm deprecate @sec-issue-finder/linux-x64@0.1.0 "Broken binary package; please upgrade to 0.1.1."
```

These commands are examples only. Do not run them unless a maintainer has decided on a rollback.

## 8. Never Do This

- Do not publish the main `sec-issue-finder` package before platform packages.
- Do not tag an unverified release as `latest`.
- Do not add arbitrary binary downloads in `postinstall`.
- Do not include the entire `target/` directory in any npm package.
- Do not publish from a dirty working tree.
- Do not publish with mismatched package versions.
- Do not claim support for a platform unless its binary package was built and smoke-tested.
- Do not claim complete vulnerability coverage.
