# v0.1.0 릴리스 체크리스트

[English](release-checklist.md)

이 checklist는 `sec-issue-finder`의 초기 open source release를 위한 것입니다.

## 범위 확인

- npm `package-lock.json`을 지원합니다.
- `pnpm-lock.yaml` 지원은 registry npm dependency를 대상으로 하며, registry version을 확인할 수 없는 local/workspace/path dependency는 skip될 수 있습니다.
- OSV 보안 권고 조회가 유일하게 구현된 advisory provider입니다.
- table과 JSON이 유일하게 구현된 output format입니다.
- `--fail-on`이 구현된 CI policy mechanism입니다.
- auto-fix는 구현되지 않았습니다.
- Dart, Rust, Yarn, Bun, Python, SARIF, CycloneDX, GitHub Actions integration은 release 전에 구현되지 않았다면 future work입니다.

## Pre-Release 검증

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

- README 사용법이 현재 CLI와 일치합니다.
- 문서가 완전한 취약점 coverage를 주장하지 않습니다.
- 문서가 미지원 ecosystem을 구현된 것처럼 말하지 않습니다.
- 테스트가 실제 OSV API를 호출하지 않습니다.
- `Cargo.toml` package metadata가 완성되어 있습니다.
- `LICENSE`, `SECURITY.md`, `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`가 존재합니다.

npm-specific release check, tarball install test, account policy check는 [docs/release.ko.md](release.ko.md)를 사용하세요.

## Cargo Publish 체크리스트

- `Cargo.toml`의 repository URL이 실제 GitHub repository와 일치하는지 확인합니다.
- crates.io에서 crate name 사용 가능 여부를 확인합니다.
- 포함 파일 확인:

```bash
cargo package --list
```

- package build:

```bash
cargo package
```

- publish dry run:

```bash
cargo publish --dry-run
```

- 준비되면 publish:

```bash
cargo publish
```

## GitHub Release 체크리스트

- project policy가 signed tag를 요구한다면 signed tag를 만들고 push합니다.
- tag format:

```bash
git tag v0.1.0
git push origin v0.1.0
```

- `v0.1.0` GitHub release를 만듭니다.
- 포함 내용:
  - 지원 ecosystem: npm `package-lock.json`
  - registry npm dependency 중심의 `pnpm-lock.yaml` 지원. registry version이 없는 local/workspace/path dependency는 skip될 수 있음
  - OSV lookup 지원
  - table 및 JSON reporter
  - `--fail-on` policy behavior
  - 알려진 제한사항
  - `SECURITY.md` 링크
- future roadmap item을 release된 기능처럼 설명하지 않습니다.

## Future Work

- npm wrapper를 위한 prebuilt binary distribution
- Dart `pubspec.lock`
- Rust `Cargo.lock`
- `yarn.lock`
- `bun.lock`
- SARIF output
- CycloneDX SBOM output
- GitHub Actions integration
- offline advisory cache
