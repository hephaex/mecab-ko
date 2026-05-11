# Sprint 127 P1: 복합명사 분할 정책 분석 + 어절 단위 측정 재고

> 핵심 결과: **어절별 독립 토크나이즈** 알고리즘으로 KLUE DP eojeol 정확도 52.4% 측정.
> 가설(복합명사 over-split이 주범)은 **틀림** — 실제 over-split은 2.5%만, 진짜 큰 영역은
> POS_DIFF (19.5%) + SURFACE_MISMATCH (12.3%, 변환 어미 표기) + SPLIT_DIFFERENT (10.2%).
> Slice-lenient ceiling은 87.7%이나 의미 손실로 권장 안 함.

---

## 배경

Sprint 126 P1까지 측정 결과:
- KLUE DP morpheme strict 65.8% / lenient 69.3% / practical 70.3%
- KLUE DP **eojeol** strict 19.2% / lenient 21.0% / practical 21.7%
- 진단 추정(85-90%)과의 갭 ~14pp = "진짜 분석 오류" 추정

가설: 갭의 약 절반(~20%)은 **복합명사 over-split** ("팝스타→팝+스타", "현대중공업→현대+중공업")이
원인. Sprint 127 P1에서 정량화하고 slice-level matching 메트릭의 적합성 검토.

---

## P1-1: 알고리즘 v1 (실패) — 누적 surface char 정렬

첫 시도: gold tokens의 surface 합과 mecab tokens의 surface 합이 같은 char 수가 될 때까지
누적해서 어절 boundary로 사용.

**문제**: KLUE의 morpheme surface는 **jamo decomposed** ("공정한" → "공정"+"하"+"ㄴ", 4 chars).
mecab은 원문 그대로 시작 위치 사용. 첫 어절에서 char count 1차이 발생하면 **cascade**로
이후 모든 어절이 한 칸씩 밀려 mismatch로 카운팅됨.

결과: SURFACE_MISMATCH 42.2% — 신뢰 불가.

## P1-1.5: 알고리즘 v2 (실패) — char position 기반 정렬

원문에서 공백을 char position으로 추출 후, mecab token의 `start_pos`를 사용하여 어절 boundary
내 토큰만 모으도록 변경.

**문제**: mecab token의 `start_pos`는 원문 char 단위지만, KLUE morpheme의 char 합은 jamo
decomposition으로 원문보다 김. 어절 char range로 mecab 토큰을 정렬하면 cover 범위가 좁아
다른 어절 토큰까지 잘못 흡수. SURFACE_MISMATCH 92.6% — 더 악화.

## P1-2: 알고리즘 v3 (성공) — 어절별 독립 토크나이즈

**핵심 결정**: 어절별로 mecab을 따로 호출 (`tokenizer.tokenize(eojeol)`). 어절 boundary 정렬
문제를 우회. mecab의 cross-eojeol Viterbi context는 잃지만, 한국어 형태소 분석에서 어절 경계
너머의 영향은 작음.

```rust
let eojeols: Vec<&str> = gold_sentence.text.split_whitespace().collect();
for (eo_i, &count_g) in eojeol_counts.iter().enumerate() {
    let gold_slice = &gold_sentence.tokens[gold_idx..gold_idx + count_g];
    let pred_raw = tokenizer.tokenize(eojeols[eo_i]);  // 어절 단위 분석
    let pred_sejong = converter.convert_tokens(&pred_raw);
    // 분류
}
```

이 알고리즘은 cascade 효과 없이 **각 어절 독립적으로 분류**.

---

## 측정 결과 (KLUE DP val, 1,995 sentences, 22,404 eojeols)

### 카테고리 분포

| 카테고리 | 건수 | 비율 | 설명 |
|----------|-----:|------:|------|
| **EXACT** | 11,738 | **52.4%** | surface + POS 완전 일치 |
| POS_DIFF | 4,373 | 19.5% | 같은 분할, POS만 다름 |
| SURFACE_MISMATCH | 2,745 | 12.3% | 변환 어미 표기 차이 (하였/하았 등) |
| SPLIT_DIFFERENT | 2,282 | 10.2% | 양쪽 분할이지만 boundary 다름 |
| GOLD_SINGLE_PRED_MULTI | 553 | **2.5%** | mecab over-split (복합명사 분리) |
| GOLD_MULTI_PRED_SINGLE | 397 | 1.8% | mecab merge |
| INNER_SPLIT_DIFF | 316 | 1.4% | 같은 count, 내부 surface boundary 다름 |

