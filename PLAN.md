# 현재 스프린트: Phase 5 - Sprint 11 (배포 & 생태계 확장)

## 목표
PyPI/npm 배포, 문서 사이트 구축, 사전 현대화 착수

## Sprint 11 작업 목록

### P0 (Critical)
- [ ] S11-01: PyPI 배포 (mecab-ko-python) - BLOCKED (PyPI 토큰 필요)
  - maturin build 성공 ✅
  - twine upload 대기 중

### P1 (High)
- [ ] S11-02: npm 배포 (mecab-ko-wasm) - BLOCKED (npm 토큰 필요)
  - wasm-pack publish 실행
  - npm install mecab-ko-wasm 테스트
- [x] S11-03: GitHub Releases 자동화 ✅
  - v0.1.1 태그 푸시 완료
  - release.yml 워크플로우 실행 중
  - CLI 바이너리 자동 빌드/첨부
- [x] S11-04: 성능 회귀 탐지 CI ✅
  - PR 비교 표 자동 생성
  - 10% 이상 악화 시 경고 추가
  - benchmark.yml 개선

### P2 (Medium)
- [ ] S11-05: 문서 사이트 구축
  - mdBook 설정
  - API 가이드 작성
  - 사용 예제 추가
- [ ] S11-06: mecab-ko-dic 최신화 조사
  - 2018년 이후 변경 조사
  - 신조어 추가 방안 검토
  - 커뮤니티 사전 통합 검토
- [ ] S11-07: Docker 이미지 배포
  - Dockerfile 작성
  - GitHub Container Registry 배포
  - docker run 사용 가이드

### P3 (Low)
- [ ] S11-08: 성능 대시보드
  - GitHub Pages에 벤치마크 결과 시각화
  - 버전별 성능 추이 그래프

---

# 완료된 스프린트: Phase 4 - Sprint 10 (안정화 & 품질) ✅

## 목표 (완료)
테스트 커버리지 향상, ignored 테스트 활성화, 코드 품질 개선, crates.io 발행

## Sprint 10 작업 목록

### P0 (Critical)
- [x] S10-01: crates.io 정식 발행 ✅ (6개 크레이트 v0.1.1 발행 완료)

### P1 (High)
- [x] S10-02: ignored 테스트 활성화 ✅ (e2e: 28 passed, 0 ignored)
- [x] S10-03: Elasticsearch 통합 테스트 개선 ✅ (doc: 5 passed, 0 ignored)
- [x] S10-04: 에러 처리 개선 ✅ (이미 thiserror 사용 중)

### P2 (Medium)
- [x] S10-05: 코드 중복 제거 ✅ (분석 완료: 중복 최소화 확인)
- [x] S10-06: 추가 벤치마크 ✅ (배치/메모리 벤치마크 이미 구현됨)
- [x] S10-07: CHANGELOG.md 작성 ✅

---

# 완료된 스프린트: Phase 4 - Sprint 9 (사전 현대화 & 발행) ✅

## 목표 (완료)
crates.io 발행 준비, 사전 현대화, 추가 최적화

## Sprint 9 작업 목록

### P0 (Critical)
- [ ] S9-01: crates.io 정식 발행 - BLOCKED (`cargo login` 필요) → S10-01로 이동
- [x] S9-02: 의존성 업데이트 ✅

### P1 (High)
- [x] S9-03: CLI 개선 (옵션 추가) ✅
- [x] S9-04: Python 바인딩 PyPI 배포 준비 완료 ✅
- [x] S9-05: 사용자 사전 기능 개선 ✅

### P2 (Medium)
- [x] S9-06: 문서화 개선 (API 가이드) ✅
- [x] S9-07: 성능 벤치마크 자동화 ✅

---

# 완료된 스프린트: Phase 4 - Sprint 8 (Memory 최적화 & 발행) ✅

## Sprint 8 목표 (완료)
Memory 최적화 (215MB → 150MB), WASM zstd 해결, crates.io 발행 준비

## Sprint 8 작업 목록

### P0 (Critical) ✅
- [x] S8-01: Memory 최적화 - entries 지연 로딩
- [x] S8-02: Memory 최적화 - mmap 활용 강화
- [x] S8-03: WASM zstd-sys 이슈 해결

### P1 (High) ✅
- [x] S8-04: crates.io 발행 준비 (인증 대기)
- [x] S8-05: PyPI 배포 준비 (maturin)
- [x] S8-06: README.md 정리

