# sec-issue-finder

[English README](README.md) | [온보딩 가이드](docs/onboarding.ko.md)

`sec-issue-finder`는 알려진 오픈 소스 보안 권고를 기준으로 의존성 lockfile을 스캔하는 Rust CLI입니다.

현재는 npm `package-lock.json`과 registry npm 의존성 중심의 `pnpm-lock.yaml` 파일을 읽고, 설치된 의존성을 정규화한 뒤 OSV에 질의하며, 결과를 표 또는 JSON 형식으로 출력합니다. 로컬 개발 환경 점검과 CI 정책 게이트에 사용하는 것을 목표로 합니다.

결과는 공개 보안 권고 데이터베이스와 lockfile에 포함된 패키지/버전 정보에 의존합니다. 이 도구는 모든 취약점을 완전하게 탐지한다고 주장하지 않습니다.

## 현재 지원 범위

- 지원 lockfile:
  - npm `package-lock.json` v2/v3
  - pnpm `pnpm-lock.yaml`의 registry npm 의존성
- OSV `/v1/querybatch`를 통한 보안 권고 조회
- 사람이 읽기 쉬운 표 출력
- 자동화에 적합한 JSON 출력
- `--fail-on` CI 정책 임계값: `low`, `moderate`, `medium`, `high`, `critical`
- 기본적으로 dev 의존성을 포함하며, `--no-dev`로 dev 의존성을 제외할 수 있음

## 이 도구가 하는 일

- 지원되는 npm ecosystem lockfile을 감지하고 파싱합니다.
- 설치된 패키지 이름, 버전, 직접/전이 의존성 여부, dev/prod 범위를 추출합니다.
- 해당 패키지 버전에 영향을 주는 알려진 취약점을 OSV에 질의합니다.
- OSV 응답을 내부 Finding 형식으로 정규화합니다.
- 출력 순서를 결정적으로 정렬합니다.
- `--fail-on`이 설정되어 있고 조건에 맞는 Finding이 있으면 0이 아닌 종료 코드로 종료합니다.

## 이 도구가 하지 않는 일

- 패키지를 설치하거나 패키지 매니저 명령을 실행하지 않습니다.
- 의존성 lifecycle script를 실행하지 않습니다.
- 취약한 의존성을 자동 수정하지 않습니다.
- pull request를 생성하지 않습니다.
- 악성코드, typosquatting, 설치 스크립트 행위 분석을 수행하지 않습니다.
- 아직 SBOM을 생성하지 않습니다.
- 아직 로컬 오프라인 취약점 데이터베이스를 유지하지 않습니다.
- 현재 Dart, Rust, Yarn, Bun, Python lockfile은 지원하지 않습니다.

## 설치

필수 조건:

- Rust 2021 호환 toolchain
- Cargo

### 소스에서 설치

이 저장소에서 Rust CLI 설치:

```bash
cargo install --path .
```

설치하지 않고 실행:

```bash
cargo run -- scan
```

### npm 전역 설치

패키지 공개 후 사용할 예정인 npm 전역 설치 명령:

```bash
npm install -g @zzozorang/sec-issue-finder
scif scan
```

Preview 제한사항: 아직 public npm 배포는 진행하지 않았습니다. main package는 optional platform package에서 prebuilt binary를 찾으므로, 이 설치 경로를 사용자에게 제공하려면 platform package를 먼저 빌드, smoke test, publish해야 합니다.

### npm 프로젝트 설치

패키지 공개 후 사용할 예정인 프로젝트 로컬 설치 명령:

```bash
npm install -D @zzozorang/sec-issue-finder
npx scif scan
```

공개 배포 전 검증은 [docs/scif-local-testing.ko.md](docs/scif-local-testing.ko.md)의 local file dependency 또는 tarball workflow를 사용하세요.

### pnpm 프로젝트 설치

패키지 공개 후 사용할 예정인 pnpm 프로젝트 로컬 설치 명령:

```bash
pnpm add -D @zzozorang/sec-issue-finder
pnpm exec scif scan
```

이 방식도 현재 preview 제한사항의 영향을 받습니다. 사용자에게 제공하려면 platform package publish가 먼저 완료되어야 합니다.

## 로컬 사용법

현재 디렉터리의 지원되는 lockfile을 자동 감지해 스캔:

```bash
sec-issue-finder scan
```

npm lockfile 스캔:

```bash
sec-issue-finder scan --lockfile package-lock.json
```

