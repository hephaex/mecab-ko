# Sprint 137 A — Connection Cost Pair Analysis (SPLIT_DIFFERENT)

> **목적**: KLUE DP per-eojeol 오류 중 SPLIT_DIFFERENT 카테고리(2,237건) 에 대해 mecab Viterbi best path의 인접 노드 쌍 `(prev.right_id, curr.left_id)` 빈도를 수집하여 matrix.def 수동 조정 후보 식별.

---

## 1. 측정 결과

### 1.1 데이터셋

- 출처: KLUE DP val (`data/eval/klue_dp_val.tsv`)
- 문장 수: 1,995
- 총 어절 수: 22,404
- SPLIT_DIFFERENT 어절 수: **2,237** (10.0%)
- 사전: mecab-ko-dic 2.1.1 + verb-inflections + klue-domain user-dict

### 1.2 Pair 분포 요약

- 고유 (right_id, left_id) 쌍: **570개**
- 총 pair 발생: **4,330건** (per-eojeol 평균 1.9 pair)
- left-id.def 엔트리: 2,693 / right-id.def 엔트리: 3,822

### 1.3 상위 30 problematic 쌍

| Rank | RID | LID | count | right.feat (prev) | left.feat (curr) | 샘플 |
|------|-----|-----|-------|-------------------|------------------|------|
| 1 | 3534 | 0 | 298 | NNG,*,T,*,*,*,*,* | BOS/EOS,*,*,*,*,*,*,BOS/EOS | 공정성\|을, 돌\|입, 천명\|을 |
| 2 | 0 | 1780 | 264 | BOS/EOS,*,*,*,*,*,*,BOS/EOS | NNG,*,*,*,*,*,*,* | 지\|검장, 주\|의, 지\|옥 |
| 3 | 3533 | 0 | 196 | NNG,*,F,*,*,*,*,* | BOS/EOS,*,*,*,*,*,*,BOS/EOS | 대\|한, 위\|한, 여\|주 |
| 4 | 5 | 1794 | 166 | EF,*,F,*,*,*,*,* | SF,*,*,*,*,*,*,BOS/EOS | 다\|., 빠집니다\|., 보인다\|. |
| 5 | 0 | 0 | 162 | BOS/EOS,*,*,*,*,*,*,BOS/EOS | BOS/EOS,*,*,*,*,*,*,BOS/EOS | 한\|다면, 나\|갈, 입\|하는 |
| 6 | 3561 | 1780 | 134 | SH,*,*,*,*,*,*,* | NNG,*,*,*,*,*,*,* | 100\|여명, 1\|천명, 2\|명의 |
| 7 | 3584 | 0 | 130 | XR,*,T,*,*,*,*,* | BOS/EOS,*,*,*,*,*,*,BOS/EOS | 탁월\|한, 비롯\|한, 비슷\|한 |
| 8 | 3533 | 1780 | 129 | NNG,*,F,*,*,*,*,* | NNG,*,*,*,*,*,*,* | 테니스\|단, 국가\|보훈, 군사\|개 |
| 9 | 8 | 3 | 114 | EP,*,T,*,*,*,*,* | EF,*,*,*,*,*,*,* | 알려졌\|다, 늘어났\|다, 옮겼\|다 |
| 10 | 3534 | 1780 | 109 | NNG,*,T,*,*,*,*,* | NNG,*,*,*,*,*,*,* | 팝\|스타, 보훈\|처, 연월\|차 |
| 11 | 0 | 1790 | 79 | BOS/EOS,... | NP,*,*,*,*,*,*,* | 지\|난, 나\|이, 주\|거 |
| 12 | 0 | 1794 | 57 | BOS/EOS,... | SF,*,*,*,*,*,*,BOS/EOS | 왔다\|., 갔습니다\|. |
| 13 | 3534 | 1800 | 50 | NNG,*,T,*,*,*,*,* | SSO,*,*,*,*,*,*,* | 억\|8, 옥\|', 기관\|( |
| 14 | 3533 | 588 | 46 | NNG,*,F,*,*,*,*,* | JX,*,*,는,*,*,*,* | 처\|는, 거리\|는 |
| 15 | 3534 | 2609 | 42 | NNG,*,T,*,*,*,*,* | XSN,*,*,적,*,*,*,* | 부분\|적, 결정\|적 |
| 16 | 3534 | 522 | 42 | NNG,*,T,*,*,*,*,* | JKS,*,*,이,*,*,*,* | 단\|이, 여명\|이 |
| 17 | 3533 | 1800 | 41 | NNG,*,F,*,*,*,*,* | SSO,*,*,*,*,*,*,* | 스타\|3, 위\|', 이모\|( |
| 18 | 3777 | 0 | 41 | XSN,*,T,적,*,*,*,* | BOS/EOS,... | 적\|으로 |