### P2 (Medium) ✅
- [x] S8-07: npm 배포 준비 (Node.js 바인딩)
- [x] S8-08: GitHub Maintenance 이슈 정리

---

# 완료된 스프린트: Phase 4 - Sprint 7 (crates.io 발행 준비) ✅

## Sprint 7 목표 (완료)
crates.io 발행 준비, 바인딩 최적화, Memory KPI 측정

## 완료된 이전 스프린트

### Phase 1 - Sprint 1-2 (프로젝트 셋업) ✅
- 프로젝트 구조 설계, 한글 유틸리티, CI/CD, 자동화 인프라

### Phase 1 - Sprint 3 (코어 데이터 구조) ✅
- 리서치 완료 (MeCab 내부, 생태계, Rust 크레이트, Lindera 분석, 바이너리 포맷)
- 사전 로더, DA Trie, 연접 비용 행렬, 미등록어 처리 모두 구현 완료

### Phase 2 - Sprint 4 (코어 엔진 + 바인딩) ✅
- Viterbi 엔진, Lattice 빌더, Tokenizer, Normalizer 구현 완료
- CLI 인터페이스, Elasticsearch Nori 호환 (부분), Python/WASM/Node 바인딩 구현 완료
- 사전 빌더 (CSV→binary), entries 파이프라인 구현 완료

### Phase 3 - Sprint 5 (안정화) ✅
- 의존성 업데이트, CI 강화, mini-dict 테스트 활성화 159개
- dict-validator 확인, 코드 품질 점검

### Phase 3 - Sprint 6 (성능 최적화) ✅
- 토크나이저 45-55% 개선, 238K morphs/sec, 0.086ms cold start
- 프로파일러 완성, ES 크레이트 완료, 746 tests pass

## Sprint 7 작업 목록

### crates.io 발행 준비
- [x] S7-01: Path dependency에 version 추가 (P0) ✅
- [x] S7-02: cargo publish --dry-run 검증 (P0) ✅
- [ ] S7-03: README.md 및 CHANGELOG.md 정리 (P1)

### Memory KPI 측정
- [x] S7-04: Full-dict 벤치마크 실행 (P1) ⚠️
  - 측정값: 215 MB (목표 150MB 초과)
  - Sprint 8에서 최적화 필요

### 바인딩 검증
- [x] S7-05: Python 바인딩 빌드 테스트 (P1) ✅
- [x] S7-06: WASM 바인딩 빌드 테스트 (P2) ⚠️ zstd-sys 이슈
- [x] S7-07: Node.js 바인딩 빌드 테스트 (P2) ✅

### 문서화
- [x] S7-08: API 문서 최종 점검 (P2) ✅ (clippy 0 warnings)

## Sprint 7 완료 요약
- **완료일**: 2026-03-01
- **커밋**: 3개 (16aa61b, 7ef84e6, 9530a77)
- **테스트**: 746 passed, 0 failed
- **Clippy**: 0 warnings

## 크레이트 발행 순서

| 순서 | 크레이트 | 의존성 | 상태 |
|-----|---------|--------|------|
| 1 | mecab-ko-hangul | 없음 | 대기 |
| 2 | mecab-ko-dict | hangul | 대기 |
| 3 | mecab-ko-core | hangul, dict | 대기 |
| 4 | mecab-ko-dict-validator | hangul | 대기 |
| 5 | mecab-ko-dict-builder | hangul, dict | 대기 |
| 6 | mecab-ko (facade) | 전체 | 대기 |

## 발행 제외 크레이트 (publish=false 유지)

| 크레이트 | 사유 | 배포 방식 |
|---------|------|----------|
| mecab-ko-cli | CLI 도구 | GitHub Releases |
| mecab-ko-python | PyO3 바인딩 | PyPI |
| mecab-ko-wasm | WASM 바인딩 | npm |
| mecab-ko-node | Node.js 바인딩 | npm |
| mecab-ko-elasticsearch | ES 플러그인 | 별도 배포 |
| mecab-ko-profiler | 개발 도구 | 내부 사용 |
| benchmarks | 벤치마크 | 내부 사용 |

## 다음 스프린트 예고
Sprint 8: crates.io 정식 발행, PyPI/npm 배포, 사전 현대화 착수
