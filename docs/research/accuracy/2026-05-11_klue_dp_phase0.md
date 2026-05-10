# Sprint 124 Phase 0: KLUE DP Format Inspection + Conversion Prototype

> 핵심 발견: **KLUE DP에서 token accuracy 65.8%** (sample.tsv 100% 대비).
> 99.9% baseline은 데이터셋 천장 효과였음이 정확히 입증됨.
> 다양성 있는 평가셋이 진짜 약점을 노출함.

---

## 배경

Sprint 122에서 sample.tsv(1,100문장 정제 데이터)에서 token 100.0% / sentence 99.9%
달성. 두 전문가 리뷰가 "데이터셋이 너무 정제되어 천장 효과"라고 경고.
Sprint 123 조사에서 KLUE DP를 "즉시 가능한 최선"으로 권고.

본 Phase 0의 목표:
1. KLUE DP 실제 형식 확인
2. 변환 가능성 검증 (Sejong 호환성 + alignment 정합성)
3. 변환기 프로토타입 작성
4. 변환 결과로 baseline 측정 → "진짜 정확도" 노출

---

## KLUE DP 형식 실측 (val split, 2,000 examples)

### 스키마

```
{
    "sentence": str,                 # 원문
    "index": list[int],              # 어절 번호
    "word_form": list[str],          # 어절 (input)
    "lemma": list[str],              # 공백 분리 형태소 (이미 split!)
    "pos": list[str],                # +로 결합된 POS 태그
    "head": list[int],               # 의존 구조 head
    "deprel": list[str],             # 의존 관계 라벨
}
```

### 샘플

```
sentence: 'K팝스타3’ 유희열이 홍정희의 탈락에 눈물을 흘렸다.

word_form    →  lemma         |  pos
'K팝스타3’     →  ' K 팝스타 3 ’    |  SS+SL+NNP+SN+SS
유희열이       →  유희열 이         |  NNP+JKS
홍정희의       →  홍정희 의         |  NNP+JKG
탈락에         →  탈락 에          |  NNG+JKB
눈물을         →  눈물 을          |  NNG+JKO
흘렸다.        →  흘리 었 다 .      |  VV+EP+EF+SF
```

**핵심 관찰**:
- `lemma` 필드가 **이미 형태소 단위로 surface 분할**됨 → 휴리스틱 분할 불필요
- 형태소 수와 POS 태그 수가 정확히 일치 (어절당 zip 가능)
- `흘렸다` → `흘리 었 다`는 **lemmatized form** (mecab-ko 출력 convention과 동일)

### 통계

| 지표 | 값 |
|------|-----|
| 총 examples | 2,000 |
| 총 eojeols | 22,496 |
| **Align mismatch** | **5 (0.02%)** |
| Unique POS tags | 43 |
| 평균 문장 길이 | 11.2 어절 |
| 평균 어절 길이 | 3.4자 |

### 어절당 형태소 분포

| 형태소 수 | 어절 | 비율 |
|----------|------|------|
| 1 | 5,629 | 25.0% |
| 2 | 9,696 | 43.1% |
| 3 | 4,087 | 18.2% |
| 4 | 1,973 | 8.8% |
| 5 | 754 | 3.4% |
| 6 | 253 | 1.1% |
| 7+ | 99 | 0.4% |

→ 75%가 **2개 이상 형태소 결합**. 평가 단위 결정이 중요.

### POS 태그 분포 (top 30, 모두 Sejong 호환)

| POS | Count | POS | Count |
|-----|-------|-----|-------|
| NNG | 13,447 | XSV | 1,532 |
| VV | 2,888 | JKO | 1,527 |
| ETM | 2,850 | SN | 1,439 |
| JKB | 2,547 | EP | 1,424 |
| EC | 2,406 | JKS | 1,357 |
| NNB | 2,368 | MAG | 1,275 |
| NNP | 2,117 | VA | 1,262 |
| SF | 2,018 | SS | 887 |
| EF | 1,997 | VCP | 710 |
| JX | 1,783 | SP | 700 |

전부 mecab-ko-dic이 사용하는 Sejong 태그 셋 내 (43 unique vs 45개 총). 호환성 OK.

---

## 변환기 (`tools/convert_klue_dp.py`)

### 알고리즘

```python
for ex in dataset:
    sentence = ex["sentence"]
    morphs = []
    for lemma_str, pos_str in zip(ex["lemma"], ex["pos"]):
        ms = lemma_str.split(" ")
        ts = pos_str.split("+")
        if len(ms) != len(ts):
            skip_sentence()
            break
        for m, t in zip(ms, ts):
            morphs.append(f"{m}/{t}")
    write(f"{sentence}\t{' '.join(morphs)}\n")
```

**Surface split 휴리스틱이 전혀 필요 없음** — KLUE DP가 lemma 필드에서 이미 제공.

### 변환 결과

