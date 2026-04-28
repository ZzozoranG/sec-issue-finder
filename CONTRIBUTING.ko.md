# 기여 가이드

`sec-issue-finder`를 개선하는 데 도움을 주셔서 감사합니다.

## 범위

이 프로젝트는 현재 OSV 보안 권고 조회를 사용하는 npm `package-lock.json` 스캔을 지원합니다. parser, scan, reporter, test가 병합되기 전까지는 향후 ecosystem 작업을 이미 구현된 기능처럼 설명하지 말아 주세요.

## 개발

`Cargo.toml`의 `rust-version`과 호환되는 최신 stable Rust toolchain을 사용하세요.

```bash
cargo fmt
cargo test
cargo clippy --all-targets --all-features
cargo build
```

pull request를 열기 전에 다음을 실행하세요.

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo build
```

## 테스트

- 테스트는 결정적으로 동작해야 합니다.
- 테스트에서 실제 OSV API를 호출하지 마세요.
- fixture 프로젝트 안에서 package manager를 실행하지 마세요.
- lockfile parser 동작에는 작고 손으로 작성한 fixture를 추가하는 것을 선호합니다.
- 신뢰할 수 없는 파일을 파싱할 때는 잘못된 입력과 누락된 필드 사례를 커버하세요.

## Pull Request

좋은 pull request는 범위가 명확하며 다음을 포함합니다.

- 동작 변경에 대한 간결한 설명
- 새 동작에 대한 테스트
- 사용자에게 보이는 동작이 바뀌는 경우 문서 업데이트
- 관련 없는 포맷 변경 최소화

## 보안에 민감한 변경

이 프로젝트의 취약점을 제보하려면 공개 issue를 열지 말고 `SECURITY.ko.md`의 절차를 사용해 주세요.
