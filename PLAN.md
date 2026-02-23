# 현재 스프린트: Phase 3 - Sprint 6 (성능 최적화, ES 완성, 프로파일러 완성) ✅

## 목표
성능 최적화 + 릴리스 준비, ES 크레이트 완성, 테스트 커버리지 확대, 프로파일러 완성

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

## Sprint 6 작업 목록

### CI 및 인프라
- [x] S6-01: CI에 Elasticsearch 크레이트 포함 (P0)
- [x] S6-02: CI에 test-allocator 테스트 추가 (P0)

### 테스트 확대
- [x] S6-03: mini-dict 기반 ignored 테스트 40개 추가 활성화 (P1)
- [x] S6-04: 로컬 전용 full-dict 테스트 인프라 (P1)
- [x] S6-05: Elasticsearch 통합 테스트 확장 (24 pass, 6 ignored) (P0)
- [x] S6-15: rustdoc ignore 테스트 활성화 (5개 이하 ignore) (P2)

### 프로파일러
- [x] S6-08: 프로파일러 실제 데이터 통합 (P0)
- [x] S6-09: 프로파일러 회귀 탐지 baseline save/compare (P1)

### 성능 최적화
- [x] S6-11: 벤치마크 실행 및 성능 기준선 확립 (P0)
- [x] S6-12: Hot path 성능 최적화 — 45-55% 개선 달성 (P0)
- [x] S6-13: Cold start 최적화 — 0.086ms (목표 200ms) 이미 충분 (P1)

### 릴리스 준비
- [x] S6-14: 릴리스 준비 (메타데이터 + cargo doc 0 경고) (P1)
- [x] S6-16: Sprint 6 전체 코드 리뷰 (P1)

## 크레이트 현황

| 크레이트 | 상태 | 비고 |
|----------|------|------|
| mecab-ko-hangul | ✅ 완료 | 한글 자소 분리/결합 |
| mecab-ko-dict | ✅ 완료 | Trie, Matrix, Loader, UserDict, HotReload, entries |
| mecab-ko-dict-builder | ✅ 완료 | CSV→binary 사전 빌더, 압축 |
| mecab-ko-core | ✅ 완료 | Viterbi, Lattice, Tokenizer, Unknown handler, Normalizer (최적화 완료) |
| mecab-ko-cli | ✅ 완료 | CLI 인터페이스 |
| mecab-ko-python | ✅ 완료 | PyO3 바인딩 (KoNLPy 호환) |
| mecab-ko-wasm | ✅ 완료 | WASM 바인딩 (wasm-bindgen) |
| mecab-ko-node | ✅ 완료 | Node.js 바인딩 (N-API) |
| mecab-ko-elasticsearch | ✅ 완료 | Nori 호환 (24 pass, 6 ignored) |
| mecab-ko-profiler | ✅ 완료 | 실제 사전 통합, baseline save/compare, 회귀 탐지 |
| mecab-ko-dict-validator | ✅ 완료 | CSV 검증, 규칙 엔진, CLI, 리포트 |
| mecab-ko (facade) | ✅ 완료 | 통합 API |
| benchmarks | ✅ 완료 | 9개 벤치마크 스위트, 기준선 문서화 |

## 성과 요약
1. **Tests**: 746 passed, 0 failed, 22 ignored (목표: 680+ pass, ≤70 ignored)
2. **CI**: ES + test-allocator 포함
3. **성능**: 토크나이저 45-55% 개선, 238K morphs/sec, 0.086ms cold start
4. **프로파일러**: `mecab-profile` CLI가 실제 사전으로 동작, baseline 비교 가능
5. **문서**: `cargo doc --workspace` 경고 0

## 다음 스프린트 예고
Sprint 7: crates.io 발행, Python/WASM 바인딩 최적화, 사전 현대화 시작
