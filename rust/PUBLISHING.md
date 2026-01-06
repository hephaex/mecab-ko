# Publishing Guide for MeCab-Ko Rust Crates

이 문서는 mecab-ko 관련 크레이트들을 crates.io에 배포하는 절차를 설명합니다.

## 배포 순서

크레이트는 의존성 순서대로 배포해야 합니다. 각 크레이트가 의존하는 크레이트가 먼저 배포되어야 합니다.

### 1단계: mecab-ko-hangul
- **의존성**: 없음
- **설명**: 한글 처리 유틸리티 (자모 분리/결합, 정규화)
- **배포 명령**:
  ```bash
  cd crates/mecab-ko-hangul
  cargo publish --dry-run  # 먼저 테스트
  cargo publish
  ```

### 2단계: mecab-ko-dict
- **의존성**: mecab-ko-hangul
- **설명**: 사전 관리 및 검색 (FST, 연접 비용)
- **배포 전 확인**: mecab-ko-hangul이 crates.io에 배포되었는지 확인
- **Cargo.toml 수정 필요**:
  ```toml
  # 배포 전에 path를 version으로 변경
  mecab-ko-hangul = "0.1.0"  # path = "../mecab-ko-hangul" 대신
  ```
- **배포 명령**:
  ```bash
  cd crates/mecab-ko-dict
  cargo publish --dry-run
  cargo publish
  ```

### 3단계: mecab-ko-core
- **의존성**: mecab-ko-dict, mecab-ko-hangul
- **설명**: 형태소 분석 핵심 엔진 (Lattice, Viterbi)
- **배포 전 확인**: mecab-ko-dict와 mecab-ko-hangul이 배포되었는지 확인
- **Cargo.toml 수정 필요**:
  ```toml
  mecab-ko-hangul = "0.1.0"
  mecab-ko-dict = "0.1.0"
  ```
- **배포 명령**:
  ```bash
  cd crates/mecab-ko-core
  cargo publish --dry-run
  cargo publish
  ```

### 4단계: mecab-ko-dict-builder
- **의존성**: mecab-ko-dict, mecab-ko-hangul
- **설명**: CSV에서 바이너리 사전 생성 도구
- **배포 전 확인**: 의존 크레이트들이 배포되었는지 확인
- **Cargo.toml 수정 필요**:
  ```toml
  mecab-ko-hangul = "0.1.0"
  mecab-ko-dict = "0.1.0"
  ```
- **배포 명령**:
  ```bash
  cd crates/mecab-ko-dict-builder
  cargo publish --dry-run
  cargo publish
  ```

### 5단계: mecab-ko-cli
- **의존성**: mecab-ko-core, mecab-ko-dict
- **설명**: 명령줄 형태소 분석 도구
- **배포 전 확인**: mecab-ko-core가 배포되었는지 확인
- **Cargo.toml 수정 필요**:
  ```toml
  mecab-ko-core = "0.1.0"
  mecab-ko-dict = "0.1.0"
  ```
- **배포 명령**:
  ```bash
  cd crates/mecab-ko-cli
  cargo publish --dry-run
  cargo publish
  ```

### 6단계: mecab-ko (facade)
- **의존성**: mecab-ko-core, mecab-ko-dict, mecab-ko-hangul
- **설명**: 통합 라이브러리 (facade 패턴)
- **배포 전 확인**: 모든 하위 크레이트가 배포되었는지 확인
- **Cargo.toml 수정 필요**:
  ```toml
  mecab-ko-core = "0.1.0"
  mecab-ko-dict = "0.1.0"
  mecab-ko-hangul = "0.1.0"

  [dependencies.mecab-ko-dict-builder]
  version = "0.1.0"
  optional = true
  ```
- **배포 명령**:
  ```bash
  cd crates/mecab-ko
  cargo publish --dry-run
  cargo publish
  ```

## 버전 관리 전략

### Semantic Versioning (SemVer)

