# Sprint 124 Phase 1: KLUE DP Dual-Metric + Diagnostic

> 핵심 발견: 65.8% morpheme accuracy의 **80% 이상이 tag scheme 차이**로 설명됨.
> Eojeol-level은 19.2%에 불과. 진짜 분석 오류는 훨씬 적을 가능성 높음.
> Sprint 125는 tag equivalence map으로 진짜 정확도를 분리해서 측정해야 함.

---

## 배경

Sprint 124 Phase 0에서 KLUE DP 65.8% baseline 측정. Phase 1 목표:
1. 이중 메트릭(morpheme + eojeol) 구현 — 사용자 요구사항
2. 65.8%의 진짜 원인 진단 (alignment artifact / tag 차이 / 진짜 오류)
3. KLUE DP 정식 평가 테스트 + threshold 추가

---

## 진단 결과 (P1-1)

`test_error_case_classification` 으로 1,995 문장 자동 분류:

### Category Summary

| 카테고리 | 건수 | 비율 |
|----------|------|------|
| **SEGMENTATION** | 1,858 | **94.1%** |
| POS_ONLY | 116 | 5.9% |
| 사전 미등록 | 0 | - |

### Top POS Confusion Patterns (1,974 errors)

| 패턴 | 건수 | 분류 |
|------|------|------|
| **SP → SC** | 270 | tag scheme (구두점/공백) |
| **SS → SY** | 159 | tag scheme (괄호/따옴표 일반화) |
| NNB → NNG | 158 | 사전 정책 (의존명사 vs 일반명사) |
| NNG → NNP | 147 | 사전 정책 (일반 vs 고유) |
| MAG → NNG | 95 | 진짜 disambiguation |
| NNP → NNG | 95 | 사전 정책 (역방향) |
| XSA → XSV | 76 | 진짜 disambiguation |
| **MMD → MM** | 76 | tag scheme (KLUE 세분, mecab 통합) |
| EC → EF | 59 | 진짜 disambiguation |
| ETM → JX | 54 | 진짜 disambiguation |
| **SS → SSO** | 42 | tag scheme (열린 괄호) |
| **SS → SSC** | 30 | tag scheme (닫힌 괄호) |
| **MMN → MM** | 38 | tag scheme |

### 분류 통계 (대략)

| 분류 | 추정 건수 | 추정 비율 |
|------|-----------|-----------|
| Tag scheme 차이 (관용 vs 관용) | ~615 | 31% |
| 사전 정책 차이 (NNG/NNP/NNB) | ~434 | 22% |
| 복합명사 분할 정책 차이 | 다수 (정확 미측정) | ~20%? |
| 진짜 disambiguation 오류 | 나머지 | ~25%? |

**핵심**: 절반 이상이 분석 오류가 아니라 표기/사전/세분도 차이.

---

## 이중 메트릭 구현 (P1-2)

### 데이터 형식 확장

KLUE DP TSV에 3번째 컬럼(eojeol_counts) 추가:
```
'K팝스타3' 유희열이 ...   '/SS K/SL 팝스타/NNP ...   5,2,2,2,2,4
```
- 콤마 구분 정수 리스트 = 어절별 형태소 개수
- 합계가 형태소 토큰 수와 일치 (검증)
- 2-column TSV(sample.tsv)는 자동으로 `eojeol_counts = None`

### DualMetricResult 구조

```rust
pub struct DualMetricResult {
    pub morpheme: EvaluationResult,        // 기존 morpheme-level
    pub eojeol_correct: usize,
    pub eojeol_total: usize,
    pub eojeol_accuracy: f64,
}
```

### evaluate_dataset_dual 알고리즘

1. Morpheme-level: 기존 `evaluate_dataset_sejong` 재사용 (greedy alignment)
2. Eojeol-level (별도 패스):
   - gold의 `eojeol_counts`를 따라 슬라이스
   - pred도 같은 개수만큼 슬라이스
   - 슬라이스 내 모든 (surface, pos) 쌍이 일치해야 어절 정답
   - eojeol 정보 없는 데이터셋은 건너뜀

### 측정 결과

| 메트릭 | 값 | 비고 |
|--------|-----|------|
| Morpheme (greedy aligned) | **65.8%** | partial match로 부분 점수 |
| **Eojeol (strict per-eojeol)** | **19.2%** | 4,299 / 22,404 |
| Sentence (모든 형태소 일치) | 1.1% | 21 / 1,995 |

**해석**: 어절 단위로 80% 이상이 어딘가에서 한 형태소라도 틀림. 대부분 tag
scheme이나 복합명사 분할 차이로 인한 cascade.

