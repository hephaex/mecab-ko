# GitHub Actions Workflows 가이드

MeCab-Ko 프로젝트의 GitHub Actions 워크플로우 완전 가이드입니다.

## 워크플로우 요약

| 파일 | 이름 | 목적 | 트리거 | 소요시간 |
|------|------|------|--------|---------|
| `ci.yml` | CI | 자동 테스트 및 린트 | Push/PR | 15-20분 |
| `release.yml` | Release | 릴리스 빌드 및 배포 | Tag push | 30-40분 |
| `docs.yml` | Documentation | 문서 생성 및 배포 | Push/PR | 10-15분 |
| `code-quality.yml` | Code Quality | 정적 분석 및 메트릭 | Push/PR | 10-15분 |
| `benchmark.yml` | Benchmarks | 성능 비교 | Push/PR | 15-20분 |
| `scheduled.yml` | Scheduled | 정기 점검 | Cron | 20-30분 |
| `dependabot.yml` | Workflow (설정) | 의존성 자동 업데이트 | Weekly | 자동 |

## 파일 위치

```
.github/
├── workflows/
│   ├── ci.yml                 # 기본 CI 파이프라인
│   ├── release.yml            # 릴리스 자동화
│   ├── docs.yml               # 문서 빌드 및 배포
│   ├── code-quality.yml       # 코드 품질 분석
│   ├── benchmark.yml          # 성능 벤치마크
│   ├── scheduled.yml          # 정기 작업
│   └── dependabot.yml         # 워크플로우 자동 PR 처리
├── dependabot.yml             # Dependabot 설정
├── pull_request_template.md   # PR 템플릿
└── WORKFLOWS.md               # 이 파일
```

## 상세 워크플로우 설명

### 1. CI Workflow (`ci.yml`)

**역할**: 모든 push와 pull request에서 자동 테스트 실행

**포함 내용**:
```
┌─ Test Suite
│  ├─ Linux (stable, beta, nightly)
│  ├─ macOS (stable, beta, nightly)
│  └─ Windows (stable, beta, nightly)
├─ Clippy Lint
├─ Rustfmt Check
├─ Code Coverage (tarpaulin)
├─ Build (all platforms)
└─ Security Audit
```

**주요 Features**:
- 3개 OS × 3개 Rust 버전 = 9개 병렬 테스트
- 자동 캐싱으로 빌드 시간 단축
- 릴리스 빌드도 검증
- SBOM 및 보안 감시

**Key Settings**:
```yaml
paths:
  - 'rust/**'
  - 'Cargo.toml'
  - 'Cargo.lock'
```

### 2. Release Workflow (`release.yml`)

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

### 3. Documentation Workflow (`docs.yml`)

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

### 4. Code Quality Workflow (`code-quality.yml`)

**역할**: 정적 분석 및 메트릭 수집

**포함 내용**:
```
├─ Code Quality Checks
│  ├─ clippy
│  ├─ rustfmt
│  └─ cargo check
├─ Dependency Audit
│  ├─ cargo-deny
│  └─ security audit
├─ Unused Dependencies (cargo-udeps)
├─ Documentation Check
└─ Complexity Analysis (tokei, cargo-metrics)
```

**PR 코멘트**: 자동으로 품질 요약을 PR에 추가

### 5. Performance Benchmark (`benchmark.yml`)

**역할**: 성능 변화 추적 및 비교

**기능**:
- Pull Request 시 base 브랜치와 자동 비교
- criterion으로 측정
- GitHub benchmark 액션으로 시각화
- PR에 결과 코멘트

### 6. Scheduled Tasks (`scheduled.yml`)

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

### 전역 설정
```yaml
env:
  CARGO_TERM_COLOR: always      # 컬러 출력
  RUST_BACKTRACE: 1             # 디버깅 정보
```

### Per-workflow
- CI: `RUST_BACKTRACE: full` (테스트 실패 시)
- 기타: job 레벨에서 설정

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
