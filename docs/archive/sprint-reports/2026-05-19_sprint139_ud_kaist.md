# Sprint 139 P2 — UD Korean-Kaist Silver Baseline 통합

> **목적**: KLUE DP 단독 평가의 도메인 편향 해소. UD Korean-Kaist (CC BY-SA 4.0)를 silver gold로 변환하여 평가 다양화.

---

## 1. 데이터셋 통합

### 1.1 출처

- **UD Korean-Kaist** (https://github.com/UniversalDependencies/UD_Korean-Kaist)
- 라이선스: **CC BY-SA 4.0** (open, redistributable)
- 형식: CoNLL-U (10-column tab-separated)
- 다운로드: `master` 브랜치의 `ko_kaist-ud-test.conllu` + `ko_kaist-ud-dev.conllu`

### 1.2 CoNLL-U 형식 발견

| Column | 내용 | 활용 |
|--------|------|------|
| 1 | token ID | (skip multi-word `1-2`) |
| 2 | form (어절) | text 재구성 |
| 3 | lemma | morpheme 분해 (`조약+에`) |
| 4 | UPOS | 너무 lossy (NOUN→NNG/NNP/NNB...) — skip |
| **5** | **XPOS (KAIST tags)** | **morpheme POS (`ncn+jca`)** — 1:N Sejong 매핑 가능 |

핵심 발견: lemma + XPOS 결합으로 morpheme 단위 silver gold 구성 가능. UPOS 변환보다 훨씬 정확.

### 1.3 변환 도구

`tools/convert_ud_kaist.py` (Python 3) — CoNLL-U → mecab-ko TSV.

50개 KAIST XPOS → Sejong tag 매핑 (`XPOS_TO_SEJONG` dict). Lossy:
- `ncpa`/`ncps` → NNG (Sejong 미세 분류 없음)
- `npd` → NP (지시대명사 흡수)
- `mmd`/`mma` → MM (관형사 분류 흡수)
- `jct` → JKB (rare/uncertain)
- `mad` → MAG (rare)

Skip 조건: unknown tag, empty morpheme, lemma/xpos count mismatch.

### 1.4 변환 결과

| 파일 | 입력 | 변환 | Skip | 변환률 |
|------|------|------|------|--------|
| `ud_kaist_test.tsv` | 2,287 sentences | 1,638 | 649 | 71.6% |
| `ud_kaist_dev.tsv` | 2,066 sentences | 1,486 | 580 | 71.9% |
| **합계** | **4,353** | **3,124** | **1,229** | **71.8%** |

---

## 2. Baseline 측정 (test split, 1,638 sentences)

| Metric | Value | KLUE DP (참조) |
|--------|-------|---------------|
| Morpheme strict | 66.3% | 66.8% |
| Morpheme practical | 68.0% (+1.7pp) | 71.6% (+4.8pp lift) |
| Per-eojeol strict | 20.7% | 20.7% |
| Per-eojeol practical | 21.8% (+1.1pp) | 23.5% (+2.8pp lift) |

### 2.1 비교 분석

**Morpheme strict 차이**: UD Kaist 66.3% vs KLUE 66.8% (-0.5pp).
거의 동일 — silver 변환이지만 mecab과 같은 형태소 분해 convention에 맞춰져 있음.

**Practical lift 차이**: UD +1.7pp vs KLUE +4.8pp.
KLUE는 SP/SC, SL/NNP, MM 그룹, NNB/NNG, VA/VV practical 동치로 +4.8pp 흡수.
UD는 KAIST 분류가 다른 카테고리에 흡수되어 lift 폭이 작음. 예시:
- KAIST `jcc` (보격조사) → Sejong JKC 매핑
- mecab은 동일 surface(예: "가")를 JKS(주격)로 분석
- JKC vs JKS는 practical에 포함되지 않음 → 차이 누적

**의미**:
- UD Kaist는 KLUE와 다른 종류의 오류 노출 (조사 세부 분류)
- 두 데이터셋이 보완적 — domain bias 해소 효과 확인

---

## 3. 활용 방안

### 3.1 단기 (Sprint 139 완료)

- ✅ silver gold 통합 완료 (1,638+1,486 sentences)
- ✅ baseline 측정 (test split)
- ✅ `test_ud_kaist_dual_metric` 신규 (ignored test)
- ✅ floor assertion: morph strict ≥ 40% (silver tolerance)

### 3.2 중기 (Sprint 140+)

**옵션 A**: Sprint 137의 SPLIT_DIFFERENT 분석을 UD Kaist에도 적용
- KLUE만으로는 특정 도메인 (뉴스/리뷰) 패턴만 봄
- UD Kaist는 다른 장르 (역사 텍스트, 학술) — 다른 problematic pairs 발견 가능

**옵션 B**: practical equivalence map 확장
- JKC ↔ JKS 동치 추가 검토 (UD와 mecab의 보격/주격 구분 차이)
- 단, KLUE에서 negative 효과 발생 가능 (case 차이가 의미적임)

**옵션 C**: accuracy-gate CI에 UD Kaist 추가
- 현재 KLUE DP + sample.tsv 두 게이트 → UD Kaist 추가로 3 게이트
- 단순 cost 변경(Sprint 138) 같은 회귀를 더 강하게 감지

### 3.3 장기 (Track B 준비)

UD Kaist train split (이번 sprint 미통합)을 학습 데이터 후보로 활용 가능.
- UD 학습 split: 23,000+ sentences
- 다른 도메인 cover (역사 텍스트, 학술적 표현)
- Track B Full CRF retrain 시 다양화된 학습 코퍼스 구성에 기여

---

## 4. 핵심 학습 포인트

### 4.1 lemma + XPOS 결합이 UPOS보다 우월

UD CoNLL-U는 UPOS(보편 태그)와 XPOS(언어별 태그) 둘 다 제공. 한국어처럼 morpheme이 풍부한 언어에서는 XPOS의 morpheme-level 매핑이 핵심. UPOS만 사용하면 NOUN→NNG/NNP/NNB 같은 lossy 변환 불가피.

### 4.2 silver 변환의 trade-off 인식

UD Kaist 71.8% 변환률 — 28%는 skip (unknown tag, mismatch). 이는 silver 변환의 본질. 100% 변환을 목표로 하면 매핑이 부정확해짐. 보수적 skip이 정확도 측면에서 안전.

### 4.3 도메인 다양화 효과는 strict보다 practical에서 가시화

morph strict 두 데이터셋 거의 동일(66.3 vs 66.8%) → mecab의 기본 동작 일관. practical 차이(+1.7 vs +4.8pp) → 두 데이터셋의 분류 convention 차이 노출. 평가 다양화는 lift 크기보다는 발견 가능한 오류 패턴의 차이에 가치.

---

## 5. 인프라

### 신규 파일

- `data/raw/ud_kaist/ko_kaist-ud-test.conllu` (3.3MB, downloaded)
- `data/raw/ud_kaist/ko_kaist-ud-dev.conllu` (3.0MB, downloaded)
- `data/eval/ud_kaist_test.tsv` (1,638 lines, mecab-ko 형식)
- `data/eval/ud_kaist_dev.tsv` (1,486 lines, mecab-ko 형식)
- `tools/convert_ud_kaist.py` (CoNLL-U → TSV 변환기)
- `test_ud_kaist_dual_metric` in `tests/accuracy_eval.rs`

### 라이선스 표기

UD Korean-Kaist는 CC BY-SA 4.0. 변환 결과(`ud_kaist_*.tsv`)와 변환 코드(`convert_ud_kaist.py`)는 동일 라이선스 상속. 각 TSV 파일 헤더에 출처 + 라이선스 명시.

---

*작성: 2026-05-19 (Sprint 139 P2)*
