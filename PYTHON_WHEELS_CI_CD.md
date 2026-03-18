# Python Wheels CI/CD 워크플로우 가이드

Python 멀티플랫폼 wheel 자동 빌드 및 PyPI 배포 시스템 설정 안내

## 목차

1. [개요](#개요)
2. [GitHub Actions 워크플로우 구조](#github-actions-워크플로우-구조)
3. [PyPI Trusted Publisher 설정](#pypi-trusted-publisher-설정)
4. [TestPyPI 설정](#testpypi-설정)
5. [워크플로우 트리거](#워크플로우-트리거)
6. [빌드 매트릭스](#빌드-매트릭스)
7. [테스트 전략](#테스트-전략)
8. [배포 프로세스](#배포-프로세스)
9. [트러블슈팅](#트러블슈팅)
10. [모니터링 및 유지보수](#모니터링-및-유지보수)

## 개요

이 워크플로우는 다음을 자동화합니다:

- **멀티플랫폼 wheel 빌드**: maturin을 사용한 자동 빌드
- **자동 테스트**: 각 플랫폼에서 빌드된 wheel 검증
- **PyPI 배포**: OIDC Trusted Publisher를 사용한 보안 배포
- **아티팩트 관리**: 자동 아티팩트 생성 및 보존

### 지원 플랫폼

| 플랫폼 | 아키텍처 | 빌드 환경 | manylinux |
|--------|---------|---------|----------|
| Linux | x86_64 | ubuntu-latest | manylinux2014 |
| Linux | aarch64 | ubuntu-latest | manylinux2014 |
| macOS | x86_64 | macos-latest | - |
| macOS | arm64 | macos-latest | - |
| Windows | x86_64 | windows-latest | - |

### Python 버전

Python 3.8 ~ 3.13 모두 지원 (pyproject.toml에서 정의)

## GitHub Actions 워크플로우 구조

워크플로우 파일: `.github/workflows/python-wheels.yml`

### 주요 Jobs

#### 1. build_wheels (병렬 실행 - 5개 플랫폼/아키텍처)

각 플랫폼별로 병렬로 wheel을 빌드합니다.

**단계:**
1. 코드 체크아웃
2. Rust 및 빌드 도구 설정
3. 캐싱 (Rust 의존성)
4. maturin을 사용한 wheel 빌드
5. 빌드된 wheel을 아티팩트로 업로드

**특징:**
- `fail-fast: false` - 한 플랫폼 실패 시 다른 플랫폼은 계속 빌드
- sccache 활성화 - 컴파일 속도 향상
- QEMU를 사용한 Linux ARM 빌드 지원

#### 2. build_sdist

소스 배포(source distribution)를 빌드합니다.

- 한 번만 빌드 (ubuntu-latest에서)
- PyPI 업로드 시 필수

#### 3. test_wheels

빌드된 wheel을 여러 플랫폼과 Python 버전에서 테스트합니다.

**테스트 환경:**
- Ubuntu + Python 3.8, 3.12, 3.13
- macOS + Python 3.11
- Windows + Python 3.11

**테스트 단계:**
1. wheel 설치
2. import 테스트
3. pytest 테스트 실행 (존재 시)

#### 4. verify_wheels

빌드된 wheel의 무결성 및 호환성을 검증합니다.

**검증 항목:**
- 휠 구조 확인
- twine으로 메타데이터 검증
- Linux wheel 호환성 감시 (auditwheel)
- 빌드 리포트 생성

#### 5. publish_to_pypi

PyPI에 wheel을 배포합니다.

**트리거 조건:**
- Release 생성 시 (자동 배포)
- workflow_dispatch + publish_to_pypi=true

**특징:**
- OIDC Trusted Publisher 사용
- 자동 메타데이터 검증
- PyPI/TestPyPI 선택 가능

#### 6. update_release_with_wheels

Release body에 wheel 정보를 추가합니다.

**Release에 추가되는 정보:**
- 빌드된 wheel 목록
- 지원 플랫폼
- 파일 크기

## PyPI Trusted Publisher 설정

Trusted Publisher는 장기 API 토큰 대신 OIDC를 사용하여 안전하게 PyPI에 배포합니다.

### 사전 요구사항

1. PyPI 프로젝트 생성 필요
2. 프로젝트 Owner 권한 필요
3. GitHub 저장소 공개 또는 프로젝트 Owner가 접근 가능해야 함

### 설정 단계

#### Step 1: PyPI 프로젝트 확인

```
https://pypi.org/project/mecab-ko-python/
```

프로젝트 페이지에서 'Manage' 클릭

#### Step 2: Trusted Publisher 추가

1. Settings > Publishing 섹션으로 이동
2. "Add Trusted Publisher" 클릭
3. 다음 정보 입력:

```
PyPI or TestPyPI: PyPI
GitHub owner or organization: hephaex
Repository name: mecab-ko
Workflow filename: python-wheels.yml
Environment name: pypi
```

4. "Add trusted publisher" 저장

#### Step 3: GitHub에서 Environment 확인

Repository Settings > Environments에서 'pypi' environment가 생성되었는지 확인

```
Deployment branches: Only allow deployments to this repository from specified environments
```

선택사항으로 배포 브랜치 제한 가능:
- main
- releases/*

#### Step 4: OIDC 토큰 설정

OIDC 토큰은 PyPI가 자동으로 관리하므로 추가 설정 불필요합니다.

### 보안 기능

```
✓ API 토큰이 GitHub Secrets에 저장되지 않음
✓ OIDC 토큰은 5분 TTL (매우 짧은 생명주기)
✓ 신뢰할 수 있는 워크플로우 실행 시에만 생성
✓ PyPI에서 모든 배포 시도 감시 가능
✓ 환경별 보호 규칙 설정 가능
```

## TestPyPI 설정

프로덕션 배포 전에 wheel을 테스트하려면 TestPyPI 사용을 권장합니다.

### TestPyPI 계정 설정

```
https://test.pypi.org/
```

1. 계정 생성 (또는 기존 계정 사용)
2. 프로젝트 'mecab-ko-python-test' 생성

### Trusted Publisher 추가

PyPI와 동일한 방법으로 TestPyPI에도 Trusted Publisher 추가:

```
PyPI or TestPyPI: TestPyPI
GitHub owner or organization: hephaex
Repository name: mecab-ko
Workflow filename: python-wheels.yml
Environment name: testpypi
```

### GitHub Environment 생성

Repository Settings > Environments > New environment

```
Environment name: testpypi
Deployment branches: No restriction (또는 특정 브랜치)
```

### 워크플로우에서 사용

workflow_dispatch 입력으로 `pypi_repository: testpypi` 선택:

```bash
# TestPyPI에 배포
gh workflow run python-wheels.yml \
  -f publish_to_pypi=true \
  -f pypi_repository=testpypi
```

### 설치 및 테스트

```bash
# TestPyPI에서 설치
pip install -i https://test.pypi.org/simple/ mecab-ko-python

# 버전 확인
python -c "import mecab_ko; print(mecab_ko.__version__)"
```

## 워크플로우 트리거

### 1. 자동 트리거: main 브랜치 푸시

```yaml
on:
  push:
    branches:
      - main
    paths:
      - 'rust/crates/mecab-ko-python/**'
      - '.github/workflows/python-wheels.yml'
```

**동작:**
- 빌드 및 테스트만 실행
- 배포하지 않음

**사용 사례:**
- Python 바인딩 코드 변경
- 워크플로우 자체 수정

### 2. Release 생성 시 자동 배포

```yaml
on:
  release:
    types:
      - created
```

**동작:**
- 전체 빌드, 테스트, 검증 실행
- 자동으로 PyPI에 배포

**사용 사례:**
- GitHub에서 Release 생성
- 모든 wheel 자동 생성 및 배포

### 3. 수동 트리거 (workflow_dispatch)

```
Actions > Python Wheels Build & Deploy > Run workflow
```

**입력 옵션:**
- `publish_to_pypi`: true/false
- `pypi_repository`: pypi or testpypi

**사용 사례:**
- TestPyPI에 수동 테스트 배포
- 빌드만 수행 (배포 없음)
- 프로덕션 PyPI에 수동 배포

### 동시성 제어

```yaml
concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true
```

- 같은 브랜치에 대한 중복 실행 자동 취소
- 최신 push만 실행

## 빌드 매트릭스

### 플랫폼별 설정

#### Linux x86_64

```yaml
platform: linux
runner: ubuntu-latest
arch: x86_64
target: x86_64-unknown-linux-gnu
manylinux: manylinux2014
```

#### Linux aarch64 (ARM)

```yaml
platform: linux
runner: ubuntu-latest
arch: aarch64
target: aarch64-unknown-linux-gnu
manylinux: manylinux2014
```

**특징:** QEMU를 사용하여 ARM 에뮬레이션

#### macOS x86_64

```yaml
platform: macos
runner: macos-latest
arch: x86_64
target: x86_64-apple-darwin
```

#### macOS arm64 (Apple Silicon)

```yaml
platform: macos
runner: macos-latest
arch: arm64
target: aarch64-apple-darwin
```

#### Windows x86_64

```yaml
platform: windows
runner: windows-latest
arch: x86_64
target: x86_64-pc-windows-msvc
```

### Python 버전

maturin 빌드 시 모든 지원 버전에 대해 wheel 생성:

```
--interpreter 3.8 3.9 3.10 3.11 3.12 3.13
```

### 병렬화 이점

- 5개 플랫폼/아키텍처 동시 빌드
- 전체 빌드 시간: ~20-30분 (플랫폼별 차이)
- `fail-fast: false` - 한 플랫폼 실패해도 다른 플랫폼 계속 진행

## 테스트 전략

### 빌드된 Wheel 테스트

test_wheels 작업에서 여러 환경 조합 테스트:

```
ubuntu-latest + Python 3.8     (최소 버전)
ubuntu-latest + Python 3.12    (최신 안정)
ubuntu-latest + Python 3.13    (최신 베타)
macos-latest + Python 3.11     (대표 버전)
windows-latest + Python 3.11   (대표 버전)
```

### 테스트 항목

1. **Import 테스트**
   ```python
   import mecab_ko
   print(mecab_ko.__version__)
   ```

2. **pytest 테스트**
   ```
   pytest tests/ -v --tb=short
   ```
   (tests 디렉토리 존재 시)

3. **의존성 검증**
   - 모든 필요한 라이브러리 설치 확인
   - 라이브러리 버전 호환성 확인

### 메타데이터 검증

```
python -m twine check dist/*/*.whl dist/*/*.tar.gz
```

검증 항목:
- wheel 파일 이름 형식
- METADATA 파일 형식
- license 정보
- 버전 형식

## 배포 프로세스

### 1. Release 생성을 통한 자동 배포

```bash
# GitHub 웹 UI에서 Release 생성
# 또는 gh CLI 사용:
gh release create v0.5.0 \
  -t "Version 0.5.0" \
  -n "Release notes here"
```

**자동 프로세스:**

```
Release created
    ↓
build_wheels (병렬, 5 플랫폼)
    ↓
build_sdist
    ↓
test_wheels (병렬, 여러 환경)
    ↓
verify_wheels
    ↓
publish_to_pypi (자동)
    ↓
update_release_with_wheels (wheel 정보 추가)
```

**예상 시간:** 30-45분

### 2. 수동 배포 (workflow_dispatch)

**Step 1:** Actions 탭 > Python Wheels Build & Deploy 선택

**Step 2:** Run workflow 클릭

**Step 3:** 옵션 선택:
- `publish_to_pypi`: true
- `pypi_repository`: pypi 또는 testpypi

**Step 4:** Run workflow

### 3. 배포 후 확인

#### PyPI 확인

```
https://pypi.org/project/mecab-ko-python/
```

- 최신 버전 표시
- wheel 다운로드 가능
- 릴리스 이력 표시

#### 설치 테스트

```bash
pip install mecab-ko-python

python -c "import mecab_ko; print(mecab_ko.__version__)"
```

#### Release Notes 확인

GitHub Release에서 wheel 정보 확인:

```
## Python Wheels Build Report

**Build Date:** 2024-03-18 10:30:00 UTC

### Built Wheels
- mecab_ko-0.5.0-cp38-cp38-manylinux_2_17_x86_64.manylinux2014_x86_64.whl (1.2 MB)
- mecab_ko-0.5.0-cp39-cp39-manylinux_2_17_x86_64.manylinux2014_x86_64.whl (1.2 MB)
...

### Source Distribution
- mecab-ko-python-0.5.0.tar.gz (50 KB)
```

## 트러블슈팅

### 문제 1: 특정 플랫폼에서 빌드 실패

**증상:** 예) Linux aarch64 빌드만 실패

**진단:**
1. GitHub Actions 로그 확인
2. 해당 플랫폼의 상세 에러 메시지 확인
3. QEMU 설정 재확인 (Linux ARM)

**해결:**
```bash
# 로컬에서 재현 (Docker 사용)
docker run --rm -it quay.io/pypa/manylinux2014_aarch64:latest

# 또는 해당 플랫폼 러너에서 스케줄링
```

### 문제 2: Wheel 구조 이상

**증상:** "Verify wheel structure" 단계 실패

**원인:**
- maturin 버전 호환성
- pyproject.toml 설정 오류
- Rust 의존성 버전 충돌

**해결:**

1. maturin 버전 확인:
```bash
cd rust/crates/mecab-ko-python
python -m pip install --upgrade maturin
```

2. pyproject.toml 검증:
```bash
python -m pip install build
python -m build --wheel
```

3. 로컬 빌드 테스트:
```bash
cd rust/crates/mecab-ko-python
maturin develop
python -c "import mecab_ko"
```

### 문제 3: PyPI 배포 실패

**증상:** "Publish to PyPI" 단계에서 403 Forbidden

**원인:**
- Trusted Publisher 설정 오류
- OIDC 토큰 생성 실패
- 환경 설정 누락

**해결:**

1. Trusted Publisher 재확인:
```
PyPI > 프로젝트 settings > Publishing
```

2. GitHub Environment 확인:
```
Repository Settings > Environments > pypi
```

3. Environment 보호 규칙 확인:
```
Deployment branches에 "main" 포함되어 있는가?
```

4. 워크플로우 권한 확인:
```yaml
permissions:
  id-token: write    # OIDC 토큰 필수
  contents: read
```

### 문제 4: TestPyPI와 PyPI 혼동

**증상:** "Package already exists" 또는 버전 충돌

**해결:**

1. 테스트 시 버전 명확히:
```
프로덕션: 0.5.0
테스트: 0.5.0.dev1 또는 0.5.0rc1
```

2. pyproject.toml에서 버전 분리:
```toml
[project]
version = "0.5.0.dev1"  # 테스트용
```

3. Release 생성 전 TestPyPI 테스트:
```bash
gh workflow run python-wheels.yml \
  -f publish_to_pypi=true \
  -f pypi_repository=testpypi
```

### 문제 5: Artifact 관련

**증상:** "Upload artifact failed" 또는 아티팩트 누락

**원인:**
- Wheel 빌드 실패 (조용히 실패)
- 아티팩트 경로 오류
- 저장 공간 부족

**해결:**

1. 로그에서 maturin 빌드 결과 확인
2. 아티팩트 경로 검증:
```bash
ls -la rust/crates/mecab-ko-python/target/wheels/
```

3. Actions 저장 공간 확인:
```
Repository Settings > Security > Storage
```

## 모니터링 및 유지보수

### 정기적인 확인

#### 월간 확인사항

1. **Python 버전 업데이트**
   - 새로운 Python 버전 릴리스 모니터링
   - pyproject.toml의 classifiers 업데이트
   - 워크플로우 테스트 매트릭스 업데이트

2. **의존성 업데이트**
   ```bash
   cargo update
   python -m pip install --upgrade maturin
   ```

3. **GitHub Actions 버전 확인**
   ```yaml
   - uses: actions/checkout@v4        # 최신 버전?
   - uses: actions/setup-python@v5    # 최신 버전?
   - uses: PyO3/maturin-action@v1    # 최신 버전?
   ```

#### 분기별 확인사항

1. **Wheel 크기 및 호환성 검토**
   - Release에서 빌드 리포트 확인
   - 비정상적인 크기 변화 감지
   - auditwheel 호환성 경고 검토

2. **테스트 커버리지**
   - 새로운 테스트 케이스 추가
   - 플랫폼별 테스트 결과 분석

3. **배포 프로세스 드릴**
   - TestPyPI에 수동 배포 시뮬레이션
   - Rollback 프로세스 검증

### 성능 최적화

#### 빌드 시간 단축

1. **Rust 캐시 활용**
```yaml
- uses: Swatinem/rust-cache@v2
  with:
    cache-all-crates: 'true'
```

2. **sccache 활성화**
```yaml
- uses: PyO3/maturin-action@v1
  with:
    sccache: 'true'
```

3. **불필요한 검사 제거**
- continue-on-error 기능 활용
- 선택적 작업 실행 (if 조건)

#### 비용 절감

- `fail-fast: false` 유지 (한 번에 모든 플랫폼 빌드)
- artifact 보존 기간: 7일 (기본값)
- 불필요한 job 병렬화 피하기

### 로그 및 아티팩트 관리

#### 아티팩트 정책

```
보존 기간: 7일
대상: wheels, sdist, wheel-report
정책: 자동 삭제 (보존 기간 후)
```

#### Release 노트 활용

각 Release의 Wheel Build Report에서:
- 지원 플랫폼 확인
- 파일 크기 추적
- 빌드 날짜/시간 기록

### 문서 업데이트

이 파일 업데이트 체크리스트:

- [ ] 새로운 Python 버전 추가
- [ ] 플랫폼 변경 (추가/제거)
- [ ] PyPI 정책 변경
- [ ] GitHub Actions 버전 업그레이드
- [ ] 알려진 문제 추가

## 참고 자료

### 공식 문서

- [maturin 문서](https://www.maturin.rs/)
- [PyPA Trusted Publishers](https://docs.pypa.io/en/latest/trusted-publishers/)
- [PyO3 가이드](https://pyo3.rs/)
- [PEP 427 - Python Distribution Wheel](https://peps.python.org/pep-0427/)
- [PEP 425 - Compatibility Tags](https://peps.python.org/pep-0425/)

### 관련 작업

- `rust/crates/mecab-ko-python/pyproject.toml` - 빌드 설정
- `rust/crates/mecab-ko-python/Cargo.toml` - Rust 설정
- `rust/crates/mecab-ko-python/python/` - Python 소스
- `.github/workflows/python-wheels.yml` - CI/CD 워크플로우

### 관련 Issue/PR 템플릿

Release 생성 시 다음 정보 포함:

```markdown
## Release v0.5.0

### Changes
- [변경 사항 설명]

### Python Wheels
- Linux x86_64 (manylinux2014)
- Linux aarch64 (manylinux2014)
- macOS x86_64
- macOS arm64 (Apple Silicon)
- Windows x86_64

### Python Versions
3.8, 3.9, 3.10, 3.11, 3.12, 3.13

### Installation
```bash
pip install mecab-ko-python
```

### Wheel Build Report
[자동으로 추가됨]
```

---

**마지막 업데이트:** 2024-03-18
**작성자:** Deployment Engineering Team
