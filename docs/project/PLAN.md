# 완료: Phase 55 - Sprint 89 (v0.7.2 릴리스 준비 + 코드 품질 + 벤치마크)

## Sprint 89 목표
v0.7.2 릴리스 준비 (버전 범프 + CHANGELOG + MSRV 문서 갱신), corrections/ 모듈 문서화, #[allow] 리뷰, FFI 호환성 확인, 벤치마크 비교

## Sprint 89 작업 목록

### Track A: 릴리스 준비
- [x] S89-01: Version bump 0.7.1 → 0.7.2 (13개 Cargo.toml) ✅
- [x] S89-02: CHANGELOG.md v0.7.2 작성 ✅
- [x] S89-03: README/문서 MSRV 1.75→1.80 갱신 (16개 파일) ✅

### Track B: 벤치마크
- [x] S89-04: normalization_bench + tokenizer_bench 실행 (criterion 기준) ✅

### Track C: 코드 품질
- [x] S89-05: #[allow(clippy::too_many_lines)] 8개 리뷰 → 전부 KEEP (정당한 사유) ✅
- [x] S89-06: corrections/ 15개 서브모듈 `//!` doc comment 추가 ✅

### Track D: FFI
- [x] S89-07: FFI crate MSRV 1.80 호환 확인 + mecab-ko-node 1.77→1.80 갱신 ✅

### Track E: 검증 + 리뷰
- [x] S89-08: 빌드/테스트/클리피 전체 검증 (1,198 pass / 0 fail) ✅
- [x] S89-09: Sprint 89 리뷰 + 문서 + Sprint 90 로드맵 ✅

---

## Sprint 90 로드맵

### P1: crates.io v0.7.2 배포
- 의존성 순서대로 7개 크레이트 publish
- 배포 후 `cargo install mecab-ko-cli` 검증
- crates.io 페이지 MSRV 1.80 확인

### P2: 벤치마크 결과 문서화
- Sprint 89에서 수집한 criterion 결과를 docs/benchmarks/에 정리
- v0.7.1 vs v0.7.2 비교표 작성
- normalization: -11.7% (disabled toggle), -1.8% (wakati)
- tokenizer: scalability/1000chars -3.3%, baseline 안정

### P3: 잔여 코드 품질
- example 파일 clippy 경고 정리 (expect/unwrap → proper error handling)
- mecab-ko-core crate-level doc comment 추가
- `hot_reload_v2.rs` MSRV 코멘트 검증 (VecDeque::new const 여부)

### P4: 통합 테스트 강화
- FFI crate 빌드 테스트 (CI ffi-tests.yml 실행)
- 시스템 사전 기반 벤치마크 (mini-dict fallback 아닌 전체 사전)

---

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
| mecab-ko-hangul | v0.7.1 | v0.7.2 | ⏳ 배포 대기 |
| mecab-ko-dict | v0.7.1 | v0.7.2 | ⏳ 배포 대기 |
| mecab-ko-core | v0.7.1 | v0.7.2 | ⏳ 배포 대기 |
| mecab-ko-dict-validator | v0.7.1 | v0.7.2 | ⏳ 배포 대기 |
| mecab-ko-dict-builder | v0.7.1 | v0.7.2 | ⏳ 배포 대기 |
| mecab-ko-dict-sync | v0.7.1 | v0.7.2 | ⏳ 배포 대기 |
| mecab-ko | v0.7.1 | v0.7.2 | ⏳ 배포 대기 |
