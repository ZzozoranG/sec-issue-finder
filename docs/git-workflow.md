# Git Workflow

[한국어](git-workflow.ko.md)

This document defines the branch and commit strategy for `sec-issue-finder`.

## Goals

- Keep release history understandable.
- Make changes easy to review.
- Keep security-sensitive scanner changes traceable.
- Avoid mixing unrelated work in one branch or commit.
- Support preview npm wrapper work without accidentally publishing.

## Branch Strategy

Use short-lived branches from `main`.

Recommended branch names:

```text
feat/<short-topic>
fix/<short-topic>
docs/<short-topic>
test/<short-topic>
chore/<short-topic>
release/<version>
```

Examples:

```text
feat/yarn-lock-parser
fix/pnpm-peer-suffix-normalization
docs/npm-preview-install
test/osv-client-errors
chore/ci-node-wrapper-checks
release/0.1.0
```

## Branch Rules

- `main` should stay releasable.
- Do not commit directly to `main` unless the repository owner explicitly chooses that workflow.
- Keep feature branches focused on one user-visible change or one internal maintenance goal.
- Rebase or merge `main` before opening a pull request if the branch is stale.
- Do not mix release packaging, parser behavior, reporter output, and documentation rewrites in one branch unless they are part of the same release task.
- Do not include generated artifacts such as `target/`, `node_modules/`, local tarballs, logs, or `.env` files.

## Commit Strategy

Use small, reviewable commits. Each commit should compile or at least leave the repository in an explainable state.

Prefer Conventional Commit style:

```text
feat: add pnpm lockfile parser
fix: skip local pnpm path dependencies
docs: document npm preview wrapper limits
test: cover feature-gated pnpm CLI scans
chore: add npm wrapper CI checks
refactor: extract advisory deduplication
```

Recommended commit types:

- `feat`: user-facing feature
- `fix`: bug fix
- `docs`: documentation-only change
- `test`: tests or fixtures
- `chore`: CI, metadata, release prep, non-runtime maintenance
- `refactor`: internal structure change without intended behavior change

## Commit Hygiene

- Keep commits scoped.
- Explain why the change exists, not only what changed.
- Include tests with behavior changes.
- Include fixture updates with parser changes.
- Include documentation updates when CLI, output schema, install flow, or release behavior changes.
- Do not commit secrets, tokens, private keys, `.env`, local registry credentials, or real customer/project lockfiles.
- Do not commit npm tarballs unless a release policy explicitly requires checked-in artifacts.

## Pull Request Checklist

Before opening a pull request, run the relevant checks.

For Rust scanner changes:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo test --features test-utils
cargo build
```

For npm wrapper changes:

```bash
npm test
npm run lint --if-present
npm pack --dry-run
```

For documentation-only changes, run at least:

```bash
rg -n "TODO|OWNER|publish|prebuilt" README.md docs *.md
```

and verify links changed by the edit.

## Release Branches

Use a release branch only when preparing a version:

```text
release/0.1.0
```

Release branches may include:

- version updates
- changelog updates
- release checklist updates
- package metadata finalization
- documentation corrections
- final CI fixes

Release branches must not include unrelated feature work.

## npm Publishing Boundary

Do not run `npm publish` from normal feature branches.

For the current preview phase:

- npm package publishing is manual and gated by release owner approval.
- npm provenance policy must be checked before publishing.
- The package currently does not include prebuilt Rust binaries.
- If prebuilt binaries are still missing, README and release notes must clearly describe the preview limitation.

## Agent Notes

When an automation agent works in this repository:

- Check this document before proposing branch names or commit grouping.
- Do not create commits unless the user explicitly asks for commits.
- Do not push branches unless the user explicitly asks for push.
- Do not publish npm packages, GitHub releases, or crates unless explicitly instructed and confirmed.
- Preserve unrelated local changes.
