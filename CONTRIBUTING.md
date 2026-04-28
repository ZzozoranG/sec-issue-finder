# Contributing

[한국어](CONTRIBUTING.ko.md)

Thanks for helping improve `sec-issue-finder`.

## Scope

This project currently supports npm `package-lock.json` and best-effort `pnpm-lock.yaml` scanning for registry npm dependencies with OSV advisory lookup. Please do not describe future ecosystem work as implemented until parser, scan, reporter, and tests are merged.

The npm wrapper is still preview/local-validation focused. It does not include prebuilt Rust binaries yet.

## Development

Use a recent stable Rust toolchain compatible with the `rust-version` in `Cargo.toml`.

```bash
cargo fmt
cargo test
cargo clippy --all-targets --all-features
cargo build
```

Before opening a pull request, run:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo test --features test-utils
cargo build
```

For npm wrapper changes, also run:

```bash
npm test
npm run lint --if-present
npm pack --dry-run
```

## Tests

- Keep tests deterministic.
- Do not call the real OSV API from tests.
- Do not run package managers inside fixture projects.
- Add small hand-written fixtures for lockfile parser behavior.
- Cover malformed input and missing-field cases when parsing untrusted files.
- Run feature-gated CLI integration tests with `cargo test --features test-utils` when changing scan behavior.

## Pull Requests

GitHub automatically pre-fills new pull requests with the checklist in `.github/pull_request_template.md`.

Good pull requests are focused and include:

- A concise explanation of the behavior change.
- Tests for new behavior.
- Documentation updates when user-facing behavior changes.
- No unrelated formatting churn.

For branch names, commit style, release branches, and publish boundaries, follow [docs/git-workflow.md](docs/git-workflow.md).

## Security-Sensitive Changes

For vulnerabilities in this project, use the process in `SECURITY.md` instead of opening a public issue.
