# Phase 84 - Sprint 118 (UNKNOWN 공백 분할 Viterbi + known_limitation 해소)

## Sprint 118 로드맵

### P1: UNKNOWN 토큰 공백 분할 — Viterbi 레벨 수정
- 조사: unknown handler의 group loop는 이미 공백에서 분할됨
- 근본 원인: Viterbi가 2-char UNKNOWN 노드를 chain하여 단일 경로 생성
- 방안 A: unknown handler에서 공백 경계 노드에 추가 비용 부여
- 방안 B: Viterbi forward pass에서 unknown→unknown 연결 시 공백 패널티 강화
- 방안 C: build_lattice에서 공백 위치에 barrier 노드 삽입
- 15건 known_limitation 해소 기대

### P2: golden test known_limitation 업데이트
- P1 수정 후 16건 중 해소된 케이스의 expected_pos를 실제 분할 결과로 갱신
- status 필드 제거 (정상 테스트 전환)

### P3: tokenizer 공백 분할 단위 테스트
- unknown word + space boundary 조합의 regression test 추가
- 엣지 케이스: 연속 공백, 탭, 혼합 공백

### P4: mini-dict entries.csv → entries.bin 자동 동기화 CI
- create_mini_dict 실행 → diff 확인 → 불일치 시 CI 실패

---

# 완료: Phase 83 - Sprint 117 (ValueEnum + run_convert 분할 + mini-dict 확장)

## Sprint 117 결과
- dict-builder: --output-format String → clap::ValueEnum (컴파일 타임 검증)
- dict-builder: run_convert → save_entries_format/verify_entries_format 헬퍼 추출
- mini-dict: 43→56 엔트리 (고/도/면/서/며/되/수/후/문제/안/모두/정말/그녀)
- create_mini_dict: entries.bin 자동 생성 (mecab-ko-dict LazyEntries::save_entries)
- UNKNOWN 공백 분할 조사: group loop는 정상, Viterbi chain이 근본 원인 → Sprint 118 P1
- 테스트: 1,181 pass / 0 fail / 52 ignored, clippy 0 warnings

---

# 완료: Phase 82 - Sprint 116 (doc-test 복원 + 미사용 API 정리 + v3 CI)

## Sprint 116 결과
- trie/owned.rs: doc-test 3건 복원
- trie/mod.rs: 380→221줄 (DictionarySearcher/EntryIndex/PrefixMatch 제거)
- lib.rs: 미사용 re-export 제거
- dict-build.yml: test-v3-format job (v2→v3→v2 round-trip)
- known_limitation 분석: 15/16건 구조적 문제 (공백 미분할), 1건 mini-dict 확장 가능
- 테스트: 1,181 pass / 0 fail / 52 ignored, clippy 0 warnings

---

# 완료: Phase 81 - Sprint 115 (trie/mod.rs 분할 + dict-builder v3 옵션 + entry_store 통합 리팩토링)

## Sprint 115 결과
- trie/mod.rs 681→380줄: Trie + TrieBuilder → trie/owned.rs 추출
- impl_lazy_store! 매크로: LazyStore/LazyStoreV3 중복 86줄 제거
- dict-builder: --output-format v2|v3 옵션 + Info MKE3 감지
- golden test: 16건 complex.json에 known_limitation 플래그
- 테스트: 1,181 pass / 0 fail / 52 ignored, clippy 0 warnings

---

# 완료: Phase 80 - Sprint 114 (entries.bin v3 SystemDictionary 통합 + mmap trie test)

## Sprint 114 목표
entries.bin v3(MKE3)를 SystemDictionary 로더에 통합하고, mmap trie 통합 테스트를 추가한다.

