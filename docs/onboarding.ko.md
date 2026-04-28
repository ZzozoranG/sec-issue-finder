# sec-issue-finder 온보딩 가이드

이 문서는 `sec-issue-finder` 프로젝트에 처음 참여하는 개발자가 빠르게 맥락을 잡고 안전하게 기여할 수 있도록 돕기 위한 안내서입니다.

영문 버전: [onboarding.md](onboarding.md)

처음 온 사람이라면 아래 순서대로 읽는 것을 권장합니다.

1. 이 문서 전체를 가볍게 읽습니다.
2. 루트의 [README.md](../README.md) 또는 [README.ko.md](../README.ko.md)를 읽어 사용자 관점의 기능을 확인합니다.
3. [CONTRIBUTING.md](../CONTRIBUTING.md) 또는 [CONTRIBUTING.ko.md](../CONTRIBUTING.ko.md)를 읽어 기여 규칙을 확인합니다.
4. 로컬에서 `cargo test`와 `cargo test --features test-utils`를 실행해 현재 상태가 깨끗한지 확인합니다.
5. 수정하려는 영역의 테스트와 fixture를 먼저 살펴본 뒤 변경을 시작합니다.

## 프로젝트 한 줄 요약

`sec-issue-finder`는 dependency lockfile을 읽고, 설치된 패키지와 버전을 정규화한 뒤, OSV 같은 공개 보안 권고 데이터베이스를 조회해 알려진 취약점을 보고하는 Rust CLI 도구입니다.

현재 v0.1.0 범위는 다음에 집중합니다.

- npm `package-lock.json` v2/v3
- pnpm `pnpm-lock.yaml`의 registry npm dependency
- OSV `/v1/querybatch`
- table 및 JSON 출력
- `--fail-on` 기반 CI 실패 정책

중요한 제한도 있습니다.

- 완전한 취약점 탐지를 보장하지 않습니다.
- 결과는 OSV와 공개 advisory 데이터의 품질과 최신성에 의존합니다.
- package manager 명령을 실행하지 않습니다.
- dependency lifecycle script를 실행하지 않습니다.
- auto-fix를 제공하지 않습니다.
- pnpm workspace/local/path dependency 처리는 보수적이며 best-effort입니다.

## 개발 환경 준비

필요한 도구:

- Rust stable toolchain
- Cargo
- `rustfmt`
- `clippy`

이 프로젝트의 Rust 최소 버전은 [Cargo.toml](../Cargo.toml)의 `rust-version`을 기준으로 합니다.

기본 확인 명령:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo test --features test-utils
cargo build
```

개발 중에는 보통 아래 명령을 자주 사용합니다.

```bash
cargo fmt
cargo test
cargo clippy --all-targets --all-features
```

## 로컬에서 실행해 보기

설치하지 않고 실행:

```bash
cargo run -- scan
```

빌드된 바이너리로 실행:

```bash
cargo build
./target/debug/sec-issue-finder scan
```

npm lockfile 지정:

```bash
cargo run -- scan --lockfile package-lock.json
```

pnpm lockfile 지정:

```bash
cargo run -- scan --lockfile pnpm-lock.yaml
```

JSON 출력:

```bash
cargo run -- scan --lockfile pnpm-lock.yaml --format json
```

dev dependency 제외:

```bash
cargo run -- scan --lockfile pnpm-lock.yaml --no-dev
```

CI 정책처럼 high 이상에서 실패:

```bash
cargo run -- scan --lockfile pnpm-lock.yaml --fail-on high
```

## 주요 명령과 의미

### `cargo fmt`

Rust 코드 포맷을 적용합니다. PR 전에는 반드시 실행하는 것이 좋습니다.

### `cargo fmt --check`

CI에서 사용하는 포맷 검증 명령입니다. 파일을 수정하지 않고 포맷이 맞는지만 확인합니다.

### `cargo test`

기본 feature set으로 unit test와 integration test를 실행합니다.

기본 테스트는 실제 OSV API를 호출하지 않습니다. OSV 클라이언트 테스트는 mocked transport를 사용하고, scan 테스트도 in-process mock client를 사용합니다.

### `cargo test --features test-utils`

`test-utils` feature를 켠 상태로 테스트를 실행합니다.

이 feature는 CLI integration test에서 spawned binary가 OSV mock response file을 읽을 수 있게 하는 내부 테스트 hook을 활성화합니다.

중요:

- `test-utils`는 일반 빌드에 필요하지 않습니다.
- release build에 켤 기능이 아닙니다.
- `SEC_ISSUE_FINDER_OSV_MOCK_RESPONSE_FILE`은 `test-utils` feature가 켜진 테스트용 빌드에서만 동작합니다.

### `cargo clippy --all-targets --all-features -- -D warnings`

CI에서 사용하는 lint 명령입니다. warning도 실패로 처리합니다.

### `cargo build`

기본 feature set으로 일반 빌드를 수행합니다. 이 명령은 `test-utils`를 켜지 않습니다.

## CI 구성

CI는 [.github/workflows/ci.yml](../.github/workflows/ci.yml)에 정의되어 있습니다.

현재 실행 순서:

1. `cargo fmt --check`
2. `cargo clippy --all-targets --all-features -- -D warnings`
3. `cargo test`
4. `cargo test --features test-utils`
5. `cargo build`

이 순서에는 의도가 있습니다.

- 먼저 포맷과 lint를 확인합니다.
- 기본 feature set 테스트를 실행합니다.
- `test-utils` feature가 필요한 CLI integration test를 별도로 실행합니다.
- 마지막으로 일반 build가 `test-utils` 없이도 통과하는지 확인합니다.

## 코드 구조

주요 디렉터리와 파일은 다음과 같습니다.

```text
src/
  main.rs
  lib.rs
  cli.rs
  types.rs
  error.rs
  scan.rs
  policy.rs
  ecosystems/
    mod.rs
    npm.rs
    pnpm.rs
    dart.rs
  clients/
    mod.rs
    osv.rs
  analyzers/
    mod.rs
    osv.rs
  reporters/
    mod.rs
    table.rs
    json.rs
