# GitHub Actions Workflows 가이드

MeCab-Ko 프로젝트의 GitHub Actions 워크플로우 완전 가이드입니다.

## 워크플로우 요약

| 파일 | 이름 | 목적 | 트리거 | 소요시간 |
|------|------|------|--------|---------|
| `ci.yml` | CI | 빌드, 테스트, 린트 | Push/PR | 20-30분 |
| `security.yml` | Security | 보안 검사 (audit, deny, geiger) | Push/PR/Daily | 10-15분 |
| `code-quality.yml` | Code Quality | 정적 분석 및 메트릭 | Push/PR/Daily | 15-20분 |
| `benchmark.yml` | Benchmarks | 성능 비교 및 측정 | Push/PR/Manual | 10-20분 |
| `release.yml` | Release | 릴리스 빌드 및 배포 | Tag push | 30-40분 |
| `docs.yml` | Documentation | 문서 생성 및 배포 | Push/PR | 10-15분 |
| `scheduled.yml` | Scheduled | 정기 점검 | Cron | 20-30분 |
| `dependabot.yml` | Workflow (설정) | 의존성 자동 업데이트 | Weekly | 자동 |

## 파일 위치

```
.github/
├── workflows/
│   ├── ci.yml                 # 기본 CI 파이프라인 (빌드, 테스트, 린트)
│   ├── security.yml           # 보안 검사 (audit, deny, unsafe code 검사)
│   ├── code-quality.yml       # 코드 품질 분석 (복잡도, 문서화, 의존성)
│   ├── benchmark.yml          # 성능 벤치마크 및 비교
│   ├── release.yml            # 릴리스 자동화
│   ├── docs.yml               # 문서 빌드 및 배포
│   ├── scheduled.yml          # 정기 작업
│   ├── e2e-tests.yml          # E2E 테스트
│   ├── elasticsearch-plugin-tests.yml # 플러그인 테스트
│   └── dependabot.yml         # Dependabot 설정 및 자동화
├── dependabot.yml             # Dependabot 설정
├── pull_request_template.md   # PR 템플릿
└── WORKFLOWS.md               # 이 파일
```

## 상세 워크플로우 설명

### 1. CI Workflow (`ci.yml`)

**역할**: 모든 push와 pull request에서 자동 빌드, 테스트, 린트 실행

**포함 내용**:
```
┌─ Rustfmt Check (빠른 포맷 검사)
├─ Clippy Lint (린트 검사)
├─ Test Suite (다중 플랫폼 테스트)
│  ├─ Linux (stable, beta, nightly)
│  ├─ macOS (stable, beta, nightly)
│  └─ Windows (stable, beta, nightly)
├─ Build (모든 플랫폼)
├─ Security Audit (cargo audit + RustSec)
├─ Code Coverage (tarpaulin)
└─ CI Status (모든 체크 요약)
```

**주요 Features**:
- 빠른 검사 먼저 실행 (fmt, clippy)
- 3개 OS × 3개 Rust 버전 = 9개 병렬 테스트
- 자동 캐싱으로 빌드 시간 단축
- Debug와 Release 빌드 모두 검증
- Rustdoc 생성 및 경고 검사

**Key Settings**:
```yaml
env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1
  RUSTFLAGS: -D warnings  # 모든 경고를 에러로 처리
paths:
  - 'rust/**'
  - 'Cargo.toml'
  - 'Cargo.lock'
```

### 2. Security Workflow (`security.yml`)

**역할**: 의존성 및 코드 보안 검사 (자동 또는 일일 스케줄)

**포함 내용**:
```
┌─ RustSec Audit (보안 데이터베이스 검사)
├─ Cargo Audit (cargo audit 도구)
├─ Cargo Deny (의존성 정책 검사)
├─ Unsafe Code Check (unsafe 사용 추적 via cargo-geiger)
├─ SAST - Clippy (엄격한 린트 검사)
├─ Unmaintained Dependencies (구식 의존성 확인)
├─ SBOM Generation (소프트웨어 BOM 생성)
└─ Security Summary (보안 체크 요약)
```

