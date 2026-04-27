# 완료: Phase 58 - Sprint 92 (문서 v0.7.2 갱신 + RELEASE_CHECKLIST + git tag 준비)

## Sprint 92 목표
문서 전체 v0.7.2 갱신, RELEASE_CHECKLIST 오버홀, CHANGELOG 갱신, git tag + GitHub Release 준비

## Sprint 92 작업 목록

### Track A: 문서
- [x] S92-01: RELEASE_CHECKLIST.md 오버홀 (v0.1.0 → v0.7.2, dict-validator/dict-sync 추가) ✅
- [x] S92-02: README 버전 참조 수정 (8파일, 0.1.0/0.7.1 → 0.7.2) ✅
- [x] S92-04: CHANGELOG v0.7.2 갱신 (clippy 완료 + crates.io 배포 기록) ✅

### Track B: 검증
- [x] S92-03: docs.rs 빌드 확인 — v0.7.2 아직 빌드 중 (v0.7.1 표시, 자동 해결 예정) ✅

### Track C: 릴리스
- [ ] S92-05: git tag v0.7.2 + GitHub Release — **사용자 승인 대기**

### Track D: 리뷰
- [x] S92-06: 빌드/테스트/클리피 검증 + 리뷰 + Sprint 93 로드맵 ✅

---

## Sprint 93 로드맵

### P1: git tag v0.7.2 + GitHub Release (S92에서 이관)
- `git tag -a v0.7.2` + `git push origin v0.7.2`
- release.yml 트리거 → GitHub Release + CLI 바이너리 5플랫폼
- python-wheels.yml 트리거 → PyPI 배포
- 사용자 승인 필요

### P2: 시스템 사전 설치 + full-dict 벤치마크
- mecab-ko-dic 설치 절차 문서화
- MECAB_DICDIR 설정 후 벤치마크 재실행
- mini-dict vs full-dict 비교

### P3: docs.rs 검증
- v0.7.2 빌드 완료 확인
- 문서 누락/경고 확인

### P4: v0.8.0 기능 계획
- 잠재 기능 목록 정리 (N-best, whitespace penalty 등)
- 로드맵 작성

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
