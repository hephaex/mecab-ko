# 변경 이력

모든 주요 변경 사항을 기록합니다.

이 프로젝트는 [Semantic Versioning](https://semver.org/)을 따릅니다.

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

### 단기 계획 (v0.3.0)
- [ ] 신조어 자동 수집 파이프라인
- [ ] PyPI/npm 정식 배포
- [ ] 정확도 95% 달성

### 중기 계획 (v0.5.0)
- [ ] mecab-ko-dic v3.0
- [ ] OpenSearch 플러그인
- [ ] 실시간 사전 업데이트

### 장기 계획 (v1.0.0)
- [ ] Production-ready Elasticsearch 플러그인
- [ ] 정확도 98% 달성
- [ ] 커뮤니티 릴리스

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

[0.2.0]: https://github.com/hephaex/mecab-ko/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/hephaex/mecab-ko/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/hephaex/mecab-ko/releases/tag/v0.1.0