**주요 Features**:
- 자동 보안 감시 (매일 2 AM UTC)
- 3가지 보안 도구로 다층 검사
- SBOM 생성으로 공급망 보안 추적
- PR/Push 시 추가 검사
- 보안 문제 발견 시 즉시 감지

**트리거**:
```yaml
on:
  push:        # 모든 push
  pull_request # 모든 PR
  schedule:    # 매일 2 AM UTC
    - cron: '0 2 * * *'
  workflow_dispatch  # 수동 실행
```

### 3. Code Quality Workflow (`code-quality.yml`)

**역할**: 정적 분석, 복잡도, 문서화 점검

**포함 내용**:
```
┌─ Code Quality Checks
│  ├─ clippy (JSON 형식)
│  ├─ rustfmt (포맷 검사)
│  └─ cargo check (컴파일 검사)
├─ Dependency Audit
│  ├─ cargo-deny
│  └─ RustSec
├─ Unused Dependencies (cargo-udeps with nightly)
├─ Documentation Check (문서화 커버리지)
├─ Complexity Analysis (tokei, cargo-metrics)
└─ Summary (품질 체크 요약)
```

**주요 Features**:
- 자동 품질 분석 (매일 3 AM UTC)
- PR에 자동 코멘트로 결과 공유
- 미사용 의존성 검사 (nightly 필요)
- 코드 통계 및 복잡도 보고서
- 문서화 커버리지 추적

### 4. Benchmark Workflow (`benchmark.yml`)

**역할**: 성능 측정 및 비교 분석

**포함 내용**:
```
┌─ Benchmark Compilation Check
├─ Run Benchmarks (현재 브랜치)
├─ Benchmark Comparison (PR 시 base와 비교)
└─ Extended Benchmarks (스케줄 또는 수동)
```

**주요 Features**:
- PR 시 자동으로 base 브랜치와 비교
- Criterion 기반 성능 측정
- GitHub benchmark 액션으로 시각화
- 결과를 PR 코멘트로 자동 공유
- 수동 트리거로 상세 벤치마크 실행 가능

**수동 실행**:
```bash
gh workflow run benchmark.yml -f full_bench=true
```

### 5. Release Workflow (`release.yml`)

**역할**: 버전 태그 푸시 시 자동 릴리스 생성

**포함 내용**:
```
┌─ Create Release (GitHub Release 생성)
├─ Build Release (6개 플랫폼)
│  ├─ Linux x86_64
│  ├─ Linux aarch64
│  ├─ macOS x86_64
│  ├─ macOS aarch64
│  └─ Windows x86_64
├─ Upload Assets
└─ Publish to crates.io (안정 버전만)
```

**릴리스 방법**:

```bash
# 1. 버전 확인
grep version rust/Cargo.toml

# 2. 변경사항 커밋
git add .
git commit -m "Release v0.1.0"

# 3. 태그 생성 및 푸시
git tag v0.1.0
git push origin main
git push origin v0.1.0

# 또는 한 번에
git push origin main --tags
```

**산출물**:
- GitHub Release 페이지에 자동 생성
- 각 플랫폼별 바이너리 압축 파일 업로드
- crates.io에 자동 배포

### 6. Documentation Workflow (`docs.yml`)

**역할**: Rustdoc 및 mdBook 문서 자동 생성 및 배포

**포함 내용**:
```
┌─ Build Rustdoc
├─ Build mdBook
├─ Combine Docs (통합 인덱스)
└─ Deploy to GitHub Pages
```

**배포 대상**:
- API 문서: Rustdoc
- 사용 가이드: mdBook
- 통합 인덱스: 커스텀 HTML

**접근 URL**:
```
https://hephaex.github.io/mecab-ko/api/mecab_ko/
https://hephaex.github.io/mecab-ko/book/
```

### 7. Scheduled Tasks (`scheduled.yml`)

**스케줄**:
```
매일 00:00 UTC
└─ 보안 감시 (security audit)

매주 일요일 02:00 UTC
├─ 의존성 업데이트 확인
├─ 모든 플랫폼/버전 테스트
├─ 아티팩트 정리
└─ 상태 리포트 생성
```