## Sprint 114 결과
- LazyStoreV3: EntryStore trait 구현 (LazyEntriesV3 래핑)
- dictionary.rs: detect_entries_format()으로 V1/V2/V3 자동 감지 + 분기 로드
- get_entries_at(): LazyEntriesV3에 연속 surface 조회 추가
- migrate_v2_to_v3(): v2→v3 마이그레이션 유틸리티
- mmap trie integration test: 3건 (로드, 결과 동일성, memory_optimized)
- 테스트: 1,184 pass / 0 fail / 52 ignored, clippy 0 warnings

---

# 완료: Phase 79 - Sprint 113 (entries.bin v3 PoC + Golden Test 300)

## Sprint 113 목표
entries.bin v3 포맷 PoC를 구현하고, golden test를 300건으로 확장한다.

## Sprint 113 결과
- entries.bin v3 (MKE3): 24B 헤더, feature_len u32, LazyEntriesV3 mmap 저장소
- detect_entries_format(): v1(MKED)/v2(MKE2)/v3(MKE3) 자동 감지
- save_entries_v3(): MKE3 직렬화 (인덱스 테이블 포함)
- common_prefix_search_at: Vec → PrefixSearchResult (SmallVec) 일관성 수정
- Golden Test: 250→300건 (basic 132, nouns 87, complex 81)
- 테스트: 1,180 pass / 0 fail / 49 ignored, clippy 0 warnings

---

# 완료: Phase 78 - Sprint 112 (SystemDictionary TrieBackend 전환 + Benchmark CI)

## Sprint 112 목표
SystemDictionary가 TrieBackend를 직접 사용하도록 전환하고, benchmark CI threshold를 설정한다.

## Sprint 112 결과
- SystemDictionary: `trie: Trie<'static>` → `trie: TrieBackend` 전환
- LoadOptions: `use_mmap_trie: bool` 추가 (memory_optimized에서 true)
- common_prefix_search_at: MmapTrie + TrieBackend에 추가 (API 완전성)
- benchmark CI: continue-on-error → core.setFailed() (10% 회귀 시 CI 실패)
- 테스트: 1,176 pass / 0 fail / 49 ignored, clippy 0 warnings

---

# 완료: Phase 77 - Sprint 111 (trie 분할 + SmallVec + Golden Test 250건)

## Sprint 111 목표
trie.rs를 분할하고, Box<dyn Iterator>를 SmallVec으로 교체하고, golden test를 250건으로 확장한다.

## Sprint 111 결과
- trie.rs (808줄) → trie/mod.rs(681) + mmap.rs(59) + backend.rs(93)
- TrieBackend: Box<dyn Iterator> → SmallVec<[(u32, usize); 16]> (힙 할당 제거)
- PrefixSearchResult 타입 별칭 추가
- mmap vs owned trie 벤치마크 추가 (trie_bench.rs)
- Golden Test: 200→250건 (basic 115, nouns 70, complex 65)
- POS 커버리지: 39→40/45 (XR 어근 추가)
- 테스트: 1,176 pass / 0 fail / 49 ignored, clippy 0 warnings

---

# 완료: Phase 76 - Sprint 110 (TrieBackend 통합 + matrix 분할)

## Sprint 110 목표
TrieBackend를 사전 로더에 통합하고, matrix 모듈을 분할한다.

## Sprint 110 결과
- loader.rs: trie 필드를 Trie<'static> → TrieBackend로 전환
- use_mmap=true 시 MmapTrie 자동 사용, 압축 파일은 Owned fallback
- matrix/mod.rs (1036줄) → mod.rs(485) + dense.rs(362) + sparse.rs(115) + mmap.rs(105)
- 테스트: 1,176 pass / 0 fail / 49 ignored, clippy 0 warnings

---

# 완료: Phase 75 - Sprint 109 (sys.dic mmap PoC + parse_matrix_header)

## Sprint 109 목표
sys.dic mmap PoC로 Trie 백엔드 분리를 시작하고, matrix 헤더 파서를 리팩토링한다.

