# 변경 내역

[English](CHANGELOG.md)

`sec-issue-finder`의 주요 변경 사항은 이 파일에 기록합니다.

이 형식은 Keep a Changelog를 참고하며, 초기 공개 릴리스 이후에는 semantic versioning을 따릅니다.

## [0.1.0] - Preview

### 추가됨

- 지원되는 dependency lockfile을 스캔하는 Rust CLI
- npm `package-lock.json` v2/v3 파싱
- registry npm 의존성 중심의 best-effort `pnpm-lock.yaml` 파싱
- OSV `/v1/querybatch` 보안 권고 조회
- 내부 Finding 모델 정규화
- table 및 JSON reporter
- `--fail-on` CI 정책 임계값
- `--no-dev` 필터링
- report의 source lockfile metadata
- `scif` 명령을 제공하는 local npm wrapper
- Rust CLI를 실행하고 JSON 출력을 파싱하는 최소 Node.js `scan()` API
- 릴리스 및 로컬 검증 문서

### 보안

- lockfile은 신뢰할 수 없는 입력으로 취급합니다.
- package manager 명령과 dependency lifecycle script를 실행하지 않습니다.
- 테스트는 mocked OSV 응답을 사용하며 실제 OSV API를 호출하지 않습니다.

### 제한사항

- npm wrapper는 preview/local validation 중심입니다.
- npm package는 아직 prebuilt Rust binary를 포함하지 않습니다.
- Rust가 없거나 `PATH`에 기존 `sec-issue-finder` binary가 없는 환경에서 바로 공개 npm install로 사용하는 것은 아직 의도한 배포 방식이 아닙니다.
- pnpm 지원은 registry npm 의존성에 초점을 둡니다. local, workspace, link, file, path-like dependency는 registry version을 확인할 수 없으면 skip될 수 있습니다.
- advisory coverage는 public OSV data에 의존하며 완전한 취약점 탐지를 보장하지 않습니다.
- auto-fix, SARIF, CycloneDX SBOM, offline advisory cache, 추가 ecosystem은 아직 구현되지 않았습니다.
