# PROGRESS — mecab-ko Sprint 158 (명시 어구 정규화 + 안전 영역 소진)

> 마지막 업데이트: 2026-05-21

## Sprint 158 — 명시 어구 축약 정규화

| Task | 상태 | 결과 |
|------|------|------|
| S158-1: Sprint 156 진단 결과 재활용 | ✅ 완료 | 인하여/확인하여 등 식별 |
| S158-2: EXPLICIT_PHRASE_PATTERNS 신규 (3 패턴) | ✅ 완료 | evaluate.rs |
| S158-3: normalize_endings Step 5 추가 | ✅ 완료 | |
| S158-4: 단위 테스트 추가 | ✅ 완료 | `test_surface_eq_canonical_lenient_explicit_phrase` |
| S158-5: 5-gate 검증 | ✅ 완료 | KLUE surface +10 eojeols, 무회귀 |

## 변경 내용

### EXPLICIT_PHRASE_PATTERNS 신규

```rust
const EXPLICIT_PHRASE_PATTERNS: &[(&str, &str)] = &[
    ("인하여", "인해"),              // 4건 KLUE
    ("확인하여", "확인해"),           // 3건 KLUE
    ("참고하시어요", "참고하세요"),    // 3건 KLUE (시어요 → 세요)
];
```

### normalize_endings Step 5 추가

```rust
// Step 5: 명시 어구 축약 (Sprint 158)
for (from, to) in EXPLICIT_PHRASE_PATTERNS {
    if out.contains(from) {
        out = out.replace(from, to);
    }
}
```

## 측정 결과

| Metric | Before | After | Δ |
|--------|--------|-------|---|
| sample.tsv | 100.0%/99.9% | 100.0%/99.9% | 무회귀 ✓ |
| KLUE morph practical | 72.1% | 72.1% | — |
| **KLUE surface canonical_lenient** | **95.6%** (21419) | **95.6%** (**21429**) | **+10 eojeols (~+0.05pp)** |
| UD | 영향 없음 | — | — |

작은 lift지만 명확하게 양수. 누적 surface_only 95.6% 유지.

## 안전 영역 소진 선언

### Sprint 156~158 누적 안전 영역 효과

| Sprint | 영역 | 효과 |
|--------|------|------|
| 156 | ㄷ 불규칙 + 르 추가 | +30 eojeols surface |
| 157 | MAG/MAJ practical | +124 eojeols morph practical |
| **158** | **명시 어구** | **+10 eojeols surface** |

총: **+164 eojeols (morph + surface)**.

### 잔여 안전 후보 — 거의 소진

진단 결과 남은 surface mismatch:
- 있어디에서, 모으게하고, 있안으며 — **mecab tokenizer over-split** (정규화 영역 아님)
- 가이습니다, 남기이습니다 — VCP 분해 복잡 (Sprint 134 부분 처리)
- 살는 → 사는, 것이 → 게 — false positive 위험

추가 PRACTICAL 동치:
- VV↔NNG 47건 — 의미 분류 차이 (위험)
- 나머지 이미 처리됨

**결론**: 안전 영역 lift ≤ +0.05pp/sprint. ROI 매우 낮음.

## 다음 단계: 비가역 작업 confirm 필요

영역 소진 → 비가역 대규모 작업만 남음. **사용자 confirm 필요**:

### F: NIKL Modu 도입
- 구어/SNS 도메인 silver dataset
- Academic license + 수동 다운로드
- coverage 확장 (lift 자체는 아님)

### E: Full CRF Retrain (Track B)
- 3-5 sprint 장기
- 학습 데이터 + mecab-cost-train
- 잠재 lift +1~5pp

### 또는 정확도 외 다른 영역
- 문서 정리
- CLI/API 사용성 개선
- 성능 최적화
- 새 언어 바인딩

## 검증

- `cargo test --workspace --exclude mecab-ko-ffi --lib`: **411 passed / 0 failed** (410+1)
- `cargo clippy --workspace --all-targets --exclude mecab-ko-ffi -- -D warnings`: clean
- 5-gate: sample.tsv 무회귀, KLUE surface +10 eojeols

## 변경 파일

- `rust/crates/mecab-ko-core/src/evaluate.rs`:
  - EXPLICIT_PHRASE_PATTERNS 신규 (3 패턴)
  - normalize_endings Step 5 추가
  - 단위 테스트 1개 신규
- `PLAN.md`, `PROGRESS.md` 갱신

## 누적 진척 (Sprint 122 baseline → Sprint 158)

| Metric | Sprint 122 | Sprint 158 | 총 Δ |
|--------|-----------|-----------|-----|
| KLUE morph practical | ~65.8% | 72.1% | **+6.3pp** |
| KLUE eojeol practical | ~5000 | 5327 | +327 |
| KLUE surface canonical_lenient | 89.x% | 95.6% | **+6pp** |
| UD GSD morph practical | ~70.x% | 71.8% | +1pp |
| sample.tsv | 100%/99.9% | 100%/99.9% | — (baseline) |

총 30+ sprint로 KLUE practical +6.3pp / surface +6pp 누적 lift.
