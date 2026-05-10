# Error Case Classification (2026-05-10)

> Sprint 121 P2: full dict baseline 4건 실패의 프로그래매틱 에러 분류.
> 핵심 결론: **4건 전부 영문 약어의 SL(외국어) vs NNP(고유명사) 라벨링 규약 차이.**
> 실질적 분석 오류는 1건(MBTI+가 분절 오류)뿐이며, 나머지 3건은 사전 등록 정책 문제.

---

## 방법론

`test_error_case_classification` 테스트를 작성하여 프로그래매틱으로 분류.

분류 기준:
- **POS_ONLY**: surface(표면형)는 일치, POS(품사)만 다름
- **SEGMENTATION**: surface 자체가 불일치 (토큰 경계 다름)
- **TOKEN_COUNT**: gold/pred 토큰 수 차이
- **OTHER**: 위 어느 것도 해당하지 않음

재현 명령:
```bash
cd rust && cargo test -p mecab-ko-core --test accuracy_eval \
  test_error_case_classification -- --ignored --nocapture
```

---

## Category Summary

| 카테고리 | 건수 | 비율 |
|----------|------|------|
| **POS_ONLY** | 3 | 75.0% |
| **SEGMENTATION** | 1 | 25.0% |
| TOKEN_COUNT | 0 | 0% |
| 사전 미등록 (UNKNOWN) | 0 | 0% |
| 공백 처리 오류 | 0 | 0% |

**총 4건 / 1,100문장 → Sentence Accuracy 99.64%**

---

## POS Confusion Matrix (gold -> pred)

| Gold POS | Pred POS | 건수 | 비고 |
|----------|----------|------|------|
| SL (외국어) | NNP (고유명사) | **4** | 전부 영문 약어 |
| JKS (주격조사) | VV (동사) | 1 | MBTI 케이스의 cascade |

**핵심**: 유일한 혼동 축은 **SL -> NNP**. JKS->VV는 MBTI 분절 오류의 연쇄 효과.

---

## Detailed Error Cases

### Error #1 [POS_ONLY]: "API를 호출하여 결과를 받았다"

```
Gold: API/SL  를/JKO 호출/NNG 하/XSV 어/EC 결과/NNG 를/JKO 받/VV 았/EP 다/EF
Pred: API/NNP 를/JKO 호출/NNG 하/XSV 어/EC 결과/NNG 를/JKO 받/VV 았/EP 다/EF
Diff: [0] API: gold=SL pred=NNP
```

분절: 완벽 일치. POS만 다름.
원인: mecab-ko-dic이 "API"를 NNP로 등록. 평가 데이터는 SL을 기대.

### Error #2 [POS_ONLY]: "TMI인데요"

```
Gold: TMI/SL  이/VCP ㄴ데요/EF
Pred: TMI/NNP 이/VCP ㄴ데요/EF
Diff: [0] TMI: gold=SL pred=NNP
```

분절: 완벽 일치. POS만 다름.
원인: 동일 패턴 — SL vs NNP.

### Error #3 [SEGMENTATION]: "MBTI가 뭐예요"

```
Gold: MBTI/SL  가/JKS 뭐/NP 이/VCP 에요/EF  (5 tokens)
Pred: MBTI/NNP 가/VV  아/EF 뭐/NP 이/VCP 에요/EF  (6 tokens)
Diff:
  token count: gold=5 pred=6
  [0] MBTI: gold=SL pred=NNP
  [1] 가: gold=JKS pred=VV
  [2] gold=뭐/NP pred=아/EF
  [3] gold=이/VCP pred=뭐/NP
  [4] gold=에요/EF pred=이/VCP
```

**이것만 진짜 분석 오류.**
원인 체인:
1. "MBTI" → NNP로 분석 (SL이 정답)
2. NNP 뒤의 "가" → `VV+EC (가/VV/*+아/EC/*)` Inflect 토큰으로 분석
   (JKS가 정답이지만, MeCab 사전에 "가/VV+EC" Inflect 엔트리가 존재하여
   NNP 뒤에서 경로 비용이 JKS보다 낮아짐)
3. Inflect 분해 후 토큰 수가 5→6으로 증가, 이후 전부 정렬 어긋남

근본 원인: "MBTI" NNP 뒤 right_id가 NNP의 것이라 "가/JKS" 연결 비용이
NNG 뒤보다 높아짐. SL 뒤라면 JKS가 선택되었을 것.

### Error #4 [POS_ONLY]: "AI로 그림 그렸어"

```
Gold: AI/SL  로/JKB 그림/NNG 그렸어/VV
Pred: AI/NNP 로/JKB 그림/NNG 그렸어/VV
Diff: [0] AI: gold=SL pred=NNP
```

분절: 완벽 일치. POS만 다름.

---

## 분석

