# PROGRESS — mecab-ko Sprint 142 (dict-builder CSV 버그 수정)

> 마지막 업데이트: 2026-05-20

## Sprint 142 B — dict-builder CSV unquoted comma surface 수정

| Task | 상태 | 비고 |
|------|------|------|
| S142-B1: CSV 파싱 코드 분석 | ✅ 완료 | entries.csv 4행 `,,1792,...` (unquoted comma surface) |
| S142-B2: CSV escape/quote 처리 수정 | ✅ 완료 | record.len()==13 + record[0]/[1] empty 보정 로직 |
| S142-B3: dict-builder round-trip 검증 | ✅ 완료 | 1,632,572 entries, 77초, 4-gate 무회귀 |
| S142-B4: 단위 테스트 + 보고서 | ✅ 완료 | 2 신규 테스트 + 보고서 |

## 핵심 발견

### Sprint 138 차단 원인 해결

Sprint 138에서 dict-builder 사용 시 "Invalid left_id at line 4" 에러 → 단일 행 (entries.csv:4) 의 surface=","가 unquoted 작성.

```
,,1792,3558,788,SC,*,*,*,*,*,*,*
```

csv 라이브러리(RFC 4180)가 13 fields로 분할 → record[0]="" (empty) → parse fail.

### 수정 (record count 보정)

```rust
let (surface, field_offset) = if record.len() == 13
    && record[0].is_empty()
    && record[1].is_empty()
{
    (",".to_string(), 1)  // surface=",", 나머지 +1 shift
} else {
    (record[0].to_string(), 0)
};
```

### Round-trip 검증

```
$ cargo run --release --bin mecab-ko-dict-builder -- \
    build -i data/mecab-ko-dic-2.1.1-20180720 -o /tmp/dict-test-rebuild

=== Build Summary ===
Entries:      1632572
Trie size:    34470912 bytes
Matrix size:  10292646 entries
Time elapsed: 77.83s
Dictionary build successful!
```

재빌드 dict로 4-gate 측정 → 모든 메트릭 동일 (회귀 0):
- sample.tsv: 100.0%/99.9%
- KLUE morph strict 66.8% / practical 71.6%
- Surface-only canonical_lenient 95.5%
- UD Kaist morph strict 66.4%

## 단위 테스트 추가 (2개)

- `test_csv_parser_unquoted_comma_surface`: 12 → 13 fields 복원 검증
- `test_csv_parser_quoted_comma_surface_still_works`: Symbol.csv 정상 파싱 유지

## 측정값 (변경 없음 — 인프라 fix만)

| 메트릭 | Sprint 141 | Sprint 142 |
|--------|-----------|-----------|
| 모든 KLUE/UD/sample.tsv 메트릭 | 동일 | 동일 |

## 핵심 학습 포인트

### 1. mecab-ko-dic은 비표준 CSV 가정

mecab-ko-dic은 일본 mecab 도구의 형식을 따름 — unquoted comma surface 허용. RFC 4180 CSV 파서는 1행만 fail하지만 dict-builder 전체 실패 야기. record count 기반 보정으로 우회.

### 2. 작은 버그가 큰 인프라 막음

이 1행 버그가 Sprint 138 Tier A 실험 인프라를 방해. 이제 정상 빌드 가능 → **Track E (Full CRF Retrain) 진입 가능**.

### 3. Round-trip 검증의 가치

단위 테스트만으로는 부족. 전체 mecab-ko-dic 재빌드 + 4-gate 회귀 검증으로 binary 형식 정합성 확인 필수.

## 검증

- `cargo test --workspace --exclude mecab-ko-ffi --lib`: all pass / 0 fail (테스트 +2개)
- `cargo clippy --workspace --all-targets -- -D warnings`: clean
- dict-builder 전체 round-trip: 1.63M entries, 77s, 4-gate 무회귀
- `test_csv_parser_unquoted_comma_surface` / `_quoted_comma_surface_still_works`: PASS

## 변경 파일

- `rust/crates/mecab-ko-dict-builder/src/lib.rs`: `parse_csv_content` (record count 보정 로직) + 2 신규 단위 테스트
- `docs/research/accuracy/2026-05-20_sprint142_dict_builder_csv_fix.md` (신규)
- `PLAN.md`, `PROGRESS.md` 갱신

## Sprint 143 후보

- B [메인]: **Full CRF Retrain (Track E)** — 이제 진입 가능
- A: 다른 mecab 결합 토큰 패턴 (Sprint 141 연장)
- C: UD Korean-GSD 통합
- D: NIKL Modu 추가