모든 크레이트는 [Semantic Versioning 2.0.0](https://semver.org/)을 따릅니다:

- **MAJOR (0.x.0)**: 호환되지 않는 API 변경
- **MINOR (0.1.x)**: 하위 호환되는 기능 추가
- **PATCH (0.1.0)**: 하위 호환되는 버그 수정

### 초기 개발 단계 (0.x.y)

현재는 0.1.0 버전으로 시작합니다. 0.x 버전에서는:
- API가 안정화되지 않았음을 나타냄
- 0.x 내에서도 breaking changes 가능
- 1.0.0 릴리스 전까지는 실험적 단계

### 버전 동기화

모든 크레이트는 workspace.package.version을 공유하여 동일한 버전을 사용합니다.

```toml
# Cargo.toml (workspace root)
[workspace.package]
version = "0.1.0"

# 각 크레이트
[package]
version.workspace = true
```

### 버전 업데이트 절차

1. **workspace Cargo.toml 버전 업데이트**
   ```toml
   [workspace.package]
   version = "0.2.0"  # 예: 0.1.0 -> 0.2.0
   ```

2. **CHANGELOG.md 업데이트** (각 크레이트 또는 루트)
   - 변경사항 기록
   - 릴리스 날짜 명시

3. **Git 태그 생성**
   ```bash
   git tag -a v0.2.0 -m "Release v0.2.0"
   git push origin v0.2.0
   ```

4. **배포 스크립트 실행**
   ```bash
   ./scripts/publish.sh --version 0.2.0
   ```

## 배포 전 체크리스트

모든 크레이트 배포 전에 다음을 확인하세요:

### 코드 품질
- [ ] `cargo test` - 모든 테스트 통과
- [ ] `cargo clippy -- -D warnings` - Clippy 경고 없음
- [ ] `cargo fmt --check` - 코드 포맷팅 확인
- [ ] `cargo doc --no-deps` - 문서 생성 확인

### 메타데이터
- [ ] `Cargo.toml`에 모든 필수 필드 존재
  - name, version, edition, authors
  - description (간결하고 명확한 설명)
  - license (MIT OR Apache-2.0)
  - repository, homepage
  - documentation (docs.rs URL)
  - readme (README.md 경로)
  - keywords (최대 5개, 소문자, 하이픈 사용)
  - categories (crates.io 카테고리 확인)
- [ ] README.md 존재 및 내용 확인
  - 크레이트 설명
  - 기본 사용 예제
  - 라이선스 명시
- [ ] LICENSE 파일 존재 (workspace 루트)

### 의존성
- [ ] 의존성 버전이 올바른지 확인
  - 로컬 개발: `path = "../other-crate"`
  - 배포용: `version = "0.1.0"` (또는 `{ version = "0.1", path = "..." }`)
- [ ] 의존하는 크레이트가 이미 배포되었는지 확인

### 빌드 및 테스트
- [ ] `cargo build --release` - 릴리스 빌드 성공
- [ ] `cargo test --all-features` - 모든 기능 테스트
- [ ] `cargo package --list` - 패키지 내용 확인
- [ ] `cargo package` - 패키징 성공 확인

### Dry-run
- [ ] `cargo publish --dry-run` - 배포 시뮬레이션 성공
- [ ] 패키지 크기 확인 (< 10MB 권장)
- [ ] 불필요한 파일 제외 확인 (.gitignore 확인)

## 배포 명령어

### 개별 크레이트 배포

```bash
# 1. 크레이트 디렉토리로 이동
cd crates/mecab-ko-hangul

# 2. Dry-run으로 테스트
cargo publish --dry-run

# 3. 문제없으면 실제 배포
cargo publish

# 4. 배포 확인
cargo search mecab-ko-hangul
```

### 스크립트를 사용한 일괄 배포

```bash
# Dry-run 모드 (실제 배포 안 함)
./scripts/publish.sh --dry-run

# 특정 버전으로 배포
./scripts/publish.sh --version 0.1.0

# 실제 배포 (주의!)
./scripts/publish.sh
```

## 배포 후 작업

### 확인 사항
1. **crates.io에서 확인**
   - https://crates.io/crates/mecab-ko-hangul
   - 버전, 설명, 문서 링크 확인

2. **docs.rs에서 문서 확인**
   - https://docs.rs/mecab-ko-hangul
   - 문서가 올바르게 생성되었는지 확인
   - 예제 코드가 작동하는지 확인

3. **설치 테스트**
   ```bash
   # 새 프로젝트에서 테스트
   cargo new test-mecab-ko
   cd test-mecab-ko
   cargo add mecab-ko
   cargo build
   ```

### Git 태그 및 릴리스
```bash
# 태그 생성
git tag -a v0.1.0 -m "Release v0.1.0"
git push origin v0.1.0

# GitHub Release 생성 (선택사항)
gh release create v0.1.0 --title "v0.1.0" --notes "Initial release"
```

### 문서 업데이트
- README.md의 설치 방법 업데이트
- CHANGELOG.md에 릴리스 기록
- 프로젝트 웹사이트 업데이트 (있는 경우)

## 문제 해결

### "failed to verify package tarball" 오류
```bash
# .cargo/config.toml에서 exclude 확인
[package]
exclude = [
    "tests/fixtures/*",
    "benches/data/*",
]
```

### 의존성 버전 오류
```bash
# 의존하는 크레이트가 배포되었는지 확인
cargo search mecab-ko-hangul

# 로컬 path를 version으로 변경
mecab-ko-hangul = "0.1.0"  # path = "../mecab-ko-hangul" 제거
```

### 패키지 크기 초과
```bash
# 패키지 내용 확인
cargo package --list

# 불필요한 파일 제외
# Cargo.toml에 추가
[package]
exclude = [
    "tests/large_files/*",
    "*.png",
    "docs/*",
]
```

### 문서 생성 실패
```bash
# 로컬에서 문서 생성 테스트
cargo doc --no-deps --open

# missing_docs 경고 확인
cargo doc --no-deps 2>&1 | grep warning
```

## crates.io API Token 관리

### 토큰 생성
1. https://crates.io/settings/tokens 접속
2. "New Token" 클릭
3. 토큰 이름 입력 (예: "mecab-ko-publisher")
4. 권한 선택: "publish-update" 또는 "publish-new"

### 토큰 저장
```bash
# cargo login으로 토큰 저장
cargo login
# 토큰 입력

# 또는 환경 변수로 설정
export CARGO_REGISTRY_TOKEN="your-token-here"
```

### 보안 주의사항
- 토큰을 Git에 커밋하지 말 것
- CI/CD에서는 Secret으로 관리
- 주기적으로 토큰 갱신

## CI/CD 자동화 (선택사항)

### GitHub Actions 예제

```yaml
name: Publish to crates.io

on:
  push:
    tags:
      - 'v*'

jobs:
  publish:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: dtolnay/rust-toolchain@stable

      - name: Publish crates
        env:
          CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}
        run: |
          ./scripts/publish.sh --version ${GITHUB_REF#refs/tags/v}
```

## 참고 자료

- [The Cargo Book - Publishing on crates.io](https://doc.rust-lang.org/cargo/reference/publishing.html)
- [crates.io Policies](https://crates.io/policies)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Semantic Versioning](https://semver.org/)
- [Keep a Changelog](https://keepachangelog.com/)

## 문의

배포 관련 문제가 있으면 GitHub Issues에 등록해주세요:
https://github.com/hephaex/mecab-ko/issues
