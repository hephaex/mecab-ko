# 현재 스프린트: Phase 5 - Sprint 12 (사전 현대화 & 기능 확장)

## 목표
신조어 사전 구축, 사용자 사전 도구 개선, API 확장

## Sprint 12 작업 목록

### P0 (Critical)
- [ ] S12-01: 신조어 시드 사전 구축
  - 2018-2024 주요 신조어 100개 수집
  - `data/user-dict/neologisms.csv` 생성
  - 품사 태그 및 비용 정보 포함

### P1 (High)
- [ ] S12-02: 사용자 사전 빌드 도구 개선
  - 신조어 자동 품사 추정 기능
  - CSV 중복 검사 기능
  - 시스템 사전과 충돌 검사
- [ ] S12-03: PyPI 배포 (S11-01 계속)
  - PyPI 토큰 설정 후 배포
  - pip install mecab-ko 테스트
- [ ] S12-04: npm 배포 (S11-02 계속)
  - npm 토큰 설정 후 배포
  - npm install mecab-ko-wasm 테스트

### P2 (Medium)
- [ ] S12-05: 국립국어원 API 연동 조사
  - 우리말샘 API 테스트
  - 한국어기초사전 API 테스트
  - 데이터 변환 파이프라인 설계
- [ ] S12-06: Streaming API 추가
  - 대용량 텍스트 스트리밍 분석
  - Iterator 기반 API
  - 메모리 효율적 처리
- [ ] S12-07: 분석 모드 확장
  - 복합어 분해 모드
  - 원형 복원 모드
  - 의미 태그 모드

### P3 (Low)
- [ ] S12-08: CLI 인터랙티브 모드
  - REPL 스타일 대화형 분석
  - 히스토리 및 자동완성

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
| 12 | 5 | 사전 현대화 & 기능 확장 | 🚧 |

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
Sprint 13: 커뮤니티 기여 시스템, 국립국어원 API 통합, v0.2.0 준비