tests/
  cli_pnpm.rs
  fixtures/
    npm/
    pnpm/
    scan/
    osv/
docs/
  release-checklist.md
  onboarding.md
  onboarding.ko.md
```

### [src/main.rs](../src/main.rs)

CLI entrypoint입니다.

역할:

- tracing 초기화
- CLI argument parsing
- `scan` 명령 실행
- reporter 출력
- policy 실패를 process exit code로 반영

가능하면 복잡한 로직을 `main.rs`에 넣지 않습니다. CLI layer는 얇게 유지하고, 실제 동작은 `scan`, `policy`, `reporters`, `clients`, `ecosystems` 모듈에 위임합니다.

### [src/cli.rs](../src/cli.rs)

`clap` 기반 CLI 정의가 들어 있습니다.

현재 주요 옵션:

- `scan`
- `--lockfile <path>`
- `--format table|json`
- `--fail-on low|moderate|medium|high|critical`
- `--include-dev`
- `--no-dev`

CLI 옵션을 추가할 때는 다음을 함께 고려해야 합니다.

- parsing test 추가
- README usage 업데이트
- 정책이나 scan config에 반영할지 여부
- JSON output schema에 영향이 있는지 여부

### [src/types.rs](../src/types.rs)

프로젝트의 핵심 domain type이 정의되어 있습니다.

중요한 타입:

- `Dependency`
- `Ecosystem`
- `Severity`
- `Advisory`
- `Finding`
- `ScanConfig`
- `ScanResult`

설계 원칙:

- ecosystem parser는 lockfile-specific 데이터를 `Dependency`로 정규화합니다.
- advisory client는 정규화된 `Dependency`만 받습니다.
- reporter는 `Finding`을 출력할 뿐, 의미를 새로 해석하지 않습니다.

### [src/error.rs](../src/error.rs)

도메인 에러 타입인 `SecFinderError`가 정의되어 있습니다.

원칙:

- library/domain layer에서는 `thiserror` 기반 typed error를 사용합니다.
- CLI entrypoint에서는 필요할 때 `anyhow`를 사용합니다.
- malformed lockfile, missing file, OSV error는 사용자에게 도움이 되는 메시지를 반환해야 합니다.

### [src/scan.rs](../src/scan.rs)

scan pipeline의 중심입니다.

현재 흐름:

1. lockfile path 결정
2. lockfile 종류에 맞는 parser 선택
3. dependency parsing
4. advisory query용 dependency deduplication
5. OSV query
6. OSV result를 internal finding으로 normalization
7. `ScanResult` 반환

중요한 설계:

- advisory query 전 dependency를 deduplicate합니다.
- dedup key는 advisory ecosystem, package name, version입니다.
- pnpm dependency도 OSV에는 ecosystem `"npm"`으로 query합니다.
- duplicate metadata는 보수적으로 merge합니다.
  - 하나라도 direct이면 `direct = true`
  - 하나라도 production이면 `dev = false`

주의:

- deduplication은 importer/path nuance를 잃을 수 있습니다.
- 현재 `Dependency` 모델에는 importer path가 없습니다.
- 향후 workspace reporting을 강화하려면 domain model 확장이 필요합니다.

### [src/ecosystems/mod.rs](../src/ecosystems/mod.rs)

lockfile parser abstraction과 parser selection이 들어 있습니다.

현재 지원:

- `package-lock.json` -> npm parser
- `pnpm-lock.yaml` -> pnpm parser

미래 확장:

- Dart `pubspec.lock`
- Rust `Cargo.lock`
- Yarn `yarn.lock`
- Bun `bun.lock`

새 ecosystem을 추가할 때는 parser가 반드시 `Vec<Dependency>`를 반환하도록 해야 합니다.

### [src/ecosystems/npm.rs](../src/ecosystems/npm.rs)

npm `package-lock.json` v2/v3 parser입니다.

하는 일:

- `packages` object 읽기
- root package metadata에서 direct dependency 판단
- package entry에서 name/version/dev 추출
- scoped package 처리
- `Dependency`로 정규화

하지 않는 일:

- `npm install` 실행
- package script 실행
- OSV query
- auto-fix

### [src/ecosystems/pnpm.rs](../src/ecosystems/pnpm.rs)

pnpm `pnpm-lock.yaml` parser입니다.

현재 범위:

- registry npm dependency 중심의 best-effort parsing
- `importers`, `packages`, `snapshots` 처리
- scoped package 처리
- peer suffix normalize
- `dependencies`, `devDependencies`, `optionalDependencies` 기반 direct/dev 판단
- local/workspace/path-like dependency skip

중요한 pnpm 정책:

- pnpm은 npm package를 설치하므로 `Dependency.ecosystem`은 `Ecosystem::Npm`입니다.
- OSV query도 ecosystem `"npm"`으로 나갑니다.
- `"pnpm"`이라는 별도 OSV ecosystem을 만들지 않습니다.
- local dependency는 registry version을 확정할 수 없으면 skip합니다.

skip되는 대표 형태:

- `file:`
- `link:`
- `workspace:`
- `portal:`
- `path:`
- `../foo`
- `./foo`
- absolute/path-like key
- version이 registry version으로 볼 수 없는 값

주의:

- pnpm workspace와 multi-importer classification은 best-effort입니다.
- 같은 package name/version이 여러 importer에서 등장하면 scan 단계에서 dedup됩니다.
- importer별 reporting은 아직 제공하지 않습니다.

### [src/clients/osv.rs](../src/clients/osv.rs)

OSV `/v1/querybatch` client입니다.

역할:

- `Dependency` list를 OSV querybatch request로 변환
- HTTP request 실행
- OSV response를 typed struct로 parse
- dependency order와 OSV result order 매핑 보존

중요:

- lockfile parsing을 하지 않습니다.
- reporter용 finding normalization을 하지 않습니다.
- raw `serde_json::Value`를 외부로 노출하지 않습니다.

테스트 전략:

- unit test에서는 mocked transport를 사용합니다.
- CLI integration test에서는 `test-utils` feature가 켜졌을 때만 file-based mock response hook을 사용합니다.
- 실제 OSV API를 테스트에서 호출하지 않습니다.

### [src/analyzers/osv.rs](../src/analyzers/osv.rs)

OSV response를 internal `Finding`으로 변환합니다.

역할:

- OSV vulnerability id, aliases, summary, details 추출
- severity normalize
- fixed version 추출
- references 추출
- dependency의 direct/dev/source metadata 보존

정책:

- missing severity는 `Severity::Unknown`으로 처리합니다.
- fixed version이 없으면 빈 list로 처리합니다.
- references가 없으면 빈 list로 처리합니다.

### [src/reporters](../src/reporters)

출력 담당 모듈입니다.

현재 reporter:

- table
- JSON

중요 원칙:

- reporter는 finding을 정렬하고 출력합니다.
- reporter에서 OSV-specific parsing을 하지 않습니다.
- reporter는 finding 의미를 변경하지 않습니다.
- output은 테스트 가능하도록 deterministic해야 합니다.

JSON output은 automation을 위한 schema를 제공합니다.

Table output은 사람이 읽기 쉬운 요약을 제공합니다.

### [src/policy.rs](../src/policy.rs)

CI failure policy를 담당합니다.

현재 정책:

- `--fail-on`이 없으면 finding이 있어도 exit code 1을 만들지 않습니다.
- `--fail-on`이 있으면 threshold 이상 severity가 있을 때 실패합니다.
- `moderate`와 `medium`은 동등하게 취급합니다.
- `unknown` severity는 기본적으로 실패시키지 않습니다.

## 테스트 구조

테스트는 크게 네 종류입니다.

### 1. Parser unit tests

위치:

- [src/ecosystems/npm.rs](../src/ecosystems/npm.rs)
- [src/ecosystems/pnpm.rs](../src/ecosystems/pnpm.rs)

fixture 위치:

- `tests/fixtures/npm`
- `tests/fixtures/pnpm`

검증 예:

- direct dependency
- dev dependency
- transitive dependency
- scoped package
- malformed lockfile
- missing version
- local/path-like dependency skip
- pnpm peer suffix normalization
- pnpm multi-importer handling

### 2. OSV client tests

위치:

- [src/clients/osv.rs](../src/clients/osv.rs)

검증 예:

- request body
- empty dependency list
- non-2xx response
- malformed response
- vulnerability response
- response order mapping

### 3. Scan pipeline tests

위치:

- [src/scan.rs](../src/scan.rs)

검증 예:

- lockfile auto-detection
- ambiguous lockfile error
- pnpm dependency OSV mapping
- advisory query deduplication
- policy interaction
- mocked OSV client behavior

### 4. CLI integration tests

위치:

- [tests/cli_pnpm.rs](../tests/cli_pnpm.rs)

특징:

- 실제 binary를 실행합니다.
- 일부 테스트는 `test-utils` feature가 필요합니다.
- `SEC_ISSUE_FINDER_OSV_MOCK_RESPONSE_FILE`은 feature-gated test hook입니다.
- 실제 OSV API를 호출하지 않습니다.

## Fixture 작성 규칙

fixture는 작고 손으로 작성하는 것이 좋습니다.

좋은 fixture:

- 한 가지 behavior만 검증합니다.
- lockfile 구조가 최소화되어 있습니다.
- dependency 수가 적습니다.
- malformed case가 명확합니다.

피해야 할 것:

- `npm install` 또는 `pnpm install`로 생성한 큰 lockfile
- `node_modules`
- 외부 네트워크가 필요한 fixture
- lifecycle script 실행
- 테스트 의도와 무관한 metadata

새 parser fixture를 추가할 때는 보통 다음 위치 중 하나를 사용합니다.

```text
tests/fixtures/npm/
tests/fixtures/pnpm/
tests/fixtures/scan/
tests/fixtures/osv/
```

## 보안 관련 개발 원칙

이 프로젝트는 lockfile을 untrusted input으로 취급합니다.

반드시 지켜야 할 원칙:

- lockfile 내용을 믿지 않습니다.
- lockfile-derived value에 대해 `unwrap` 또는 `expect`를 피합니다.
- package manager를 실행하지 않습니다.
- dependency name이나 version을 shell command에 전달하지 않습니다.
- dependency lifecycle script를 실행하지 않습니다.
- malformed JSON/YAML은 panic이 아니라 typed error 또는 safe skip으로 처리합니다.
- local/path-like dependency는 registry version을 확정할 수 없으면 skip합니다.
- 테스트에서 실제 OSV API를 호출하지 않습니다.

## pnpm support를 수정할 때 주의할 점

pnpm support는 현재 release scope에서 가장 섬세한 영역입니다.

수정 전 확인할 파일:

- [src/ecosystems/pnpm.rs](../src/ecosystems/pnpm.rs)
- [src/scan.rs](../src/scan.rs)
- [tests/fixtures/pnpm](../tests/fixtures/pnpm)
- [tests/fixtures/scan](../tests/fixtures/scan)
- [tests/cli_pnpm.rs](../tests/cli_pnpm.rs)
- [README.md](../README.md)

주요 기대 동작:

- scoped package `@scope/pkg`는 정상 parsing합니다.
- peer suffix는 가능한 경우 base package/version으로 normalize합니다.
- `file:`, `link:`, `workspace:`, `portal:`, `path:`는 registry version이 없으면 skip합니다.
- relative/absolute path-like key는 bogus dependency를 만들지 않아야 합니다.
- 같은 name/version duplicate는 OSV query 전에 dedup됩니다.
- pnpm dependency는 OSV ecosystem `"npm"`으로 query됩니다.
- `--no-dev`는 dev dependency를 제외하되 production dependency가 남아 있으면 OSV query를 계속 수행합니다.

pnpm 변경 후 권장 테스트:

```bash
cargo test pnpm
cargo test --features test-utils --test cli_pnpm
cargo test
cargo test --features test-utils
```

## OSV client를 수정할 때 주의할 점

OSV client는 외부 API와 닿는 영역이지만 테스트에서는 실제 네트워크를 사용하면 안 됩니다.

수정 시 확인할 점:

- request body가 OSV schema와 맞는지
- dependency order와 response order mapping이 유지되는지
- empty dependency list에서 HTTP request를 보내지 않는지
- non-2xx status가 typed error로 반환되는지
- malformed response가 typed error로 반환되는지
- raw `serde_json::Value`를 public API로 노출하지 않는지

`SEC_ISSUE_FINDER_OSV_MOCK_RESPONSE_FILE` 관련 주의:

- 이 환경 변수는 사용자 기능이 아닙니다.
- `test-utils` feature에서만 컴파일됩니다.
- CLI integration test에서 spawned binary를 mock하기 위한 내부 hook입니다.
- 일반 build나 release build에 의존하면 안 됩니다.

## Reporter를 수정할 때 주의할 점

Reporter 변경은 사용자 output에 직접 영향을 줍니다.

수정 전 확인할 파일:

- [src/reporters/mod.rs](../src/reporters/mod.rs)
- [src/reporters/table.rs](../src/reporters/table.rs)
- [src/reporters/json.rs](../src/reporters/json.rs)

주의할 점:

- output ordering은 deterministic해야 합니다.
- JSON schema를 바꾸면 README 예시도 업데이트해야 합니다.
- source metadata를 제거하면 npm/pnpm 구분이 어려워집니다.
- reporter에서 advisory provider-specific parsing을 하지 않습니다.

현재 정렬 기준은 severity, direct/transitive, prod/dev, package name, advisory id 순서입니다.

## Policy를 수정할 때 주의할 점

Policy 변경은 CI exit code에 영향을 줍니다.

수정 전 확인할 파일:

- [src/policy.rs](../src/policy.rs)
- [src/main.rs](../src/main.rs)
- [src/cli.rs](../src/cli.rs)

현재 정책:

- threshold가 없으면 실패하지 않습니다.
- `low` threshold는 low 이상에서 실패합니다.
- `high` threshold는 high/critical에서 실패합니다.
- `critical` threshold는 critical에서만 실패합니다.
- `moderate`와 `medium`은 동등합니다.
- `unknown`은 기본적으로 실패하지 않습니다.

정책을 바꾸면 CLI behavior와 README를 함께 업데이트해야 합니다.

## 새 ecosystem parser를 추가하는 방법

예를 들어 Dart `pubspec.lock` parser를 추가한다고 가정하면, 대략 다음 순서로 진행합니다.

1. domain model에 필요한 ecosystem이 있는지 확인합니다.
2. `src/ecosystems/<name>.rs`를 추가합니다.
3. `LockfileParser` trait를 구현합니다.
4. parser output은 반드시 `Vec<Dependency>`로 정규화합니다.
5. `src/ecosystems/mod.rs`에서 lockfile filename 기반 parser selection을 추가합니다.
6. advisory client가 이해할 ecosystem mapping을 확인합니다.
7. fixture를 작게 추가합니다.
8. parser unit test를 추가합니다.
9. scan pipeline test를 추가합니다.
10. CLI behavior가 바뀌면 integration test와 README를 업데이트합니다.

중요:

- parser가 advisory API를 호출하면 안 됩니다.
- advisory client가 lockfile을 parse하면 안 됩니다.
- reporter가 ecosystem-specific parsing을 하면 안 됩니다.

## 흔한 작업별 체크리스트

### Lockfile parser를 고칠 때

- [ ] malformed input에서 panic이 나지 않는가?
- [ ] missing file error가 유용한가?
- [ ] missing field를 안전하게 처리하는가?
- [ ] scoped package가 깨지지 않는가?
- [ ] local/path-like dependency가 안전하게 처리되는가?
- [ ] fixture가 작고 의도가 명확한가?
- [ ] `cargo test <ecosystem>`을 실행했는가?
- [ ] user-facing behavior라면 README를 업데이트했는가?

### OSV client를 고칠 때

- [ ] real OSV API를 호출하지 않는 테스트인가?
- [ ] request body test가 있는가?
- [ ] non-2xx test가 있는가?
- [ ] malformed response test가 있는가?
- [ ] response order mapping이 보존되는가?
- [ ] typed response struct를 유지하는가?

### Reporter를 고칠 때

- [ ] output이 deterministic한가?
- [ ] empty findings test가 있는가?
- [ ] one finding test가 있는가?
- [ ] multiple sorted findings test가 있는가?
- [ ] JSON schema 변경 시 README 예시도 바뀌었는가?

### CLI를 고칠 때

- [ ] `clap` parsing test가 있는가?
- [ ] scan pipeline이 얇게 유지되는가?
- [ ] exit code behavior가 명확한가?
- [ ] integration test가 필요한가?
- [ ] README usage가 맞는가?

## PR 전 최종 확인

PR을 열기 전에 최소한 아래를 실행하세요.

```bash
cargo fmt
cargo test
cargo test --features test-utils
cargo clippy --all-targets --all-features
cargo build
```

release에 가까운 변경이라면 CI와 동일하게 warning deny도 확인하세요.

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo test --features test-utils
cargo build
```