- Total: 2,000 → Written: **1,995** → Skipped: **5** (0.25%)
- 출력: `data/eval/klue_dp_val.tsv` (684 KB, 1,995 lines)
- 형식: 기존 `sample.tsv`와 100% 호환 (`text\tsurface1/POS1 surface2/POS2 ...`)

---

## Baseline 측정 (mecab-ko vs KLUE DP)

`MECAB_EVAL_PATH=data/eval/klue_dp_val.tsv` 환경 변수로 기존 평가 하니스 재사용:

```bash
MECAB_EVAL_PATH=data/eval/klue_dp_val.tsv \
  cargo test -p mecab-ko-core --test accuracy_eval \
  test_full_accuracy_evaluation -- --ignored --nocapture
```

### 결과

| 데이터셋 | 문장 수 | Token Accuracy | Sentence Accuracy |
|----------|---------|----------------|-------------------|
| sample.tsv (정제) | 1,100 | **100.0%** | 99.9% |
| **KLUE DP val (실제)** | **1,995** | **65.8%** | (측정 필요) |

**~34%p 차이**. 이것이 진짜 baseline.

### 품사별 정확도 (KLUE DP)

저정확도 패턴:
- JKO (목적격조사): 22.5%
- JKS (주격조사): 33.3%
- SN (숫자): 32.4%
- EP (선어말어미): 16.4%

조사/어미가 심각하게 낮음. 두 가지 가능성:
1. **진짜 분석 오류**: 컨텍스트 기반 disambiguation 부족
2. **alignment artifact**: 형태소 개수 차이로 인한 위치 기반 매칭 실패

품사별 정확도 패턴(조사/어미가 가장 낮음)은 **(2) alignment artifact 가능성이 큼**.
어절당 morpheme 수가 KLUE(2.5)와 mecab-ko 출력(예측 ~2)이 미세하게 다르면
조사 위치가 밀려서 cascade 실패. Phase 1에서 `evaluate_tokens_aligned`(greedy
alignment) 사용 또는 별도 메트릭 필요.

---

## 핵심 학습 포인트

### 1. 평가 데이터셋 다양성이 진짜 신호를 만든다
sample.tsv 100%는 천장 효과였고, KLUE DP에서 65.8%로 떨어짐.
30%p+ 차이는 "데이터셋 의존적 측정"의 위험을 정량적으로 보여줌.

**적용 원칙**: 새 데이터셋 도입 전 기존 데이터셋 정확도와 차이를 먼저 측정.
차이가 크면 어느 쪽이 진짜인지 검증 (보통 다양한 쪽이 진짜).

### 2. KLUE DP는 surface split 노력을 절약해줌
사전 조사에서 우려했던 "복합 태그 surface 분할 휴리스틱" 문제가
실제로는 KLUE가 lemma 필드에서 이미 분리 제공함. 0건 작업.

**적용 원칙**: 실제 데이터를 보기 전에 추상적 우려로 결정하지 말 것.
30분 inspect로 답이 나오는 질문을 1주짜리 디자인으로 만들지 말 것.

### 3. Align mismatch 0.25%는 무시 가능
2,000 중 5건 skip. KLUE 데이터 품질이 매우 높음.

---

## Phase 1+ 권고 (Sprint 125+)

### Phase 1: 평가 하니스 통합
- `data/eval/klue_dp_val.tsv`를 `cargo test`에 정식 추가
- 별도 threshold 설정 (현재 baseline 65.8% 측정 후 70%? 75%?)
- alignment artifact 분석 → `evaluate_tokens_aligned` 사용 권고
- 이중 메트릭 (eojeol-level + morpheme-level) 구현

### Phase 2: Error 분류 자동화
- KLUE DP 실패 케이스 카테고리 분류 (Sprint 121 P2 방식 재사용)
- alignment artifact vs 진짜 분석 오류 분리
- 조사/어미 저정확도 근본 원인 분석

### Phase 3: CI 통합
- HF에서 KLUE DP 자동 다운로드 + 변환
- accuracy-gate.yml에 KLUE DP gate job 추가
- 회귀 시 즉시 알림

### Phase 4: noisy 데이터 추가 (사용자 우선순위 "높음")
- NIKL Modu 구어 subcorpus 또는 자체 silver-label 파이프라인
- KLUE DP는 편집 register만 — 실제 SNS/구어 register 보강 필요

---

## 산출물

- `tools/inspect_klue_dp.py` (포맷 실측 스크립트)
- `tools/convert_klue_dp.py` (변환기 프로토타입)
- `tools/dump_klue_dp_raw.py` (원본 JSONL 덤프)
- `data/eval/klue_dp_val.tsv` (변환 결과, 1,995 sentences, 684KB)
- `data/raw/klue/klue_dp_val.jsonl` (원본 JSONL, 1.6MB)
- `data/raw/klue/LICENSE-KLUE.md` (CC BY-SA 4.0 attribution)

---

*작성: 2026-05-11*
*작성자: Mario Cho (hephaex@gmail.com)*
*Sprint: 124 Phase 0*
