# MeCab-Ko GitHub Actions CI/CD 워크플로우 개선 사항

## 개요

MeCab-Ko 프로젝트의 GitHub Actions CI/CD 파이프라인이 다음과 같이 개선되었습니다:
- **더 강화된 보안 검사** (별도 `security.yml` 워크플로우)
- **향상된 CI 파이프라인** (병렬화 최적화, 빠른 피드백)
- **개선된 벤치마크 워크플로우** (PR 자동 비교, 수동 트리거)
- **상세한 설정 문서화** (WORKFLOWS.md 업데이트)

## 개선된 워크플로우

### 1. CI 워크플로우 (`ci.yml`) - 강화

#### 주요 개선 사항:
```
✅ 빠른 검사 먼저 실행 (fmt → clippy)
✅ 테스트 병렬 실행 (9개 조합: 3 OS × 3 Rust 버전)
✅ 세 가지 보안 검사 통합 (audit + RustSec)
✅ 코드 커버리지 자동 생성 및 업로드
✅ CI 상태 요약 job 추가
```

#### 구성:
```yaml
1. Rustfmt Check      (빠른 실행 - ~1분)
2. Clippy Lint        (빠른 실행 - ~2분)
3. Test Suite         (병렬 9개 - ~15분)
4. Build              (병렬 3개 - ~10분)
5. Security Audit     (멀티 도구 - ~5분)
6. Code Coverage      (tarpaulin - ~5분)
7. CI Status          (요약 - ~1초)
```

#### 환경 변수:
```yaml
CARGO_TERM_COLOR: always
RUST_BACKTRACE: 1
RUSTFLAGS: -D warnings        # 모든 경고를 에러로 변환
```

### 2. Security 워크플로우 (`security.yml`) - 신규

#### 목적:
- **보안-중심** CI 파이프라인
- 일일 자동 스캔 (2 AM UTC)
- PR/Push 시 추가 검사

#### 포함 도구:
```
┌─ RustSec Audit          # 공식 보안 데이터베이스
├─ Cargo Audit            # 의존성 취약점 검사
├─ Cargo Deny             # 의존성 정책 검사
├─ Unsafe Code Check      # cargo-geiger로 unsafe 추적
├─ Clippy (Strict Mode)   # 엄격한 린트 검사
├─ Unmaintained Deps      # 구식 의존성 확인
├─ SBOM Generation        # 소프트웨어 BOM 생성
└─ Security Summary       # 종합 보고서
```

#### 트리거:
```yaml
on:
  push:        # 모든 push
  pull_request # 모든 PR
  schedule:    # 매일 2 AM UTC
    - cron: '0 2 * * *'
  workflow_dispatch  # 수동 실행
```

#### 사용 사례:
```bash
# 수동 보안 검사
gh workflow run security.yml

# 특정 이벤트에서 자동 실행
# - 모든 PR에서 자동 실행
# - 매일 2 AM UTC에 자동 스캔
```

### 3. Code Quality 워크플로우 (`code-quality.yml`) - 개선

#### 개선 사항:
```
✅ 일일 스케줄 추가 (3 AM UTC)
✅ Cargo.lock 추적 추가
✅ Unused dependencies 검사 (nightly)
✅ Documentation 커버리지
✅ Complexity 분석
✅ PR 자동 코멘트
```

#### 트리거:
```yaml
on:
  push, pull_request, schedule (3 AM UTC), workflow_dispatch
```

### 4. Benchmark 워크플로우 (`benchmark.yml`) - 강화

#### 주요 개선 사항:
```
✅ 3단계 분리:
   1. Compilation Check  (fast fail)
   2. Run Benchmarks     (현재 브랜치)
   3. Benchmark Compare  (PR 비교)
   4. Extended Benchmarks (선택적)

✅ PR 자동 비교 기능
✅ 수동 트리거로 상세 벤치마크 실행
✅ 90일 아티팩트 보관
```

#### 수동 트리거:
```bash
# 상세 벤치마크 실행
gh workflow run benchmark.yml -f full_bench=true
```

## 보안 강화 - SLSA 및 공급망 보안

### 포함된 보안 검사:

1. **RustSec Database**
   - 공식 Rust 보안 경고 데이터베이스
   - 자동 업데이트

2. **Cargo Audit**
   - 의존성 취약점 검사
   - 세 가지 심각도 수준

3. **Cargo Deny**
   - 라이센스 검사
   - Advisories 검사
   - 정책 기반 필터링

4. **Unsafe Code Tracking**
   - `cargo-geiger` 통합
   - Unsafe 사용 시각화

5. **SBOM Generation**
   - 소프트웨어 명세서 자동 생성
   - 공급망 보안 추적

## 워크플로우 실행 흐름

### PR 제출 시:
```
1. 기본 검사 (fmt, clippy) 실행
   ↓ (병렬)
2. 테스트 스위트 (9개 조합) 실행
3. 다중 플랫폼 빌드 실행
4. 보안 검사 실행
   ↓
5. 코드 커버리지 생성
   ↓
6. 모든 체크 완료 (CI Status)
```