**자동 이슈 생성**:
- 보안 취약점 발견 시
- 의존성 업데이트 가능 시

## Dependabot 설정

위치: `.github/dependabot.yml`

**자동화 범위**:
```yaml
cargo:          # Rust 의존성
github-actions: # GitHub Actions
docker:         # Docker 이미지
```

**PR 자동 관리**:
- 의존성 업데이트 PR 자동 생성
- 자동 머지 및 승인 (dependabot.yml 워크플로우)

## Pull Request 템플릿

위치: `.github/pull_request_template.md`

PR 생성 시 자동으로 제공되는 템플릿:
- 변경 사항 설명
- 변경 타입 (버그, 기능, 리팩토링 등)
- 관련 이슈
- 테스트 방법
- 체크리스트

## 필수 설정 체크리스트

### Repository Secrets
```
☐ CODECOV_TOKEN      (선택) - 커버리지 업로드용
☐ CARGO_REGISTRY_TOKEN (선택) - crates.io 배포용
```

### Repository Settings
```
☐ GitHub Pages 활성화
  Settings → Pages → Deploy from a branch (gh-pages)

☐ Branch Protection Rule (권장)
  Settings → Branches → Add rule
  - main 또는 master 브랜치
  - Require status checks
  - Require code reviews
```

### Dependabot
```
☐ Dependabot 알림 활성화
  Settings → Code security and analysis
  → Enable Dependabot alerts
  → Enable Dependabot security updates
```

## 환경 변수

### CI 워크플로우
```yaml
env:
  CARGO_TERM_COLOR: always      # 컬러 출력
  RUST_BACKTRACE: 1             # 기본 디버깅 정보
  RUSTFLAGS: -D warnings        # 모든 경고를 에러로 처리

steps:
  # 테스트 실패 시 상세 정보
  - name: Run tests
    env:
      RUST_BACKTRACE: full      # 전체 스택 추적
```

### Security 워크플로우
```yaml
env:
  CARGO_TERM_COLOR: always
  RUSTFLAGS: -D warnings        # 보안 중심
```

### Benchmark 워크플로우
```yaml
env:
  CARGO_TERM_COLOR: always
  RUSTFLAGS: -D warnings
```

## 성능 최적화

### 캐싱 전략
```yaml
cache:
  - ~/.cargo/registry (의존성)
  - ~/.cargo/git (git 의존성)
  - target/ (빌드 아티팩트)
```

**캐시 키**: `${{ hashFiles('**/Cargo.lock') }}`
- Cargo.lock 변경 시 새로 생성

### 병렬 실행
- Test Suite: 9개 조합 병렬
- Code Quality: 5개 job 병렬
- Release Build: 6개 플랫폼 병렬

## 문제 해결

### 캐시 문제
```bash
# GitHub UI에서 캐시 제거
Actions → Caches → 해당 캐시 삭제
```

### 워크플로우 디버깅
```bash
# 로컬 재현
cd rust
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt -- --check
```

### 로그 수집
```bash
# gh CLI 사용
gh run list --workflow ci.yml
gh run view <run-id> --log
```

## 문서 (CICD_SETUP.md 참고)

자세한 설정 및 문제 해결:
- `/home/mare/mecab-ko/CICD_SETUP.md`

## 추가 자료

- [GitHub Actions 문서](https://docs.github.com/en/actions)
- [Rust 공식 가이드](https://www.rust-lang.org/what/wg-cli/)
- [Cargo 출판 가이드](https://doc.rust-lang.org/cargo/reference/publishing.html)

## 빠른 참조

### 테스트 실행
```bash
cd /home/mare/mecab-ko/rust
cargo test --verbose
```

### 릴리스 생성
```bash
git tag v0.1.0
git push origin v0.1.0
```

### 문서 로컬 빌드
```bash
cd /home/mare/mecab-ko/rust
cargo doc --no-deps --open
```

### 코드 품질 확인
```bash
cd /home/mare/mecab-ko/rust
cargo clippy --all-targets -- -D warnings
cargo fmt -- --check
```
