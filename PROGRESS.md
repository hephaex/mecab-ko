# PROGRESS — mecab-ko Sprint 147 (VV/XSV practical 동치)

> 마지막 업데이트: 2026-05-20

## Sprint 147 A — XSV practical 동치 추가

| Task | 상태 | 비고 |
|------|------|------|
| S147-A1: XSV+EP mecab/gold 확인 | ✅ 완료 | mecab은 "했/VV+EP", gold는 "하/XSV + 였/EP" — POS scheme 차이 |
| S147-A2: XSV+EC mecab/gold 확인 | ✅ 완료 | 같은 패턴 — "해/VV+EC" vs "하/XSV + 여/EC" |
| S147-A3: PRACTICAL group에 XSV 추가 | ✅ 완료 | `VA/VV/XSV` 동치, 단위 테스트 1개 |
| S147-A4: 5-gate + lift 측정 | ✅ 완료 | 3 silver 모두 practical morph +0.2~0.4pp |

## 핵심 발견

### POS scheme 차이 → 단순 분리 불가

mecab CLI 직접 확인:
- "했" → mecab: `VV+EP` (1 token) vs gold: `하/XSV + 였/EP` (2 tokens)
- "됐" → mecab: `VV+EP` vs gold: `되/XSV + 었/EP`
- "해" → mecab: `VV+EC` vs gold: `하/XSV + 여/EC`

→ mecab "하" = VV, gold "하" = XSV. **POS 분류 convention 차이** (surface 분리 단위는 같음).
→ surface 분리 시도 시 POS 부정확. **practical 동치만 적절**.

### PRACTICAL 그룹 확장 (Sprint 147 A)

```rust
&["VA", "VV", "XSV"]  // 기존 VA/VV에 XSV 추가
```

언어학적 정당성:
- VA/VV (Sprint 136): "있다" 형용사/동사 분류 논쟁
- VV/XSV (Sprint 147): "하/되" 본동사/접사 분류 논쟁
- 모두 한국어 문법 진행 중 convention 차이

## 측정 결과 (3 silver 모두 lift)

| Metric | Before | After | Δ |
|--------|--------|-------|---|
| sample.tsv | 100.0%/99.9% | 동일 | 무회귀 |
| KLUE morph strict | 66.8% | 66.8% | — (strict 미영향) |
| **KLUE morph practical** | 71.6% | **71.9%** | **+0.3pp** |
| KLUE eo practical | 5262건 | **5281건** | +19 |
| **UD Kaist morph practical** | 68.1% | **68.3%** | **+0.2pp** |
| UD Kaist eo practical | 4193건 | **4200건** | +7 |
| **UD GSD morph practical** | 71.3% | **71.7%** | **+0.4pp** |
| UD GSD eo practical | 2907건 | **2918건** | +11 |
| Surface-only | 동일 | 동일 | — |

### 일관된 도메인 lift = 진짜 효과

3 silver 모두 +0.2~0.4pp morph + 일관된 eojeol +건수 → 도메인 독립적, 진짜 convention 차이 흡수.

## 핵심 학습 포인트

### 1. 분석 빈도 → mecab CLI → 적합한 접근 식별

Sprint 145 분석 "XSV+EP 413건"은 raw feature. mecab CLI 확인 후 실제 출력 "VV+EP"임을 발견. surface 분리 → practical 동치 전환.

### 2. POS scheme 차이는 practical 동치로 처리

mecab/gold 분류 차이는 conventional disagreement. surface 분리 단위는 같으나 분류만 다름 → split 불가, lenient만 적절.

### 3. Conservative vs Practical 분리 가치

Conservative 변경 없음 — 정밀 평가 보존. Practical만 lift → trade-off 명확.

### 4. 일관된 도메인 lift = 신뢰도 높음

3 silver 모두 +0.2~0.4pp → 단일 anomaly 아님, 진짜 효과.

## 검증

- `cargo test --workspace --exclude mecab-ko-ffi --lib`: **399 passed / 0 failed** (398 + 1 신규)
- `cargo clippy --workspace --all-targets -- -D warnings`: clean
- `test_pos_tags_equivalent_practical_includes_xsv`: PASS
- `test_full_accuracy_evaluation`: PASS (sample.tsv 100.0%/99.9%)
- KLUE/UD: 3 silver 모두 practical lift

## 변경 파일

- `rust/crates/mecab-ko-core/src/evaluate.rs`:
  - `TAG_EQUIVALENCE_GROUPS_PRACTICAL`에 XSV 추가
  - `test_pos_tags_equivalent_practical_includes_xsv` 신규
- `docs/research/accuracy/2026-05-20_sprint147_xsv_practical_equivalence.md` (신규)
- `PLAN.md`, `PROGRESS.md` 갱신

## Sprint 148 후보

- A: VV+EP 명시 동사 분리 (542건)
- B [메인]: Full CRF Retrain (Track E)
- C: NIKL Modu 수동 다운로드
- D: ETM+ETM "라는" 조사
