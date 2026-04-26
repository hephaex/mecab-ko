# 완료: Phase 53 - Sprint 87 (per-call 할당 제거 성능 최적화)

## Sprint 87 목표
corrections/ 내 22개 매 호출 HashMap/HashSet → 정적 할당 전환, verb_splitting 분할 검토

## Sprint 87 작업 목록

### Track A: 성능 최적화
- [x] S87-01: 매 호출 HashMap/Vec 패턴 조사 (22개 발견) ✅
- [x] S87-02: OnceLock/const 정적 전환 (6개 파일, 22개 할당) ✅

### Track B: 구조 검토
- [x] S87-03: verb_splitting.rs 764줄 분할 검토 → 불필요 ✅
- [x] S87-04: 빌드/테스트/클리피 전체 검증 (1,198 pass / 0 fail) ✅

### Track C: 리뷰 + 문서
- [x] S87-05: Sprint 87 리뷰 + 문서 + Sprint 88 로드맵 ✅

---

## Sprint 88 로드맵

### P1: 벤치마크 측정 + 성능 분석
- Sprint 87 정적 전환의 실제 성능 효과 벤치마크 측정
- `cargo bench` 기준 before/after 비교
- 프로파일링으로 다음 hot path 식별

### P2: tag_map.rs 정적 전환
- `sejong/tag_map.rs:9` HashMap::new() → OnceLock 또는 LazyLock 전환
- corrections/ 외부의 마지막 per-call 할당

### P3: CI/CD 개선
- benchmark dashboard CI 커밋이 main에 직접 push → 별도 브랜치 전략
- GitHub Actions 워크플로우 최적화 검토

### P4: v0.7.2 준비
- MSRV 1.75 → 1.80 업그레이드 검토 (LazyLock 사용 가능)
- OnceLock → LazyLock 전환으로 코드 단순화 가능
- FFI crates (python, wasm, node) 호환성 확인

---

# 완료: Phase 52 - Sprint 86 (v0.7.1 릴리스 + 보호 테스트 확충)

## Sprint 86 작업 목록
- [x] S86-01: Remote 충돌 해결 (force push + rebase) ✅
- [x] S86-02: v0.7.1 annotated tag 재설정 + push ✅
- [x] S86-03: crates.io 배포 7/7 완료 ✅
- [x] S86-04: 보호 테스트 +4 ✅
- [x] S86-05: Sprint 86 리뷰 + 문서 ✅

---

# 완료: Phase 51 - Sprint 85 (corrections/ 디렉토리 전환 + 추가 분할)

## Sprint 85 작업 목록
- [x] S85-01: corrections.rs -> corrections/ (6,516줄 -> 16 files) ✅
- [x] S85-02: sentence_final/xsv_ec_ef 추가 분할 ✅
- [x] S85-03: 빌드/테스트/클리피 전체 통과 ✅
- [x] S85-04: v0.7.1 tag + push ✅
- [x] S85-05: Sprint 85 리뷰 ✅

---

# 완료: Sprint 80-84 (corrections 분할 1-4차)
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