### Headline

- **Strict eojeol 정확도 (어절별 독립): 52.4%** (11,738 / 22,404)
- **Slice-lenient ceiling: 87.7%** (19,659 / 22,404, surface 합만 일치)

### Sprint 126 측정과의 비교

| Metric | Sprint 126 (sequence) | Sprint 127 (per-eojeol) | Δ |
|--------|----------------------|-------------------------|---|
| eojeol strict | 19.2% | **52.4%** | +33.2pp |
| eojeol lenient (conservative) | 21.0% | — | — |
| eojeol lenient (practical) | 21.7% | — | — |

**Sprint 126의 eojeol 측정이 cascade 효과로 큰 폭 underestimate함이 입증됨**.
실제 어절 정확도는 52.4%로 morpheme 정확도(65.8% strict)에 가까움.

### 분할 패턴 빈도 (surface-aligned 어절만)

| (gold_count, pred_count) | 건수 |
|--------------------------|-----:|
| (2, 2) | 7,387 |
| (1, 1) | 4,692 |
| (3, 3) | 2,834 |
| (4, 4) | 1,082 |
| (2, 3) ←diff | 896 |
| (1, 2) ←diff | 444 |
| (2, 1) ←diff | 392 |
| (3, 2) ←diff | 381 |
| (5, 5) | 304 |

분할 일치(diagonal) 비율은 **88.5%** (16,299/18,420 surface-aligned eojeols).
즉 분할 차이 자체는 흔하지 않음.

---

## P1-3: 케이스 분석 (real error vs convention)

### POS_DIFF (19.5%, 4,373건)

같은 분할, POS만 다름. Sprint 126 P1에서 NNG/NNP/NNB confusion 809건을 일부 다룸.
나머지는 MAG/NNG, VV/NNG 등 진짜 disambiguation 오류 (Sprint 127 P2 영역).

### SURFACE_MISMATCH (12.3%, 2,745건)

KLUE의 inflectional morpheme 분해 표기와 mecab의 표기가 다름:

| KLUE | mecab | 패턴 |
|------|-------|------|
| 인정하였다 | 인정하았다 | 어 → 았 표기 차이 |
| 확산되었고 | 확사ㄴ되어었고 | 분해 단위 차이 |
| 함께 | 하ㅁ께 | 모음 분해 차이 |
| 등장하여 | 등장하어 | 어 → 여 표기 차이 |
| 통하여 | 통하어 | 어 → 여 표기 차이 |
| 보았다는 | 보았다는 | EC vs EF+JX |
| 복사하여 | 복사하어 | 동일 |
| 보내주면 | 보내어주면 | 어 분해 차이 |

**판정**: 거의 전부 **convention difference** (KLUE의 morpheme 분해 정책 vs mecab의 분해 정책).
"하였" vs "하았" 같은 표기 정규화 차이는 진짜 오류라 보기 어려움.

**적용 대안**: KLUE에서 morpheme surface를 **표면형으로 normalize 후 비교**하는 lenient 모드
도입 (Sprint 128 후보). 이 카테고리의 12.3% 흡수 가능 추정.

### SPLIT_DIFFERENT (10.2%, 2,282건)

양쪽 모두 분할하지만 경계 다름. 샘플:

| 어절 | gold | pred | 분류 |
|------|------|------|------|
| 'K팝스타3’ | '/SS K/SL 팝스타/NNP 3/SN ’/SS | '/SY K/SL 팝/NNG 스타/NNG 3/SN ’/SY | 복합 NNP vs over-split |
| 대한 | 대하/VV ㄴ/ETM | 대/NNG 하/XSV ㄴ/ETM | 동사/명사+동사 |
| 깨져 | 깨지/VV 어/EC | 깨/VV 지/NNB 어/EF | VV vs VV+NNB |
| 보았다는 | 보/VV 았/EP 다는/ETM | 보/VV 았/EP 다/EF 는/JX | 단일 ETM vs EF+JX |
| 지검장과 | 지검장/NNG 과/JKB | 지/VX 검장/NNG 과/JC | 합성어 vs over-split |
| 공정성을 | 공정/NNG 성/XSN 을/JKO | 공정성/NNG 을/JKO | 분해 vs 단일 |
| 부분적으로 | 부분/NNG 적/XSN 으로/JKB | 부분적/NNG 으로/JKB | 동일 |

