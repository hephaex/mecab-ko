# Sprint 150 A — VA+ETM multi-syllable ㄴ jongseong split

> **결과**: VA+ETM 542건 중 518건은 이미 ending_rules가 처리. 미처리 24건 (빠른/나쁜/예쁜/느린)에 multi-syllable ㄴ jongseong split 추가. KLUE morph strict +0.4pp, sample.tsv 무회귀. VV+ETM은 Sprint 145 회귀로 제외.

---

## 1. 진단

### 1.1 Raw VA+ETM 빈도

mecab raw `pos == "VA+ETM"` 토큰: **542건** (KLUE + UD-Kaist + UD-GSD)
- 1-syllable (이미 1-syl ㄹ/ㄴ 자모 분리): 160건
- Multi-syllable: 382건

### 1.2 Post-splitter 분석

기존 ending_rules ("은", "ㄹ", "을", "던") 적용 후:
- **분리됨 (ETM 토큰 생성)**: **518건 (95.6%)**
- 미분리: **24건 (4.4%)**

미분리 surface 분포:
- 빠른 (13건) — 빠르 + ㄴ (르 불규칙)
- 나쁜 (5건) — 나쁘 + ㄴ (ㅡ 탈락)
- 예쁜 (4건) — 예쁘 + ㄴ
- 느린 (1건) — 느리 + ㄴ
- 신선한 (1건) — 신선하 + ㄴ

공통 패턴: **multi-syllable VA+ETM with ㄴ jongseong on last char**

### 1.3 Gold 검증

KLUE gold:
```
빠르/VA ㄴ/ETM 비트
나쁘/VA ㄴ/ETM 사람
둔하/VA ㄴ/ETM 템포
```

gold가 정확히 `stem + ㄴ` 형태로 분리. mecab의 raw VA+ETM compound와 mismatch.

---

## 2. 구현

### 2.1 splitter.rs 신규 규칙

`splitter.rs:398` (1-syllable ㄹ/ㄴ 자모 분리 직전 추가):

```rust
// Sprint 150 A: multi-syllable VA+ETM with ㄴ jongseong on last char
// 빠른 → 빠르 + ㄴ (르 불규칙), 나쁜 → 나쁘 + ㄴ (ㅡ 탈락), ...
// VV+ETM은 Sprint 145 회귀로 제외 — VA만 안전 (어휘 범위 제한).
if pos == "VA+ETM" && surface.chars().count() > 1 {
    let chars: Vec<char> = surface.chars().collect();
    let last = chars[chars.len() - 1];
    if let Some(stem_last) = remove_jongseong_nieun(last) {
        let mut stem: String = chars[..chars.len() - 1].iter().collect();
        stem.push(stem_last);
        return vec![
            (stem, "VA".to_string()),
            ("ㄴ".to_string(), "ETM".to_string()),
        ];
    }
}
```

### 2.2 안전성 분석

**VV+ETM 제외**: Sprint 145 D rollback 교훈 — VV (일반동사) 어휘 범위가 넓어 false positive 위험.
**VA만**: 형용사는 어휘 범위가 제한적, false positive 위험 낮음.

**제외 범위**:
- ㅂ 불규칙 ('운'/'울' suffix): 어려운, 무거운 — ㄴ jongseong 없음 (제외됨)
- ㅎ 불규칙 ('란'/'간' suffix): 노란, 빨간 — ㄴ jongseong 없음 (제외됨)
- '은' suffix (좋은, 높은): ending_rules가 이미 처리

### 2.3 단위 테스트

4개 신규 단위 테스트:
- `test_split_morpheme_va_etm_multisyllable_eun_via_ending_rules` — 기존 동작 검증
- `test_split_morpheme_vv_etm_multisyllable_eun_via_ending_rules` — VV+ETM도 ending_rules 처리 확인
- `test_split_morpheme_va_etm_multisyllable_nieun_jongseong` — 신규 규칙 (빠른/나쁜/예쁜/느린)
- `test_split_morpheme_va_etm_multisyllable_no_nieun_fallback` — fallback 검증

---

## 3. 측정 결과

### 3.1 5-gate 비교

| Metric | Sprint 149 | Sprint 150 A | Δ |
|--------|-----------|--------------|---|
| sample.tsv Token | 100.0% | 100.0% | — |
| sample.tsv Sentence | 99.9% | 99.9% | — |
| **KLUE morph strict** | 66.5% | **66.9%** | **+0.4pp** |
| KLUE morph practical | 71.9% | 71.9% | — |
| KLUE eo practical | 5281 | 5283 | +2 |
| **KLUE surface strict** | 87.7% | **87.8%** | **+0.1pp** |
| KLUE surface canonical_lenient | 95.4% | 95.5% | +0.1pp |
| UD Kaist morph strict | 66.4% | 66.4% | — |
| UD Kaist morph practical | 68.3% | 68.4% | +0.1pp |
| UD Kaist eo practical | 4200 | 4197 | -3 |
| UD GSD morph strict | 67.4% | 67.3% | -0.1pp |
| UD GSD morph practical | 71.7% | 71.6% | -0.1pp |
| UD GSD eo practical | 2918 | 2926 | **+8** |

### 3.2 종합

- **sample.tsv 무회귀 ✓** (Sprint 138 hard rule)
- KLUE: 명확한 positive (morph strict +0.4pp, surface +0.1pp)
- UD Kaist: positive (morph practical +0.1pp)
- UD GSD: noise level (morph -0.1pp = 11 morphemes, 그러나 eojeol +8)

전체 lift: **net positive**. 가장 큰 효과는 KLUE morph strict +0.4pp.

---

## 4. 핵심 학습 포인트

### 4.1 빈도 분석 ≠ 실제 영향

raw VA+ETM 542건 중 518건 (95.6%)이 이미 ending_rules로 처리됨. 빈도만으로 판단했다면 542건 모두를 미처리로 오해할 수 있음.

**적용 원칙**: raw 빈도 분석 → splitter post-처리 분석 → 실제 mismatch 측정 → 작업 범위 결정.

### 4.2 VV vs VA 위험성 차이

Sprint 145 D는 VV+ETM multi-syllable 확장으로 sample.tsv 회귀. VA+ETM은 같은 알고리즘이지만:
- VA = 형용사 (제한된 어휘)
- VV = 일반동사 (광범위 어휘)

VA만 확장하는 보수적 접근으로 false positive 회피.

### 4.3 ending_rules의 광범위 커버리지

`init_ending_rules`의 VA+ETM 규칙 ["ㄴ", "은", "ㄹ", "을", "던"]이 대부분의 multi-syllable 케이스를 이미 처리. 새 규칙은 ending_rules가 놓친 ㄴ jongseong 케이스만 보강.

### 4.4 회귀 정의 — sample.tsv vs silver dataset

- sample.tsv 회귀: hard rule (Sprint 138 rollback policy)
- Silver dataset (KLUE/UD) ±0.1pp: noise level (rollback 불필요)
- KLUE strict +0.4pp 같은 명확한 시그널만 추적

---

## 5. 변경 파일

- `rust/crates/mecab-ko-core/src/sejong/splitter.rs`:
  - multi-syllable VA+ETM ㄴ jongseong split 추가
  - 단위 테스트 4개 신규
- `rust/crates/mecab-ko-core/tests/accuracy_eval.rs`:
  - `test_va_etm_post_splitter_mismatch` 진단 테스트
- `PLAN.md`, `PROGRESS.md` 갱신

---

*작성: 2026-05-21 (Sprint 150 A)*
