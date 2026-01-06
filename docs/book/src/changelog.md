# 변경 이력

모든 주요 변경 사항을 기록합니다.

이 프로젝트는 [Semantic Versioning](https://semver.org/)을 따릅니다.

## [Unreleased]

### 추가됨 (Added)

- 초기 Rust 구현
- `mecab-ko-core`: 형태소 분석 핵심 엔진
  - Lattice 구조
  - Viterbi 알고리즘 (기본 구현)
  - 미등록어 처리
- `mecab-ko-dict`: 사전 관리 라이브러리
  - 사용자 사전 지원 (CSV 형식)
  - Double-Array Trie 구현
  - 연접 비용 매트릭스
- `mecab-ko-hangul`: 한글 유틸리티
  - 자모 분리/결합
  - 문자 유형 분류
  - 종성 판별
- `mecab-ko-cli`: 명령줄 도구
  - 다양한 출력 포맷 (default, wakati, json, csv, pos, simple, dump)
  - 사용자 사전 지원
  - N-best 출력
- `mecab-ko`: 통합 라이브러리

### 변경됨 (Changed)

- (없음)

### 사용 중단 (Deprecated)

- (없음)

### 제거됨 (Removed)

- (없음)

### 수정됨 (Fixed)

- (없음)

### 보안 (Security)

- (없음)

---

## 로드맵

### 단기 계획 (v0.2.0)

- [ ] 바이너리 사전 로더 완전 구현
- [ ] Viterbi 알고리즘 최적화
- [ ] N-best 경로 검색
- [ ] 띄어쓰기 패널티 구현

### 중기 계획 (v0.5.0)

- [ ] Python 바인딩 (PyO3)
- [ ] mecab-ko-dic v3.0 사전
- [ ] 성능 벤치마크

### 장기 계획 (v1.0.0)

- [ ] WASM 지원
- [ ] Elasticsearch 플러그인
- [ ] 정확도 95% 이상 달성

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
