# PROGRESS — mecab-ko Sprint 170 (B-1 영역 종합 + 정리 sprint)

> 마지막 업데이트: 2026-05-27

## Sprint 170 — B-1 바인딩 영역 종합 진단

| Task | 상태 | 결과 |
|------|------|------|
| S170-1: NIKL Modu 6번째 체크 | ⏸ 미다운로드 | 변동 없음 |
| S170-2: Python 바인딩 테스트 인벤토리 | ✅ 완료 | **30+ 테스트 함수 (매우 풍부)** |
| S170-3: Node 바인딩 테스트 인벤토리 | ✅ 완료 | **31 it() 테스트** |
| S170-4: WASM 바인딩 (Sprint 169에서 처리) | ✅ 완료 | 11 테스트 |
| S170-5: B-1 영역 종합 평가 | ✅ 완료 | **영역 충분히 검증됨** |

## 바인딩 테스트 종합

| 바인딩 | 테스트 함수 | 구조 |
|--------|----------|------|
| **Python** | **30+** (TestMecab, TestEdgeCases, TestPOSFiltering, TestConsistency, TestOutputFormats, TestMemoryAndPerformance, konlpy_compat) | pytest classes |
| **Node** | **31** (constructor, tokenize, morphs, nouns, pos, parse, getVersion, Thread safety, Edge cases, Performance) | vitest describe/it |
| **WASM** | **11** (Sprint 169 확장 후) | wasm_bindgen_test |

**총 72+ 테스트** 3 바인딩 합산.

## Python tests 세부

`test_mecab.py` (158줄): 기본 API (creation, morphs, nouns, pos, parse, wakati, empty/english/mixed/special/numbers, custom dict, module metadata, konlpy 호환)

`test_advanced.py` (292줄): 고급 (unicode, long text, repeated chars, whitespace, punctuation, mixed scripts, emoji, special Korean, POS filtering nouns/verbs/adjectives, consistency, output formats, memory/performance)

## Node tests 세부

`index.test.ts` (320줄): constructor (2), tokenize (5), morphs (3), nouns (3), pos (3), parse (6), getVersion (2), Thread safety (1), Edge cases (5), Performance (1)

## B-1 영역 평가

3 바인딩 모두 충분히 검증됨. 추가 테스트의 marginal value 매우 낮음.

남은 잠재 작업:
- 바인딩 간 API 일관성 검증 (동일 입력 → 동일 출력)
- 새 기능 추가 (사용자 요청 시)
- 문서 추가 정리

## 검증

- `cargo test --workspace --exclude mecab-ko-ffi --lib`: 변경 없음 (411 pass)
- 5-gate sample.tsv: 영향 없음 (코드 변경 0)

## 변경 파일

- (코드 변경 없음 — 진단 sprint)
- `PLAN.md`, `PROGRESS.md` 갱신

## Sprint 171+ — 사용자 결정 필요

### 자동 진행 가능한 영역 거의 소진

| 영역 | 상태 |
|------|------|
| 정확도 lift (S122~167) | 종료 |
| WASM tests | Sprint 169 완료 |
| Python/Node tests | 충분 (S170 진단) |
| Docs 정리 | Sprint 160/161/162 완료 |
| CI 정리 | Sprint 149/152 완료 |

### 가능 옵션 (사용자 결정 필요)

1. **B-2 성능 진단** — mecab-ko-profiler 활용 측정 sprint
2. **유지보수 모드** — sprint cycle 공식 종료
3. **외부 입수** — NIKL Modu / Sejong 다운로드 시 재개
4. **사용자 명시 신규 영역** — 새 기능 등