## 문서 업데이트 기준

다음에 해당하면 문서를 업데이트해야 합니다.

- CLI option이 추가/변경됨
- supported lockfile이 추가됨
- output schema가 변경됨
- policy behavior가 변경됨
- security model이나 limitation이 달라짐
- release scope가 달라짐

업데이트 대상:

- [README.md](../README.md)
- [README.ko.md](../README.ko.md)
- [CONTRIBUTING.md](../CONTRIBUTING.md)
- [CONTRIBUTING.ko.md](../CONTRIBUTING.ko.md)
- [docs/release-checklist.md](release-checklist.md)
- 필요한 경우 이 문서

## 현재 release readiness 상태

현재 v0.1.0 기준으로 다음은 준비되어 있습니다.

- npm `package-lock.json` parser
- pnpm `pnpm-lock.yaml` best-effort registry npm dependency parser
- OSV querybatch client
- OSV result normalization
- table reporter
- JSON reporter
- `--fail-on` policy
- default feature tests
- `test-utils` feature-gated CLI integration tests
- CI에서 두 test mode 실행
- conservative pnpm documentation

release 전 남은 대표 확인 사항:

- `Cargo.toml`의 repository placeholder를 실제 GitHub URL로 바꾸기
- `cargo publish --dry-run` 실행
- GitHub release note 작성
- tag 생성 전 release checklist 확인

