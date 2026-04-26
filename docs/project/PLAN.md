# 완료: Phase 52 - Sprint 86 (v0.7.1 릴리스 + 보호 테스트 확충)

## Sprint 86 목표
v0.7.1 crates.io 배포 완료, 보호 테스트 52→56, remote 충돌 해결

## Sprint 86 작업 목록

### Track A: v0.7.1 릴리스
- [x] S86-01: Remote 충돌 해결 (force push + rebase) ✅
- [x] S86-02: v0.7.1 annotated tag 재설정 + push ✅
- [x] S86-03: crates.io 배포 7/7 완료 ✅

### Track B: 품질 개선
- [x] S86-04: 보호 테스트 +4 (xsv_morpheme_split, verb_splitting, sentence_final_endings, xsv_and_ec_ef) ✅
- [x] S86-05: Sprint 86 리뷰 + 문서 + 개선 계획 ✅

### Track C: 리뷰 수정
- [x] S86-R1: mod.rs 죽은 import 제거 (review H1) ✅
- [x] S86-R2: tests.rs rustfmt 적용 (review M1) ✅
- [x] S86-R3: workspace keyword 수정 (crates.io 20자 제한) ✅

---

## Sprint 87 로드맵

### P1: 성능 최적화 — LazyLock/phf 정적 맵 전환
- particle_and_ending.rs: `particle_map` 매 호출 HashMap → `LazyLock<HashMap>` 또는 `phf_map!`
- verb_splitting.rs: `verb_gi_words` 매 호출 Vec → static slice 또는 `phf_set!`
- compound_and_irregular.rs: 유사 패턴 확인 및 전환
- 예상 효과: hot path 할당 제거, 벤치마크 수치 개선

### P2: tests.rs 분할 검토
- tests.rs 728줄 → 800줄 초과 시 분할
- 분할 기준: 오케스트레이터 통합 테스트 vs 서브함수 직접 호출 보호 테스트
- 현재는 800줄 이하이므로 모니터링만

### P3: verb_splitting.rs 764줄 검토
- 현재 최대 파일 (764줄)
- 논리적 분할점 존재 여부 확인
- 필요 시 pass 그룹별 분리

### P4: CI/CD 개선
- benchmark dashboard CI 커밋이 rebase 충돌 유발 — auto-merge 또는 별도 브랜치 전략 검토
- FFI crates (python, wasm, node) 테스트 자동화 확인

---

# 완료: Phase 51 - Sprint 85 (corrections/ 디렉토리 전환 + 추가 분할)

## Sprint 85 작업 목록
- [x] S85-01: corrections.rs -> corrections/ (6,516줄 -> 16 files) ✅
- [x] S85-02: sentence_final 추가 분할 (838->440+413) ✅
- [x] S85-02: xsv_and_ec_ef 추가 분할 (990->499+495) ✅
- [x] S85-03: 빌드/테스트/클리피 전체 통과 ✅
- [x] S85-04: v0.7.1 tag + push ✅
- [x] S85-05: Sprint 85 리뷰 + PLAN/PROGRESS 업데이트 ✅

---

# 완료: Phase 50 - Sprint 84 (대형 서브함수 분할 + 보호 테스트)

## Sprint 84 작업 목록
- [x] S84-01: Sprint 83 함수 보호 테스트 3개 ✅
- [x] S84-02: sentence_final 분할 (1,821->986+1,020) ✅
- [x] S84-03: verb_and_morpheme 분할 (1,401->762+638) ✅

---

# 완료: Sprint 80-83 (corrections 분할 1-4차)
# 완료: Sprint 73-77 (v0.7.1, CI 통합, 분석)
# 완료: Sprint 57-72 (100% 정확도, v0.6.0, 메모리 최적화)
# 아카이브: Sprint 1-56 (.history/ 및 Git 히스토리 참조)

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
