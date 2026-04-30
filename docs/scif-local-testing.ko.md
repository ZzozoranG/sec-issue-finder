# Local scif 테스트

[English](scif-local-testing.md)

이 문서는 npm publish 전 local `scif` wrapper를 테스트하는 절차를 설명합니다. `npm publish`를 실행하지 않고, GitHub release 설정도 하지 않습니다.

목표는 local npm wrapper가 `scif` 명령을 제공하고, 실제 npm/pnpm 프로젝트에서 인자를 Rust CLI로 올바르게 전달하는지 확인하는 것입니다.

source checkout 테스트 flow는 preview/local validation 중심입니다. 이 flow는 publish된 platform package에 의존하지 않습니다. 이 checkout에서 `npm link` 또는 file dependency install을 테스트하기 전 이 저장소에서 Rust CLI를 먼저 빌드하세요.

`npm link`로 이 checkout을 직접 연결하면 wrapper가 `target/debug/sec-issue-finder` 또는 `target/release/sec-issue-finder`를 찾을 수 있습니다. matching platform package 없이 main package tarball만 설치한 경우에는 보통 이 checkout의 `target/` 디렉터리를 볼 수 없으므로, `npx scif` 또는 `pnpm exec scif` 실행 전에 빌드된 binary를 `PATH`에 넣어야 합니다.

prebuilt platform tarball 테스트는 [npm-prebuilt-smoke-test.md](npm-prebuilt-smoke-test.md)를 사용하세요.

## Rust CLI 빌드

이 저장소에서 먼저 Rust CLI를 빌드합니다.

```bash
cargo build
```

이 명령은 `target/debug/sec-issue-finder`를 생성합니다. 이 checkout 또는 `npm link`를 통해 실행할 때 local `scif` wrapper가 이 binary를 찾을 수 있습니다.

release mode binary가 필요하면 다음을 사용하세요.

```bash
cargo build --release
```

wrapper는 matching optional platform package를 먼저 찾고, 그다음 설치된 package root의 `target/release`, `target/debug`, `PATH` 순서로 binary를 찾습니다. main-package-only tarball install에는 `target/`이 포함되지 않으므로, 이 source-checkout tarball 테스트는 matching platform package tarball도 설치하지 않는 한 `PATH`에 `sec-issue-finder`가 필요합니다.

## npm link 테스트

이 저장소 checkout에서 빠르게 smoke test할 때 유용합니다.

이 저장소에서:

```bash
npm link
```

그 다음 실행:

```bash
scif scan --help
scif scan --lockfile package-lock.json
scif scan --lockfile pnpm-lock.yaml
```

## 실제 npm 프로젝트에서 file dependency 테스트

`package-lock.json`이 있는 별도 npm 프로젝트에서 이 flow를 사용하세요.

```bash
cargo build
```

`cargo build`는 이 저장소에서 먼저 실행해야 합니다. 이후 npm 프로젝트로 이동해서 이 저장소를 local file dependency로 설치합니다.

```bash
npm install -D ../sec-finder
```

npm 프로젝트에서 실행:

```bash
npx scif scan --help
npx scif scan --lockfile package-lock.json
npx scif scan --lockfile package-lock.json --format json
npx scif scan --lockfile package-lock.json --no-dev
npx scif scan --lockfile package-lock.json --fail-on high
```

## 실제 pnpm 프로젝트에서 file dependency 테스트

`pnpm-lock.yaml`이 있는 별도 pnpm 프로젝트에서 이 flow를 사용하세요.

이 저장소에서 Rust CLI를 먼저 빌드합니다.

```bash
cargo build
```

그 다음 pnpm 프로젝트로 이동해서 이 저장소를 local file dependency로 설치합니다.

```bash
pnpm add -D ../sec-finder
```

pnpm 프로젝트에서 실행:

```bash
pnpm exec scif scan --help
pnpm exec scif scan --lockfile pnpm-lock.yaml
pnpm exec scif scan --lockfile pnpm-lock.yaml --format json
pnpm exec scif scan --lockfile pnpm-lock.yaml --no-dev
pnpm exec scif scan --lockfile pnpm-lock.yaml --fail-on high
```

## publish 없이 npm tarball 테스트

이 flow는 public npm registry를 사용하지 않으면서 실제 npm install에 가장 가까운 방식입니다.

이 저장소에서 Rust CLI를 빌드하고 local tarball을 만듭니다.

```bash
cargo build
npm pack
```

`npm pack`은 다음과 같은 파일을 생성합니다.

```text
zzozorang-sec-issue-finder-0.1.0.tgz
```

별도 npm 프로젝트를 만들고 tarball을 설치합니다.

```bash
mkdir /tmp/scif-test
cd /tmp/scif-test
npm init -y
npm install /path/to/zzozorang-sec-issue-finder-0.1.0.tgz
```

테스트 프로젝트에서 wrapper를 실행합니다.

```bash
npx scif scan --help
npx scif scan --lockfile package-lock.json
npx scif scan --lockfile package-lock.json --format json
npx scif scan --lockfile package-lock.json --no-dev
npx scif scan --lockfile package-lock.json --fail-on high
```