## Sprint 109 결과
- MmapTrie: DoubleArray<memmap2::Mmap> 기반 zero-copy trie (unsafe는 Mmap::map만)
- TrieBackend enum: Owned(Trie<'static>) vs Mmap(MmapTrie), 검색 메서드 위임
- parse_matrix_header(): v2/v3 자동 감지 로직 DenseMatrix/MmapMatrix에서 추출
- MatrixHeader 구조체로 헤더 정보 캡슐화
- 테스트: 1,176 pass / 0 fail / 49 ignored, clippy 0 warnings

---

# 완료: Phase 74 - Sprint 108 (matrix.bin MKM3 헤더 + mini-dict 재빌드)

## Sprint 108 목표
matrix.bin에 MKM3 헤더를 추가하고, 확장된 mini-dict CSV로 바이너리를 재빌드한다.

## Sprint 108 결과
- matrix.bin v3: MKM3 magic(4B) + version(1B) + flags(1B) + reserved(2B) + lsize(u32) + rsize(u32) = 16B 헤더
- v2/v3 자동 감지: from_bin_bytes(), MmapMatrix::from_file() 모두 지원
- to_bin_bytes_v3() 추가 (기존 to_bin_bytes() v2 유지)
- mini-dict: create_mini_dict.rs 43엔트리 + 44x44 matrix + 41키 trie 재빌드
- 테스트: 1,173 pass / 0 fail / 49 ignored, clippy 0 warnings

---

# 완료: Phase 73 - Sprint 107 (LRU O(1) 교체 + Golden Test 200건)

## Sprint 107 목표
entries.bin의 hand-rolled LRU를 lru 크레이트로 교체하고, golden test를 200건으로 확장한다.

## Sprint 107 결과
- LRU 캐시: hand-rolled O(n) → lru crate O(1) eviction
- Read path: RwLock::write() → RwLock::read() + peek() (write lock 불필요)
- Golden Test: 155→200건 (basic 95, nouns 55, complex 50)
- POS 커버리지: 38→39/45 (SE 추가)
- mini-dict: entries.csv 21→43 엔트리 (CSV만, binary rebuild 별도)
- 테스트: 1,170 pass / 0 fail / 49 ignored, clippy 0 warnings

---

# 완료: Phase 72 - Sprint 106 (v0.8.0 Binary Dict v3 설계 + Golden Test 확장)

## Sprint 106 목표
v2 dict 코드 분석 결과를 바탕으로 v3 바이너리 사전 포맷 설계를 시작하고, golden test를 155건 이상으로 확장한다.

## Sprint 106 결과
- v3 dict schema 설계 문서 작성 (docs/design/v3-dict-schema.md, 374줄)
- User dict 개선 설계 문서 작성 (docs/design/user-dict-improvement.md, 449줄)
- Golden Test: 144건 → 155건 (basic 80, nouns 40, complex 35)
- POS 커버리지: 33 → 38/45 (VCN, JC, MAJ, JKV, XSA 추가)
- Difficulty level 1-5 전 테스트 케이스에 도입
- 테스트: 1,170 pass / 0 fail / 49 ignored, clippy 0 warnings

---

# 완료: Phase 71 - Sprint 105 (Golden Test 100→144 확장 + v2 Dict 분석)

## Sprint 105 목표
Golden Test 테스트셋을 현재 100건 → 150건 이상으로 확장하고, 문법 카테고리별 커버리지 갭을 해소한다. 부수적으로 v2 바이너리 사전 포맷 분석 문서를 작성하여 v0.8.0 설계 기반을 마련한다.

## Sprint 105 결과
- Golden Test: 100건 → 144건 (basic 75, nouns 40, complex 29)
- POS 커버리지: 33/45 태그 (30+ 목표 달성)
- 카테고리/설명: 전 테스트 케이스에 category/description 추가
- POS 커버리지 리포트 테스트 + 카테고리 통계 테스트 추가
- v2 dict 코드 분석 문서 작성 (docs/design/v2-dict-code-analysis.md, 473줄)
- 테스트: 1,169 pass / 0 fail / 96 ignored, clippy 0 warnings

## 배경
- Sprint 101-104: CI/테스트 인프라 정비 완료 (4개 연속 스프린트)
- 현재 golden test: basic.json 50건, nouns.json 30건, complex.json 20건 = **총 100건**
- 목표 DIC-009: 정확도 검증용 golden test 1,000+문장 (이번 스프린트에서 150+ 달성)
- POS 커버리지 갭: basic.json에 JKO, JKG, ETN, SN, SL, SP, SS, SW, NR, XPN, XSN 미포함
- mini-dict는 21 엔트리 → 대부분 golden test가 mini-dict에서는 structural 검증만 수행
- v2 dict 포맷 분석 문서가 이미 존재 (`docs/dictionary-format-v2.md`) — v3 설계 전 코드 기반 gap 분석 필요

## Sprint 105 작업 목록

### Track A: Golden Test 데이터 확장 (DIC-009)

- [ ] S105-01: basic.json 조사/어미 커버리지 확장 (+15건)
  - **설명**: JKO(목적격), JKG(관형격), JKB(부사격), JKC(보격) 등 조사 분류별 테스트 추가. ETN(명사형 전성어미), ETM(관형형 전성어미), EC(연결어미) 유형별 예문 추가.
  - **난이도**: Medium
  - **트랙**: Data
  - **파일**: `rust/crates/mecab-ko/tests/golden/basic.json`
  - **POS 갭 해소**: JKO, JKG, JKC, ETN 추가

- [ ] S105-02: basic.json 숫자/외래어/기호 커버리지 (+10건)
  - **설명**: SN(숫자), SL(외국어), SW(기타기호), SP(쉼표), SS(따옴표) 등 기호계 POS 테스트 추가. "100개", "API 호출", "2026년" 등 혼합형 문장.
  - **난이도**: Easy
  - **트랙**: Data
  - **파일**: `rust/crates/mecab-ko/tests/golden/basic.json`
  - **POS 갭 해소**: SN, SL, SW, SP, SS, NR 추가

- [ ] S105-03: nouns.json 복합명사/신조어 확장 (+10건)
  - **설명**: 3음절+ 복합명사, 의학/법률/학술 도메인 명사, 최신 신조어 추가. "생성형인공지능", "탄소중립", "메타인지" 등.
  - **난이도**: Easy
  - **트랙**: Data
  - **파일**: `rust/crates/mecab-ko/tests/golden/nouns.json`

- [ ] S105-04: complex.json 다양한 문체 확장 (+15건)
  - **설명**: 구어체, 문어체, 뉴스체, 학술체 등 문체별 긴 문장 추가. 이중 부정, 피동/사동 구문, 관계절 중첩 등 통사적 복잡성 높은 문장.
  - **난이도**: Hard
  - **트랙**: Data
  - **파일**: `rust/crates/mecab-ko/tests/golden/complex.json`

### Track B: Golden Test 인프라 개선

- [ ] S105-05: category/description 필드 체계화
  - **설명**: 모든 golden test case에 `description` (한줄 설명)과 `category` (문법 분류) 필드 추가. category 값: "greeting", "question", "particle", "number", "foreign", "compound-noun", "colloquial", "formal", "news", "academic", "passive", "causative" 등.
  - **난이도**: Medium
  - **트랙**: Code + Data
  - **파일**: `rust/crates/mecab-ko/tests/golden/*.json`

- [ ] S105-06: POS 커버리지 리포트 테스트 추가
  - **설명**: `integration_golden.rs`에 전체 golden test의 POS 태그 커버리지를 집계하는 테스트 추가. 전체 45개 POS 태그 중 몇 개가 커버되는지 리포트. 30개 이상 커버 필수 assert.
  - **난이도**: Medium
  - **트랙**: Code
  - **파일**: `rust/crates/mecab-ko/tests/integration_golden.rs`

- [ ] S105-07: golden test 카테고리별 통계 테스트 추가
  - **설명**: category 필드 기반으로 카테고리별 테스트 수, 통과율을 출력하는 테스트 함수 추가.
  - **난이도**: Easy
  - **트랙**: Code
  - **파일**: `rust/crates/mecab-ko/tests/integration_golden.rs`

### Track C: v2 Dict 코드 분석 (DIC-010 준비, lightweight)

- [ ] S105-08: v2 바이너리 포맷 코드 분석 메모
  - **설명**: `lazy_entries.rs` (entries.bin v2), `trie.rs` (sys.dic), `matrix/mod.rs` (matrix.bin)의 실제 코드를 읽고, 현재 v2 포맷의 구조/제약/성능 특성을 정리. `docs/dictionary-format-v2.md`의 코드 맵 섹션 갱신 또는 `docs/design/v2-dict-code-analysis.md` 신규 작성.
  - **난이도**: Medium
  - **트랙**: Design
  - **파일**: `docs/design/v2-dict-code-analysis.md` (신규)
  - **산출물**: v3 설계 시 참조할 v2 제약 목록 (mmap 부재, 압축 미지원, 엔디안 이슈 등)

### Track D: 검증

- [ ] S105-09: cargo test 전체 통과 확인 + clippy 검증
  - **설명**: 추가된 golden test 데이터와 코드가 빌드/테스트/clippy를 모두 통과하는지 검증.
  - **난이도**: Easy
  - **트랙**: Verify
  - **파일**: 전체

- [ ] S105-10: Sprint 106 로드맵 작성
  - **설명**: PLAN.md에 Sprint 106 로드맵 추가.
  - **난이도**: Easy
  - **트랙**: Planning
  - **파일**: `docs/project/PLAN.md`

## 의존성

```
S105-01 ──┐
S105-02 ──┤
S105-03 ──┼──→ S105-05 (category/description 추가) ──→ S105-07 (카테고리 통계)
S105-04 ──┘                                        └──→ S105-06 (POS 커버리지)
                                                         │
S105-08 (독립, 병렬 진행 가능)                              ↓
                                                    S105-09 (검증)
                                                         ↓
                                                    S105-10 (로드맵)
```

## 현재 Golden Test 현황 (Sprint 105 시작 시점)

| 파일 | 건수 | POS 커버리지 |
|------|------|-------------|
| basic.json | 50 | EC, EF, EP, EP+EF, ETM, IC, JKB, JKS, JX, MAG, MM, NNB, NNG, NP, SF, VA, VCP, VV, VX, XSV (20개) |
| nouns.json | 30 | NNG, NNP (2개) |
| complex.json | 20 | EC, EF, EP, ETM, ETN, JKB, JKG, JKO, JKQ, JKS, JX, MAG, MM, NNB, NNG, NNP, NP, NR, SF, SL, SN, SP, SS, SW, VA, VCP, VV, VX, XPN, XSN, XSV (31개) |
| **전체 (중복 제거)** | **100** | **33개** |

### Sprint 105 목표

| 파일 | 목표 건수 | 주요 추가 항목 |
|------|----------|--------------|
| basic.json | 75 | 조사 분류, 어미 유형, 숫자/외래어/기호 |
| nouns.json | 40 | 복합명사, 도메인 전문어, 신조어 |
| complex.json | 35 | 구어체, 문어체, 피동/사동, 이중부정 |
| **전체** | **150** | POS 35개+ 커버리지 |

## Sprint 106 로드맵 (미리보기)

### P1: v0.8.0 Binary Dict v3.0 설계 착수 (DIC-010)
- S105-08 분석 결과를 바탕으로 v3 스키마 구체화
- mmap 지원 PoC (memmap2 크레이트 통합)
- Zstd 압축 레이어 설계

### P2: Golden Test 250건 확장 (DIC-009 계속)
- 구문 분석 난이도별 분류 (Level 1-5)
- 오분석 사례 수집 (mecab-ko-dic 알려진 오류)
- mini-dict 확장 (21 → 50 엔트리) 검토

### P3: 사용자 사전 개선 설계 (RST-011)
- hot-reload v2 안정화
- domain overlay API 설계
- 사용자 사전 포맷 표준화

### P4: Benchmark CI 개선
- full-dict 벤치마크 결과 자동 비교
- 성능 회귀 자동 감지 threshold 설정

---

# 완료: Phase 70 - Sprint 104 (E2E CI false-green 수정 — mini-dict + graceful skip + MSRV)

## Sprint 104 목표
E2E CI의 continue-on-error로 마스킹된 실제 실패 19건 수정

## Sprint 104 작업 목록

- [x] S104-01: Python E2E 테스트 — mini-dict 단어 사용 (4건 실패 수정) ✅
- [x] S104-02: Node.js E2E 테스트 — 바인딩 미설치 시 graceful skip (12건) ✅
- [x] S104-03: CLI E2E — Rust 1.80.0 제거 (icu_* 1.83+ 요구, 3건) ✅
- [x] S104-04: job-level continue-on-error 제거 (cli/python/nodejs) ✅
- [x] S104-05: 빌드/테스트/clippy 검증 + 커밋 ✅

### 근본 원인
- Python: 테스트 텍스트가 mini-dict에 없는 단어 사용 → 빈 결과 반환 → assert 실패
- Node.js: mecab-ko-node 패키지 미설치 → import 실패 → 전체 12 테스트 실패
- CLI MSRV: icu_* v2.1.1이 Rust 1.83+ 요구 → 1.80.0 빌드 실패
- 모든 실패가 job-level continue-on-error로 마스킹되어 CI는 false green

---

# 완료: Phase 69 - Sprint 103 (E2E FFI 테스트 확장 — Python 13, Node.js 12, WASM 5)

---

# 완료: Phase 68 - Sprint 102 (E2E CLI 테스트 확장 + full-dict 벤치마크 CI)

---

# 완료: Phase 67 - Sprint 101 (dict-build CI .zst 검증 수정 + SHA256 checksum)

## Sprint 101 목표
dict-build.yml CI 실패 수정 — 압축 출력 파일(.zst) 검증, SHA256 checksum 추가

## Sprint 101 작업 목록

- [x] S101-01: dict-build.yml 파일 검증 — .zst 압축 파일 지원 ✅
- [x] S101-02: bitbucket 다운로드에 SHA256 checksum 추가 ✅
- [x] S101-03: tokenize-test/generate-report 조건부 실행 (빌드 성공 시만) ✅

---

# 완료: Phase 66 - Sprint 100 (벤치마크 + dict-build CI + e2e scaffolding)

---

# 완료: Phase 65 - Sprint 99 (CI nightly Rust continue-on-error)

---

# 완료: Phase 64 - Sprint 98 (Python Bindings CI 테스트 dict-aware skip)

---

# 완료: Phase 63 - Sprint 97 (python-wheels Octokit + e2e-ffi-tests virtualenv/compat 수정)

---

# 완료: Phase 62 - Sprint 96 (excluded FFI crate workspace root 해결 + CI 추가 수정)

---

# 완료: Phase 61 - Sprint 95 (CI 워크플로우 6건 수정 + accuracy tests ignore)

---

# 완료: Phase 60 - Sprint 94 (CI 워크플로우 수정 + Actions Node.js 24 대응)

## Sprint 94 목표
CI 워크플로우 3개 실패 수정, GitHub Actions Node.js 20 deprecation 대응 (v4→v5)

## Sprint 94 작업 목록

### Track A: CI 워크플로우 수정
- [x] S94-01: python-wheels.yml — PyO3/maturin-action@v2 → @v1 (v2 미존재) ✅
- [x] S94-02: npm-publish.yml — wasm-pack workspace root 해결 (repo root에서 실행) ✅
- [x] S94-03: docker.yml — id-token: write + attestations: write 권한 추가 ✅

### Track B: Actions 업데이트
- [x] S94-04: 전체 16개 워크플로우 actions v4→v5 (checkout, upload/download-artifact, setup-node) ✅

### Track C: 검증
- [x] S94-05: 빌드/테스트/클리피 검증 + 리뷰 + Sprint 95 로드맵 ✅

### CI 워크플로우 수정 상세
| 워크플로우 | 문제 | 수정 |
|-----------|------|------|
| python-wheels.yml | `maturin-action@v2` 미존재 | `@v1` (최신 v1.50.0) |
| npm-publish.yml | wasm-pack workspace root 못 찾음 | repo root에서 crate path 인자로 실행 |
| docker.yml | attest-build-provenance OIDC 토큰 없음 | `id-token: write` + `attestations: write` 권한 |

### 잔여 CI 이슈 (Sprint 94 범위 외)
- ci.yml: 테스트 30개 실패 (사전 데이터 미포함 — 구조적 문제)
- dict-build.yml: 사전 빌드 런타임 에러 (bitbucket 다운로드/파싱)

---

## Sprint 95 로드맵

### P1: CI 워크플로우 검증
- push 후 python-wheels, npm-publish, docker 워크플로우 재실행 확인
- ci.yml 테스트 실패 원인 분석 (사전 데이터 의존성)

### P2: 시스템 사전 벤치마크
- `brew install mecab-ko-dic` 후 MECAB_DICDIR 설정
- full-dict 벤치마크 실행 + mini-dict 비교 리포트

### P3: v0.8.0 기능 계획 구체화
- ISSUE_BACKLOG에서 P0 항목 선정
- Binary Dict v3 (DIC-010), 사전 검증 (DIC-009), 사용자 사전 개선 (RST-011)

### P4: ci.yml 사전 데이터 테스트 해결
- mini-dict 또는 fixture 기반 CI 테스트 전략 수립
- dict-build.yml 안정화

---

# 완료: Phase 59 - Sprint 93 (git tag v0.7.2 + GitHub Release + docs.rs 검증 + MSRV 문서)
# 완료: Phase 58 - Sprint 92 (문서 v0.7.2 갱신 + RELEASE_CHECKLIST + git tag 준비)

---

# 완료: Phase 56 - Sprint 90 (벤치마크 문서화 + 코드 품질 + 배포 준비)
# 완료: Phase 55 - Sprint 89 (v0.7.2 릴리스 준비 + 코드 품질 + 벤치마크)
# 완료: Phase 54 - Sprint 88 (MSRV 1.80 + LazyLock + CI 수정)
# 완료: Phase 53 - Sprint 87 (per-call 할당 제거 성능 최적화)
# 완료: Phase 52 - Sprint 86 (v0.7.1 릴리스 + 보호 테스트 확충)
# 완료: Phase 51 - Sprint 85 (corrections/ 디렉토리 전환)
# 완료: Sprint 80-84 (corrections 분할 1-4차)
# 아카이브: Sprint 1-72 (.history/ 및 Git 히스토리 참조)

## 크레이트 발행 현황

| 크레이트 | crates.io | 로컬 | 상태 |
|---------|-----------|------|------|
| mecab-ko-hangul | v0.7.2 | v0.7.2 | ✅ 배포 완료 |
| mecab-ko-dict | v0.7.2 | v0.7.2 | ✅ 배포 완료 |
| mecab-ko-core | v0.7.2 | v0.7.2 | ✅ 배포 완료 |
| mecab-ko-dict-validator | v0.7.2 | v0.7.2 | ✅ 배포 완료 |
| mecab-ko-dict-builder | v0.7.2 | v0.7.2 | ✅ 배포 완료 |
| mecab-ko-dict-sync | v0.7.2 | v0.7.2 | ✅ 배포 완료 |
| mecab-ko | v0.7.2 | v0.7.2 | ✅ 배포 완료 |