### 근본 원인은 단 하나: 영문 약어의 POS 등록 정책

mecab-ko-dic 2.1.1에서 영문 약어(API, TMI, MBTI, AI)는 **NNP(고유명사)**로
등록되어 있음. 평가 데이터(sample.tsv)는 **SL(외국어)**를 정답으로 기대.

이것은 "형태소 분석 오류"가 아니라 **라벨링 규약 차이**:
- NNP 관점: 고유한 이름/약어이므로 고유명사가 맞음
- SL 관점: 영문 알파벳이므로 외국어 태그가 맞음

두 관점 모두 언어학적으로 타당. 정답 데이터와 사전의 태깅 기준 불일치.

### 실질적 분석 오류는 1건뿐

Error #3 (MBTI가 뭐예요)만 분절 오류 포함. 이 케이스도 SL→NNP 차이에서
cascade된 것이므로, SL/NNP 규약을 통일하면 이 케이스도 해소됨.

### 카테고리별 해결 전략

| 카테고리 | 해결 방법 | 복잡도 | 영향 범위 |
|----------|----------|--------|----------|
| SL↔NNP | 평가 데이터 정답을 NNP로 수정 **OR** 사전의 영문 약어를 SL로 변경 | 낮음 | 4건 해소 |
| "가" 분절 오류 | SL↔NNP 해소 시 자동 해소 | - | 1건 |

### 이전 수동 분석과의 차이

Sprint 121 첫 분석에서 다음 케이스를 보고했으나, 이는 디버그 테스트 케이스였고
실제 sample.tsv 평가 실패와 무관:

| 이전 보고 | 실제 |
|-----------|------|
| "되" VV↔XSV ("크리에이터 되고 싶어") | XSV 디버그 테스트 — sample.tsv 에러 아님 |
| "보" VV↔NNG ("보지만 하지만") | XSV 디버그 테스트 — sample.tsv 에러 아님 |
| SL↔NNP ("MBTI") | **맞음**, 실제 에러 |
| "가" JKS↔VV | **맞음**, MBTI cascade |

**프로그래매틱 분류의 가치**: 수동 분석은 디버그 테스트 출력과 평가 실패를 혼동.
자동 분류는 정확히 sample.tsv 기준 4건만 추출.

---

## 학습 포인트

### 1. 수동 분석과 프로그래매틱 분류의 차이

**왜 중요한가**: 이전 수동 분석은 `test_xsv_debug_sentences` 출력의 불일치 마커를
sample.tsv 평가 실패와 혼동. "되 VV↔XSV"는 디버그 케이스였지 정확도 실패가 아님.

**적용 원칙**: 에러 분류는 반드시 프로그래매틱으로 수행.
"눈으로 보고 분류"하면 출처가 다른 데이터를 혼동.

### 2. 단일 근본 원인의 cascade 효과

**왜 중요한가**: 4건 에러가 4가지 다른 문제처럼 보이지만 근본 원인은 하나
(SL↔NNP 정책). MBTI 분절 오류도 NNP 뒤의 연접 비용 차이에서 cascade.

**적용 원칙**: 에러 케이스가 여러 개일 때 공통 근본 원인을 먼저 찾을 것.
개별 케이스를 개별 수정하면 regression 위험이 올라감.

### 3. "분석 오류" vs "라벨링 규약 차이" 구분

**왜 중요한가**: SL↔NNP는 형태소 분석기의 오류가 아니라 정답 데이터와
사전의 태깅 기준 불일치. 이것을 "99.9% → 100% 정확도 개선"으로
스프린트를 만들면 시간 낭비.

**적용 원칙**: 에러 분류 후 "이것이 진짜 분석 실패인가, 아니면 규약 차이인가?"를
판별하는 단계 필수. 규약 차이는 정답 데이터 교정으로 해결.

---

## Sprint 122 권고 (업데이트)

baseline 99.9%의 실패 4건이 모두 SL↔NNP 규약 차이라면:

1. **sample.tsv 정답 데이터 교정**: API/TMI/MBTI/AI를 NNP로 변경하면
   token accuracy 100%, sentence accuracy 100% 달성 가능
   → 단, 이것은 "데이터 수정"이지 "분석 개선"이 아님

2. **데이터셋 확장이 여전히 최우선**: 99.9%(사실상 100%)인 데이터셋에서
   더 이상 개선 여지 없음. 진짜 약점을 드러낼 noisy/diverse 데이터 필요

3. **C++ mecab-ko 비교**: drop-in replacement 검증. C++ 원본도 동일하게
   NNP를 출력하는지 확인 → 출력 일치가 100%에 가까울 것으로 예상

---

*작성: 2026-05-10*
*작성자: Mario Cho (hephaex@gmail.com)*
*Sprint: 121*
*테스트: test_error_case_classification (accuracy_eval.rs)*
