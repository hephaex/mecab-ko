# 진행 상황

## 마지막 업데이트: 2026-02-24

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
- [x] PLAN.md, PROGRESS.md 업데이트 (실제 구현 상태 반영)
- [x] 의존성 업데이트 (wasm-bindgen 0.2.111, tempfile 3.25, rkyv 0.8.15 등 58개)
- [x] find_dicdir()에 test-fixtures/mini-dict 폴백 추가
- [x] 159개 ignored 테스트 활성화 (502→661 pass, 259→95 ignored)
- [x] mecab-ko-dict-validator 확인 (이미 완전 구현됨, 19 tests + 1 doc test 통과)
- [x] 코드 품질 점검 (라이브러리 0 경고, 벤치마크 컴파일 에러 수정)

### Phase 3 - Sprint 6 작업 (S6-03)
- [x] S6-03: mini-dict로 추가 ignored 테스트 40개 활성화 (670→710 pass, 95→55 ignored)

## 크레이트 실제 상태

| 크레이트 | 상태 | 비고 |
|----------|------|------|
| mecab-ko-hangul | ✅ 완료 | 한글 자소 분리/결합 |
| mecab-ko-dict | ✅ 완료 | Trie, Matrix, Loader, UserDict, HotReload, FileWatcher, entries |
| mecab-ko-dict-builder | ✅ 완료 | CSV→binary 사전 빌더, 압축, entries 저장 |
| mecab-ko-core | ✅ 완료 | Viterbi, Lattice, Tokenizer, Unknown handler, Normalizer |
| mecab-ko-cli | ✅ 완료 | CLI 인터페이스 |
| mecab-ko-python | ✅ 완료 | PyO3 바인딩 (KoNLPy 호환) |
| mecab-ko-wasm | ✅ 완료 | WASM 바인딩 (wasm-bindgen) |
| mecab-ko-node | ✅ 완료 | Node.js 바인딩 (N-API, TypeScript) |
| mecab-ko-elasticsearch | ⚠️ 부분 | Nori 호환 (8 테스트 ignored) |
| mecab-ko-profiler | ⚠️ 부분 | 성능 프로파일러 기초 |
| mecab-ko-dict-validator | ✅ 완료 | CSV 검증, 규칙 엔진, CLI, 리포트 (19 tests) |
| mecab-ko (facade) | ✅ 완료 | 통합 API |

## 테스트 현황
- **통과**: 710개
- **실패**: 0개
- **무시됨**: 55개 (full dict 필요, 메모리 프로파일러 feature, rust,ignore doc tests)
- **Clippy**: 경고 없음
- **브랜치**: main

### Full-Dict 테스트 인프라 (2026-02-24 추가)
- `rust/crates/mecab-ko-core/src/test_utils.rs` 생성
  - `full_dict_test!()` 매크로: `MECAB_KO_FULL_DICT` 미설정 시 테스트 스킵
  - `full_dict_path()` 함수: env var로부터 PathBuf 반환
- `mecab-ko-core/Cargo.toml`에 `test-utils` 피처 추가
- `lib.rs`에 `#[cfg(any(test, feature = "test-utils"))] pub mod test_utils` 추가
- `scripts/test-full-dict.sh` 생성 (실행 권한 포함)

## 블로커/이슈
- 없음

## GitHub 이슈
- #6: 커뮤니티 질문 (프로젝트 목표, 성능, 사전 계획 등) - 미응답
- #1-5, #7-8: 의존성 자동 업데이트 알림 (7건, semver-compatible 완료)

## 다음 세션에서 할 일
1. GitHub Issue #6 응답 (Q4, Q5는 프로젝트 오너 작성 필요)
2. Sprint 6 계획: 성능 최적화, 프로파일러 완성, 릴리스 준비
3. (선택) full-dict 테스트에 `full_dict_test!()` 매크로 적용하여 `#[ignore]` 어노테이션 대체
4. 남은 55개 ignored 분류:
   - 2개: full dict 필요 (symbols/numbers) - 영구적으로 ignored
   - 1개: 성능 비교 테스트 (test_batch_vs_sequential) - 비결정적
   - 11개: --features test-allocator 필요 (profiler)
   - 40개: rust,ignore doc tests (소스 문서 예제)
