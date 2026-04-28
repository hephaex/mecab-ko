# 완료: Phase 59 - Sprint 93 (git tag v0.7.2 + GitHub Release + docs.rs 검증 + MSRV 문서)

## Sprint 93 목표
git tag v0.7.2 + GitHub Release, docs.rs 빌드 검증, MSRV 문서 갱신, v0.8.0 로드맵 조사

## Sprint 93 작업 목록

### Track A: 릴리스
- [x] S93-01: git tag v0.7.2 + GitHub Release — 태그 push 완료, release.yml 트리거 ✅

### Track B: 검증
- [x] S93-02: docs.rs v0.7.2 빌드 확인 — 7/7 크레이트 100% documented ✅
- [x] S93-03: 시스템 사전 설치 문서 검증 + MSRV 1.75→1.80 갱신 (5파일) ✅

### Track C: 조사
- [x] S93-04: v0.8.0 기능 후보 조사 — ISSUE_BACKLOG 분석, TODO/FIXME 스캔 완료 ✅

### Track D: 리뷰
- [x] S93-05: 빌드/테스트/클리피 검증 + 리뷰 + Sprint 94 로드맵 ✅

### CI 워크플로우 상태 (v0.7.2 태그)
- release.yml: ✅ GitHub Release 생성, 바이너리 빌드 진행 중
- python-wheels.yml: ❌ maturin build 실패 (기존 이슈)
- npm-publish.yml: ❌ wasm-pack build 실패 (기존 이슈)
- docker.yml: ❌ OIDC 토큰 설정 누락 (기존 이슈)

---

## Sprint 94 로드맵

### P1: CI 워크플로우 수정 (3개)
- python-wheels.yml: maturin build 에러 진단 + 수정
- npm-publish.yml: wasm-pack build 에러 진단 + 수정
- docker.yml: OIDC 토큰 / permissions 설정

### P2: 시스템 사전 벤치마크
- `brew install mecab-ko-dic` 후 MECAB_DICDIR 설정
- full-dict 벤치마크 실행 + mini-dict 비교 리포트

### P3: GitHub Actions Node.js 20 deprecation
- actions/checkout@v4 → v5 (Node.js 24 지원) 검토
- 전체 워크플로우 업데이트

### P4: v0.8.0 기능 계획 구체화
- ISSUE_BACKLOG에서 P0 항목 선정
- Binary Dict v3 (DIC-010), 사전 검증 (DIC-009), 사용자 사전 개선 (RST-011)

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