---

## 2. 패턴 해석

### 2.1 NNG 분해 (Rank 1, 2, 3, 7, 8, 10) — 핵심 문제

**합산: 298+264+196+130+129+109 = 1,126건 (50.3%)**

쌍 (NNG, BOS/EOS) 와 (BOS/EOS, NNG) 는 어절 내 NNG 분해의 직접 표지:
- `공정성|을` — gold가 한 token으로 보는데 mecab이 분리
- `지|검장` — "지검장"이 NNG_compound인데 분리됨
- `대|한` — "대한"이 분리됨
- `탁월|한` — XR + 한
- `테니스|단`, `팝|스타` — NNG+NNG 복합어 분리

**원인**: `(NNG-right-context, BOS-left-context)` 의 connection cost가 너무 낮음 → 어절 중간에서 끝내는 것을 선호.

**조정 후보**: `matrix.def`에서 `(3534, 0)`, `(3533, 0)`, `(0, 1780)` cost를 일정량(+200~500) 상향하면 NNG 끝남/시작에 페널티 부여 → split 회피.

### 2.2 EF + SF 분리 (Rank 4, 12)

**합산: 166+57 = 223건 (10%)**

`다|.`, `빠집니다|.` — 종결어미 + 문장부호 분리.
실제로 mecab이 이걸 분리하는 것은 자연스럽다. KLUE는 결합 표기 사용.
**조정 어려움**: cost 조정 시 다른 SF 위치 어절에 영향.

### 2.3 EP + EF (Rank 9)

114건. `알려졌|다`, `옮겼|다` — 어말어미 분리.
mecab의 분해가 형태론적으로 정확. KLUE convention 차이.
**조정 회피**: 형태소 분석 정확도 측면에서는 mecab이 옳음.

### 2.4 NNG + JKS/JX/XSN 분리 (Rank 14, 15, 16, 18)

**합산: 46+42+42+41 = 171건 (7.6%)**

`단|이`, `처|는`, `부분|적`, `적|으로` — 명사+조사/접사 결합.
mecab 분해가 표준 형태론. KLUE는 일부 어절을 단일 token 처리.
**조정 어려움**: JKS/JX/XSN은 분리되어야 하는 형태소.

### 2.5 SH/SN/SSO + NNG (Rank 6, 13, 17, etc.)

수치/특수문자 + NNG 결합 분리 (`100|여명`, `억|8`, `(|현`).
**조정 가능**: 수치+한자명사 결합 패턴.

---

## 3. Sprint 138 조정 후보

### 3.1 Tier A — 안전한 조정 (NNG 복합 분해 회피)

**Target 쌍** (예상 효과 큰 순):

| Pair | 의미 | 현재 패턴 | 조정 방향 |
|------|------|----------|---------|
| (3534, 0) | NNG-T → BOS | 어절 중간 NNG-T로 종결 선호 | +300 cost |
| (3533, 0) | NNG-F → BOS | 어절 중간 NNG-F로 종결 선호 | +300 cost |
| (0, 1780) | BOS → NNG | 어절 중간에서 NNG로 시작 선호 | +300 cost |
| (3534, 1780) | NNG-T → NNG | NNG+NNG 복합어 분해 | +500 cost |
| (3533, 1780) | NNG-F → NNG | NNG+NNG 복합어 분해 | +500 cost |