pnpm lockfile 스캔:

```bash
sec-issue-finder scan --lockfile pnpm-lock.yaml
```

pnpm lockfile을 스캔하고 JSON 출력:

```bash
sec-issue-finder scan --lockfile pnpm-lock.yaml --format json
```

dev 의존성 제외:

```bash
sec-issue-finder scan --no-dev
```

high 또는 critical Finding이 있을 때 실패:

```bash
sec-issue-finder scan --lockfile pnpm-lock.yaml --fail-on high
```

low 이상의 알려진 심각도 Finding이 있으면 실패:

```bash
sec-issue-finder scan --fail-on low
```

심각도가 unknown인 Finding은 기본적으로 정책 실패를 일으키지 않습니다.

## 로컬 scif Wrapper 테스트

이 저장소에는 짧은 `scif` 명령을 제공하는 npm wrapper가 포함되어 있습니다. public npm publish 전에는 두 가지 방식으로 검증할 수 있습니다.

- source checkout 테스트: wrapper가 `target/release/sec-issue-finder` 또는 `target/debug/sec-issue-finder`를 fallback으로 사용합니다.
- local prebuilt tarball 테스트: wrapper가 `@zzozorang/sec-issue-finder-darwin-arm64` 같은 현재 플랫폼용 package를 찾아 실행합니다.

npm package는 아직 preview 단계입니다. Rust 없이 public npm install만으로 사용하려면 platform package artifact를 먼저 빌드, smoke test, publish해야 합니다.

자세한 절차는 [docs/scif-local-testing.ko.md](docs/scif-local-testing.ko.md)를 참고하세요.

- `npm link` smoke test
- `npm install -D ../sec-finder`와 `npx scif ...`
- `pnpm add -D ../sec-finder`와 `pnpm exec scif ...`
- publish 없이 `npm pack` tarball 설치 테스트

prebuilt platform package를 포함한 local tarball 테스트는 [docs/npm-prebuilt-smoke-test.md](docs/npm-prebuilt-smoke-test.md)를 참고하세요.

## Lockfile 자동 감지

`--lockfile`을 제공하지 않은 경우:

- `package-lock.json`만 있으면 해당 파일을 스캔합니다.
- `pnpm-lock.yaml`만 있으면 해당 파일을 스캔합니다.
- `package-lock.json`과 `pnpm-lock.yaml`이 둘 다 있으면 ambiguity error를 반환하고 `--lockfile`을 명시하라고 안내합니다.

이 동작은 여러 package manager lockfile이 있는 프로젝트에서 의도하지 않은 파일을 조용히 스캔하는 것을 피하기 위한 것입니다.

## OSV Ecosystem 매핑

pnpm은 npm 패키지를 설치합니다. 따라서 `pnpm-lock.yaml`에서 파싱한 의존성은 별도의 `"pnpm"` ecosystem이 아니라 OSV ecosystem `"npm"`으로 질의합니다.

이 스캐너에는 별도의 OSV ecosystem `"pnpm"`이 없습니다. advisory ecosystem과 source lockfile은 별도로 추적됩니다.

보고서에는 source lockfile 정보가 포함되어 의존성이 `package-lock.json`에서 왔는지 `pnpm-lock.yaml`에서 왔는지 확인할 수 있습니다.

## CI 사용법

셸 단계 예시:

```bash
sec-issue-finder scan --format table --fail-on high
```

JSON 아티팩트 생성 예시:

```bash
sec-issue-finder scan --format json --fail-on high > sec-issue-finder-report.json
```

도구 설치 후 GitHub Actions 단계 예시:

```yaml
- name: Scan npm dependencies
  run: sec-issue-finder scan --lockfile package-lock.json --format table --fail-on high
```

`--fail-on`을 사용하면 설정된 임계값 이상인 Finding이 있을 때 명령이 실패 상태로 종료됩니다. `--fail-on`이 없으면 Finding은 보고되지만 종료 코드 1을 발생시키지 않습니다.

## JSON 출력

출력 형태 예시:

