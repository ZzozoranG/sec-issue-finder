# Git Workflow

[English](git-workflow.md)

이 문서는 `sec-issue-finder`의 브랜치 및 커밋 전략을 정의합니다.

## 목표

- 릴리스 이력을 이해하기 쉽게 유지합니다.
- 변경 사항을 리뷰하기 쉽게 만듭니다.
- 보안 스캐너의 민감한 변경 사항을 추적 가능하게 유지합니다.
- 서로 관련 없는 작업을 하나의 branch나 commit에 섞지 않습니다.
- 실수로 publish하지 않으면서 preview npm wrapper 작업을 진행할 수 있게 합니다.

## 브랜치 전략

두 가지 primary branch family를 사용합니다.

- `feature/<short-topic>`: 일반 개발, bug fix, 문서, 테스트, maintenance 변경
- `version/<version>`: release stabilization 및 version-specific release 준비

권장 branch 이름:

```text
feature/<short-topic>
version/<version>
```

예시:

```text
feature/yarn-lock-parser
feature/pnpm-peer-suffix-normalization
feature/npm-preview-install-docs
feature/osv-client-error-tests
feature/ci-node-wrapper-checks
version/0.1.0
```

## 브랜치 규칙

- `main`은 releasable한 상태로 유지합니다.
- 저장소 owner가 명시적으로 선택한 workflow가 아니라면 `main`에 직접 commit하지 않습니다.
- `feature/*` branch는 하나의 user-visible change 또는 하나의 internal maintenance 목표에 집중합니다.
- `version/*` branch는 release stabilization, version metadata, changelog finalization, release checklist update, final release fix에만 사용합니다.
- branch가 오래되었다면 pull request 전에 `main`을 rebase 또는 merge합니다.
- 관련 없는 feature work를 `version/*` branch에 섞지 않습니다.
- 같은 feature 또는 version task가 아니라면 release packaging, parser behavior, reporter output, documentation rewrite를 한 branch에 섞지 않습니다.
- `target/`, `node_modules/`, local tarball, log, `.env` 같은 generated artifact를 포함하지 않습니다.

## 커밋 전략

작고 리뷰 가능한 commit을 사용합니다. 각 commit은 가능하면 compile 가능한 상태여야 하며, 그렇지 않다면 이유를 설명할 수 있어야 합니다.

Conventional Commit 스타일을 권장합니다.

```text
feat: add pnpm lockfile parser
fix: skip local pnpm path dependencies
docs: document npm preview wrapper limits
test: cover feature-gated pnpm CLI scans
chore: add npm wrapper CI checks
refactor: extract advisory deduplication
```

권장 commit type:

- `feat`: 사용자에게 보이는 기능
- `fix`: bug fix
- `docs`: 문서만 변경
- `test`: test 또는 fixture
- `chore`: CI, metadata, release prep, runtime과 무관한 maintenance
- `refactor`: 의도한 behavior change 없는 내부 구조 변경

## 커밋 Hygiene

- commit 범위를 작게 유지합니다.
- 무엇이 바뀌었는지만이 아니라 왜 바뀌었는지 설명합니다.
- behavior change에는 test를 포함합니다.
- parser change에는 fixture update를 포함합니다.
- CLI, output schema, install flow, release behavior가 바뀌면 documentation update를 포함합니다.
- secret, token, private key, `.env`, local registry credential, 실제 고객/프로젝트 lockfile을 commit하지 않습니다.
- release policy가 명시적으로 요구하지 않는 한 npm tarball을 commit하지 않습니다.

## Pull Request 체크리스트

pull request를 열기 전에 관련 check를 실행합니다.

Rust scanner 변경:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo test --features test-utils
cargo build
```

npm wrapper 변경:

```bash
npm test
npm run lint --if-present
npm pack --dry-run
```

문서만 변경한 경우 최소 다음을 실행합니다.

```bash
rg -n "TODO|OWNER|publish|prebuilt" README.md docs *.md
```

그리고 변경한 link가 올바른지 확인합니다.

## Version Branch

version 준비 시에만 version branch를 사용합니다.

```text
version/0.1.0
```

Version branch에 포함할 수 있는 항목:

- version update
- changelog update
- release checklist update
- package metadata finalization
- documentation correction
- final CI fix

Version branch에는 관련 없는 feature work를 포함하지 않습니다.

Merge 순서:

1. 완료된 `feature/*` branch를 `main`으로 merge합니다.
2. release scope가 freeze되면 `main`에서 `version/<version>` branch를 만듭니다.
3. `version/<version>`에는 release stabilization 변경만 적용합니다.
4. release approval 후 `version/<version>`을 다시 `main`으로 merge합니다.

## npm Publishing 경계

일반 feature branch에서 `npm publish`를 실행하지 않습니다.

현재 preview 단계에서는:

- npm package publish는 manual이며 release owner approval이 필요합니다.
- publish 전 npm provenance policy를 확인해야 합니다.
- package는 아직 prebuilt Rust binary를 포함하지 않습니다.
- prebuilt binary가 없다면 README와 release note가 preview limitation을 명확히 설명해야 합니다.

## Agent Notes

자동화 agent가 이 저장소에서 작업할 때:

- branch name이나 commit grouping을 제안하기 전에 이 문서를 확인합니다.
- 사용자가 명시적으로 commit을 요청하지 않으면 commit을 만들지 않습니다.
- 사용자가 명시적으로 push를 요청하지 않으면 branch를 push하지 않습니다.
- 명시적 지시와 확인 없이 npm package, GitHub release, crate를 publish하지 않습니다.
- 관련 없는 local change를 보존합니다.