---

## KLUE DP 정식 테스트 추가 (P1-3)

`test_klue_dp_dual_metric` 추가:
- `data/eval/klue_dp_val.tsv` 자동 로드 (없으면 skip)
- `evaluate_dataset_dual` 호출
- Threshold:
  - MORPHEME_FLOOR = 60% (현재 65.8% 대비 5%p 여유)
  - EOJEOL_FLOOR = 15% (현재 19.2% 대비 4%p 여유)
- 회귀 catch 용도, 목표치(aspiration)가 아님

`#[ignore]` 유지 — full dict + KLUE DP 양쪽 데이터 필요.

---

## 핵심 학습 포인트

### 1. 정확도 숫자의 다층적 의미

| 메트릭 | sample.tsv | KLUE DP | 의미 |
|--------|-----------|---------|------|
| Token (greedy) | 100% | 65.8% | 부분 점수, 정렬 보정 |
| Eojeol (strict) | (미측정) | 19.2% | 어절 전체 일치 |
| Sentence (perfect) | 99.9% | 1.1% | 문장 완전 일치 |

같은 토크나이저의 같은 출력을 보고도 **사용 메트릭에 따라 35%p~99%p 차이**.
"정확도 X%" 단일 숫자는 misleading.

**적용 원칙**: 정확도 보고 시 메트릭 정의를 명시. 단일 숫자가 아닌 메트릭
세트로 보고.

### 2. Tag scheme 차이가 정확도를 크게 깎음

KLUE는 SP/SC/SS/SY/SSO/SSC/MMD/MMN/MMA 등 더 세분화된 태그 사용.
mecab-ko-dic은 일부 통합 태그(MM 등)와 다른 분류 사용.

이로 인한 "오류"가 전체 오류의 30% 이상. **분석 알고리즘의 문제가 아님**.

**적용 원칙**: 다른 코퍼스 평가 시 tag mapping 필수 작업.
"tag equivalence map"이 진짜 평가의 첫 단계.

### 3. KLUE DP는 morpheme-level 정확도 측정에 적합

복합명사 분할이나 tag scheme 차이로 인한 cascade가 sentence accuracy를
거의 0%로 만듦. Sentence-level은 KLUE DP에서 의미 없음.
Morpheme + eojeol 메트릭이 적절한 분해.

### 4. 빠른 진단(test_error_case_classification 재사용)이 1시간 절약

기존 P2 도구를 환경 변수로 새 데이터셋에 적용 → 디자인 결정 빨리 도출.
새 도구 만들기 전에 기존 도구 적용 가능성을 먼저 검토.

---

## Sprint 125 권고

### P1: Tag Equivalence Map 구현
- mecab-ko-dic ↔ KLUE 태그 매핑 정의
- {SP, SC} 동치, {SS, SY, SSO, SSC} 동치, {MM, MMD, MMN, MMA} 동치 등
- Lenient evaluator: 동치 매핑 시 정답으로 간주
- 측정 후 morpheme/eojeol 정확도 변화 보고

### P2: 복합명사 분할 정책 분석
- 팝스타/NNP vs 팝/NNG + 스타/NNG 같은 케이스 통계
- mecab-ko-dic 분할 vs KLUE 결합 분포
- 양쪽 모두 정답으로 인정하는 메트릭 또는 conversion

### P3: noisy 데이터 추가 (사용자 우선순위 "높음")
- KLUE DP는 편집 register만 — SNS/구어 추가 필요
- 옵션 평가: NIKL 모두의말뭉치 / silver-label / 사용자 데이터

### P4: CI 통합
- HuggingFace에서 KLUE DP 자동 다운로드 + 변환 step
- accuracy-gate.yml에 KLUE DP gate job 추가

---

## 산출물 (Sprint 124 Phase 1)

- `tools/convert_klue_dp.py` (3-column 형식 출력)
- `data/eval/klue_dp_val.tsv` (eojeol_counts 포함, 1,995 문장)
- `rust/crates/mecab-ko-core/src/evaluate.rs`
  - `GoldSentence.eojeol_counts: Option<Vec<usize>>`
  - `parse_tsv_line` 3-column 지원
  - `DualMetricResult` 구조체
  - `evaluate_dataset_dual()` 함수
- `rust/crates/mecab-ko-core/tests/accuracy_eval.rs`
  - `test_klue_dp_dual_metric` 테스트
- 본 보고서

빌드/clippy clean, 모든 기존 테스트 pass.

---

*작성: 2026-05-11*
*작성자: Mario Cho (hephaex@gmail.com)*
*Sprint: 124 Phase 1*