`/path/to/zzozorang-sec-issue-finder-0.1.0.tgz`는 `npm pack`으로 생성된 tarball의 절대 경로로 바꾸세요.

중요: main package tarball에는 JavaScript wrapper만 포함됩니다. 이 source-checkout preview flow에서는 matching platform package, local `target/` binary, 또는 `PATH`의 `sec-issue-finder` 중 하나를 찾을 수 있어야 `scif`가 성공합니다.

## publish 없이 pnpm tarball 테스트

이 저장소에서 tarball을 만듭니다.

```bash
cargo build
npm pack
```

별도 pnpm 프로젝트를 만들고 tarball을 설치합니다.

```bash
mkdir /tmp/scif-pnpm-test
cd /tmp/scif-pnpm-test
pnpm init
pnpm add -D /path/to/zzozorang-sec-issue-finder-0.1.0.tgz
```

테스트 프로젝트에서 wrapper를 실행합니다.

```bash
pnpm exec scif scan --help
pnpm exec scif scan --lockfile pnpm-lock.yaml
pnpm exec scif scan --lockfile pnpm-lock.yaml --format json
pnpm exec scif scan --lockfile pnpm-lock.yaml --no-dev
pnpm exec scif scan --lockfile pnpm-lock.yaml --fail-on high
```

테스트 프로젝트에 아직 `pnpm-lock.yaml`이 없다면 작은 dependency를 먼저 추가하세요.

```bash
pnpm add lodash
```

## 동작 체크리스트

실제 프로젝트에서 테스트하면서 다음을 확인하세요.

- [ ] `scif scan --help`, `npx scif scan --help`, `pnpm exec scif scan --help`가 scan help를 출력합니다.
- [ ] `npx scif scan --lockfile package-lock.json`가 npm lockfile을 인식하고 스캔합니다.
- [ ] `pnpm exec scif scan --lockfile pnpm-lock.yaml`가 pnpm lockfile을 인식하고 스캔합니다.
- [ ] `--format json`이 stdout에 깨끗한 JSON을 출력합니다.
- [ ] `--format json` 출력에 table output이나 log가 섞이지 않습니다.
- [ ] `--no-dev`가 dev dependency를 제외합니다.
- [ ] `--fail-on high`가 기대한 exit code를 반환합니다.
- [ ] lockfile이 없을 때 이해 가능한 error message가 출력됩니다.
- [ ] `package-lock.json`과 `pnpm-lock.yaml`이 모두 있고 `--lockfile`을 생략하면 ambiguity error가 이해 가능하게 출력됩니다.

## Tarball 설치 체크리스트

`zzozorang-sec-issue-finder-0.1.0.tgz`를 임시 npm/pnpm 프로젝트에 설치한 뒤 다음을 확인하세요.

- [ ] npm 프로젝트에서 `npx scif scan --help`가 help를 출력합니다.
- [ ] pnpm 프로젝트에서 `pnpm exec scif scan --help`가 help를 출력합니다.
- [ ] `npx scif scan --lockfile package-lock.json`가 npm lockfile을 스캔합니다.
- [ ] `pnpm exec scif scan --lockfile pnpm-lock.yaml`가 pnpm lockfile을 스캔합니다.
- [ ] `--format json`이 stdout에 valid JSON을 출력합니다.
- [ ] `--fail-on high`가 테스트 프로젝트의 finding에 맞는 exit code를 반환합니다.
- [ ] Rust binary를 찾을 수 없을 때 `cargo build` 또는 `cargo build --release`가 필요하다는 error message가 출력됩니다.
- [ ] 테스트에 `npm publish`가 필요하지 않습니다.
- [ ] 테스트에 GitHub release 또는 npm provenance 설정이 필요하지 않습니다.

## Exit Code 확인

`--fail-on high` 명령을 실행한 직후 exit code를 확인하세요.

npm:

```bash
npx scif scan --lockfile package-lock.json --fail-on high
echo $?
```

pnpm:

```bash
pnpm exec scif scan --lockfile pnpm-lock.yaml --fail-on high
echo $?
```

기대 동작:

- `0`: scan이 완료되었고 설정된 policy가 실패하지 않았습니다.
- `1`: policy가 finding과 매칭되어 실패했거나 operational error가 발생했습니다.

## Missing Binary 문제 해결

Rust binary가 없으면 `scif`는 다음 명령이 필요하다고 안내합니다.

```bash
cargo build
```

또는:

```bash
cargo build --release
```

package manager가 checkout을 link하지 않고 파일을 복사했다면 rebuild 후 local file dependency를 다시 설치하세요.

```bash
npm install -D ../sec-finder
```

또는:

```bash
pnpm add -D ../sec-finder
```

## 중요한 경계

- 이 local test를 위해 `npm publish`를 실행하지 마세요.
- 이 local test를 위해 prebuilt binary를 구현하거나 요구하지 마세요.
- Rust test에서 mocked client를 사용하는 경우가 아니라면 runtime scan은 OSV를 질의합니다.
- pnpm 지원은 registry npm dependency 중심의 best effort입니다. local, workspace, path-like dependency는 registry version이 없으면 skip될 수 있습니다.