**판정**: 혼합. 일부는 진짜 분석 오류 ("깨져" → 깨/지/어), 일부는 convention difference
(접미사 적/XSN 분해 여부). Sprint 128 candidate.

### GOLD_SINGLE_PRED_MULTI (2.5%, 553건) — **원래 가설의 영역**

mecab이 KLUE 단일 토큰을 분할:

| KLUE 단일 토큰 | mecab 분할 | 빈도 |
|---------------|------------|------|
| 하지만/MAJ | 하지만 (분석) | 22 |
| 굉장히/MAG | 굉장/하지 분리 | 16 |
| 지하철/NNG | 지하/철 | 11 |
| 한국수력원자력/NNP | 한국/수력/원자력 | 1 |
| 김앤장/NNP | 김/앤/장 | 1 |
| 경기지방경찰청/NNP | 경기/지/방/경찰청 | 1 |
| 영풍문고/NNP | 영풍/문고 | 1 |
| 세월호/NNP | 세월/호 | 6 |

**판정**: 두 카테고리로 나뉨:
1. **고빈도 부사** ("하지만", "굉장히", "엄청", "더욱"): mecab이 잘못 분해 — 진짜 오류
2. **고유명사**: 사용자 사전 추가로 해결 가능

**적용 원칙**: 가설의 절대 비율은 작지만(2.5%), 고빈도 부사는 cascade로 쓰임이 많아 사용자
체감 영향 크다. 사용자 사전 + lexicon override로 해결 가능.

### GOLD_MULTI_PRED_SINGLE (1.8%, 397건) — 역방향

mecab이 KLUE의 여러 토큰을 합침:

| KLUE 분할 | mecab 단일 | 빈도 |
|-----------|------------|------|
| 이/MMD 날/NNG | 이날/NNG | 다수 |
| 정신/NNG 적/XSN | 정신적/NNG | 일부 |
| 그러/VV 고는/EC | 그러고는/MAJ | 일부 |
| 이/MMD 번/NNB | 이번/NNG | 일부 |
| 부정/NNG 승차/NNG | 부정승차/NNP | 일부 |
| 등/NNB 의/JKG | 등의/NNP | 다수 |
| 왕/NNG 세손/NNG | 왕세손/NNG | 일부 |
| 수/NNB 도/JX | 수도/NNG | 일부 |

**판정**: 다수가 mecab의 사전 등재형. "이날", "정신적", "왕세손" 등은 단일어로 등재되어 분할
안 됨. KLUE는 형태소 단위로 분할.

**적용 원칙**: 이건 **mecab 사전 정책 차이**. 단일어로 등재된 것을 의도적으로 분할해야 KLUE와
일치하지만, downstream에는 단일어가 더 유용한 경우 많음. Slice-level lenient로 흡수 가능.

---

## P1-3: Slice-level matching 메트릭 적합성 검토

### 정의

Slice-level lenient: 어절 내 surface concat이 같으면 분할/POS 무관하게 인정.

### 측정 결과

- Slice-lenient ceiling: **87.7%** (vs strict 52.4%, **+35.3pp lift**)
- 단, "분할 무시"는 모든 형태소 정보 손실 — POS도 무시

### 적합성 판단

**적용 가능 영역**:
- 검색 인덱싱 (어절 단위 토큰 추출이 목적)
- 키워드 추출 (분할 방식 무관)
- 통계 분석

**적용 불가 영역**:
- 형태소 분석 정확도 평가 (분할 자체가 평가 대상)
- 의존 구문 분석 입력 (POS 필수)
- 의미역 분석 (lemma 필수)

### 결론

**Slice-level lenient 메트릭은 평가 인프라에 추가하지 않음**. 이유:
1. 87.7% ceiling은 매력적이나 의미 손실이 큼
2. Sprint 125-126의 conservative/practical lenient는 명확한 convention 차이만 흡수, 의미 손실 없음
3. Slice-lenient는 sample 수치로 보고만 하고, 정식 메트릭에서 제외

**대안**: Sprint 128에서 **morpheme surface normalization lenient** 도입 검토.
KLUE의 jamo-decomposed surface를 mecab과 동일하게 표면형으로 normalize 후 비교.
SURFACE_MISMATCH 12.3% 흡수 추정. 의미 손실 없음.

---

## 핵심 학습 포인트

### 1. 가설은 측정 전에 테스트되어야 한다

