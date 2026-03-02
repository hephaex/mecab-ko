# 현재 스프린트: Phase 7 - Sprint 16 (고급 토큰화 & N-best)

## 목표
N-best 경로 탐색 개선, 사용자 정의 분석 모드, 토큰화 성능 최적화

## Sprint 16 작업 목록

### P0 (Critical)
- [x] S16-01: N-best 경로 탐색 개선 ✅
  - N-best Viterbi 알고리즘 최적화
  - 메모리 효율적인 경로 저장
  - N-best 결과 API 개선

### P1 (High)
- [x] S16-02: 사용자 정의 분석 모드 ✅
  - AnalysisMode enum 확장
  - 품사 필터링 모드
  - 명사 추출 전용 모드
  - 동사/형용사 원형 복원 모드
- [ ] S16-03: PyPI 배포 (S15-03 계속) - BLOCKED
  - PyPI 토큰 설정 후 배포
- [ ] S16-04: npm 배포 (S15-04 계속) - BLOCKED
  - npm 토큰 설정 후 배포

### P2 (Medium)
- [x] S16-05: Lattice 시각화 도구 ✅
  - DOT/Graphviz 출력
  - HTML 인터랙티브 뷰어 (d3-graphviz)
  - 디버깅용 lattice dump (Text, JSON)
- [ ] S16-06: 토큰화 캐싱
  - 반복 입력 캐싱
  - LRU 캐시 구현
  - 캐시 히트율 모니터링
- [ ] S16-07: 병렬 토큰화
  - Rayon 기반 배치 병렬 처리
  - 스레드풀 설정
  - 처리량 벤치마크

### P3 (Low)
- [ ] S16-08: v0.3.0 준비
  - Breaking changes 정리
  - CHANGELOG 업데이트
  - Migration guide 작성

---

# 완료된 스프린트: Phase 7 - Sprint 15 (사전 품질 & 정확도) ✅

## 목표 (완료)
정확도 측정 인프라 구축, 사전 품질 개선, Unknown 단어 처리 강화

## Sprint 15 작업 목록

### P0 완료
- [x] S15-01: 정확도 측정 인프라 구축 ✅

### P1 완료/BLOCKED
- [x] S15-02: 사전 품질 검증 도구 개선 ✅
- [ ] S15-03: PyPI 배포 - BLOCKED → S16-03
- [ ] S15-04: npm 배포 - BLOCKED → S16-04

### P2 완료
- [x] S15-05: Unknown 단어 처리 개선 ✅
- [x] S15-06: 복합명사 분해 개선 ✅
- [x] S15-07: 성능 벤치마크 CI 통합 ✅

### P3 완료
- [x] S15-08: 문서 사이트 개선 ✅

---

# 완료된 스프린트: Phase 6 - Sprint 14 (v0.2.0 릴리스 & 사전 자동화) ✅

## 목표 (완료)
v0.2.0 정식 릴리스, 신조어 수집 워크플로우 검증, 사전 빌드 자동화 개선

## Sprint 14 작업 목록

### P0 완료
- [x] S14-01: v0.2.0 릴리스 준비 ✅

### P1 완료/BLOCKED
- [ ] S14-02: 신조어 수집 워크플로우 테스트 - 대기 (secret 필요)
- [ ] S14-03: PyPI 배포 - BLOCKED → S15-03
- [ ] S14-04: npm 배포 - BLOCKED → S15-04

### P2 완료
- [x] S14-05: 한국어기초사전 API 클라이언트 ✅
- [x] S14-06: CLI collect 서브커맨드 ✅
- [x] S14-07: 사전 빌드 자동화 ✅

### P3 완료
- [x] S14-08: 성능 벤치마크 문서화 ✅

---

# 완료된 스프린트: Phase 6 - Sprint 13 (커뮤니티 & API 통합) ✅

## 목표 (완료)
커뮤니티 기여 시스템 구축, 국립국어원 API 클라이언트 구현, v0.2.0 준비

## Sprint 13 작업 목록

### P0 (Critical)
- [x] S13-01: 커뮤니티 기여 가이드라인 ✅

### P1 (High)
- [x] S13-02: 국립국어원 API 클라이언트 ✅
- [ ] S13-03: PyPI 배포 - BLOCKED → S14-03
- [ ] S13-04: npm 배포 - BLOCKED → S14-04

### P2 (Medium)
- [x] S13-05: 사전 데이터 변환기 ✅
- [x] S13-06: CLI 사전 동기화 명령 ✅
- [x] S13-07: v0.2.0 Breaking Changes 정리 ✅

