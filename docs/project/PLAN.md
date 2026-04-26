# 완료: Phase 54 - Sprint 88 (MSRV 1.80 + LazyLock + CI 수정)

## Sprint 88 목표
MSRV 1.80 업그레이드, OnceLock→LazyLock 전환, tag_map 정적화, CI benchmark 자동커밋 제거

## Sprint 88 작업 목록

### Track A: 성능 + 코드 단순화
- [x] S88-01: 벤치마크 인프라 확인 (10개 벤치 파일) ✅
- [x] S88-02: tag_map.rs per-call HashMap → LazyLock 전환 ✅
- [x] S88-04: MSRV 1.75→1.80 + OnceLock→LazyLock (accessor fn 4개 제거) ✅

### Track B: CI/CD
- [x] S88-03: benchmark.yml main 자동커밋 → artifact 업로드로 전환 ✅

### Track C: 검증 + 리뷰
- [x] S88-05: 빌드/테스트/클리피 전체 검증 (1,198 pass / 0 fail) ✅
- [x] S88-06: Sprint 88 리뷰 + 문서 + Sprint 89 로드맵 ✅

---

## Sprint 89 로드맵

### P1: v0.7.2 릴리스
- MSRV 1.80 + 성능 최적화 반영 버전
- CHANGELOG 작성
- crates.io 7개 크레이트 업데이트 배포
- README/문서 MSRV 표기 갱신

### P2: 벤치마크 before/after 비교
- Sprint 87-88 최적화(25 per-call 제거 + LazyLock) 효과 정량 측정
- tokenizer_bench, normalization_bench 기준
- 결과를 docs/benchmarks/ 에 기록

### P3: 잔여 코드 품질 개선
- OnceLock 잔여 확인: tests/common/mod.rs (SYSTEM_DICT_AVAILABLE) — 테스트용이므로 유지
- corrections/ 모듈 문서화 (각 서브파일 모듈 doc comment)
- `#[allow(clippy::too_many_lines)]` 잔여 2개 (mod.rs, tag_map.rs) 검토

### P4: FFI crate 업데이트
- python/wasm/node crate가 MSRV 1.80 호환 확인
- mecab-ko-node: 이미 1.77 → 1.80으로 자동 상속
- FFI 통합 테스트 실행

---

# 완료: Phase 53 - Sprint 87 (per-call 할당 제거 성능 최적화)

## Sprint 87 작업 목록
- [x] S87-01: 매 호출 HashMap/Vec 패턴 조사 (22개 발견) ✅
- [x] S87-02: OnceLock/const 정적 전환 (6개 파일, 25개 할당) ✅
- [x] S87-03: verb_splitting.rs 분할 불필요 확인 ✅
- [x] S87-04: 전체 검증 ✅
- [x] S87-05: 리뷰 + Sprint 88 로드맵 ✅

---

# 완료: Phase 52 - Sprint 86 (v0.7.1 릴리스 + 보호 테스트 확충)
# 완료: Phase 51 - Sprint 85 (corrections/ 디렉토리 전환)
# 완료: Sprint 80-84 (corrections 분할 1-4차)
# 완료: Sprint 73-77 (v0.7.1, CI 통합, 분석)
# 아카이브: Sprint 1-72 (.history/ 및 Git 히스토리 참조)

## 크레이트 발행 현황

| 크레이트 | crates.io | 로컬 | 상태 |
|---------|-----------|------|------|
| mecab-ko-hangul | v0.7.1 | v0.7.1 | ✅ 배포 완료 |
| mecab-ko-dict | v0.7.1 | v0.7.1 | ✅ 배포 완료 |
| mecab-ko-core | v0.7.1 | v0.7.1 | ✅ 배포 완료 |
| mecab-ko-dict-validator | v0.7.1 | v0.7.1 | ✅ 배포 완료 |
| mecab-ko-dict-builder | v0.7.1 | v0.7.1 | ✅ 배포 완료 |
| mecab-ko-dict-sync | v0.7.1 | v0.7.1 | ✅ 배포 완료 |
| mecab-ko | v0.7.1 | v0.7.1 | ✅ 배포 완료 |
