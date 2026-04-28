# v0.1.0 Release Checklist

This checklist is for the initial open source release of `sec-issue-finder`.

## Scope Check

- npm `package-lock.json` is supported.
- `pnpm-lock.yaml` support covers registry npm dependencies, with local/workspace/path dependencies skipped when no registry version is available.
- OSV advisory lookup is the only implemented advisory provider.
- Table and JSON are the only implemented output formats.
- `--fail-on` is the implemented CI policy mechanism.
- Auto-fix is not implemented.
- Dart, Rust, Yarn, Bun, Python, SARIF, CycloneDX, and GitHub Actions integration are future work unless implemented before release.

## Pre-Release Verification

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo build
```

Confirm:

- README usage matches the current CLI.
- No docs claim complete vulnerability coverage.
- No docs claim unsupported ecosystems are implemented.
- Tests do not call the real OSV API.
- `Cargo.toml` package metadata is complete.
- `LICENSE`, `SECURITY.md`, `CONTRIBUTING.md`, and `CODE_OF_CONDUCT.md` are present.

## Cargo Publish Checklist

- Replace the repository placeholder in `Cargo.toml` with the real GitHub URL.
- Confirm crate name availability on crates.io.
- Review included files:

```bash
cargo package --list
```

- Build the package:

```bash
cargo package
```

- Dry run publish:

```bash
cargo publish --dry-run
```

- Publish when ready:

```bash
cargo publish
```

## GitHub Release Checklist

- Create and push a signed tag if the project policy requires signed tags.
- Tag format:

```bash
git tag v0.1.0
git push origin v0.1.0
```

- Create a GitHub release for `v0.1.0`.
- Include:
  - Supported ecosystem: npm `package-lock.json`
  - pnpm `pnpm-lock.yaml` support for registry npm dependencies, with local/workspace/path dependencies skipped when no registry version is available
  - OSV lookup support
  - Table and JSON reporters
  - `--fail-on` policy behavior
  - Known limitations
  - Link to `SECURITY.md`
- Do not describe future roadmap items as released functionality.

## Future Work

- npm wrapper package for easier installation from JavaScript projects.
- Dart `pubspec.lock`
- Rust `Cargo.lock`
- `yarn.lock`
- `bun.lock`
- SARIF output
- CycloneDX SBOM output
- GitHub Actions integration
- Offline advisory cache