**기대 효과**: SPLIT_DIFFERENT 1,126건 중 일부 흡수 → per-eojeol strict +0.5-1.0pp 추정 (회귀 확인 필수).

**리스크**: Cost +300 ~ +500 조정이 다른 어절(올바른 분해 케이스)에 영향. matrix.def 수정 후 sample.tsv + KLUE DP morph + per-eojeol practical 4-mode 회귀 검증 필수.

### 3.2 Tier B — 조정 회피 (형태론적으로 mecab이 정확)

- EF/SF 분리 (rank 4, 12) — 문장 부호는 분리 정상
- EP/EF 분리 (rank 9) — 어말어미 분해는 정확
- NNG/JKS/JX/XSN 분리 (rank 14-18) — 조사/접사 분리는 정확

### 3.3 Tier C — 보류

XR/탁월/비롯/비슷 (rank 7) — XR(어근)는 별도 카테고리. XR+한(VV) 결합이 형태론적으로 어떻게 처리되는지 추가 분석 필요.

---

## 4. 실험 절차 (Sprint 138 권장)

1. **백업**: `matrix.def` → `matrix.def.s137-baseline`
2. **수정**: Tier A 5쌍 cost +300~+500 (스크립트로 정확한 라인 찾아 수정)
3. **재빌드**: `cargo run --bin mecab-ko-dict-builder -- ...` (dict-builder 재실행)
4. **회귀 검증** (모두 통과해야 함):
   - `test_full_accuracy_evaluation` (sample.tsv 100%/99.9%)
   - `test_klue_dp_dual_metric` (morph 60%+ / eo 15%+)
   - `test_klue_dp_dual_metric_lenient` (practical ≥ lenient)
   - `test_klue_dp_eojeol_surface_only` (strict 50%+ / canon 80%+)
5. **측정**: 동일 분석 테스트 재실행 → SPLIT_DIFFERENT 건수 감소량 확인

---

## 5. 인프라

추가된 분석 테스트: `test_klue_dp_split_diff_connection_pairs` in `tests/accuracy_eval.rs`
- 실행: `cargo test --package mecab-ko-core --test accuracy_eval test_klue_dp_split_diff_connection_pairs -- --ignored --nocapture`
- 출력: 상위 30 (right_id, left_id) 쌍 + left-id.def/right-id.def 자질열 + 샘플 surface

새 API: `Tokenizer::lattice()` — `tokenize()` 직후 Viterbi 결과 포함된 lattice 접근.
- 주의: `tokenize_to_lattice()`는 Viterbi 미실행, `best_path()` 빈 결과
- 사용 시 항상 `tokenize()` 호출 후 `lattice()` 접근

---

## 6. 결론

**확인된 사실**:
1. SPLIT_DIFFERENT 2,237 어절의 절반 (50.3%)이 NNG 분해(상위 6 패턴)에 집중
2. EF/SF, EP/EF, NNG+조사 분리는 형태론적으로 mecab이 정확 → 조정 부적합
3. Tier A 5쌍 조정으로 +0.5-1.0pp lift 추정 (회귀 위험은 있음)

**Sprint 138 진입점**: Tier A 5쌍 cost +300~+500 실험, sample.tsv 무회귀 + KLUE 동일 metric pass 조건.

**위험 신호**:
- 부적절한 조정 시 morph strict 정확도 회귀 가능
- matrix.def는 binary로 변환됨 — 텍스트 수정 후 dict-builder 재실행 필요
- (id, 0) 쌍 조정은 어절 경계 처리에 광범위한 영향

---

*작성: 2026-05-19 (Sprint 137 Track A)*
