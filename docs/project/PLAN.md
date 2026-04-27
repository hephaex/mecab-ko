# 완료: Phase 57 - Sprint 91 (crates.io v0.7.2 배포 + CI 수정 + clippy 정리)

## Sprint 91 목표
crates.io v0.7.2 배포 (7개), ffi-tests.yml CI 확인, test/example clippy 경고 정리, 시스템 사전 확인

## Sprint 91 작업 목록

### Track A: 배포
- [x] S91-01: crates.io v0.7.2 배포 — 7/7 크레이트 성공 ✅

### Track B: CI
- [x] S91-02: ffi-tests.yml → e2e-ffi-tests.yml로 이미 존재 확인, Cargo.toml 참조 수정 ✅

### Track C: 코드 품질
- [x] S91-03: integration test clippy #[allow] 확장 (3파일), 경고 127→36 감소 ✅

### Track D: 환경
- [x] S91-04: 시스템 사전 미설치 확인 → full-dict 벤치마크 불가 ✅

### Track E: 검증 + 리뷰
- [x] S91-05: 빌드/테스트/클리피 검증 + 리뷰 + Sprint 92 로드맵 ✅

---

## Sprint 92 로드맵

### P1: v0.7.2 git tag + GitHub Release
- `git tag v0.7.2` + `git push --tags`
- GitHub Release 생성 (CHANGELOG v0.7.2 내용 포함)
- e2e-ffi-tests.yml release 트리거 검증

### P2: 잔여 test/example clippy 정리 (36개)
- accuracy_eval (13), convert_to_v2 (7), memory_measure (3+2), 기타
- example 파일: Result+? 패턴 적용
- accuracy_eval: 대량 테스트 파일, #[allow] 그룹으로 처리

### P3: 시스템 사전 설치 + full-dict 벤치마크
- mecab-ko-dic 설치 절차 문서화
- MECAB_DICDIR 설정 후 벤치마크 재실행
- mini-dict vs full-dict 비교

### P4: 문서 개선
- docs/RELEASE_CHECKLIST.md 갱신 (v0.7.2 절차 반영)
- README 설치 예제 검증
- API 문서 빌드 확인 (docs.rs)

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
