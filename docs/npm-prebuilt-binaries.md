# npm Prebuilt Binary Distribution

This document describes the planned npm prebuilt binary strategy for `sec-issue-finder`.

It is a design document only. Do not publish npm packages, add release automation, or change runtime code from this document alone.

## 1. Goal

The target user experience for Milestone 5 is:

```bash
npm install -g @zzozorang/sec-issue-finder
scif scan --help
```

This must work on a user's local machine without requiring a Rust toolchain, `cargo`, or a local source checkout.

The npm package should keep the existing JavaScript wrapper command named `scif`, but the wrapper should be able to resolve and execute a prebuilt `sec-issue-finder` Rust binary for the current platform.

## 2. Why optionalDependencies instead of postinstall download

The preferred distribution model is platform-specific npm packages installed through `optionalDependencies`, not a `postinstall` script that downloads binaries.

Reasons:

- npm installs packages from the registry using normal package metadata, integrity hashes, lockfiles, and audit tooling.
- Users and CI systems can inspect exactly which binary package was installed.
- Package provenance can be attached to each platform package when the release process supports it.
- Installs do not need to execute a network-downloading `postinstall` script.
- Corporate and restricted environments are more likely to allow npm package installation than arbitrary install-time downloads.
- The wrapper can fail clearly when the current platform is unsupported or the optional package was not installed.

Tradeoff:

- The release process becomes more complex because multiple npm packages must be built, checked, and published in the correct order.

## 3. Package Roles

### Main package: `@zzozorang/sec-issue-finder`

Responsibilities:

- Provide the public npm package users install.
- Provide the `scif` executable through the `bin` field.
- Include JavaScript wrapper code and TypeScript declarations.
- Resolve the current platform's optional binary package.
- Fall back to local development binaries where appropriate.
- Print clear errors for unsupported platforms or missing binaries.

The main package should not contain all platform binaries.

### Platform packages

Responsibilities:

- Contain exactly one prebuilt `sec-issue-finder` binary for one OS and CPU architecture.
- Use npm `os` and `cpu` metadata so npm only installs compatible packages.
- Expose the binary at a stable internal path consumed by the main package resolver.
- Avoid JavaScript wrapper logic unless a tiny package metadata helper becomes necessary.

Planned package names:

```text
@zzozorang/sec-issue-finder-darwin-arm64
@zzozorang/sec-issue-finder-darwin-x64
@zzozorang/sec-issue-finder-linux-x64
@zzozorang/sec-issue-finder-linux-arm64
@zzozorang/sec-issue-finder-win32-x64
```

## 4. Supported Platform List

Planned platform mapping:

| npm package | `process.platform` | `process.arch` | binary name |
|---|---:|---:|---|
| `@zzozorang/sec-issue-finder-darwin-arm64` | `darwin` | `arm64` | `sec-issue-finder` |
| `@zzozorang/sec-issue-finder-darwin-x64` | `darwin` | `x64` | `sec-issue-finder` |
| `@zzozorang/sec-issue-finder-linux-x64` | `linux` | `x64` | `sec-issue-finder` |
| `@zzozorang/sec-issue-finder-linux-arm64` | `linux` | `arm64` | `sec-issue-finder` |
| `@zzozorang/sec-issue-finder-win32-x64` | `win32` | `x64` | `sec-issue-finder.exe` |

## 5. Minimum First Supported Platforms

The first implementation should support at least:

- `darwin-arm64`
- `linux-x64`
- `win32-x64`

This covers Apple Silicon local development, common Linux CI and server environments, and common Windows developer machines.

## 6. Additional Platforms If Practical

Add these when build and smoke-test coverage is reliable:

- `darwin-x64`
- `linux-arm64`

These are useful, but they should not block the first prebuilt-binary milestone if release automation or test infrastructure is not ready. `darwin-x64` targets Intel macOS and is excluded from the first artifact matrix because GitHub-hosted Intel macOS runner availability can leave the release workflow queued for a long time.

`linux-arm64` is also intentionally not part of the first GitHub Actions release artifact matrix. Add it after runner or cross-compilation support is verified with a real smoke test.

## 7. npm Package Structure Draft

One possible repository layout:

```text
npm/
  bin.js
  index.js
  index.d.ts
  lib/
    binary.js
    errors.js
packages/
  darwin-arm64/
    package.json
    bin/
      sec-issue-finder
  darwin-x64/
    package.json
    bin/
      sec-issue-finder
  linux-x64/
    package.json
    bin/
      sec-issue-finder
  linux-arm64/
    package.json
    bin/
      sec-issue-finder
  win32-x64/
    package.json
    bin/
      sec-issue-finder.exe
```

The main `package.json` would eventually include optional dependencies similar to:

```json
{
  "optionalDependencies": {
    "@zzozorang/sec-issue-finder-darwin-arm64": "0.1.0",
    "@zzozorang/sec-issue-finder-linux-x64": "0.1.0",
    "@zzozorang/sec-issue-finder-win32-x64": "0.1.0"
  }
}
```

Each platform package should use strict `files` metadata so only package metadata, license/readme if needed, and the binary are included.

## 8. Binary Resolver Order

The main wrapper should resolve the binary in this order:

1. Explicit `binaryPath` option from the Node API.
2. `SEC_ISSUE_FINDER_BINARY_PATH` environment variable.
3. Platform optional package for the current `process.platform` and `process.arch`.
4. Local development release binary:

   ```text
   target/release/sec-issue-finder
   target/release/sec-issue-finder.exe
   ```

