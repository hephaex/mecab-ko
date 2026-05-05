# 완료: Phase 66 - Sprint 100 (벤치마크 + dict-build CI + e2e scaffolding)

## Sprint 100 목표
시스템 사전 벤치마크, dict-build CI 안정화, E2E 테스트 기초 구조

## Sprint 100 작업 목록

### Track A: 벤치마크 (병렬)
- [x] S100-01: Full-dict 벤치마크 (MECAB_DICDIR=data/dict-output, 816K entries) ✅
- [x] S100-02: Mini-dict vs Full-dict 비교 리포트 ✅

### Track B: CI/인프라 (병렬)
- [x] S100-03: dict-build.yml — bitbucket curl 다운로드 추가 ✅
- [x] S100-05: DEFAULT_DICDIR_PATHS에 Homebrew ARM 경로 추가 ✅

### Track C: E2E scaffolding (병렬)
- [x] S100-04: tests/e2e/ 디렉토리 생성 (CLI, Python, Node.js stubs) ✅

### Track D: 계획
- [x] S100-06: v0.8.0 기능 계획 — ISSUE_BACKLOG 분석 완료 ✅

### 벤치마크 결과: Mini-dict vs Full-dict (816K entries)

| Benchmark | Full-Dict | Mini-Dict | Ratio |
|-----------|----------|-----------|-------|
| tokenize/short (5자) | 7.82 µs | 1.54 µs | 5.1x |
| tokenize/medium (70자) | 86.70 µs | 11.33 µs | 7.7x |
| tokenize/long (200자) | 270.21 µs | 87.65 µs | 3.1x |
| by_text_type/news | 142.05 µs | — | — |
| by_text_type/technical | 245.57 µs | — | — |
| tokenizer_creation | 127.37 ms | 75.17 µs | 1,694x |
| consecutive/5_texts | 590.26 µs | 187.08 µs | 3.2x |
| throughput/1KB | 2.10 ms | — | — |
| throughput/10KB | 110.84 ms | — | — |

**핵심 발견:**
- 토크나이저 생성 시간: full-dict 127ms vs mini-dict 75µs (1,694x 차이 — 사전 로딩 비용)
- 분석 속도: full-dict이 3~8x 느림 (trie 탐색 범위 증가)
- Full-dict throughput: ~1 MiB/s (1KB), ~213 KiB/s (10KB)

---

## Sprint 101 로드맵

### P1: v0.8.0 기능 계획 구체화
- DIC-010: Binary Dict v3.0 설계 (압축 효율 + mmap 지원)
- DIC-009: 사전 검증 테스트셋 구축 (golden test 1,000+ 문장)
- RST-011: 사용자 정의 사전 개선

### P2: dict-build.yml CI 검증
- Push 후 bitbucket 다운로드 동작 확인
- dict-build 워크플로우 전체 성공 검증

### P3: E2E 테스트 실질 구현
- tests/e2e/ stub을 실제 테스트로 확장
- e2e-ffi-tests 워크플로우와 연동

### P4: Full-dict 벤치마크 기준선 자동화
- CI Performance Benchmarks에 MECAB_DICDIR 옵션 추가
- full-dict 벤치마크 regression 자동 감지

### P5: checksum 검증 추가
- dict-build.yml curl 다운로드에 SHA256 검증 추가 (코드 리뷰 권고)

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
