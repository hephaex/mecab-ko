# 현재 스프린트: Phase 6 - Sprint 13 (커뮤니티 & API 통합)

## 목표
커뮤니티 기여 시스템 구축, 국립국어원 API 클라이언트 구현, v0.2.0 준비

## Sprint 13 작업 목록

### P0 (Critical)
- [x] S13-01: 커뮤니티 기여 가이드라인 ✅
  - CONTRIBUTING.md 업데이트 (신조어 추가 가이드)
  - PR 템플릿 개선 (사전 변경 섹션)
  - 이슈 템플릿 추가 (word-request, bug-report, analysis-error, feature-request)
  - CODE_OF_CONDUCT.md 작성 (Contributor Covenant 2.0)

### P1 (High)
- [ ] S13-02: 국립국어원 API 클라이언트 (Phase 1)
  - `mecab-ko-dict-sync` 크레이트 생성
  - OpenDictClient 구조체 구현
  - 우리말샘 검색 API 연동
  - API 키 환경변수 처리
- [ ] S13-03: PyPI 배포 (S12-03 계속) - BLOCKED
  - PyPI 토큰 설정 후 배포
- [ ] S13-04: npm 배포 (S12-04 계속) - BLOCKED
  - npm 토큰 설정 후 배포

### P2 (Medium)
- [ ] S13-05: 사전 데이터 변환기
  - 국립국어원 → MeCab 포맷 변환
  - 품사 태그 매핑 테이블 구현
  - 비용 자동 계산 로직
- [ ] S13-06: CLI 사전 동기화 명령
  - `mecab-ko sync` 서브커맨드
  - `--source opendict` 옵션
  - CSV 출력 및 병합 기능
- [ ] S13-07: v0.2.0 Breaking Changes 정리
  - API 변경 사항 문서화
  - Migration Guide 작성
  - CHANGELOG 업데이트

### P3 (Low)
- [ ] S13-08: 신조어 자동 수집 파이프라인 설계
  - GitHub Actions 스케줄 워크플로우
  - 주간 신조어 PR 자동 생성
  - 리뷰어 자동 할당

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
| 13 | 6 | 커뮤니티 & API 통합 | 🚧 |

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
Sprint 14: 국립국어원 API 동기화 도구 완성, 자동 사전 업데이트 CI/CD
