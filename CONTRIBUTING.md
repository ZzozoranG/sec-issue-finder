# Contributing

Thanks for helping improve `sec-issue-finder`.

## Scope

This project currently supports npm `package-lock.json` scanning with OSV advisory lookup. Please do not describe future ecosystem work as implemented until parser, scan, reporter, and tests are merged.

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
cargo build
```

## Tests

- Keep tests deterministic.
- Do not call the real OSV API from tests.
- Do not run package managers inside fixture projects.
- Add small hand-written fixtures for lockfile parser behavior.
- Cover malformed input and missing-field cases when parsing untrusted files.

## Pull Requests

Good pull requests are focused and include:

- A concise explanation of the behavior change.
- Tests for new behavior.
- Documentation updates when user-facing behavior changes.
- No unrelated formatting churn.

## Security-Sensitive Changes

For vulnerabilities in this project, use the process in `SECURITY.md` instead of opening a public issue.
