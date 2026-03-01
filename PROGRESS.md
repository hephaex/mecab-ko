# 진행 상황

## 마지막 업데이트: 2026-03-01

## Sprint 7 진행 중

### 완료
- [x] S7-01: Path dependency에 version 추가 (7개 크레이트)
- [x] S7-02: cargo publish --dry-run 검증
  - ✅ mecab-ko-hangul (이미 crates.io에 존재)
  - ✅ mecab-ko-dict
  - ✅ mecab-ko-core
  - ✅ mecab-ko-dict-validator
  - ⚠️ mecab-ko-dict-builder (crates.io의 mecab-ko-dict 0.1.0에 save_entries 함수 없음)
  - ✅ mecab-ko (facade)
- [x] S7-04: Full-dict Memory KPI 측정
  - Peak Memory: **215 MB** (목표 150MB 초과)
  - Cold Start: **0.13s** (목표 0.2s 달성)
  - 상세: docs/research/benchmarks/sprint7-memory-kpi.md
- [x] S7-05: Python 바인딩 테스트 ✅
  - maturin build 성공
  - wheel 설치 및 import 성공
  - `Mecab().parse()` 동작 확인
- [x] S7-06: WASM 바인딩 테스트 ⚠️
  - wasm-pack build 실패
  - 원인: zstd-sys가 wasm32 타겟 미지원
  - 해결: Sprint 8에서 zstd 제거 또는 pure-Rust 대안
- [x] S7-07: Node.js 바인딩 테스트 ✅
  - cargo build 성공
  - libmecab_ko_node.dylib 생성 (669KB)

### 발행 순서 노트
- crates.io에 이미 mecab-ko-hangul 0.1.0 존재
- mecab-ko-dict-builder 발행 전에 mecab-ko-dict를 0.1.1로 업데이트 필요

### Memory 최적화 필요 (Sprint 8)
- entries 지연 로딩
- mmap 활용 강화
- String interning

## 완료된 작업

### Phase 1 - Sprint 1-2 (프로젝트 셋업)
- [x] RST-002: 프로젝트 구조 설계 (Cargo workspace 13개 크레이트)
- [x] RST-008: 한글 자소 분리/결합 유틸리티 (`mecab-ko-hangul` 구현 완료, 7 tests)
- [x] 프로파일러 기초 구현 (`mecab-ko-profiler` 부분 구현, 6 tests)
- [x] CI/CD 파이프라인 기초 설정
- [x] 프로젝트 자동화 인프라 구축 (.claude agents, skills, commands)

### Phase 1 - Sprint 3 (코어 데이터 구조)
- [x] research: MeCab 알고리즘 내부 구조 조사
- [x] research: 한국어 NLP 생태계 조사
- [x] research: Rust 데이터 구조 크레이트 조사
- [x] research: MeCab 바이너리 사전 포맷 상세
- [x] research: Lindera 소스 코드 심층 분석

### Phase 2 - Sprint 4 (코어 엔진 + 바인딩)
- [x] SystemDictionary entries 로딩, Multi-entry lookup
- [x] mecab-ko-dic 2.1.1 빌드 (816,283 엔트리)
- [x] Viterbi, Lattice, Tokenizer, Unknown handler, Normalizer
- [x] CLI, Python/WASM/Node 바인딩, Elasticsearch Nori 호환

### Phase 3 - Sprint 5 (안정화)
- [x] 의존성 업데이트 (wasm-bindgen 0.2.111, tempfile 3.25, rkyv 0.8.15 등 58개)
- [x] find_dicdir()에 test-fixtures/mini-dict 폴백 추가
- [x] 159개 ignored 테스트 활성화 (502→661 pass, 259→95 ignored)
- [x] mecab-ko-dict-validator 확인 (19 tests + 1 doc test)
- [x] 코드 품질 점검 (라이브러리 0 경고)

