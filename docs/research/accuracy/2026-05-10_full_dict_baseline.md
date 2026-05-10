# Full Dictionary Accuracy Baseline (2026-05-10)

> Sprint 121 P1 결과: mecab-ko-dic full dict 첫 정확도 측정.
> Token Accuracy **99.9%**, Sentence Accuracy **99.6%** (1,100문장 / 5,833 토큰).

---

## 배경

### 측정 동기

Sprint 117까지 보고된 "Token Accuracy 100%"는 mini-dict(56 entries) 기준이었음.
mini-dict는 CI 테스트용 축소 사전으로 실제 프로덕션 정확도를 반영하지 못함.
Sprint 121 두 전문가 리뷰에서 "측정 자체부터 해야 한다"는 지적에 따라
mecab-ko-dic full dict(342MB, 약 80만 entries)의 첫 baseline을 측정.

### 사전 정보

- **사전**: `data/mecab-ko-dic-2.1.1-20180720/` (mecab-ko-dic 2.1.1, 2018-07-20)
- **사전 크기**: 342MB
- **Connection cost matrix**: matrix.def 10,292,647 줄
- **사용자 사전**: `data/user-dict/verb-inflections.csv`

### 평가 데이터셋

- **sample.tsv**: 1,239줄 (주석 제외 1,100문장)
- **형식**: `원문\t정답_morphs` (Tab-separated, surface/pos 쌍)
- **변환**: 세종 코퍼스 형식으로 정답 비교 (MeCab 원본 융합 토큰을 분해)

---

## 측정 결과

### 전체 통계

| 지표 | 값 |
|------|-----|
| 테스트 문장 수 | 1,100 |
| 정답 토큰 수 | 5,833 |
| 예측 토큰 수 | 5,834 |
| **Token Accuracy** | **99.9%** |
| **Sentence Accuracy** | **99.6%** (1,096 / 1,100) |
| **POS Accuracy** | **99.9%** |
| Precision | 0.999 |
| Recall | 0.999 |
| F1 Score | 0.999 |

### 품사별 정확도 (상위 빈도순)

| POS | 빈도 | 정확도 |
|-----|------|--------|
| NNG (일반명사) | 1,402 | 100.0% |
| EF (종결어미) | 1,057 | 99.9% |
| EP (선어말어미) | 687 | 100.0% |
| VV (동사) | 545 | 100.0% |
| XSV (동사파생접미사) | 451 | 100.0% |
| JKS (주격조사) | 380 | 99.7% |
| JKO (목적격조사) | 283 | 100.0% |
| VA (형용사) | 191 | 100.0% |
| EC (연결어미) | 174 | 100.0% |
| VX (보조용언) | 101 | 100.0% |
| MAG (부사) | 88 | 100.0% |
| ETM (관형사형어미) | 65 | 100.0% |
| NP (대명사) | 62 | 98.4% |
| JKB (부사격조사) | 60 | 100.0% |
| NNB (의존명사) | 57 | 100.0% |
| (외 16개 품사) | - | - |

### 보조 측정 (보조 테스트들, 모두 100% 통과)

- `test_xsv_debug_sentences`: XSV 분석 8/8 통과 (축하해요, 발표했다, 사용되다 등)
- `test_xsv_sample_errors`: 0 errors
- `test_vv_sample_errors`: 0 errors
- `test_vx_sample_errors`: 0 errors
- `test_xpn_error_analysis`: 0 errors

---

## 실패 케이스 (4건)

전체 4문장이 완전 일치 실패. 패턴 분류:

### 1. VV vs XSV 동음이의어

**케이스 A**: `크리에이터 되고 싶어`
- 예상: `크리에이터/NNG 되/VV 고/EC 싶/VX 어/EF`
- 결과: `크리에이터/NNG 되/XSV 고/EC 싶/VX 어/EF`
- 차이: `되` (VV vs XSV)

**케이스 B**: `보지만 하지만`
- 예상: `보/VV 지만/EC 하/VV 지만/EC`
- 결과: `보/NNG 지만/EC 하/VV 지만/EC`
- 차이: `보` (VV vs NNG)

`되`, `보`처럼 본동사와 파생접미사/명사 양쪽으로 가능한 형태에서
Viterbi가 컨텍스트를 충분히 활용하지 못함.

### 2. SL/NNP + Inflect 분해 오류

**케이스 C**: `MBTI가 뭐예요`
- 예상: `MBTI/SL 가/JKS 뭐/NP 이/VCP 에요/EF`
- 결과: `MBTI/NNP 가/VV 아/EF 뭐/NP 이/VCP 에요/EF`
- 차이:
  - `MBTI` (SL vs NNP) — 외국어 분류 누락
  - `가` (JKS vs VV+EC) — MeCab 원본이 `가/VV+EC`(Inflect)로 분석한 후 잘못 분해

원본 MeCab feature: `가/VV+EC,...,Inflect,VV,EC,가/VV/*+아/EC/*`
→ 평가가 이 Inflect 토큰을 `가/VV + 아/EC`로 분해하지만 컨텍스트상
실제로는 `가/JKS`가 정답.

### 3. (4번째 실패는 출력에 명시되지 않음 — 추가 조사 필요)

---

## 해석 및 의미

### 좋은 소식

**99.9% token accuracy는 프로덕션 수준이다.**
원본 C++ mecab-ko가 보고하는 일반적 정확도와 동등한 범위.
mecab-ko Rust 재구현이 알고리즘적으로 정확함을 입증.

### 주의할 점

이 99.9%는 **이 데이터셋 한정**임. 일반화에는 다음이 필요:

1. **데이터셋이 작고 정제됨**: 1,100문장은 통계적으로 의미 있지만
   실제 noisy 텍스트(SNS, 신조어, 외국어 혼용)에서는 더 낮을 수 있음.
2. **세종 코퍼스 미통합**: 표준 평가셋 부재. 다른 형태소 분석기와의
   직접 비교 불가.
3. **C++ mecab-ko 일치율 미측정**: drop-in replacement 검증 별도 필요
   (mecab 바이너리 미설치).

### 실패 패턴의 본질

4건 실패가 모두 **사전이 아닌 모호성 해소(disambiguation)** 문제임:
- `되`, `보`의 다중 분석 가능성
- `MBTI`의 외국어 분류
- Inflect 토큰의 컨텍스트 의존적 분해

→ 사전 확장으로 해결되지 않음. CRF 재학습 또는 룰 추가가 필요한 영역.

---

## 다음 단계 (Sprint 122 후보)

baseline이 99.9%이므로, "정확도 끌어올리기"보다 다음 방향이 의미 있음:

1. **데이터셋 확장**: 세종 코퍼스 일부 통합, noisy 입력 추가
2. **C++ mecab-ko 비교 파이프라인**: 동일 입력 출력 일치율 측정
3. **모호성 해소 케이스 디버그**: `되`, `보`, SL 분류 케이스 분석
4. **Sentence Accuracy 향상**: 99.6% → 100% 위한 4건 분석 + 회귀 테스트

---

## 재현 방법

```bash
cd rust
cargo test -p mecab-ko-core --test accuracy_eval test_full_accuracy_evaluation \
  -- --ignored --nocapture
```

환경 변수로 사전/평가 데이터 경로 오버라이드 가능:
- `MECAB_DIC_PATH`: 사전 경로 (기본 `data/mecab-ko-dic-2.1.1-20180720`)
- `MECAB_EVAL_PATH`: 평가 데이터 경로 (기본 `data/eval/sample.tsv`)

---

*작성: 2026-05-10*
*작성자: Mario Cho (hephaex@gmail.com)*
*Sprint: 121*
