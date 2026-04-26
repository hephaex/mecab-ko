# 완료: Phase 51 - Sprint 85 (corrections/ 디렉토리 전환 + 추가 분할)

## Sprint 85 목표
corrections.rs 6,516줄 단일 파일 -> corrections/ 디렉토리 전환, 800줄 초과 파일 추가 분할

## Sprint 85 작업 목록

### Track A: corrections/ 디렉토리 전환
- [x] S85-01: corrections.rs -> corrections/ (6,516줄 -> 16 files) ✅
- [x] S85-02: sentence_final 추가 분할 (838->440+413) ✅
- [x] S85-02: xsv_and_ec_ef 추가 분할 (990->499+495) ✅

### Track B: 검증
- [x] S85-03: 빌드/테스트/클리피 전체 통과 (1,194 pass / 0 fail / 18 ignored) ✅

### Track C: 릴리스 (보류)
- [ ] S85-P0: remote 충돌 해결 (force push 필요)
- [ ] S85-04: v0.7.1 tag + push + crates.io 배포

### Track D: 리뷰 + 문서
- [x] S85-05: Sprint 85 리뷰 + PLAN/PROGRESS 업데이트 ✅

---

## Sprint 86 로드맵

### P1: v0.7.1 릴리스 (S85에서 이월)
- remote 충돌 해결 (force push)
- v0.7.1 annotated tag 재설정 (HEAD)
- crates.io 배포 (7개 크레이트)

### P2: 보호 테스트 확충
- sentence_final_endings, xsv_morpheme_split 보호 테스트 추가
- 목표: 52 -> 56+ corrections 보호 테스트

### P3: 성능 최적화 + v0.8.0 준비
- LazyLock/phf 적용 (particle_map, verb_gi_words 등 매 호출 HashMap 제거)
- verb_splitting 764줄 -- 필요 시 추가 분할

---

# 완료: Phase 50 - Sprint 84 (대형 서브함수 분할 + 보호 테스트)

## Sprint 84 작업 목록
- [x] S84-01: Sprint 83 함수 보호 테스트 3개 ✅
- [x] S84-02: sentence_final 분할 (1,821->986+1,020) ✅
- [x] S84-03: verb_and_morpheme 분할 (1,401->762+638) ✅

---

# 완료: Phase 49 - Sprint 83 (corrections 분할 4차)

## Sprint 83 작업 목록
- [x] S83-01: verb_and_morpheme 추출 (2,633->1,238) ✅
- [x] S83-02: compound_and_irregular 추출 (1,238->867) ✅
- [x] S83-03: suffix_and_dependency 추출 (867->300) ✅
- [x] S83-04: 보호 테스트 5개 ✅

---

# 완료: Phase 48 - Sprint 82 (corrections 분할 3차)

## Sprint 82 작업 목록
- [x] S82-01: post_conjugation 추출 ✅
- [x] S82-02: particle_and_ending 추출 ✅
- [x] S82-03: 보호 테스트 5개 ✅
- [x] S82-04: CHANGELOG + README 버전 수정 ✅

---

# 완료: Sprint 80-81 (corrections 분할 1-2차)
# 완료: Sprint 73-77 (v0.7.1, CI 통합, 분석)
# 완료: Sprint 57-72 (100% 정확도, v0.6.0, 메모리 최적화)
# 아카이브: Sprint 1-56 (.history/ 및 Git 히스토리 참조)

## 크레이트 발행 현황

| 크레이트 | crates.io 최신 | 로컬 | 상태 |
|---------|--------------|------|------|
| mecab-ko-hangul | v0.5.0 | v0.7.1 | 배포 대기 |
| mecab-ko-dict | v0.5.0 | v0.7.1 | 배포 대기 |
| mecab-ko-core | v0.5.0 | v0.7.1 | 배포 대기 |
| mecab-ko-dict-validator | v0.5.0 | v0.7.1 | 배포 대기 |
| mecab-ko-dict-builder | v0.5.0 | v0.7.1 | 배포 대기 |
| mecab-ko-dict-sync | v0.5.0 | v0.7.1 | 배포 대기 |
| mecab-ko | v0.5.0 | v0.7.1 | 배포 대기 |