```json
{
  "schema_version": "1.0",
  "generated": {
    "tool": "sec-issue-finder",
    "format": "json"
  },
  "summary": {
    "total": 1,
    "critical": 0,
    "high": 1,
    "moderate": 0,
    "medium": 0,
    "low": 0,
    "unknown": 0,
    "direct": 1,
    "transitive": 0,
    "prod": 1,
    "dev": 0
  },
  "findings": [
    {
      "severity": "high",
      "package": {
        "name": "example-package",
        "installed_version": "1.0.0",
        "ecosystem": "npm",
        "package_url": "pkg:npm/example-package@1.0.0",
        "source_file": "pnpm-lock.yaml"
      },
      "advisory": {
        "id": "GHSA-example",
        "aliases": ["CVE-0000-0000"],
        "source": "osv",
        "summary": "Example advisory summary",
        "details": null,
        "url": "https://example.test/advisory"
      },
      "dependency_type": "direct",
      "scope": "prod",
      "fixed_versions": ["1.0.1"],
      "references": ["https://example.test/advisory"]
    }
  ]
}
```

정확한 Finding은 스캔 시점에 OSV에서 제공되는 데이터에 따라 달라집니다.

## 보안 모델

- lockfile 내용은 신뢰할 수 없는 입력으로 취급합니다.
- 파서는 lockfile을 데이터로 읽으며 스크립트를 실행하지 않습니다.
- 스캐너는 패키지 이름을 사용해 shell 명령을 실행하지 않습니다.
- 테스트는 보안 권고 응답을 mock 처리하며 실제 OSV API를 호출하지 않습니다.
- 향후 오프라인 캐시가 추가되기 전까지 런타임 스캔은 공개 보안 권고 서비스를 질의합니다.
- 현재 OSV 사용에는 secret이나 token이 필요하지 않습니다.

## 제한 사항

- 보안 권고 커버리지는 OSV와 upstream 공개 보안 권고 데이터에 의존합니다.
- OSV의 가용성, rate limit, 응답 품질이 스캔 결과에 영향을 줄 수 있습니다.
- 심각도 정규화는 보수적으로 동작하며, 심각도 데이터가 없거나 인식되지 않으면 `unknown`으로 보고할 수 있습니다.
- 현재는 npm ecosystem lockfile인 `package-lock.json`과 `pnpm-lock.yaml`만 지원합니다.
- pnpm 지원은 현재 registry npm 의존성에 초점을 둡니다.
- pnpm local `workspace:`, `link:`, `file:` 의존성은 registry package version이 없으면 skip될 수 있습니다.
- pnpm peer dependency suffix는 가능한 경우 정규화합니다. 예를 들어 `react-dom@18.2.0(react@18.2.0)`은 `react-dom@18.2.0`으로 처리됩니다.
- 스캐너는 취약점이 애플리케이션에서 실제로 도달 가능하거나 악용 가능하다는 것을 증명하지 않습니다.
- 이 스캐너는 의존성 검토, 패치 테스트, 더 넓은 공급망 보안 통제를 대체하지 않습니다.

## 로드맵

- Dart `pubspec.lock`
- Rust `Cargo.lock`
- `yarn.lock`
- `bun.lock`
- SARIF 출력
- CycloneDX SBOM 출력
- GitHub Actions 통합
- 오프라인 보안 권고 캐시

로드맵 항목은 계획된 작업이며, 현재 지원 범위에 문서화되어 있지 않다면 아직 지원되는 기능이 아닙니다.

## 기여

기여를 환영합니다. parser fixture, OSV 응답 edge case, reporter 출력 테스트, CI 통합 예시 등이 특히 도움이 됩니다.

프로젝트가 처음이라면 [온보딩 가이드](docs/onboarding.ko.md)부터 읽어보세요. 영문 버전은 [docs/onboarding.md](docs/onboarding.md)에 있습니다.

릴리스 전 점검은 [docs/release.ko.md](docs/release.ko.md)를 참고하세요. 변경 내역은 [CHANGELOG.ko.md](CHANGELOG.ko.md)에 정리되어 있습니다.

브랜치와 커밋 전략은 [docs/git-workflow.ko.md](docs/git-workflow.ko.md)를 참고하세요.

pull request를 열기 전에 [CONTRIBUTING.ko.md](CONTRIBUTING.ko.md)를 읽어 주세요. 취약점 제보는 [SECURITY.ko.md](SECURITY.ko.md)의 비공개 절차를 사용해 주세요.

변경 사항을 제출하기 전에 다음을 실행하세요.

```bash
cargo fmt
cargo test
cargo clippy --all-targets --all-features
```

변경은 집중된 범위로 유지하고, 동작 변경에는 테스트를 추가하며, 테스트에서 실제 네트워크 호출을 피해주세요.