### Phase 3 - Sprint 6 (성능 최적화, ES 완성, 프로파일러 완성) ✅
- [x] S6-01: CI에 Elasticsearch 크레이트 포함
- [x] S6-02: CI에 test-allocator 테스트 별도 job 추가
- [x] S6-03: mini-dict로 추가 ignored 테스트 40개 활성화 (670→710 pass)
- [x] S6-04: Full-dict 테스트 인프라 (test_utils.rs, test-full-dict.sh)
- [x] S6-05: Elasticsearch 통합 테스트 12개 추가 (24 pass, 6 ignored)
- [x] S6-08: 프로파일러 실제 사전 데이터 통합 (mecab-profile CLI 완전 재작성)
- [x] S6-09: 프로파일러 회귀 탐지 (`baseline save/compare` 서브커맨드)
- [x] S6-11: 9개 벤치마크 스위트 실행, KPI 기준선 확립
- [x] S6-12: Hot path 성능 최적화 (토크나이저 45-55% 개선)
  - SpacePositions: HashSet → sorted Vec + binary_search
  - SpacePenalty: linear scan → binary_search
  - parse_features: Vec → splitn iterator
  - Lattice: byte_to_char binary search helper
- [x] S6-13: Cold start 최적화 (0.086ms, 이미 목표 200ms 충족)
- [x] S6-14: 릴리스 준비 (cargo doc 0 경고, 메타데이터 정리)
- [x] S6-15: rustdoc ignore → no_run/runnable 전환 (11→5 ignore)
- [x] S6-16: Sprint 6 전체 코드 리뷰

## 크레이트 실제 상태

| 크레이트 | 상태 | 비고 |
|----------|------|------|
| mecab-ko-hangul | ✅ 완료 | 한글 자소 분리/결합 |
| mecab-ko-dict | ✅ 완료 | Trie, Matrix, Loader, UserDict, HotReload, FileWatcher, entries |
| mecab-ko-dict-builder | ✅ 완료 | CSV→binary 사전 빌더, 압축, entries 저장 |
| mecab-ko-core | ✅ 완료 | Viterbi, Lattice, Tokenizer, Unknown handler, Normalizer (최적화) |
| mecab-ko-cli | ✅ 완료 | CLI 인터페이스 |
| mecab-ko-python | ✅ 완료 | PyO3 바인딩 (KoNLPy 호환) |
| mecab-ko-wasm | ✅ 완료 | WASM 바인딩 (wasm-bindgen) |
| mecab-ko-node | ✅ 완료 | Node.js 바인딩 (N-API, TypeScript) |
| mecab-ko-elasticsearch | ✅ 완료 | Nori 호환 (24 pass, 6 ignored) |
| mecab-ko-profiler | ✅ 완료 | 실제 사전 통합, baseline save/compare |
| mecab-ko-dict-validator | ✅ 완료 | CSV 검증, 규칙 엔진, CLI, 리포트 |
| mecab-ko (facade) | ✅ 완료 | 통합 API |

## 테스트 현황
- **통과**: 746개
- **실패**: 0개
- **무시됨**: 22개
- **Clippy**: 경고 없음
- **cargo doc**: 경고 없음
- **브랜치**: main

## 성능 KPI (mini-dict 기준)

| KPI | 목표 | 측정값 | 상태 |
|-----|------|--------|------|
| Morphemes/sec | 150K | ~238K | PASS |
| Cold start | < 200ms | 0.086ms | PASS |
| Memory per instance | < 150MB | N/A (mini-dict) | TBD |

### 벤치마크 개선 (S6-12)

| Input Size | Before | After | Improvement |
|-----------|--------|-------|-------------|
| 10 chars | 8.6µs | 3.8µs | -55% |
| 50 chars | 77.5µs | 44.9µs | -42% |
| 100 chars | 198µs | 141µs | -31% |
| 500 chars | 3055µs | 2165µs | -29% |
| 1000 chars | 9978µs | 8413µs | -16% |

## 블로커/이슈
- 없음

## GitHub 이슈
- #6: 커뮤니티 질문 (프로젝트 목표, 성능, 사전 계획 등) - ✅ 응답 완료

## 다음 세션에서 할 일
1. Sprint 7 계획: crates.io 발행, Python/WASM 바인딩 최적화, 사전 현대화
2. Full-dict 벤치마크로 Memory KPI 측정
