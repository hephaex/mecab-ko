# PROGRESS — mecab-ko Sprint 169 (B-1: WASM 바인딩 테스트 확장)

> 마지막 업데이트: 2026-05-27

## Sprint 169 — B-1: 언어 바인딩 강화 (WASM 우선)

| Task | 상태 | 결과 |
|------|------|------|
| S169-1: NIKL Modu 5번째 체크 | ⏸ 미다운로드 | 사용자 결정 → B-1 |
| S169-2: 3 바인딩 인벤토리 진단 | ✅ 완료 | Python 성숙, Node 320줄 테스트, **WASM tests 0개** |
| S169-3: WASM 현재 테스트 분석 | ✅ 완료 | 5개 단위 테스트 (기본만) |
| S169-4: WASM tests 6개 신규 추가 | ✅ 완료 | nouns/wakati/empty/positions/optional/json_format |
| S169-5: 빌드 검증 (wasm32 target) | ✅ 완료 | cargo check 정상 |

## 바인딩 인벤토리

| 바인딩 | 소스 | 테스트 | 비고 |
|--------|------|--------|------|
| mecab-ko-python | lib.rs | tests/ 3개 | 가장 성숙 (BND-003 문서, examples, validate 스크립트) |
| mecab-ko-node | lib.rs | index.test.ts 320줄 | vitest, pre-built binary 포함 |
| **mecab-ko-wasm** | **lib.rs** | **tests/ 0개, inline 5개** | **테스트 가장 부족 → Sprint 169 우선** |

## 변경 내용

### WASM 테스트 6개 신규 (lib.rs:323+)

```rust
#[wasm_bindgen_test] fn test_nouns_extraction()       // nouns() API
#[wasm_bindgen_test] fn test_wakati_split()           // wakati() API
#[wasm_bindgen_test] fn test_empty_input()            // edge case
#[wasm_bindgen_test] fn test_token_positions()        // start/end
#[wasm_bindgen_test] fn test_token_optional_fields()  // reading/lemma Option
#[wasm_bindgen_test] fn test_pos_json_array_format()  // pos() JSON
```

기존 5개 + 6 신규 = **11개 WASM 테스트** (+120%).

### 커버리지 영역

- ✅ Mecab::new() (creation)
- ✅ morphs() (기존)
- ✅ tokenize() (기존)
- ✅ pos() (기존 + 신규 JSON format)
- ✅ nouns() (신규)
- ✅ wakati() (신규)
- ✅ Token reading/lemma (신규)
- ✅ Token positions (신규)
- ✅ Empty input edge case (신규)
- ✅ Token JSON serialization (기존)

## 검증

- `cargo check --target wasm32-unknown-unknown`: clean (16.6초)
- WASM 빌드 정상 (wasm-bindgen 인식)
- 다른 바인딩 영향 없음

## 변경 파일

- `rust/crates/mecab-ko-wasm/src/lib.rs`: 6개 wasm_bindgen_test 추가
- `PLAN.md`, `PROGRESS.md` 갱신

## Sprint 170 후보

### B-1 계속 (다른 바인딩)

- **Node**: 이미 320줄 테스트 — 보완 영역 식별 필요
- **Python**: tests/ 3개 — 확장 가능

### 또는 다른 영역

- B-2: 성능 진단 sprint
- 유지보수 모드
- NIKL Modu / Sejong 입수 시 정확도 재개