5. Local development debug binary:

   ```text
   target/debug/sec-issue-finder
   target/debug/sec-issue-finder.exe
   ```

6. `sec-issue-finder` or `sec-issue-finder.exe` on `PATH`.

For the Node API, `scan({ binaryPath })` intentionally has higher priority than `SEC_ISSUE_FINDER_BINARY_PATH` so tests and embedding applications can force a specific binary without mutating the process environment.

The resolver should keep stdout clean. It must not print informational logs during normal execution because `scif scan --format json` and the Node `scan()` API depend on parseable JSON stdout.

If no binary is found, the error should include:

- Current platform and architecture.
- Expected optional package name.
- The local development paths that were checked.
- A short note that the current package may be unsupported or the optional dependency may not have installed.

## 9. GitHub Actions Release Build Strategy

Use a release-oriented workflow separate from normal CI.

Suggested jobs:

- Build Rust binary on each target platform.
- Run a smoke test for the produced binary:

  ```bash
  ./sec-issue-finder scan --help
  ```

- Copy the binary into the matching platform package directory.
- Generate or verify platform package metadata.
- Run `npm pack --dry-run` for each platform package.
- Upload each platform package tarball as a GitHub Actions artifact.
- Build and pack the main package after platform tarballs are available.
- Run an install smoke test with the main package and matching local platform tarball.

Suggested matrix:

| Runner | Target package |
|---|---|
| `macos-latest` | `@zzozorang/sec-issue-finder-darwin-arm64` |
| `ubuntu-latest` | `@zzozorang/sec-issue-finder-linux-x64` |
| `windows-latest` | `@zzozorang/sec-issue-finder-win32-x64` |

Linux ARM64 and macOS x64 may require additional runners, cross-compilation, or a separate release strategy.

Do not automatically publish from this workflow until the package contents, provenance policy, and maintainer approval flow are finalized.

### Preparing a platform package locally

After building a release binary, copy it into the matching platform package with:

```bash
node scripts/prepare-platform-package.mjs \
  --platform linux \
  --arch x64 \
  --binary target/release/sec-issue-finder
```

Windows uses the `.exe` binary name:

```bash
node scripts/prepare-platform-package.mjs \
  --platform win32 \
  --arch x64 \
  --binary target/release/sec-issue-finder.exe
```

Use `--dry-run` to validate package metadata and paths without copying the binary:

```bash
node scripts/prepare-platform-package.mjs \
  --platform darwin \
  --arch arm64 \
  --binary target/release/sec-issue-finder \
  --dry-run
```

The script validates the platform package name and version against the main `package.json`. It does not publish anything.

## 10. npm Publish Order

When release publication is explicitly approved:

1. Publish platform packages first:

   ```text
   @zzozorang/sec-issue-finder-darwin-arm64
   @zzozorang/sec-issue-finder-linux-x64
   @zzozorang/sec-issue-finder-win32-x64
   ```

2. Publish the main package last:

   ```text
   sec-issue-finder
   ```

Reason:

- The main package depends on platform packages through `optionalDependencies`.
- If the main package is published first, users may install it before the referenced platform packages exist.
- Publishing platform packages first makes the first public install path more reliable.

Before publishing, verify the npm scope and package names are available and controlled by the maintainer account or organization. The `@zzozorang` npm scope is owned by the project organization; do not publish the main package with `optionalDependencies` that point to packages the project does not control.

## 11. Smoke Test Procedure

Before public npm publish, test from tarballs.

### Platform package smoke test

For each platform package:

```bash
npm pack
tar -tf sec-issue-finder-*.tgz
```

Confirm:

- The package contains one binary.
- The binary has the expected name.
- The binary is executable on Unix platforms.
- Windows package uses `.exe`.
- No source tree, `target/`, fixtures, `.env`, or local artifacts are included.

### Main package install smoke test

From a clean temporary project:

```bash
npm init -y
npm install /path/to/zzozorang-sec-issue-finder-0.1.0.tgz
npx scif scan --help
```

Then test a minimal npm lockfile:

```bash
npx scif scan --lockfile package-lock.json
npx scif scan --lockfile package-lock.json --format json
```

For pnpm:

```bash
pnpm init
pnpm add -D /path/to/zzozorang-sec-issue-finder-0.1.0.tgz
pnpm exec scif scan --help
pnpm exec scif scan --lockfile pnpm-lock.yaml
pnpm exec scif scan --lockfile pnpm-lock.yaml --format json
```

Confirm:

- `scif scan --help` works without Rust installed.
- JSON output is clean stdout and valid JSON.
- stderr is used for errors.
- Exit code behavior matches `--fail-on`.
- Unsupported platforms produce a clear error.
- Missing optional binary package produces a clear error.

## 12. Known Risks

- npm scope ownership must be secured before publication.
- Platform packages must be published before the main package.
- Windows binary names require `.exe` handling in package layout and resolver logic.
- macOS code signing and notarization are outside this milestone.
- Linux glibc versus musl compatibility requires separate review.
- Cross-compilation can produce binaries that build successfully but fail at runtime if not smoke-tested on the target OS.
- Optional dependencies can be omitted by user install settings such as `--omit=optional`; the wrapper must explain this clearly.
- Package version skew can happen if the main package and platform packages are not published with matching versions.

## 13. Out of Scope for This Milestone

Do not implement these in Milestone 5:

- Running `npm publish`.
- Implementing `postinstall` downloads.
- Adding automatic release publication.
- Implementing macOS notarization.
- Claiming support for platforms that are not built and smoke-tested.
- Bundling every platform binary directly into the main package.
- Requiring users to install Rust for the normal npm install path.