### P3 (Low)
- [x] S13-08: 신조어 자동 수집 파이프라인 설계 ✅

---

# 완료된 스프린트: Phase 5 - Sprint 12 (사전 현대화 & 기능 확장) ✅

## 목표 (완료)
신조어 사전 구축, 사용자 사전 도구 개선, API 확장

## Sprint 12 작업 목록

### P0 (Critical)
- [x] S12-01: 신조어 시드 사전 구축 ✅
  - 2018-2024 주요 신조어 123개 수집
  - `data/user-dict/neologisms.csv` 생성
  - 품사 태그 및 비용 정보 포함
  - README.md 문서 추가

### P1 (High)
- [x] S12-02: 사용자 사전 빌드 도구 개선 ✅
  - 신조어 자동 품사 추정 기능 (`estimate_pos()`)
  - CSV 중복 검사 기능 (`check_csv_duplicates()`)
  - 시스템 사전과 충돌 검사 (`check_system_conflicts()`)
- [ ] S12-03: PyPI 배포 - BLOCKED → S13-03
- [ ] S12-04: npm 배포 - BLOCKED → S13-04

### P2 (Medium)
- [x] S12-05: 국립국어원 API 연동 조사 ✅
- [x] S12-06: Streaming API 확인 ✅
- [x] S12-07: 분석 모드 확인 ✅

### P3 (Low)
- [x] S12-08: CLI 인터랙티브 모드 ✅

---

# 완료된 스프린트: Phase 5 - Sprint 11 (배포 & 생태계 확장) ✅

## 목표 (완료)
PyPI/npm 배포 준비, 문서 사이트 구축, 사전 현대화 조사

## Sprint 11 작업 목록

### P0 (Critical)
- [ ] S11-01: PyPI 배포 (mecab-ko-python) - BLOCKED (PyPI 토큰 필요) → S12-03
- [ ] S11-02: npm 배포 (mecab-ko-wasm) - BLOCKED (npm 토큰 필요) → S12-04

### P1 (High)
- [x] S11-03: GitHub Releases 자동화 ✅
- [x] S11-04: 성능 회귀 탐지 CI ✅

### P2 (Medium)
- [x] S11-05: 문서 사이트 구축 ✅
- [x] S11-06: mecab-ko-dic 최신화 조사 ✅
- [x] S11-07: Docker 이미지 배포 ✅

### P3 (Low)
- [x] S11-08: 성능 대시보드 ✅

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

# 완료된 스프린트 요약

| Sprint | Phase | 목표 | 상태 |
|--------|-------|------|------|
| 1-2 | 1 | 프로젝트 셋업 | ✅ |
| 3 | 1 | 코어 데이터 구조 | ✅ |
| 4 | 2 | 코어 엔진 + 바인딩 | ✅ |
| 5 | 3 | 안정화 | ✅ |
| 6 | 3 | 성능 최적화 | ✅ |
| 7 | 4 | crates.io 발행 준비 | ✅ |
| 8 | 4 | Memory 최적화 | ✅ |
| 9 | 4 | 사전 현대화 & 발행 | ✅ |
| 10 | 4 | 안정화 & 품질 | ✅ |
| 11 | 5 | 배포 & 생태계 확장 | ✅ |
| 12 | 5 | 사전 현대화 & 기능 확장 | ✅ |
| 13 | 6 | 커뮤니티 & API 통합 | ✅ |
| 14 | 6 | v0.2.0 릴리스 & 사전 자동화 | ✅ |
| 15 | 7 | 사전 품질 & 정확도 | ✅ |
| 16 | 7 | 고급 토큰화 & N-best | 🚧 |

## 크레이트 발행 현황

| 크레이트 | 버전 | 플랫폼 | 상태 |
|---------|------|--------|------|
| mecab-ko-hangul | v0.1.1 | crates.io | ✅ |
| mecab-ko-dict | v0.1.1 | crates.io | ✅ |
| mecab-ko-core | v0.1.1 | crates.io | ✅ |
| mecab-ko-dict-validator | v0.1.1 | crates.io | ✅ |
| mecab-ko-dict-builder | v0.1.1 | crates.io | ✅ |
| mecab-ko | v0.1.1 | crates.io | ✅ |
| mecab-ko-python | - | PyPI | BLOCKED |
| mecab-ko-wasm | - | npm | BLOCKED |
| mecab-ko-cli | v0.1.1 | GitHub Releases | ✅ |
| mecab-ko (docker) | latest | GHCR | ✅ |

## 다음 스프린트 예고
Sprint 17: 스트리밍 API, 메모리 최적화, v0.3.0 릴리스 준비
