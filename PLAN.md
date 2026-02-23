# 현재 스프린트: Phase 3 - Sprint 5 (안정화 및 CI 강화)

## 목표
프로젝트 안정화: 의존성 업데이트, CI 테스트 커버리지 확대, 사전 검증기 구현

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

## 작업 목록

### 안정화 (Stabilization)
- [x] update: 의존성 업데이트 (wasm-bindgen 0.2.111, tempfile 3.25, rkyv 0.8.15 등)
- [x] test: CI용 소형 테스트 사전 fixture 연동 (find_dicdir 폴백 추가)
- [x] test: ignored 테스트 159개 활성화 (661 pass / 0 fail / 95 ignored)

### 구현 (Implementation)
- [x] implement: mecab-ko-dict-validator 사전 검증기 확인 (이미 완료 상태, 19 tests + 1 doc test)

### 검증 (Verification)
- [x] review: 전체 코드베이스 품질 점검 (라이브러리 0 경고, 벤치마크 컴파일 에러 수정)

### 문서/커뮤니티 (Documentation/Community)
- [ ] docs: GitHub Issue #6 커뮤니티 질문 응답 (담당: tech-writer, P2)

## 의존성
- dict-validator → (선행 완료)
- 코드 리뷰 → dict-validator 완료 후

## 크레이트 현황

| 크레이트 | 상태 | 비고 |
|----------|------|------|
| mecab-ko-hangul | ✅ 완료 | 한글 자소 분리/결합 |
| mecab-ko-dict | ✅ 완료 | Trie, Matrix, Loader, UserDict, HotReload, entries |
| mecab-ko-dict-builder | ✅ 완료 | CSV→binary 사전 빌더, 압축 |
| mecab-ko-core | ✅ 완료 | Viterbi, Lattice, Tokenizer, Unknown handler, Normalizer |
| mecab-ko-cli | ✅ 완료 | CLI 인터페이스 |
| mecab-ko-python | ✅ 완료 | PyO3 바인딩 (KoNLPy 호환) |
| mecab-ko-wasm | ✅ 완료 | WASM 바인딩 (wasm-bindgen) |
| mecab-ko-node | ✅ 완료 | Node.js 바인딩 (N-API) |
| mecab-ko-elasticsearch | ⚠️ 부분 | Nori 호환 (8 테스트 ignored) |
| mecab-ko-profiler | ⚠️ 부분 | 성능 프로파일러 기초 |
| mecab-ko-dict-validator | ✅ 완료 | CSV 검증, 규칙 엔진, CLI, 리포트 (19 tests) |
| mecab-ko (facade) | ✅ 완료 | 통합 API |
| benchmarks | ✅ 완료 | 벤치마크 (컴파일 에러 수정 완료) |

## 다음 스프린트 예고
Sprint 6: 성능 최적화, 프로파일러 완성, 사전 커버리지 확대, 릴리스 준비