## 도움을 요청할 때 포함하면 좋은 정보

이슈나 PR에서 질문할 때는 아래 정보를 포함하면 빠르게 답을 받을 수 있습니다.

- 실행한 명령
- 기대한 결과
- 실제 결과
- 사용한 lockfile 종류
- 관련 fixture 경로
- 실패한 test 이름
- 에러 메시지
- 변경한 파일 목록

예:

```text
Command: cargo test pnpm_peer_suffix
Expected: react-dom@18.2.0으로 normalize
Actual: dependency가 생성되지 않음
Fixture: tests/fixtures/pnpm/peer-suffix.yaml
Changed files: src/ecosystems/pnpm.rs
```

## 마지막으로 기억할 것

이 프로젝트의 핵심은 “lockfile을 안전하게 읽고, dependency를 정규화하고, advisory provider와 reporter를 분리하는 것”입니다.

헷갈릴 때는 아래 원칙으로 돌아오면 됩니다.

- parser는 lockfile만 parse합니다.
- client는 advisory API만 다룹니다.
- analyzer는 provider response를 internal finding으로 바꿉니다.
- reporter는 finding을 출력합니다.
- policy는 exit 여부만 판단합니다.
- tests는 외부 네트워크에 의존하지 않습니다.
- 문서는 지원 범위를 과장하지 않습니다.