### Push (main/master/develop):
```
1. 위의 PR 체크 +
2. 일일 스케줄 확인 (보안, 품질)
```

### 일일 스케줄:
```
2 AM UTC  → Security 워크플로우 (심층 보안 검사)
3 AM UTC  → Code Quality 워크플로우 (복잡도, 문서화)
```

## 성능 최적화

### 캐싱 전략:
```yaml
# 각 워크플로우는 다음을 캐시합니다:
- ~/.cargo/registry          # 다운로드한 의존성
- ~/.cargo/git               # Git 의존성
- rust/target                # 빌드 아티팩트
```

### 캐시 키:
```yaml
# Cargo.lock 기반 캐싱
${{ runner.os }}-cargo-build-target-${{ matrix.rust }}-${{ hashFiles('**/Cargo.lock') }}
```

### 병렬 실행:
```
- Test Suite:    9개 조합 (3 OS × 3 Rust)
- Build:         3개 플랫폼 병렬
- Code Quality:  6개 job 병렬
- Security:      7개 job 병렬
```

## 필수 설정

### Repository Secrets (선택):
```
☐ CODECOV_TOKEN      - 커버리지 업로드용
☐ CARGO_REGISTRY_TOKEN - crates.io 배포용
```

### Branch Protection (권장):
```
Settings → Branches → Add rule for main/master
- ✓ Require status checks to pass
- ✓ Require code reviews
- ✓ Dismiss stale pull request approvals
```

### Dependabot (권장):
```
Settings → Code security and analysis
- ✓ Enable Dependabot alerts
- ✓ Enable Dependabot security updates
```

## 로컬 테스트 명령어

### CI 검사 재현:
```bash
cd /home/mare/mecab-ko/rust

# 포맷 검사
cargo fmt -- --check

# 린트 검사
cargo clippy --all-targets --all-features -- -D warnings

# 테스트 (Debug + Release)
cargo test --verbose
cargo test --release --verbose

# 빌드
cargo build --release

# 문서 생성
cargo doc --no-deps --release

# 보안 검사
cargo audit
```

### 보안 도구 로컬 설치:
```bash
# cargo-audit
cargo install cargo-audit
cargo audit

# cargo-deny
cargo install cargo-deny
cargo deny check all

# cargo-geiger (unsafe 코드 추적)
cargo install cargo-geiger
cargo geiger

# tarpaulin (커버리지)
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

## 문제 해결

### 워크플로우가 실패하는 경우:

1. **캐시 문제**:
   ```bash
   # GitHub UI에서 캐시 제거
   Actions → Caches → 관련 캐시 삭제
   ```

2. **특정 플랫폼 실패**:
   ```bash
   # 로컬에서 해당 플랫폼 테스트
   rustup target add <target>
   cargo build --target <target>
   ```

3. **보안 검사 실패**:
   ```bash
   cargo audit              # 취약점 확인
   cargo deny check all     # 정책 확인
   cargo-geiger             # unsafe 코드 확인
   ```

## GitHub Actions 실행 모니터링

```bash
# 최근 워크플로우 확인
gh run list --workflow ci.yml

# 특정 워크플로우 상태 확인
gh run view <run-id> --log

# 워크플로우 수동 트리거
gh workflow run security.yml
gh workflow run benchmark.yml -f full_bench=true

# 캐시 현황 확인
gh cache list
```

## 문서 참조

- **주 문서**: `/home/mare/mecab-ko/.github/WORKFLOWS.md`
- **이 파일**: `/home/mare/mecab-ko/.github/CI-CD-IMPROVEMENTS.md`
- **Project Plan**: `/home/mare/mecab-ko/docs/PROJECT_PLAN.md`

## 핵심 통계

### 워크플로우 수:
- **CI**: 1개 (빌드, 테스트, 린트)
- **Security**: 1개 (보안 검사 - 매일)
- **Code Quality**: 1개 (정적 분석 - 매일)
- **Benchmark**: 1개 (성능 측정)
- **기타**: Release, Docs, E2E, Dependabot (5개)
- **총합**: 11개 워크플로우

### 병렬 실행:
- **PR 시**: 최대 30+ job 동시 실행
- **일일 스케줄**: 13+ job 동시 실행

### 평균 실행 시간:
- **CI**: 20-30분
- **Security**: 10-15분
- **Code Quality**: 15-20분
- **Benchmark**: 10-20분

## 다음 단계 (Future Enhancements)

1. **SLSA Level 3** 구현 (provenance 생성)
2. **Container Scanning** (Docker 이미지)
3. **License Compliance** 자동화
4. **Performance Regression** 자동 감지
5. **Fuzzing** 통합 (cargo fuzz)
6. **Code Coverage** 트렌드 추적

---

**마지막 업데이트**: 2026-01-27
**담당자**: MeCab-Ko DevOps Team
