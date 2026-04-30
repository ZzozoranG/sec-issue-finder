# Local scif Testing

[한국어](scif-local-testing.ko.md)

This document covers local `scif` wrapper testing before npm publishing. It does not run `npm publish` or configure GitHub releases.

The goal is to verify that the local npm wrapper exposes `scif` and forwards arguments to the Rust CLI from real npm and pnpm projects.

The source-checkout test flow is preview/local-validation focused. It does not rely on published platform packages. Before testing `npm link` or file dependency installs from this checkout, build the Rust CLI from this repository.

For `npm link` from this checkout, the wrapper can find `target/debug/sec-issue-finder` or `target/release/sec-issue-finder`. For packed main-package tarball installs without a matching platform package, the wrapper usually cannot see this checkout's `target/` directory, so put the built binary on `PATH` before running `npx scif` or `pnpm exec scif`.

For prebuilt platform tarball testing, use [npm-prebuilt-smoke-test.md](npm-prebuilt-smoke-test.md).

## Build The Rust CLI

From this repository, build the Rust CLI first:

```bash
cargo build
```

This creates `target/debug/sec-issue-finder`, which the local `scif` wrapper can find when running from this checkout or through `npm link`.

For a release-mode binary, use:

```bash
cargo build --release
```

The wrapper searches for a matching optional platform package first, then the installed package root's `target/release` path, then `target/debug`, then `PATH`. Main-package-only tarball installs do not include `target/`, so this source-checkout tarball test requires `sec-issue-finder` on `PATH` unless a matching platform package tarball is installed too.

## Test With npm link

This is useful for a quick smoke test from the repository checkout.

From this repository:

```bash
npm link
```

Then run:

```bash
scif scan --help
scif scan --lockfile package-lock.json
scif scan --lockfile pnpm-lock.yaml
```

## Test From A Real npm Project Using File Dependency

Use this flow from a separate npm project that has a `package-lock.json`.

```bash
cargo build
```

The `cargo build` command should be run from this repository before installing it into the npm project. Then move to the npm project and install this repository as a local file dependency:

```bash
npm install -D ../sec-finder
```

Run these commands from the npm project:

```bash
npx scif scan --help
npx scif scan --lockfile package-lock.json
npx scif scan --lockfile package-lock.json --format json
npx scif scan --lockfile package-lock.json --no-dev
npx scif scan --lockfile package-lock.json --fail-on high
```

## Test From A Real pnpm Project Using File Dependency

Use this flow from a separate pnpm project that has a `pnpm-lock.yaml`.

Build the Rust CLI from this repository first:

```bash
cargo build
```

Then move to the pnpm project and install this repository as a local file dependency:

```bash
pnpm add -D ../sec-finder
```

Run these commands from the pnpm project:

```bash
pnpm exec scif scan --help
pnpm exec scif scan --lockfile pnpm-lock.yaml
pnpm exec scif scan --lockfile pnpm-lock.yaml --format json
pnpm exec scif scan --lockfile pnpm-lock.yaml --no-dev
pnpm exec scif scan --lockfile pnpm-lock.yaml --fail-on high
```

## Test From An npm Tarball Without Publishing

This flow is closest to a real npm install while still avoiding the public npm registry.

From this repository, build the Rust CLI and create a local tarball:

```bash
cargo build
npm pack
```

The `npm pack` command creates a file such as:

```text
zzozorang-sec-issue-finder-0.1.0.tgz
```

Then create a separate npm project and install the tarball:

```bash
mkdir /tmp/scif-test
cd /tmp/scif-test
npm init -y
npm install /path/to/zzozorang-sec-issue-finder-0.1.0.tgz
```

Run the wrapper from the test project:

```bash
npx scif scan --help
npx scif scan --lockfile package-lock.json
npx scif scan --lockfile package-lock.json --format json
npx scif scan --lockfile package-lock.json --no-dev
npx scif scan --lockfile package-lock.json --fail-on high
```

Replace `/path/to/zzozorang-sec-issue-finder-0.1.0.tgz` with the absolute path to the tarball created by `npm pack`.

