# S15-08: 문서 사이트 개선

## 세션 개요

- **날짜**: 2026-03-03
- **작업**: Sprint 15 - P3 문서 사이트 개선
- **상태**: 완료

## 작업 내용

### 1. mdBook 구조 정리

**SUMMARY.md 업데이트**:
- 튜토리얼 섹션 추가
- 벤치마크 섹션 분리
- API 레퍼런스에 개요 페이지 추가

변경된 구조:
```
# Summary
[소개](introduction.md)

# 시작하기
- 설치
- 빠른 시작

# 튜토리얼 (신규)
- 기본 사용법
- 고급 기능
- 웹 서버 통합

# 사용 가이드
- CLI 사용법
- 사용자 사전
- 출력 포맷

# API 레퍼런스
- 개요 (신규 링크)
- Rust/Python/Node.js/WASM

# 고급 주제
- 사전 빌드/성능 튜닝/Elasticsearch/커스텀 분석기

# 성능 벤치마크 (신규 섹션)
- 성능 대시보드
- 벤치마크 가이드

# 레퍼런스
- 품사 태그/사전 포맷/바이너리 포맷

# 개발자 가이드
- 프로젝트 구조/빌드 프로세스/컨트리뷰션 가이드

# 부록
- FAQ/변경 이력/마이그레이션 가이드
```

### 2. 신규 튜토리얼 문서

**tutorials/basic-usage.md** (기본 사용법):
- 환경 설정
- 첫 번째 분석 (Rust/Python)
- 토큰 정보 활용
- 품사 필터링
- 사용자 사전 적용
- 키워드 추출기 예제

**tutorials/advanced-features.md** (고급 기능):
- N-best 분석
- 복합명사 분해 (DecompoundMode)
- 스트리밍 처리
- 정확도 평가 (CLI, API)
- 사전 품질 검증
- Unknown 단어 패턴
- 문서 분석 파이프라인 예제

**tutorials/web-integration.md** (웹 서버 통합):
- Actix-web 통합
- Axum 통합
- REST API 설계
- 성능 최적화 (풀링, 캐싱, Rate Limiting)
- Docker 배포
- 클라이언트 예제 (Python, JavaScript)

### 3. 벤치마크 가이드

**benchmarks/guide.md**:
- 벤치마크 환경 설정
- 벤치마크 실행 방법
- 9가지 벤치마크 종류 설명
- 결과 분석 방법
- CI 통합 워크플로우
- 커스텀 벤치마크 작성

### 4. 기존 문서 업데이트

**introduction.md**:
- v0.2.0 버전 표시
- 성능 지표 테이블 추가
- v0.2.0 주요 변경사항 요약
- 프로젝트 구조 업데이트 (모든 크레이트 포함)

**installation.md**:
- Rust 버전 1.75+ 요구
- Python 3.8+, Node.js 18+ 요구사항 추가
- 버전 0.2로 업데이트
- Feature flags 테이블 확장

**changelog.md**:
- v0.2.0 전체 변경사항 반영
- 로드맵 업데이트 (v0.3.0, v0.5.0, v1.0.0)

**faq.md**:
- Python 바인딩 정보 업데이트
- WASM 바인딩 정보 업데이트

**benchmarks/index.md**:
- v0.2.0 KPI 추가

**book.toml**:
- description 추가
- additional-css/js 설정
- search 설정 개선
- playground 설정 추가

## 파일 변경 요약

### 생성된 파일
| 파일 | 설명 |
|------|------|
| `docs/book/src/tutorials/basic-usage.md` | 기본 사용법 튜토리얼 |
| `docs/book/src/tutorials/advanced-features.md` | 고급 기능 튜토리얼 |
| `docs/book/src/tutorials/web-integration.md` | 웹 서버 통합 튜토리얼 |
| `docs/book/src/benchmarks/guide.md` | 벤치마크 가이드 |

### 수정된 파일
| 파일 | 변경 내용 |
|------|-----------|
| `docs/book/src/SUMMARY.md` | 구조 재정리 |
| `docs/book/src/introduction.md` | v0.2.0 정보 추가 |
| `docs/book/src/installation.md` | 버전 및 요구사항 업데이트 |
| `docs/book/src/quick-start.md` | 버전 업데이트 |
| `docs/book/src/changelog.md` | v0.2.0 변경사항 |
| `docs/book/src/faq.md` | 바인딩 정보 업데이트 |
| `docs/book/src/benchmarks/index.md` | v0.2.0 KPI 추가 |
| `docs/book/src/developer/contributing.md` | Rust 버전 업데이트 |
| `docs/book/book.toml` | 설정 개선 |
| `PROGRESS.md` | S15-08 완료 기록 |
| `PLAN.md` | S15-08 완료 체크 |

## 검증

1. 모든 신규 문서 파일 생성 확인
2. SUMMARY.md 링크 유효성 확인
3. 버전 번호 일관성 확인 (0.2.0)
4. Rust 버전 요구사항 일관성 확인 (1.75+)

## 다음 단계

Sprint 15 완료. Sprint 16 준비:
- 고급 토큰화 기능
- N-best 개선
- 사용자 정의 분석 모드

## 기술 노트

### mdBook 설정 개선
- `create-missing = true`: 누락 파일 자동 생성
- `use-hierarchical-index = true`: 계층적 검색 인덱스
- `playground.line-numbers = true`: 코드 블록 라인 번호

### 문서 구조 원칙
1. 시작하기: 5분 내 첫 분석 가능
2. 튜토리얼: 점진적 학습 경로
3. 가이드: 특정 작업 수행 방법
4. 레퍼런스: 상세 API 정보
5. 부록: FAQ, 변경 이력, 마이그레이션
