# 완료: Phase 68 - Sprint 102 (E2E CLI 테스트 확장 + full-dict 벤치마크 CI)

## Sprint 102 목표
E2E CLI 테스트를 stub에서 실제 테스트로 확장, full-dict 벤치마크 CI job 추가

## Sprint 102 작업 목록

- [x] S102-01: E2E CLI 테스트 확장 (2 stub → 7 real tests) ✅
- [x] S102-02: benchmark.yml에 full-dict-benchmark job 추가 ✅
- [x] S102-03: 빌드/테스트/clippy 검증 + 커밋 ✅

### 수정 내용
- `tests/e2e/cli/test_basic.sh`: PASS/FAIL/SKIP 추적, 사전 게이팅, portable grep
- `benchmark.yml`: workflow_dispatch `full_dict` 입력, bitbucket 다운로드 + SHA256 검증
- macOS grep -P 비호환 수정 (grep -E + literal tab 사용)

---

## Sprint 103 로드맵

### P1: dict-build CI 전체 결과 검증
- Push 후 dict-build + benchmark 워크플로우 성공 확인
- full-dict-benchmark manual trigger 테스트

### P2: v0.8.0 기능 계획 구체화
- DIC-010: Binary Dict v3.0 설계 (압축 효율 + mmap 지원)
- DIC-009: 사전 검증 테스트셋 구축 (golden test 1,000+ 문장)
- RST-011: 사용자 정의 사전 개선

### P3: E2E 테스트 추가 확장
- Python/Node.js FFI E2E 테스트 실질 구현
- e2e-ffi-tests 워크플로우와 연동

### P4: 코드 품질 + 문서
- README에 E2E 테스트 실행 방법 추가
- CI 워크플로우 문서 정리

---

# 완료: Phase 67 - Sprint 101 (dict-build CI .zst 검증 수정 + SHA256 checksum)

## Sprint 101 목표
dict-build.yml CI 실패 수정 — 압축 출력 파일(.zst) 검증, SHA256 checksum 추가

## Sprint 101 작업 목록

- [x] S101-01: dict-build.yml 파일 검증 — .zst 압축 파일 지원 ✅
- [x] S101-02: bitbucket 다운로드에 SHA256 checksum 추가 ✅
- [x] S101-03: tokenize-test/generate-report 조건부 실행 (빌드 성공 시만) ✅

---

# 완료: Phase 66 - Sprint 100 (벤치마크 + dict-build CI + e2e scaffolding)

---

# 완료: Phase 65 - Sprint 99 (CI nightly Rust continue-on-error)

---

# 완료: Phase 64 - Sprint 98 (Python Bindings CI 테스트 dict-aware skip)

---

# 완료: Phase 63 - Sprint 97 (python-wheels Octokit + e2e-ffi-tests virtualenv/compat 수정)

---

# 완료: Phase 62 - Sprint 96 (excluded FFI crate workspace root 해결 + CI 추가 수정)

---

# 완료: Phase 61 - Sprint 95 (CI 워크플로우 6건 수정 + accuracy tests ignore)

---

# 완료: Phase 60 - Sprint 94 (CI 워크플로우 수정 + Actions Node.js 24 대응)

## Sprint 94 목표
CI 워크플로우 3개 실패 수정, GitHub Actions Node.js 20 deprecation 대응 (v4→v5)

## Sprint 94 작업 목록

### Track A: CI 워크플로우 수정
- [x] S94-01: python-wheels.yml — PyO3/maturin-action@v2 → @v1 (v2 미존재) ✅
- [x] S94-02: npm-publish.yml — wasm-pack workspace root 해결 (repo root에서 실행) ✅
- [x] S94-03: docker.yml — id-token: write + attestations: write 권한 추가 ✅

### Track B: Actions 업데이트
- [x] S94-04: 전체 16개 워크플로우 actions v4→v5 (checkout, upload/download-artifact, setup-node) ✅

### Track C: 검증
- [x] S94-05: 빌드/테스트/클리피 검증 + 리뷰 + Sprint 95 로드맵 ✅

### CI 워크플로우 수정 상세
| 워크플로우 | 문제 | 수정 |
|-----------|------|------|
| python-wheels.yml | `maturin-action@v2` 미존재 | `@v1` (최신 v1.50.0) |
| npm-publish.yml | wasm-pack workspace root 못 찾음 | repo root에서 crate path 인자로 실행 |
| docker.yml | attest-build-provenance OIDC 토큰 없음 | `id-token: write` + `attestations: write` 권한 |

### 잔여 CI 이슈 (Sprint 94 범위 외)
- ci.yml: 테스트 30개 실패 (사전 데이터 미포함 — 구조적 문제)
- dict-build.yml: 사전 빌드 런타임 에러 (bitbucket 다운로드/파싱)

---

## Sprint 95 로드맵

### P1: CI 워크플로우 검증
- push 후 python-wheels, npm-publish, docker 워크플로우 재실행 확인
- ci.yml 테스트 실패 원인 분석 (사전 데이터 의존성)

### P2: 시스템 사전 벤치마크
- `brew install mecab-ko-dic` 후 MECAB_DICDIR 설정
- full-dict 벤치마크 실행 + mini-dict 비교 리포트

### P3: v0.8.0 기능 계획 구체화
- ISSUE_BACKLOG에서 P0 항목 선정
- Binary Dict v3 (DIC-010), 사전 검증 (DIC-009), 사용자 사전 개선 (RST-011)

### P4: ci.yml 사전 데이터 테스트 해결
- mini-dict 또는 fixture 기반 CI 테스트 전략 수립
- dict-build.yml 안정화

---

# 완료: Phase 59 - Sprint 93 (git tag v0.7.2 + GitHub Release + docs.rs 검증 + MSRV 문서)
# 완료: Phase 58 - Sprint 92 (문서 v0.7.2 갱신 + RELEASE_CHECKLIST + git tag 준비)

---

# 완료: Phase 56 - Sprint 90 (벤치마크 문서화 + 코드 품질 + 배포 준비)
# 완료: Phase 55 - Sprint 89 (v0.7.2 릴리스 준비 + 코드 품질 + 벤치마크)
# 완료: Phase 54 - Sprint 88 (MSRV 1.80 + LazyLock + CI 수정)
# 완료: Phase 53 - Sprint 87 (per-call 할당 제거 성능 최적화)
# 완료: Phase 52 - Sprint 86 (v0.7.1 릴리스 + 보호 테스트 확충)
# 완료: Phase 51 - Sprint 85 (corrections/ 디렉토리 전환)
# 완료: Sprint 80-84 (corrections 분할 1-4차)
# 아카이브: Sprint 1-72 (.history/ 및 Git 히스토리 참조)

## 크레이트 발행 현황

| 크레이트 | crates.io | 로컬 | 상태 |
|---------|-----------|------|------|
| mecab-ko-hangul | v0.7.2 | v0.7.2 | ✅ 배포 완료 |
| mecab-ko-dict | v0.7.2 | v0.7.2 | ✅ 배포 완료 |
| mecab-ko-core | v0.7.2 | v0.7.2 | ✅ 배포 완료 |
| mecab-ko-dict-validator | v0.7.2 | v0.7.2 | ✅ 배포 완료 |
| mecab-ko-dict-builder | v0.7.2 | v0.7.2 | ✅ 배포 완료 |
| mecab-ko-dict-sync | v0.7.2 | v0.7.2 | ✅ 배포 완료 |
| mecab-ko | v0.7.2 | v0.7.2 | ✅ 배포 완료 |
