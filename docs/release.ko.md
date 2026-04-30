# 릴리스 체크리스트

[English](release.md)

이 문서는 `sec-issue-finder`를 public npm release 직전에 점검하기 위한 checklist입니다. checklist일 뿐이며, 모든 blocking item이 해결되고 release owner가 명시적으로 승인하기 전까지 `npm publish`를 실행하지 마세요.

## 현재 릴리스 범위

대상 버전: `0.1.0` preview.

포함:

- `sec-issue-finder` Rust CLI
- `scif` npm wrapper 명령
- npm `package-lock.json` 스캔
- registry npm dependency 중심의 best-effort `pnpm-lock.yaml` 스캔
- OSV 보안 권고 조회
- table 및 JSON 출력
- `--fail-on` CI policy
- Rust CLI를 실행하는 local Node.js `scan()` API

포함하지 않음:

- prebuilt binary 배포
- auto-fix
- GitHub release 자동화
- npm provenance 자동화
- SARIF
- CycloneDX SBOM
- offline advisory cache
- 완전한 pnpm workspace coverage
- 완전한 취약점 탐지 보장

## Preview Binary 배포 경고

main npm package에는 JavaScript wrapper가 포함되며, optional platform package에서 prebuilt Rust binary를 찾습니다.

넓은 public npm release 전에 다음 중 하나를 확인해야 합니다.

- main package보다 지원 platform package를 먼저 publish합니다.
- 특정 target user를 위한 platform package가 없다면 release를 명확히 preview로 표시합니다.
- Rust binary를 안전하게 download 또는 build하는 installer 전략을 문서화합니다.

프로젝트가 관리하지 않는 package를 가리키는 `optionalDependencies`가 있는 상태로 main package를 publish하지 마세요.

## 필수 로컬 점검

저장소 root에서 모두 실행합니다.

```bash
npm test
npm run build --if-present
npm run lint --if-present
npm pack --dry-run
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo test --features test-utils
cargo build
```

확인:

- [ ] 모든 명령이 로컬에서 통과합니다.
- [ ] CI가 동일한 Rust 및 npm wrapper check를 실행합니다.
- [ ] 테스트가 실제 OSV API를 호출하지 않습니다.
- [ ] feature-gated CLI test가 `cargo test --features test-utils`로 실행됩니다.

## Tarball 설치 테스트

tarball 생성:

```bash
cargo build
npm pack
```

### npm Tarball 테스트

```bash
mkdir /tmp/scif-test
cd /tmp/scif-test
npm init -y
npm install /path/to/zzozorang-sec-issue-finder-0.1.0.tgz
npx scif scan --help
```

`package-lock.json`이 있는 실제 npm 프로젝트에서 추가 확인:

```bash
npx scif scan --lockfile package-lock.json
npx scif scan --lockfile package-lock.json --format json
npx scif scan --lockfile package-lock.json --fail-on high
```

### pnpm Tarball 테스트

```bash
mkdir /tmp/scif-pnpm-test
cd /tmp/scif-pnpm-test
pnpm init
pnpm add -D /path/to/zzozorang-sec-issue-finder-0.1.0.tgz
pnpm exec scif scan --help
```

`pnpm-lock.yaml`이 있는 실제 pnpm 프로젝트에서 추가 확인:

```bash
pnpm exec scif scan --lockfile pnpm-lock.yaml
pnpm exec scif scan --lockfile pnpm-lock.yaml --format json
pnpm exec scif scan --lockfile pnpm-lock.yaml --fail-on high
```

확인:

- [ ] publish 없이 npm tarball install이 동작합니다.
- [ ] publish 없이 pnpm tarball install이 동작합니다.
- [ ] npm과 pnpm에서 `scif scan --help`가 동작합니다.
- [ ] fixture 또는 실제 test project에서 npm/pnpm lockfile scan이 동작합니다.
- [ ] JSON output이 stdout의 valid JSON입니다.
- [ ] `--fail-on`이 기대한 exit code를 반환합니다.
- [ ] missing binary error가 이해 가능합니다.

## Package Contents 확인

실행:

```bash
npm pack --dry-run
```

확인:

- [ ] `LICENSE`가 포함됩니다.
- [ ] `README.md`가 포함됩니다.
- [ ] `package.json`이 포함됩니다.
- [ ] `npm/` 아래 runtime file이 포함됩니다.
- [ ] `.env` file이 포함되지 않습니다.
- [ ] `target/`이 포함되지 않습니다.
- [ ] `.github/`이 포함되지 않습니다.
- [ ] `tests/fixtures/`가 포함되지 않습니다.
- [ ] packaging strategy가 의도적으로 바뀐 경우가 아니라면 Rust source file이 포함되지 않습니다.
- [ ] local test artifact가 포함되지 않습니다.

