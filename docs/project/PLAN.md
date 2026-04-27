# 완료: Phase 56 - Sprint 90 (벤치마크 문서화 + 코드 품질 + 배포 준비)

## Sprint 90 목표
벤치마크 결과 문서화, example clippy 경고 수정, VecDeque const 검증, FFI 호환성 확인, crates.io dry-run

## Sprint 90 작업 목록

### Track A: 문서
- [x] S90-01: docs/benchmarks/v0.7.2-benchmark-report.md 작성 ✅
- [x] S90-03: mecab-ko-core crate-level doc 확인 (이미 존재) ✅

### Track B: 코드 품질
- [x] S90-02: example clippy 경고 수정 (expect→?, unwrap→unwrap_or, 3파일) ✅
- [x] S90-04: hot_reload_v2.rs VecDeque::new() const 검증 → 1.82 필요, KEEP ✅

### Track C: FFI + 배포
- [x] S90-05: FFI crate 빌드 검증 → ffi-tests.yml 미존재 발견 ✅
- [x] S90-06: crates.io dry-run → hangul, dict-sync 성공, 나머지 의존성 순서 확인 ✅

### Track D: 검증 + 리뷰
- [x] S90-07: 빌드/테스트/클리피 전체 검증 (1,198 pass / 0 fail / lib 0 warnings) ✅
- [x] S90-08: Sprint 90 리뷰 + 문서 + Sprint 91 로드맵 ✅

---

## Sprint 91 로드맵

### P1: crates.io v0.7.2 배포 (사용자 승인 필요)
- 배포 순서: hangul → dict → core → dict-validator → dict-builder → dict-sync → mecab-ko
- 각 단계 publish 후 다음 단계 의존성 해결 확인
- 배포 후 `cargo install mecab-ko-cli` 검증

### P2: ffi-tests.yml CI 워크플로우 생성
- 참조된 .github/workflows/ffi-tests.yml이 실제로 존재하지 않음
- Python (maturin + pytest), WASM (wasm-pack test), Node.js (npm test) 별도 job
- 주 1회 또는 release 태그 시 트리거

### P3: test/example clippy 경고 정리
- 테스트 파일 127개 경고 (expect_used, unwrap_used, panic 등)
- 우선순위: 핵심 테스트 > 통합 테스트 > example
- `clippy::pedantic` 그룹 정리

### P4: 시스템 사전 기반 벤치마크
- 시스템 사전 설치 후 전체 사전 벤치마크 실행
- mini-dict vs full-dict 비교
- corrections/ 최적화 효과 정량 측정

---

# 완료: Phase 55 - Sprint 89 (v0.7.2 릴리스 준비 + 코드 품질 + 벤치마크)
# 완료: Phase 54 - Sprint 88 (MSRV 1.80 + LazyLock + CI 수정)
# 완료: Phase 53 - Sprint 87 (per-call 할당 제거 성능 최적화)
# 완료: Phase 52 - Sprint 86 (v0.7.1 릴리스 + 보호 테스트 확충)
# 완료: Phase 51 - Sprint 85 (corrections/ 디렉토리 전환)
# 완료: Sprint 80-84 (corrections 분할 1-4차)
# 완료: Sprint 73-77 (v0.7.1, CI 통합, 분석)
# 아카이브: Sprint 1-72 (.history/ 및 Git 히스토리 참조)

## 크레이트 발행 현황

| 크레이트 | crates.io | 로컬 | 상태 |
|---------|-----------|------|------|
| mecab-ko-hangul | v0.7.1 | v0.7.2 | ⏳ dry-run 통과 |
| mecab-ko-dict | v0.7.1 | v0.7.2 | ⏳ 의존성 순서 대기 |
| mecab-ko-core | v0.7.1 | v0.7.2 | ⏳ 의존성 순서 대기 |
| mecab-ko-dict-validator | v0.7.1 | v0.7.2 | ⏳ 의존성 순서 대기 |
| mecab-ko-dict-builder | v0.7.1 | v0.7.2 | ⏳ 의존성 순서 대기 |
| mecab-ko-dict-sync | v0.7.1 | v0.7.2 | ⏳ dry-run 통과 |
| mecab-ko | v0.7.1 | v0.7.2 | ⏳ 의존성 순서 대기 |
