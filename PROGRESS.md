# PROGRESS — mecab-ko Sprint 141 (VCP+ETM/EC splitter fix)

> 마지막 업데이트: 2026-05-20

## Sprint 141 A — VCP 결합 토큰 분리

| Task | 상태 | 비고 |
|------|------|------|
| S141-A1: XSN/VCP 동치 실험 — 가설 검증 | ✅ 완료 (가설 폐기) | XSN/VCP는 동치 아닌 진짜 의미 차이. 실제 문제는 mecab의 결합 토큰 |
| S141-A2: 측정 분석 + 결정 | ✅ 완료 | splitter 패턴 추가로 UD +0.1pp lift, KLUE 무변경, sample.tsv 무회귀 |
| S141-A3: 보고서 | ✅ 완료 | `docs/research/accuracy/2026-05-20_sprint141_vcp_split_fix.md` |

## 핵심 발견

### 초기 가설 폐기

Sprint 140 분석의 (3777, 2240) XSN(적) → VCP(인) pair는 SPLIT_DIFFERENT로 분류됐으나, mecab과 gold 모두 같은 분해 방식 (XSN + VCP + ETM) 사용. 차이는 **mecab의 결합 토큰 표기**:
- mecab: `인/VCP+ETM` (1 token)
- gold: `이/VCP + ㄴ/ETM` (2 tokens)

→ 동치 추가가 아니라 **splitter 패턴 추가**가 정답.

### 구현 (splitter.rs)

VCP+ETM 패턴:
- `인` → `이/VCP + ㄴ/ETM`
- `일` → `이/VCP + ㄹ/ETM`
- `라는` → `이/VCP + 라는/ETM`

VCP+EC 패턴 (명시 surface 8개):
- 라, 며, 라서, 라고, 라며, 라면, 라야, 라든지 → `이/VCP + X/EC`

### 단위 테스트 4개 신규

- `test_split_morpheme_vcp_etm_in` / `_il` / `_la` (각 분리 검증)
- `test_split_morpheme_vcp_etm_unrelated_surface_no_split` (overcorrect 방지)

## 측정값

| 메트릭 | Before | After | Δ |
|--------|--------|-------|---|
| sample.tsv Token | 100.0% | 100.0% | — |
| sample.tsv Sentence | 99.9% | 99.9% | — |
| KLUE morph strict | 66.8% | 66.8% | — |
| KLUE morph practical | 71.6% | 71.6% | — |
| KLUE eo practical | 23.5% | 23.5% | — |
| **UD Kaist morph strict** | **66.3%** | **66.4%** | **+0.1pp** |
| **UD Kaist morph practical** | **68.0%** | **68.1%** | **+0.1pp** |
| UD Kaist eo strict (count) | 3989 | 3987 | -2 (-0.01pp) |
| UD Kaist eo practical (count) | 4194 | 4193 | -1 |

### 분석

- UD morph +0.1pp = 형태론적 정확성 향상
- KLUE 무변경 = VCP+ETM 패턴 빈도 차이 (UD 92건 vs KLUE 27건, 학술 텍스트 특성)
- UD eojeol -2건 = 일부 어절에서 mecab 결합 토큰이 gold와 우연히 일치했던 경우
- VCP+EC 추가 효과 없음 (mecab이 VCP+EC를 별도 출력하는 빈도 매우 낮음)

→ 형태론적 정확성 향상이 미미한 회귀(-0.01pp)를 정당화 → **유지**

## 핵심 학습 포인트

### 1. 가설은 데이터로 검증해야 함

SPLIT_DIFFERENT pair 빈도만 보고 동치 추가를 결정하면 안 됨. 실제 mecab vs gold 분해 방식 직접 비교 필수.

### 2. SejongConverter splitter가 형태론 정규화의 진짜 진입점

matrix.def cost 조정(Sprint 138 실패)은 너무 거침. splitter 패턴 추가는:
- 형태론적 정확
- KLUE/UD 일관 적용
- sample.tsv 회귀 위험 거의 없음 (특정 결합 토큰만 분리)

### 3. 도메인별 빈도 차이는 패턴 영향 분포의 의미

VCP+ETM UD 92건 vs KLUE 27건 → 변경의 KLUE 영향 작음 → 도메인 특화 변경이 도메인 독립 변경보다 회귀 위험 낮음.

## 검증

- `cargo test --workspace --exclude mecab-ko-ffi --lib`: **396 passed / 0 failed** (392 + 4 신규)
- `cargo clippy --workspace --all-targets -- -D warnings`: clean
- `test_full_accuracy_evaluation`: PASS (sample.tsv 100.0%/99.9%)
- `test_klue_dp_dual_metric_lenient`: PASS (변화 없음)
- `test_ud_kaist_dual_metric`: PASS (morph strict 66.3→66.4%)

## 변경 파일

- `rust/crates/mecab-ko-core/src/sejong/splitter.rs`: VCP+ETM/EC 패턴 + 4 단위 테스트
- `docs/research/accuracy/2026-05-20_sprint141_vcp_split_fix.md` (신규)
- `PLAN.md`, `PROGRESS.md` 갱신

## Sprint 142 후보

- A: 다른 mecab 결합 토큰 패턴 조사 (NNG+VCP+EC 등)
- B: dict-builder CSV 버그 수정 (Track D 선행)
- C: NIKL Modu 또는 OpenKorPOS 추가
- D: Full CRF retrain (B 선행)
