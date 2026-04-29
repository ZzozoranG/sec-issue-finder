## Summary

Describe what this PR changes and why.

## Type of Change

- [ ] Feature
- [ ] Bug fix
- [ ] Documentation
- [ ] Test
- [ ] CI / release maintenance
- [ ] Refactor

## Scope

- [ ] Rust scanner
- [ ] npm `package-lock.json` parser
- [ ] pnpm `pnpm-lock.yaml` parser
- [ ] OSV client / advisory handling
- [ ] Reporters / output schema
- [ ] Policy / exit code behavior
- [ ] npm wrapper / Node API
- [ ] Documentation only

## Labels

Add labels so Release Drafter can place this PR in the right release note category.

- [ ] `feature` / `enhancement`
- [ ] `bug` / `fix`
- [ ] `security`
- [ ] `documentation`
- [ ] `maintenance`
- [ ] `dependencies`
- [ ] `ci`
- [ ] `skip-changelog`

## Behavior Checklist

- [ ] User-facing behavior is described clearly.
- [ ] Unsupported ecosystems are not described as implemented.
- [ ] The change does not claim complete vulnerability coverage.
- [ ] The change does not add or imply auto-fix unless explicitly implemented.
- [ ] JSON output remains clean stdout when `--format json` is used.
- [ ] Exit code behavior is intentional and documented when changed.

## Security Checklist

- [ ] Lockfiles are treated as untrusted input.
- [ ] No package manager commands are executed from parser or scanner logic.
- [ ] No dependency lifecycle scripts are executed.
- [ ] No dependency names, package paths, or lockfile values are passed to shell commands.
- [ ] Malformed user-controlled input returns a clean error instead of panicking.
- [ ] No secrets, tokens, `.env` files, private keys, local registry credentials, or real customer lockfiles are included.
- [ ] Tests do not call the real OSV API.

## Test Checklist

Run the checks relevant to this PR.

- [ ] `cargo fmt --check`
- [ ] `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] `cargo test`
- [ ] `cargo test --features test-utils`
- [ ] `cargo build`
- [ ] `npm test`
- [ ] `npm run lint --if-present`
- [ ] `npm pack --dry-run`

## Documentation Checklist

- [ ] README is updated when user-facing behavior changes.
- [ ] Korean documentation is updated when English documentation changes.
- [ ] English documentation is updated when Korean documentation changes.
- [ ] CHANGELOG is updated for release-visible changes.
- [ ] Release checklist/docs are updated for packaging, CI, or publish-adjacent changes.

## npm Wrapper / Publish Boundary

- [ ] This PR does not run `npm publish`.
- [ ] This PR does not add npm publish automation.
- [ ] This PR does not configure GitHub release or npm provenance unless explicitly scoped.
- [ ] If npm wrapper behavior changes, `scif` still preserves Rust CLI stdout/stderr/exit code.
- [ ] If package contents change, `npm pack --dry-run` output was reviewed.
- [ ] If prebuilt binaries are still absent, docs clearly state the preview limitation.

## Notes for Reviewers

Mention risky areas, intentional tradeoffs, or follow-up tasks.
