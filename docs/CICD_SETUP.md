# CI/CD Pipeline Setup Guide

이 문서는 MeCab-Ko 프로젝트의 GitHub Actions CI/CD 파이프라인 설정 및 사용 방법을 설명합니다.

## 목차

1. [파이프라인 개요](#파이프라인-개요)
2. [워크플로우 설명](#워크플로우-설명)
3. [필수 설정](#필수-설정)
4. [선택적 설정](#선택적-설정)
5. [트리거 조건](#트리거-조건)
6. [모니터링 및 디버깅](#모니터링-및-디버깅)
7. [문제 해결](#문제-해결)

## 파이프라인 개요

MeCab-Ko 프로젝트는 다음과 같은 자동화된 CI/CD 파이프라인을 제공합니다:

```
Push/PR → CI (Test, Lint) → Code Quality → Docs Build
                                     ↓
                              GitHub Pages Deploy
                                     ↓
Tag Push → Release Build → Multi-platform Build → GitHub Releases
                ↓
         crates.io Publish (stable releases only)
```

## 워크플로우 설명

### 1. CI Workflow (`ci.yml`)

**목적**: 모든 푸시와 PR에 대한 자동 테스트 및 검사

**실행 단계**:
- Test Suite: 3개 OS(Linux, macOS, Windows) × 3개 Rust 버전(stable, beta, nightly)에서 테스트
- Clippy Lint: 코드 품질 검사
- Rustfmt Check: 코드 포맷팅 검사
- Code Coverage: tarpaulin을 이용한 커버리지 측정
- Build: 모든 플랫폼에서 빌드
- Security Audit: 의존성 보안 감시

**트리거**:
```yaml
- main, master, develop 브랜치의 push/PR
- rust/ 또는 Cargo.toml 변경 시에만
```

**소요 시간**: 약 15-20분 (모든 job 병렬 실행)

### 2. Release Workflow (`release.yml`)

**목적**: 버전 태그 푸시 시 자동 릴리스 생성 및 바이너리 배포

**실행 단계**:
- Create Release: GitHub Release 생성
- Build Release: 6개 플랫폼 빌드
  - Linux (x86_64, aarch64)
  - macOS (x86_64, aarch64)
  - Windows (x86_64)
- Upload Assets: 각 플랫폼별 바이너리 업로드
- Publish Crates: crates.io에 배포 (안정 버전만)

**트리거**:
```yaml
- v* 태그 푸시 (예: v0.1.0)
- 또는 workflow_dispatch로 수동 실행
```

**사용 예**:
```bash
git tag v0.1.0
git push origin v0.1.0
```

**소요 시간**: 약 30-40분

### 3. Documentation Workflow (`docs.yml`)

**목적**: rustdoc 및 mdbook 문서 생성 및 GitHub Pages 배포

**실행 단계**:
- Build Rustdoc: Rust API 문서 생성
- Build mdBook: 사용자 가이드 빌드
- Combine Docs: 통합 문서 생성
- Deploy Pages: GitHub Pages에 배포 (main/master 브랜치만)

**트리거**:
```yaml
- rust/, docs/ 변경 시
- main 또는 master 브랜치의 push
```

**접근**:
- API 문서: `https://your-repo.github.io/api/mecab_ko/`
- 사용자 가이드: `https://your-repo.github.io/book/`

**소요 시간**: 약 10-15분

### 4. Code Quality Workflow (`code-quality.yml`)

**목적**: 정적 분석, 의존성 감시, 문서 커버리지 확인

**실행 단계**:
- Code Quality Checks: clippy, rustfmt, cargo check
- Dependency Audit: cargo-deny로 라이선스 및 보안 검사
- Unused Dependencies: cargo-udeps로 미사용 의존성 확인
- Documentation Check: 문서화 누락 확인
- Complexity Analysis: 코드 복잡도 분석

**트리거**:
```yaml
- push/PR 시 rust/ 변경
```

**결과**: PR에 코멘트로 요약 작성

### 5. Performance Benchmark Workflow (`benchmark.yml`)

**목적**: 성능 변화 추적 및 비교

**실행 단계**:
- 현재 브랜치에서 벤치마크 실행
- PR인 경우, base 브랜치와 비교
- 결과를 GitHub에 저장 및 시각화

**트리거**:
```yaml
- 벤치마크 코드 변경 시
```

**결과**: 성능 저하 감지 시 경고

### 6. Scheduled Tasks (`scheduled.yml`)

**목적**: 정기적인 보안 및 유지보수 자동화

**스케줄**:
- 매일 00:00 UTC: 보안 감시
- 매주 일요일 02:00 UTC: 종합 점검
  - 의존성 업데이트 확인
  - 모든 플랫폼/Rust 버전 테스트
  - 아티팩트 정리

### 7. Dependabot

**목적**: 의존성 자동 업데이트

**설정**:
```yaml
cargo:
  - 주간 월요일 업데이트
  - 자동 PR 생성

github-actions:
  - 주간 월요일 업데이트

docker:
  - 주간 수요일 업데이트
```

## 필수 설정

### 1. Repository Secrets 설정

GitHub Repository Settings → Secrets and variables → Actions에서 다음을 설정:

#### `CODECOV_TOKEN` (선택적)
- Codecov에서 토큰 생성
- 용도: 코드 커버리지 업로드

```
Settings → Secrets → New repository secret
Name: CODECOV_TOKEN
Value: <codecov 토큰>
```

#### `CARGO_REGISTRY_TOKEN` (선택적, crates.io 배포 시)
- crates.io 계정에서 토큰 생성
- 용도: crates.io에 패키지 배포

```bash
# crates.io에서 토큰 생성
cargo login
# ~/.cargo/credentials.toml에서 토큰 복사

# GitHub Secrets에 추가
Name: CARGO_REGISTRY_TOKEN
Value: <crates.io 토큰>
```

### 2. GitHub Pages 활성화

Repository Settings → Pages:
- Source: Deploy from a branch
- Branch: gh-pages
- Folder: / (root)

워크플로우에서 자동으로 gh-pages 브랜치를 생성합니다.

### 3. Branch Protection Rules 설정 (권장)

Repository Settings → Branches → Add rule:

```
Branch name pattern: main OR master

Require status checks to pass before merging:
- ✓ Test Suite (ubuntu-latest, stable, beta, nightly)
- ✓ Clippy Lint
- ✓ Rustfmt Check
- ✓ Build

Require conversation resolution before merging: ✓
Require code reviews before merging: ✓
```

## 선택적 설정

### 1. Slack 알림 추가

`.github/workflows/ci.yml`에 다음 job 추가:

```yaml
notify-slack:
  name: Notify Slack
  runs-on: ubuntu-latest
  needs: [test, clippy, fmt, coverage, build]
  if: always()
  steps:
    - name: Send Slack notification
      uses: 8398a7/action-slack@v3
      with:
        status: ${{ job.status }}
        text: 'CI Pipeline Result: ${{ job.status }}'
        webhook_url: ${{ secrets.SLACK_WEBHOOK }}
      if: always()
```

### 2. Discord 알림

Webhook 기반 Discord 알림 추가 가능:

```yaml
- name: Notify Discord
  uses: sarisia/actions-status-discord@v1
  if: always()
  with:
    webhook_url: ${{ secrets.DISCORD_WEBHOOK }}
```

### 3. Email 알림

GitHub 기본 알림 설정 활용:
- Settings → Notifications → Email

## 트리거 조건

### 자동 실행 조건

```yaml
# CI (ci.yml)
- push to main, master, develop
- pull request to main, master, develop
- rust/** 파일 변경
- Cargo.toml 변경

# Release (release.yml)
- v* 태그 푸시 (v0.1.0 형식)

# Docs (docs.yml)
- rust/** 또는 docs/** 변경
- push to main/master

# Scheduled
- 매일 00:00 UTC (보안)
- 매주 일요일 02:00 UTC (종합)
```

### 수동 실행

GitHub Actions 탭에서:
1. 워크플로우 선택
2. "Run workflow" 버튼 클릭
3. 필요시 입력값 입력

```bash
# CLI로 수동 실행 (gh 설치 필요)
gh workflow run ci.yml --ref main
gh workflow run release.yml --ref main -f tag=0.1.0
```

## 모니터링 및 디버깅

### 1. 워크플로우 상태 확인

GitHub Actions 탭에서:
- 실시간 실행 상태 확인
- 각 job의 로그 확인
- 실행 시간 분석

### 2. 실패 원인 파악

```bash
# 로컬에서 재현
cd rust
cargo test --verbose
cargo clippy --all-targets -- -D warnings
cargo fmt -- --check
```

### 3. 캐시 문제

캐시가 원인인 경우:
- GitHub Actions 탭 → 워크플로우 → "Clear caches"
- 또는 특정 캐시 키 삭제

### 4. 로그 수집

```bash
# 워크플로우 실행 로그 다운로드
gh workflow view ci.yml --log
gh run view <run-id> --log
```

## 문제 해결

### 문제 1: 테스트 실패

**증상**: "Test Suite" job 실패

**해결**:
```bash
# 로컬에서 테스트 재현
cd rust
cargo test --verbose

# nightly 테스트
rustup toolchain install nightly
cargo +nightly test

# 상세 출력
RUST_BACKTRACE=full cargo test -- --nocapture
```

### 문제 2: Clippy 경고

**증상**: "Clippy Lint" job 실패

**해결**:
```bash
cd rust
cargo clippy --all-targets --all-features

# 특정 경고 무시 필요 시
#[allow(clippy::style_warning)]
```

### 문제 3: 포맷팅 오류

**증상**: "Rustfmt Check" 실패

**해결**:
```bash
cd rust
cargo fmt
git diff  # 변경 확인
```

### 문제 4: 빌드 실패

**증상**: "Build" job 실패

**해결**:
```bash
# 특정 플랫폼에서 테스트
cargo build --release --target x86_64-unknown-linux-gnu

# 문제 있는 플랫폼용 러스트 설치
rustup target add x86_64-unknown-linux-gnu
```

### 문제 5: 문서 생성 실패

**증상**: "Build rustdoc" 실패

**해결**:
```bash
cd rust
cargo doc --no-deps --release
cargo doc --open  # 생성된 문서 확인
```

### 문제 6: 릴리스 배포 실패

**증상**: "Publish to crates.io" 실패

**해결**:
```bash
# 1. 버전 확인
grep version rust/crates/mecab-ko/Cargo.toml

# 2. 로컬에서 배포 시뮬레이션
cargo publish --manifest-path rust/crates/mecab-ko/Cargo.toml --dry-run

# 3. 토큰 확인
cat ~/.cargo/credentials.toml
```

## 성능 최적화

### 캐시 전략

현재 설정:
- Cargo 레지스트리: 버전별 캐싱
- 빌드 아티팩트: Rust 버전별 캐싱
- 의존성 그래프: Cargo.lock 기반

### 병렬 실행

현재 워크플로우:
- Test Suite: 9개 조합 병렬 (stable/beta/nightly × Linux/macOS/Windows)
- Code Quality: 5개 job 병렬
- Release Build: 6개 플랫폼 병렬

### 실행 시간 감소

```yaml
# 특정 OS에서만 테스트 필요한 경우
runs-on: ubuntu-latest  # macos, windows 제거

# 특정 Rust 버전만 필요한 경우
toolchain: stable  # beta, nightly 제거
```

## 비용 최적화

GitHub Actions 요금:
- Public 리포지토리: **무료**
- Private 리포지토리: 월 2,000분 무료 (GitHub Pro 기준)

최적화 팁:
- 불필요한 OS 조합 제거
- 장기 실행 워크플로우 분리
- 캐시 활용으로 재실행 시간 단축

## 추가 자료

- [GitHub Actions 공식 문서](https://docs.github.com/en/actions)
- [Rust GitHub Actions](https://github.com/dtolnay/rust-toolchain)
- [Cargo 공식 문서](https://doc.rust-lang.org/cargo/)
- [clippy 린트 규칙](https://rust-lang.github.io/rust-clippy/)

## 지원

문제가 발생하면:
1. GitHub Issues에서 "ci/cd" 라벨로 이슈 검색
2. 워크플로우 로그 첨부하여 이슈 등록
3. 로컬에서 재현 방법 포함