Important: the main package tarball contains the JavaScript wrapper only. For this source-checkout preview flow, `scif` succeeds only if it can find a matching platform package, a local `target/` binary, or `sec-issue-finder` on `PATH`.

## Test From A pnpm Tarball Without Publishing

Create the tarball from this repository:

```bash
cargo build
npm pack
```

Then create a separate pnpm project and install the tarball:

```bash
mkdir /tmp/scif-pnpm-test
cd /tmp/scif-pnpm-test
pnpm init
pnpm add -D /path/to/zzozorang-sec-issue-finder-0.1.0.tgz
```

Run the wrapper from the test project:

```bash
pnpm exec scif scan --help
pnpm exec scif scan --lockfile pnpm-lock.yaml
pnpm exec scif scan --lockfile pnpm-lock.yaml --format json
pnpm exec scif scan --lockfile pnpm-lock.yaml --no-dev
pnpm exec scif scan --lockfile pnpm-lock.yaml --fail-on high
```

If the test project does not already have a `pnpm-lock.yaml`, add a small dependency first:

```bash
pnpm add lodash
```

## Behavior Checklist

Check these items while testing in real projects:

- [ ] `scif scan --help`, `npx scif scan --help`, or `pnpm exec scif scan --help` prints the scan help.
- [ ] `npx scif scan --lockfile package-lock.json` recognizes and scans an npm lockfile.
- [ ] `pnpm exec scif scan --lockfile pnpm-lock.yaml` recognizes and scans a pnpm lockfile.
- [ ] `--format json` writes clean JSON to stdout.
- [ ] `--format json` does not mix table output or logs into stdout.
- [ ] `--no-dev` excludes dev dependencies.
- [ ] `--fail-on high` returns the expected exit code.
- [ ] Missing lockfiles produce an understandable error message.
- [ ] If `package-lock.json` and `pnpm-lock.yaml` both exist and `--lockfile` is omitted, the ambiguity error is understandable.

## Tarball Install Checklist

Check these items after installing `zzozorang-sec-issue-finder-0.1.0.tgz` into temporary npm and pnpm projects:

- [ ] `npx scif scan --help` prints help in an npm project.
- [ ] `pnpm exec scif scan --help` prints help in a pnpm project.
- [ ] `npx scif scan --lockfile package-lock.json` scans an npm lockfile.
- [ ] `pnpm exec scif scan --lockfile pnpm-lock.yaml` scans a pnpm lockfile.
- [ ] `--format json` prints valid JSON to stdout.
- [ ] `--fail-on high` returns the expected exit code for the findings present in the test project.
- [ ] If the Rust binary cannot be found, the error message explains that `cargo build` or `cargo build --release` is needed.
- [ ] The test does not require `npm publish`.
- [ ] The test does not require a GitHub release or npm provenance setup.

## Exit Code Checks

After running a `--fail-on high` command, inspect the exit code immediately.

For npm:

```bash
npx scif scan --lockfile package-lock.json --fail-on high
echo $?
```

For pnpm:

```bash
pnpm exec scif scan --lockfile pnpm-lock.yaml --fail-on high
echo $?
```

Expected behavior:

- `0`: scan completed and the configured policy did not fail.
- `1`: scan failed because the policy matched a finding or because an operational error occurred.

## Missing Binary Troubleshooting

If the Rust binary is missing, `scif` asks you to run:

```bash
cargo build
```

or:

```bash
cargo build --release
```

After rebuilding, reinstall the local file dependency if your package manager copied files instead of linking the checkout:

```bash
npm install -D ../sec-finder
```

or:

```bash
pnpm add -D ../sec-finder
```

## Important Boundaries

- Do not run `npm publish` for this local test.
- Do not implement or require prebuilt binaries for this local test.
- Runtime scans query OSV unless you are running Rust tests with mocked clients.
- pnpm support is best effort for registry npm dependencies. Local, workspace, and path-like dependencies may be skipped when no registry version is available.
