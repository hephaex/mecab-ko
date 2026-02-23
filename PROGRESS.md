# 진행 상황

## 마지막 업데이트: 2026-02-23

## 완료된 작업

### Phase 1 - Sprint 1-2 (프로젝트 셋업)
- [x] RST-002: 프로젝트 구조 설계 (Cargo workspace 13개 크레이트)
- [x] RST-008: 한글 자소 분리/결합 유틸리티 (`mecab-ko-hangul` 구현 완료, 7 tests)
- [x] 프로파일러 기초 구현 (`mecab-ko-profiler` 부분 구현, 6 tests)
- [x] CI/CD 파이프라인 기초 설정
- [x] 프로젝트 자동화 인프라 구축 (.claude agents, skills, commands)

### Phase 1 - Sprint 3
- [x] research: MeCab 알고리즘 내부 구조 조사 → `docs/research/algorithms/mecab-internals.md`
- [x] research: 한국어 NLP 생태계 조사 → `docs/research/ecosystem/korean-nlp-ecosystem.md`
- [x] research: Rust 데이터 구조 크레이트 조사 → `docs/research/rust-crates/data-structures-and-algorithms.md`
- [x] research: MeCab 바이너리 사전 포맷 상세 → `docs/research/algorithms/mecab-binary-format.md`
- [x] research: Lindera 소스 코드 심층 분석 → `docs/research/ecosystem/lindera-analysis.md`

### Phase 2 - RST-003: 사전 로더 구현
- [x] SystemDictionary entries 로딩 파이프라인 구현 (CSV + binary 포맷)
- [x] SystemDictionary entries 저장 기능 (save_entries_bin, save_entries_csv)
- [x] Multi-entry lookup 지원 (같은 surface에 여러 품사)
- [x] DictionaryBuilder에 entries 저장 연동
- [x] mecab-ko-dic 2.1.1 다운로드 및 빌드 (816,283 엔트리)
- [x] 빌드 결과물: sys.dic(16MB), matrix.bin(20MB), entries.bin(54MB), entries.csv(59MB)

### Phase 2 - 토크나이저 테스트 수정
- [x] yada Trie 정렬 이슈 수정 (build → build_unsorted)
- [x] 테스트 context ID 범위 수정 (100,101 → 5,6, 10x10 matrix 내)
- [x] 모든 #[ignore] 토크나이저 테스트 활성화 (13개 테스트 통과)

## 크레이트 실제 상태

| 크레이트 | 상태 | 비고 |
|----------|------|------|
| mecab-ko-hangul | ✅ 완료 | 한글 자소 분리/결합 |
| mecab-ko-dict | ✅ 완료 | Trie, Matrix, Loader, UserDict, HotReload, FileWatcher, entries |
| mecab-ko-dict-builder | ✅ 완료 | CSV→binary 사전 빌더, 압축, entries 저장 |
| mecab-ko-core | ✅ 완료 | Viterbi, Lattice, Tokenizer, Unknown handler, Normalizer |
| mecab-ko-cli | ✅ 완료 | CLI 인터페이스 |
| mecab-ko-elasticsearch | ⚠️ 부분 | Nori 호환 분석기/필터 (사전 의존) |
| mecab-ko-profiler | ⚠️ 부분 | 성능 프로파일러 기초 |
| mecab-ko-dict-validator | ❌ 스텁 | 사전 검증 |
| mecab-ko-python | ❌ 스텁 | PyO3 바인딩 |
| mecab-ko-wasm | ❌ 스텁 | WASM 바인딩 |
| mecab-ko-node | ❌ 스텁 | Node.js 바인딩 |

## 블로커/이슈
- 없음

## 프로젝트 상태
- **빌드**: 통과
- **테스트**: 전체 통과 (0 failed)
- **Clippy**: 경고 없음
- **브랜치**: main

## 다음 세션에서 할 일
1. PLAN.md 업데이트 (실제 구현 상태 반영)
2. 사전 데이터를 test-fixtures에 통합하여 CI에서도 테스트 가능하도록
3. `--ignored` 테스트 (실제 사전 필요) CI 환경 구성
4. mecab-ko-elasticsearch Nori 호환 테스트 활성화
5. Python/WASM 바인딩 구현 시작
