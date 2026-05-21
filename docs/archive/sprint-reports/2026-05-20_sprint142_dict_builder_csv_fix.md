# Sprint 142 B — dict-builder CSV unquoted comma surface 수정

> **결과**: Sprint 138에서 발견된 "Invalid left_id at line 4" 에러 해결. dict-builder 전체 mecab-ko-dic 재빌드 round-trip 성공 (1.63M entries, 77초). 4-gate 무회귀. Track E (Full CRF retrain) 진입 가능.

---

## 1. 버그 분석

### 1.1 원인

`mecab-ko-dic-2.1.1-20180720/entries.csv` 4행:

```
,,1792,3558,788,SC,*,*,*,*,*,*,*
```

- surface가 `,` (쉼표) 그 자체
- mecab-ko-dic 원본은 **unquoted**로 작성 (`,,` 두 쉼표)
- csv 라이브러리는 표준 RFC 4180 quoted CSV 가정 → 13 fields로 분할:
  - `record[0]` = "" (empty), `record[1]` = "" (empty), `record[2]` = "1792", ...
- 기존 dict-builder: `surface = record[0]` (empty), `left_id = record[1].parse()` (fail) → "Invalid left_id at line 4"

다른 CSV는 properly quoted (`Symbol.csv`: `","`, `1792,...`):
- csv 라이브러리가 정상 파싱 → 12 fields

따라서 **entries.csv 단일 행만 문제**.

### 1.2 전체 mecab-ko-dic 검색

```bash
grep -E "^,," data/mecab-ko-dic-2.1.1-20180720/*.csv
```

결과: 1행만 매칭 (`entries.csv:,,1792,...`). 다른 unquoted edge case 없음.

---

## 2. 수정

### `src/lib.rs:328-352` — record count 기반 surface 복원

```rust
// Sprint 142 B: mecab-ko-dic entries.csv는 surface가 ","인 항목을
// unquoted로 작성. csv 파서가 13 fields로 분할 → record[0]/[1] empty.
// 대응: record.len() == 13 AND record[0]/record[1] 모두 empty이면
// surface=",", 나머지 field shift.
let (surface, field_offset) = if record.len() == 13
    && record[0].is_empty()
    && record[1].is_empty()
{
    (",".to_string(), 1)
} else {
    (record[0].to_string(), 0)
};

// 나머지 필드는 `field_offset` 보정해서 접근
left_id: record[1 + field_offset].parse()...
right_id: record[2 + field_offset].parse()...
...
```

### 안전성

- **명시 조건**: `record.len() == 13` 그리고 `record[0]`과 `record[1]` 모두 empty여야 surface="," 복원
- 다른 13-field CSV (있다면)는 영향 없음 (보통 record[0]는 비어있지 않음)
- 기존 12-field 케이스는 변화 없음 (`field_offset = 0`)
- Quoted comma surface (Symbol.csv `","`)는 12 fields로 정상 파싱

---

## 3. 검증

### 3.1 단위 테스트 (5개, 2개 신규)

| 테스트 | 결과 |
|--------|------|
| `test_csv_parser_basic` | ✅ |
| `test_csv_parser_encoding` | ✅ |
| `test_csv_parser_with_comments` | ✅ |
| **`test_csv_parser_unquoted_comma_surface`** (신규) | ✅ |
| **`test_csv_parser_quoted_comma_surface_still_works`** (신규) | ✅ |

### 3.2 dict-builder Full Round-trip

```bash
cargo run --release --bin mecab-ko-dict-builder -- \
  build -i data/mecab-ko-dic-2.1.1-20180720 \
        -o /tmp/dict-test-rebuild
```

**결과**:
- 빌드 성공 (이전 Sprint 138에서는 fail)
- 1,632,572 entries 파싱
- Trie size: 34MB, Matrix size: 10M entries
- 소요 시간: 77.83s

### 3.3 재빌드된 dict로 4-gate 회귀 검증

`MECAB_DIC_PATH=/tmp/dict-test-rebuild` 로 모든 평가 테스트 실행:

| Gate | Before (원본) | After (재빌드) | Δ |
|------|--------------|----------------|---|
| sample.tsv Token | 100.0% | 100.0% | — |
| sample.tsv Sentence | 99.9% | 99.9% | — |
| KLUE morph strict | 66.8% | 66.8% | — |
| KLUE eo strict | 20.7% | 20.7% | — |
| KLUE morph practical | 71.6% | 71.6% | — |
| KLUE eo practical | 23.5% | 23.5% | — |
| Surface-only strict | 87.7% | 87.7% | — |
| Surface-only canonical | 91.6% | 91.6% | — |
| Surface-only canonical_lenient | 95.5% | 95.5% | — |
| UD Kaist morph strict | 66.4% | 66.4% | — |
| UD Kaist morph practical | 68.1% | 68.1% | — |

**전 메트릭 동일 — 완벽한 round-trip 달성**.

---

## 4. 핵심 학습 포인트

### 4.1 mecab-ko-dic은 비표준 CSV 가정

mecab-ko-dic은 일본 mecab 도구로 처리되는 형식을 따름. 이 도구는 unquoted comma surface도 처리 (line의 첫 N field를 surface로 인식). 표준 RFC 4180 CSV 파서는 이를 못 다룸 — surface가 "," 인 1행에 한정된 문제이지만 **dict-builder 전체 실패** 야기.

**적용 원칙**: mecab-ko-dic 같은 legacy 도구 데이터 처리 시 RFC 4180 가정 깨는 edge case 가능. record count 기반 보정으로 우회.

### 4.2 작은 버그가 큰 인프라 막음

이 1행 버그가 Sprint 138 Tier A matrix 실험에서 dict-builder 우회 도구(`matrix_def_to_bin`)를 별도 작성하게 함. 이제 정상 빌드 가능 → **Track E (Full CRF Retrain) 진입 가능**.

### 4.3 Round-trip 검증의 가치

수정 후 단순 unit test 통과만으로는 부족. **전체 mecab-ko-dic 재빌드 + 4-gate 회귀 검증**으로 binary 형식의 정합성까지 확인. 모든 정확도 메트릭 동일 → 수정이 다른 곳에 영향 없음 보장.

---

## 5. Sprint 143 후보

### 후보 A: 다른 mecab 결합 토큰 패턴 (Sprint 141 연장)
NNG+VCP+EC, VV+EP+EF 등 추가 splitter 패턴.

### 후보 B [메인]: Full CRF Retrain (Track E, 이제 진입 가능)

**선행 조건 충족**: Sprint 142 B로 dict-builder 정상 작동.

**작업 단계** (3-5 sprint 예상):
1. 학습 코퍼스 준비 (Sejong + KLUE train + UD Kaist train)
2. `legacy/src/mecab-cost-train` (C++) 빌드 + 실행
3. 새 `model.def` → `matrix.def` + `left/right-id.def` 재생성
4. `cargo run --bin mecab-ko-dict-builder` 재실행 → binary
5. 4-gate 회귀 검증 + lift 측정

**리스크**:
- 학습 코퍼스 라이선스 (Sejong 비공개 / KLUE+UD CC BY-SA)
- left/right-id.def 변경 시 기존 binary 호환성 깨짐
- 학습 시간 (수 시간)

### 후보 C: UD Korean-GSD 통합
변환기 재사용, 0.5 sprint, 위험 낮음.

### 후보 D: NIKL Modu 추가
academic license, manual download, 0.5-1 sprint.

---

## 6. 변경 파일

- `rust/crates/mecab-ko-dict-builder/src/lib.rs`:
  - `parse_csv_content` (line ~320): record count 기반 surface 복원 로직 추가
  - `test_csv_parser_unquoted_comma_surface` / `_quoted_comma_surface_still_works` 신규 단위 테스트
- `docs/research/accuracy/2026-05-20_sprint142_dict_builder_csv_fix.md` (본 문서)
- `PLAN.md`, `PROGRESS.md` 갱신

---

*작성: 2026-05-20 (Sprint 142 B)*