## Metadata 확인

`package.json`의 final value를 확인합니다.

- [ ] `name`
- [ ] `version`
- [ ] `description`
- [ ] `license`
- [ ] `repository`
- [ ] `bugs`
- [ ] `homepage`
- [ ] `bin`
- [ ] `main`
- [ ] `types`
- [ ] `files`

release 전에 repository URL이 public GitHub repository와 일치하는지 확인하세요.

## README 확인

확인:

- [ ] README 설치 방법이 실제 배포 방식과 일치합니다.
- [ ] README가 현재 preview binary limitation을 설명합니다.
- [ ] README가 npm 사용법을 문서화합니다.

```bash
npm install -g @zzozorang/sec-issue-finder
scif scan
npm install -D @zzozorang/sec-issue-finder
npx scif scan
```

- [ ] README가 pnpm 사용법을 문서화합니다.

```bash
pnpm add -D @zzozorang/sec-issue-finder
pnpm exec scif scan
```

- [ ] README가 미지원 ecosystem을 구현된 것처럼 말하지 않습니다.
- [ ] README가 완전한 취약점 coverage를 주장하지 않습니다.
- [ ] README가 auto-fix가 있다고 암시하지 않습니다.

## Release Drafter 확인

Release Drafter는 `main`에 변경 사항이 push되면 GitHub release draft를 갱신합니다.

없다면 maintainer가 repository label을 먼저 만들어야 합니다.

```text
feature
enhancement
bug
fix
security
documentation
maintenance
dependencies
ci
skip-changelog
breaking
```

release publish 전에 확인:

- [ ] merge된 pull request title이 release note에 적합합니다.
- [ ] merge된 pull request에 적절한 label이 있습니다: `feature`, `bug`, `security`, `documentation`, `maintenance`, `dependencies`, `ci`
- [ ] release note에 포함하지 않을 pull request에는 `skip-changelog` label이 있습니다.
- [ ] 생성된 GitHub release draft를 maintainer가 검토했습니다.
- [ ] 검토된 release draft를 기준으로 `CHANGELOG.md`와 `CHANGELOG.ko.md`를 수동 업데이트했습니다.
- [ ] release draft가 미지원 ecosystem, 완전한 vulnerability coverage, auto-fix를 주장하지 않습니다.

Release Drafter는 npm package를 publish하지 않으며 manual release checklist를 대체하지 않습니다.

## Repository Maintenance 확인

release 전에 repository automation 설정을 확인합니다.

- [ ] CODEOWNERS가 의도한 maintainer user 또는 organization team을 가리킵니다.
- [ ] organization team을 사용한다면 해당 team이 repository write access를 가지고 있습니다.
- [ ] project policy라면 branch protection 또는 ruleset에서 CODEOWNER review를 요구합니다.
- [ ] Dependabot이 Cargo, npm, GitHub Actions에 대해 활성화되어 있습니다.
- [ ] Dependabot label이 존재합니다: `dependencies`, `rust`, `npm`, `github-actions`
- [ ] Dependabot pull request는 일반 code change처럼 review하며 CI 없이 auto-merge하지 않습니다.

## npm 계정 및 Publishing 정책

publish 전 확인:

- [ ] npm account ownership 확인
- [ ] npm account 2FA 활성화 확인
- [ ] organization/package access policy 확인
- [ ] npm provenance policy 확인
- [ ] 이번 release에서 provenance가 필요한지 확인
- [ ] package name availability 확인
- [ ] final version number 확인

release owner가 승인한 뒤에만 사용할 publish command:

```bash
npm publish
```

provenance가 필요하고 지원되는 CI 환경에서 release를 실행한다면, 승인된 project policy를 따르세요. release 중 임의로 provenance setting을 만들지 마세요.

## Publish 후 확인

publish 후 clean temporary project에서 확인합니다.

```bash
npm install -g @zzozorang/sec-issue-finder
scif scan --help
```

```bash
npm install -D @zzozorang/sec-issue-finder
npx scif scan --help
```

```bash
pnpm add -D @zzozorang/sec-issue-finder
pnpm exec scif scan --help
```

prebuilt binary가 아직 구현되지 않았다면, 이 명령들은 사용자가 binary requirement를 이해하는 명확한 preview release에서만 사용해야 합니다.
