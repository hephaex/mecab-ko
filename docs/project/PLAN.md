# 완료: Phase 62 - Sprint 96 (excluded FFI crate workspace root 해결 + CI 추가 수정)

## Sprint 96 목표
Sprint 95 push 후 CI 검증, excluded FFI 크레이트 workspace 상속 근본 해결

## Sprint 96 작업 목록

### Track A: Excluded FFI Crate Workspace Root 해결
- [x] S96-01: mecab-ko-python Cargo.toml — workspace 상속 제거, 직접 값 지정 ✅
- [x] S96-02: mecab-ko-wasm Cargo.toml — workspace 상속 제거, 직접 값 지정 ✅
- [x] S96-03: mecab-ko-node Cargo.toml — workspace 상속 제거, 직접 값 지정 ✅

### Track B: CI 워크플로우 추가 수정
- [x] S96-04: search-plugins.yml — Prepare artifacts step에 shell: bash 추가 ✅
- [x] S96-05: e2e-ffi-tests.yml — MSRV 1.75→1.80, wasm-pack installer URL 통일 ✅

### Track C: 테스트 분리
- [x] S96-06: generate_gold.rs #[ignore] 추가 (sys.dic 의존, CI 패닉 방지) ✅

### 근본 원인 분석
| 문제 | 근본 원인 | 해결 |
|------|----------|------|
| python-wheels, wasm-bindings, node-bindings workspace root 실패 | `exclude`된 크레이트에서 `*.workspace = true` 사용 — Cargo가 workspace root를 찾을 수 없음 | workspace 상속 제거, 직접 값 지정 |
| search-plugins Windows copy 실패 | `cp ... \` backslash 연속이 PowerShell에서 해석 실패 | shell: bash 추가 |
| ci.yml test 실패 | generate_gold_standards가 sys.dic 필요 | #[ignore] 추가 |
| e2e-ffi-tests MSRV | 1.75.0 → LazyLock 빌드 실패 | 1.80.0으로 변경 |

### Sprint 95 검증 결과
| 워크플로우 | Sprint 95 수정 후 | Sprint 96 수정 |
|-----------|-----------------|----------------|
| python-wheels.yml | ❌ workspace root | ✅ workspace 상속 제거 |
| search-plugins.yml | ❌ copy step PowerShell | ✅ shell: bash 추가 |
| ci.yml | ❌ generate_gold_standards 패닉 | ✅ #[ignore] 추가 |
| e2e-ffi-tests.yml | ❌ wasm workspace root + MSRV | ✅ workspace 상속 제거 + 1.80 |
| docker.yml | ✅ SUCCESS | — |
| code-quality.yml | ✅ SUCCESS | — |
| security.yml | ✅ SUCCESS | — |
| docs.yml | ✅ SUCCESS | — |
| benchmark.yml | ✅ SUCCESS | — |

---

## Sprint 97 로드맵

### P1: CI 재검증
- push 후 전체 워크플로우 통과 확인
- npm-publish는 다음 태그 시 확인

### P2: 시스템 사전 벤치마크
- `brew install mecab-ko-dic` 후 full-dict 벤치마크
- mini-dict vs full-dict 성능 비교 리포트

### P3: v0.8.0 기능 계획 구체화
- Binary Dict v3, 사전 검증, 사용자 사전 개선 우선순위 결정

### P4: dict-build.yml 안정화
- bitbucket 다운로드/파싱 오류 해결

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