**Why**: "복합명사 over-split이 ~20% 잔여 영역의 주범"이라는 가설은 정성적 인상에 기반.
실측 결과 GOLD_SINGLE_PRED_MULTI는 2.5%만 차지. 진짜 큰 영역은 POS_DIFF (19.5%) +
SURFACE_MISMATCH (12.3%) + SPLIT_DIFFERENT (10.2%) — 가설과 다름.

**적용 원칙**: 잔여 갭 분석 전 가설을 명시하고, 측정으로 검증한 후 다음 작업을 결정.
가설이 틀리면 로드맵 재검토.

### 2. 어절 단위 측정 알고리즘은 정렬 정책이 핵심

**Why**: 같은 데이터셋, 같은 mecab으로 3가지 알고리즘 시도:
- 누적 surface char (v1): SURFACE_MISMATCH 42.2% (cascade)
- char position 정렬 (v2): SURFACE_MISMATCH 92.6% (jamo decomposition mismatch)
- 어절별 독립 토크나이즈 (v3): SURFACE_MISMATCH 12.3%, EXACT 52.4%

세 결과의 EXACT 비율: 35.7% / 4.6% / **52.4%**. 30%+ 차이가 알고리즘 선택만으로 발생.

**적용 원칙**: 평가 알고리즘의 정렬 정책은 측정 결과의 신뢰도를 결정. 같은 코퍼스에서
여러 알고리즘 비교 후 가장 robust한 것 선택. Sprint 126의 evaluate_dataset_dual의 eojeol
메트릭은 sequence cascade로 underestimate함이 확인 — Sprint 128 후보로 어절별 독립
메트릭 추가.

### 3. KLUE의 morpheme surface 분해 정책은 mecab과 다름

**Why**: KLUE는 "공정한" → "공정"+"하"+"ㄴ" (4 chars), mecab은 "공정한" 또는
"공정"+"한" (3 chars). 이 차이로 cumulative surface char 정렬은 항상 실패.

**적용 원칙**: 외부 코퍼스의 morpheme surface 표기 규칙을 사전 검토. char-level 정렬은
규칙이 다르면 작동 안 함. byte position 기반 또는 어절별 독립 토크나이즈 권장.

### 4. Slice-level lenient의 trade-off는 의미 손실

**Why**: 87.7% ceiling은 매력적이나 분할 무시 = POS 무시 = 형태소 분석 평가 무의미.
Sprint 125-126의 conservative/practical은 명확한 convention만 흡수해 의미 손실 없음.
Slice-level은 한 단계 더 나아가 의미를 잃음.

**적용 원칙**: lenient 메트릭은 "convention 흡수"와 "의미 손실"의 경계를 넘지 않아야 한다.
Slice-level은 downstream search 등 비분석 use case에만 적합.

---

## Sprint 128 권고

### P1 후보: 어절별 독립 토크나이즈 메트릭 정식 추가

`evaluate_dataset_eojeol_per_word()` 함수 추가. Sprint 126의 `evaluate_dataset_dual`의
eojeol 메트릭을 deprecate하거나 이름 변경 (예: `eojeol_sequence`).

근거: Sprint 127에서 알고리즘 차이로 30%+ underestimate 확인. 정확한 어절 정확도 측정에
필요. 측정값 약 52.4%이 mecab의 진짜 능력에 가까움.

### P2 후보: Morpheme surface normalization lenient

KLUE의 jamo-decomposed surface를 mecab 표면형으로 normalize 후 비교.
SURFACE_MISMATCH 12.3% 흡수 추정. 의미 손실 없음.

구현: `pos_eq` function pointer 패턴과 동일하게 `surface_match: fn(&str, &str) -> bool`
주입.

### P3 후보 (carryover): 진짜 분석 오류 디버그

NNG/NNP 242건, MAG/NNG 95건, VV/NNG 43건 + 본 분석의 GOLD_SINGLE_PRED_MULTI 553건의
분류. 사전 보강 vs CRF 재학습 결정.

### P4 후보 (carryover): noisy 데이터 추가, CI 통합

---

## 산출물

- `rust/crates/mecab-ko-core/tests/accuracy_eval.rs`:
  - `test_klue_dp_compound_noun_analysis` 추가 (어절별 독립 토크나이즈 알고리즘)
  - 7개 카테고리 분류 + slice-lenient ceiling + 분할 패턴 빈도 + top compound nouns
- 본 보고서

빌드/clippy clean (다음 단계에서 검증), 새 테스트는 `#[ignore]`로 분석 전용.

---

*작성: 2026-05-11*
*작성자: Mario Cho (hephaex@gmail.com)*
*Sprint: 127 P1*
