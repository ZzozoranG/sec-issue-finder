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

Use two primary branch families:

- `feature/<short-topic>` for normal development, fixes, documentation, tests, and maintenance changes.
- `version/<version>` for release stabilization and version-specific release preparation.

Recommended branch names:

```text
feature/<short-topic>
version/<version>
```

Examples:

```text
feature/yarn-lock-parser
feature/pnpm-peer-suffix-normalization
feature/npm-preview-install-docs
feature/osv-client-error-tests
feature/ci-node-wrapper-checks
version/0.1.0
```

## Branch Rules

- `main` should stay releasable.
- Do not commit directly to `main` unless the repository owner explicitly chooses that workflow.
- Keep `feature/*` branches focused on one user-visible change or one internal maintenance goal.
- Use `version/*` branches only for release stabilization, version metadata, changelog finalization, release checklist updates, and final release fixes.
- Rebase or merge `main` before opening a pull request if the branch is stale.
- Do not mix unrelated feature work into `version/*` branches.
- Do not mix release packaging, parser behavior, reporter output, and documentation rewrites in one branch unless they are part of the same feature or version task.
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

## Version Branches

Use a version branch only when preparing a version:

```text
version/0.1.0
```

Version branches may include:

- version updates
- changelog updates
- release checklist updates
- package metadata finalization
- documentation corrections
- final CI fixes

Version branches must not include unrelated feature work.

Merge order:

1. Merge completed `feature/*` branches into `main`.
2. Create `version/<version>` from `main` when the release scope is frozen.
3. Apply only release stabilization changes to `version/<version>`.
4. Merge `version/<version>` back into `main` after release approval.

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
