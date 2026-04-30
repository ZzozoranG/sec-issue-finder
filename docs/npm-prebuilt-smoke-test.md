# npm Prebuilt Binary Smoke Test

This guide verifies the prebuilt binary npm package flow before publishing anything to the npm registry.

It tests this local tarball flow:

1. Build the Rust release binary.
2. Copy the binary into the matching platform npm package.
3. Create a platform package tarball.
4. Create the main package tarball.
5. Install both tarballs into a clean temporary project.
6. Confirm `npx scif scan --help` works.

Do not run `npm publish` during this test.

## 1. Prerequisites

- Rust is required only on the maintainer or release machine that builds the binary.
- End users should not need Rust once prebuilt binary packages are published.
- This test is a publish-before local tarball validation flow.
- Run commands from the repository root unless a step explicitly changes directories.

## 2. Check Your Current Platform

```bash
node -p "process.platform + '-' + process.arch"
```

Expected values map to package directories:

| Output | Platform package directory |
|---|---|
| `darwin-arm64` | `packages/darwin-arm64` |
| `darwin-x64` | `packages/darwin-x64` |
| `linux-x64` | `packages/linux-x64` |
| `linux-arm64` | `packages/linux-arm64` |
| `win32-x64` | `packages/win32-x64` |

The first release artifact matrix focuses on `darwin-arm64`, `linux-x64`, and `win32-x64`. `darwin-x64` and `linux-arm64` are planned but require separate runner or cross-compilation validation.

## 3. Build the Release Binary

```bash
cargo build --release
```

Expected binary paths:

```text
target/release/sec-issue-finder
target/release/sec-issue-finder.exe
```

Use the `.exe` path on Windows.

## 4. Prepare the Platform Package

Choose the command for your platform.

### macOS arm64

```bash
node scripts/prepare-platform-package.mjs \
  --platform darwin \
  --arch arm64 \
  --binary target/release/sec-issue-finder
```

### Linux x64

```bash
node scripts/prepare-platform-package.mjs \
  --platform linux \
  --arch x64 \
  --binary target/release/sec-issue-finder
```

### Windows x64

```bash
node scripts/prepare-platform-package.mjs \
  --platform win32 \
  --arch x64 \
  --binary target/release/sec-issue-finder.exe
```

The script copies the binary into:

```text
packages/<platform-arch>/bin/sec-issue-finder
packages/win32-x64/bin/sec-issue-finder.exe
```

On non-Windows platforms, it also sets the executable bit.

## 5. Pack the Platform Package

From the matching platform package directory:

```bash
cd packages/<platform-arch>
npm pack
```

Example:

```bash
cd packages/linux-x64
npm pack
```

This creates a tarball similar to:

```text
sec-issue-finder-linux-x64-0.1.0.tgz
```

Move back to the repository root before continuing:

```bash
cd ../..
```

## 6. Pack the Main Package

From the repository root:

```bash
npm pack
```

This creates:

```text
sec-issue-finder-0.1.0.tgz
```

## 7. Install Tarballs in a Temporary Project

Create a clean test project:

```bash
mkdir /tmp/scif-prebuilt-test
cd /tmp/scif-prebuilt-test
npm init -y
```

Install the platform package tarball first, then the main package tarball:

```bash
npm install /path/to/sec-issue-finder-linux-x64-0.1.0.tgz
npm install --omit=optional --ignore-scripts --no-audit --no-fund /path/to/sec-issue-finder-0.1.0.tgz
npx scif scan --help
```

Replace `sec-issue-finder-linux-x64-0.1.0.tgz` with the tarball for your platform.

Why install the platform package first?

- Local tarball installs do not behave exactly like a registry install with published `optionalDependencies`.
- Installing the platform package manually makes the local test deterministic.
- `--omit=optional` prevents npm from trying to fetch unpublished platform packages from the public registry during local tarball tests.
- After registry publication, the main package should pull the matching platform package through `optionalDependencies`.

## 8. Global Install Test

Use a separate shell or terminal after installing to reduce the chance that local development paths hide packaging problems.

```bash
npm install -g /path/to/sec-issue-finder-linux-x64-0.1.0.tgz
npm install -g /path/to/sec-issue-finder-0.1.0.tgz
scif scan --help
```

Replace the platform tarball path with the tarball for your OS and architecture.

Uninstall after testing if needed:

```bash
npm uninstall -g sec-issue-finder
npm uninstall -g @sec-issue-finder/linux-x64
```

Adjust the scoped package name for your platform.

## 9. What to Verify

Confirm:

- `scif scan --help` works in a new project without a Rust toolchain.
- `scif` does not depend on `target/debug`.
- `scif` does not depend on a development `sec-issue-finder` binary on `PATH`.
- `npx scif scan --format json` keeps JSON output clean on stdout.
- Errors go to stderr.
- Unsupported platform errors clearly show detected platform and architecture.
- Missing optional package errors explain how to reinstall, verify platform support, or use `SEC_ISSUE_FINDER_BINARY_PATH`.

To reduce accidental PATH fallback during smoke tests, run from a clean shell and avoid adding the repository `target/` directories to `PATH`.

## 10. Common Problems

### Local tarball optional dependency behavior differs from registry installs

The main package's `optionalDependencies` point to registry package names. Before those packages are published, npm cannot fetch them from the registry.

For local smoke tests, install the matching platform package tarball manually before installing the main package tarball.

### The wrapper finds a local development binary

The resolver intentionally supports local development fallbacks:

```text
target/release/sec-issue-finder
target/debug/sec-issue-finder
PATH sec-issue-finder
```

For a realistic prebuilt test, run from a clean temporary directory and do not add development binaries to `PATH`.

### Windows binary name is wrong

Windows platform packages must contain:

```text
bin/sec-issue-finder.exe
```

Non-Windows packages must contain:

```text
bin/sec-issue-finder
```

### Platform package is missing

If the matching platform package is not installed, `scif` should fail with a clear message that includes:

- Detected platform.
- Detected architecture.
- Expected optional package name.
- Suggested fixes.

## 11. Minimal Command Summary

Linux x64 example:

```bash
cargo build --release
node scripts/prepare-platform-package.mjs --platform linux --arch x64 --binary target/release/sec-issue-finder
cd packages/linux-x64
npm pack
cd ../..
npm pack
mkdir /tmp/scif-prebuilt-test
cd /tmp/scif-prebuilt-test
npm init -y
npm install /path/to/sec-issue-finder-linux-x64-0.1.0.tgz
npm install --omit=optional --ignore-scripts --no-audit --no-fund /path/to/sec-issue-finder-0.1.0.tgz
npx scif scan --help
```
