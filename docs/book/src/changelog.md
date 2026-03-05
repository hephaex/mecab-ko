# 변경 이력

모든 주요 변경 사항을 기록합니다.

이 프로젝트는 [Semantic Versioning](https://semver.org/)을 따릅니다.

## [0.4.0] - 2026-03-05 🎉 crates.io 정식 배포

### 추가됨 (Added)

#### crates.io 배포
- 6개 크레이트 정식 배포 완료
  - `mecab-ko` (파사드 크레이트)
  - `mecab-ko-hangul` (한글 유틸리티)
  - `mecab-ko-dict` (사전 로더)
  - `mecab-ko-core` (핵심 엔진)
  - `mecab-ko-dict-builder` (사전 빌드 도구)
  - `mecab-ko-dict-validator` (사전 검증 도구)

#### 세종 코퍼스 호환 모드 강화 (mecab-ko-core)
- `apply_decomposition_corrections()`: 오분석 패턴 보정
- `apply_token_merges()`: 잘못 분해된 토큰 병합
- `apply_lexicon_overrides()`: 고빈도 어휘 매핑
- `apply_context_corrections()`: 컨텍스트 기반 품사 보정
- EP (선어말어미) 패턴 확장: 과거, 추측, 높임, 회상
- EC (연결어미) 패턴 확장: 이유, 조건, 시간, 양보
- ETM/ETN 패턴 확장: 관형형, 명사형

#### 고유명사 사전 확장
- ~200개 고유명사 추가
  - 도시: 안양, 안산, 파주, 김해, 창원, 청주, 전주, 포항, 원주
  - 서울 구/동: 강남, 서초, 송파, 명동, 홍대, 이태원, 잠실
  - 국가: 멕시코, 네덜란드, 싱가포르, 말레이시아 등 30+
  - 기업/브랜드: 틱톡, 넷플릭스, 테슬라, 쿠팡, 배달의민족
  - 대학: 서울대, 연세대, 고려대, 카이스트, 포스텍
  - 유명인: 이순신, 세종대왕, 손흥민, 방탄소년단, 블랙핑크

#### 신조어 사전 v3.0
- 511개 신조어 (123 → 511, +315%)
- AI/ML: Claude, Gemini, Midjourney, RAG, AGI
- 소셜미디어: Threads, Bluesky, Shorts, 크리에이터
- MZ세대: 갓생, 무지출, 킹받다, 레게노
- 경제: HBM, 밈주식, DSR
- 기술: Rust, Kubernetes, Docker

#### 평가 데이터셋 확장
- 160 → **300문장** 확장
- 추가 카테고리: 일상대화, 뉴스, 기술, 신조어, 불규칙, 복합

#### CI 자동 정확도 측정
- `dict-build.yml`에 accuracy-test job 추가
- 기준선 대비 회귀 탐지
- 정확도 이력 JSON 아티팩트 (90일 보관)
- GitHub Step Summary 통합

### 변경됨 (Changed)

- 모든 크레이트 v0.4.0 버전 업그레이드
- 정확도 기준선 업데이트: 23.8% (300문장, 세종 모드)

### 측정 결과 (300문장 데이터셋, 세종 모드)

| 지표 | v0.3.x (160문장) | v0.4.0 (300문장) |
|------|------------------|------------------|
| Token Accuracy | 29.6% | **23.8%** |
| Sentence Accuracy | 14.4% | 7.3% |
| F1 Score | 0.295 | 0.229 |

> **Note**: 확장된 데이터셋에는 복잡한 문장, 신조어, 불규칙 활용이 포함되어 절대값이 낮아짐

### 성능 (회귀 없음)

- 처리 속도: 6,250 문장/초
- 처리량: 3.0-3.7M chars/sec
- v0.3.0 대비 회귀 없음

---

## [0.3.1] - 2026-03-04

### 추가됨 (Added)

#### 세종 코퍼스 호환 모드 (mecab-ko-core)
- `SejongToken`: 세종 코퍼스 형식 토큰 구조체
- `SejongConverter`: 복합 태그 분리 변환기
- `EndingRule`: 어미 분리 규칙 (VV+EF, VA+EF, EC, ETM 등)
- 지원 어미 패턴:
  - 다, 아/어, 았/었 (과거형)
  - ㄴ/은, ㄹ (관형형)
  - 고, 니다/습니다 (연결/종결)
- `is_compound_tag()`: 복합 품사 태그 감지
- `split_compound_tag()`: 복합 태그 분리
- `format_sejong()`: 세종 형식 문자열 출력

#### CLI 개선 (mecab-ko-cli)
- `evaluate --sejong`: 세종 호환 모드로 정확도 평가
- `evaluate_dataset_sejong()`: 세종 모드 평가 함수

#### 사전 현대화 계획
- mecab-ko-dic v3.0 현대화 계획 문서
- 목표: 816K → 1M+ 엔트리
- Phase 1-4 로드맵 (Sprint 20-26)

### 측정 결과

| 지표 | 기존 | 세종 모드 | 개선 |
|------|------|-----------|------|
| Token Accuracy | 15.2% | 16.8% | +1.6%p |
| Sentence Accuracy | 8.1% | 10.0% | +1.9%p |
| F1 Score | 0.165 | 0.183 | +0.018 |

---

## [0.3.0] - 2026-03-03

### 추가됨 (Added)

#### K-best 경로 탐색 (mecab-ko-core)
- `ImprovedNbestSearcher`: K-best Viterbi 알고리즘
- `NbestPath`, `NbestResult` 구조체
- 각 노드에서 K개의 최선 후보 유지
- 이터레이터 지원 (`iter()`, `IntoIterator`)

#### 사용자 정의 분석 모드 (mecab-ko-core)
- `AnalysisMode` enum: Full, NounsOnly, VerbsOnly 등 10가지 모드
- `PosFilter`: 품사 필터링 (접두사/정확 매칭)
- `AnalyzerConfig`: 분석 설정 조합
- 편의 함수: `extract_nouns()`, `extract_verbs()` 등

#### Lattice 시각화 (mecab-ko-core)
- `LatticeViz`: 시각화 도구
- DOT, HTML, Text, JSON 출력 포맷
- d3-graphviz 기반 인터랙티브 뷰어

#### 토큰화 캐싱 (mecab-ko-core)
- `TokenCache`: LRU 캐시 (스레드 안전)
- `CachingTokenizer<T>`: 토크나이저 래퍼
- 캐시 히트/미스 통계

#### 스트리밍 API 개선 (mecab-ko-core)
- `ProgressStreamingTokenizer`: 진행률 콜백
- `LargeFileProcessor`: 대용량 파일 처리
- 스마트 문장 경계 청킹
- 오버랩 청킹 지원

#### npm 배포
- `mecab-ko-wasm` v0.3.0 npm에서 설치 가능

---

## [0.2.0] - 2026-03-02

### 추가됨 (Added)

#### 사전 동기화 (mecab-ko-dict-sync)
- `OpenDictClient`: 국립국어원 API 연동 클라이언트
- `DictConverter`: NIKL에서 MeCab-Ko 형식으로 변환
- 30+ 품사 태그 매핑 (명사->NNG, 고유명사->NNP, 동사->VV 등)
- 빈도 기반 비용 계산 (high=0, medium=500, low=1000)
- `UserEntry::to_csv_line()`: MeCab-Ko 호환 CSV 출력

#### CLI 개선 (mecab-ko-cli)
- `sync` 서브커맨드: 사전 동기화
  - `--source opendict`: NIKL OpenDict API
  - `--query`: 검색어
  - `--api-key`: API 키
  - `--output`: CSV 출력 파일
- `evaluate` 서브커맨드: 정확도 평가
  - Token/Sentence/POS Accuracy 측정
  - Precision/Recall/F1 계산
  - 품사별 정확도 리포트
- `--benchmark N`: 성능 측정 옵션
- `--stats`: 분석 통계 옵션
- REPL 7가지 출력 포맷 전환

#### 사용자 사전 개선 (mecab-ko-dict)
- `validate()`: 항목 검증
- `stats()`: 사전 통계
- `remove_duplicates()`: 중복 제거
- `remove_surface()`: 표면형으로 삭제
- `estimate_pos()`: 자동 품사 추정
- `check_csv_duplicates()`: CSV 검증
- `check_system_conflicts()`: 시스템 사전 충돌 검사

#### 사전 품질 검증 (mecab-ko-dict-validator)
- 품사 태그 분포 분석
- 비용 값 분포 분석 (평균/중앙값/표준편차)
- 이상치 탐지 (3-sigma, IQR)
- 일관성 검사 강화
- `--analyze`, `--fix` CLI 플래그

#### Unknown 단어 처리 개선 (mecab-ko-core)
- `WordPattern` 열거형: Plain, ProperNoun, CamelCase, HangulAlphaMix, NumberUnit, Emoji
- 패턴별 비용 조정
- 품사 태그 추정 개선

#### 복합명사 분해 개선
- 종성 패턴 분석 알고리즘 개선
- 접미사 자동 감지: 들, 님, 분, 꾼
- 접두사 자동 감지: 신, 구, 총, 부, 전, 후
- Character offset 정확도 개선

#### 커뮤니티 기여
- CONTRIBUTING.md: 신조어 추가 가이드
- 4개 이슈 템플릿
- CODE_OF_CONDUCT.md (Contributor Covenant 2.0)
- PR 템플릿

#### 신조어 사전
- 123개 신조어 (2018-2024)
- `data/user-dict/neologisms.csv`

### 변경됨 (Changed)

- 최소 Rust 버전: 1.75
- Node.js 22 지원
- Python 3.13 지원

### 수정됨 (Fixed)

- WASM 빌드 (`wasm-opt = false`)
- dict-sync 크레이트 Clippy 경고

### 문서

- 마이그레이션 가이드
- NIKL API 조사 문서
- mecab-ko-dic 현대화 계획

---

## [0.1.1] - 2026-03-01

### 추가됨 (Added)

- crates.io 발행 (6개 크레이트)
- GitHub Releases 자동화
- 성능 회귀 탐지 CI
- mdBook 문서 사이트
- Docker 이미지 (GHCR)
- Chart.js 성능 대시보드

### 수정됨 (Fixed)

- LazyEntries, mmap 메모리 최적화
- WASM zstd-sys 이슈 (optional feature)

---

## [0.1.0] - 2026-03-01

### 추가됨 (Added)

#### 핵심 라이브러리 (mecab-ko-core)
- Viterbi 알고리즘 구현
- Lattice 구조
- 토크나이저
- Zero-cost 추상화

#### 한글 유틸리티 (mecab-ko-hangul)
- 자모 분리/결합
- 음절 처리
- 문자 유형 분류
- 종성 판별

#### 사전 관리 (mecab-ko-dict)
- Double-Array Trie
- 연접 비용 매트릭스
- 사용자 사전 (CSV)
- 메모리 매핑

#### CLI (mecab-ko-cli)
- 다양한 출력 포맷
- N-best 출력
- 배치 처리

#### 바인딩
- Python (PyO3, KoNLPy 호환)
- Node.js (napi-rs)
- WASM (wasm-bindgen)

#### Elasticsearch (mecab-ko-elasticsearch)
- Nori 호환 분석기
- 토큰 필터
- Decompound 모드

#### CI/CD
- GitHub Actions 워크플로우
- 벤치마크 자동화
- 문서 배포

---

## 레거시 버전 (C/C++ 구현)

### [0.9.2] (mecab-0.996)
- dicdir 값 수정
- Java SWIG 메모리 누수 수정

### [0.9.1] (mecab-0.996)
- 새 사전 항목 추가 버그 수정
- 자동 문맥 ID 조회

### [0.9.0] (mecab-0.996)
- MeCab 0.996 기반
- 좌측 공백 연접 비용 증가 기능

---

## 로드맵

### 단기 계획 (v0.4.x - v0.5.0)
- [x] crates.io 정식 배포 ✅
- [ ] mecab-ko-dic v3.0 (100만+ 엔트리)
- [ ] 정확도 40%+ 달성 (사전 품질 개선)
- [ ] PyPI 배포
- [ ] 신조어 자동 수집 파이프라인 실행

### 중기 계획 (v0.5.0)
- [ ] 정확도 70%+ 달성
- [ ] OpenSearch 플러그인
- [ ] 실시간 사전 업데이트

### 장기 계획 (v1.0.0)
- [ ] Production-ready Elasticsearch 플러그인
- [ ] 정확도 90%+ 달성
- [ ] PyPI 정식 배포

---

## 버전 정책

### 버전 번호

- **MAJOR**: 하위 호환성이 깨지는 API 변경
- **MINOR**: 하위 호환성을 유지하는 기능 추가
- **PATCH**: 하위 호환성을 유지하는 버그 수정

### 지원 정책

- 최신 버전만 지원
- 보안 문제는 이전 MINOR 버전까지 패치 제공

---

## 기여자

프로젝트에 기여해 주신 모든 분들께 감사드립니다.

- hephaex ([@hephaex](https://github.com/hephaex)) - 프로젝트 리더

기여하고 싶으시다면 [GitHub](https://github.com/hephaex/mecab-ko)에서 참여해 주세요.

---

[0.4.0]: https://github.com/hephaex/mecab-ko/compare/v0.3.1...v0.4.0
[0.3.1]: https://github.com/hephaex/mecab-ko/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/hephaex/mecab-ko/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/hephaex/mecab-ko/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/hephaex/mecab-ko/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/hephaex/mecab-ko/releases/tag/v0.1.0
