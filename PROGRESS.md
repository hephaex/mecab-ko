# PROGRESS — mecab-ko Sprint 143 (UD Korean-GSD 통합)

> 마지막 업데이트: 2026-05-20

## Sprint 143 C — UD Korean-GSD Silver Baseline 통합

| Task | 상태 | 비고 |
|------|------|------|
| S143-C1: UD GSD 다운로드 + 형식 확인 | ✅ 완료 | GSD XPOS는 Sejong 직접 사용 (KAIST와 다름) |
| S143-C2: 변환기 작성 + 통합 | ✅ 완료 | `tools/convert_ud_gsd.py` (identity mapping) |
| S143-C3: 기준 측정 + 보고서 | ✅ 완료 | morph strict 67.4% (KLUE에 가장 가까움) |

## 핵심 발견

### GSD XPOS = Sejong 직접 사용

UD CoNLL-U는 column 5(XPOS)를 데이터셋마다 자유롭게 정의:
- **KAIST**: `ncn`, `pvg`, `jca`, `ecs`... (소문자 약어, Sejong 매핑 필요)
- **GSD**: `NNG`, `NNP`, `JKO`, `VV`... (Sejong 태그 직접) → **identity mapping** (변환 손실 없음)

→ GSD는 KAIST 변환기 재사용 **불가**. 신규 변환기 (identity mapping) 작성.

### 변환률 비교

| 데이터셋 | 입력 | 변환 | 변환률 |
|---------|------|------|--------|
| UD GSD | 1,939 | 1,904 | **98.2%** (identity, lossless) |
| UD Kaist | 4,353 | 3,124 | 71.8% (KAIST→Sejong lossy) |

### Baseline 비교 (3 silver)

| Metric | KLUE DP | UD Kaist | **UD GSD** |
|--------|---------|----------|--------|
| Morph strict | 66.8% | 66.3% | **67.4%** |
| Morph practical | 71.6% | 68.1% | 71.3% |
| Per-eojeol strict | 20.7% | 20.7% | **23.1%** |
| Per-eojeol practical | 23.5% | 21.8% | **25.8%** |

→ GSD가 KLUE에 가장 가까움 (현대 뉴스/web 도메인). KAIST는 학술 텍스트로 낮음.

## 측정값 (변경 없음 — 평가 데이터 추가만)

| 메트릭 | Sprint 142 | Sprint 143 |
|--------|-----------|-----------|
| 기존 4-gate | 동일 | 동일 (회귀 0) |
| **UD GSD morph strict** | (신규) | **67.4%** |
| **UD GSD morph practical** | (신규) | **71.3%** |

## 핵심 학습 포인트

### 1. UD CoNLL-U는 데이터셋마다 XPOS scheme 다름

같은 UD format이라도 column 5(XPOS) 자유 정의 → 데이터셋별 별도 변환기 필요.

### 2. 도메인 다양성은 ~1pp 변동 폭

3 silver: 66.3~67.4% (1.1pp 변동). 단일 도메인 평가로는 안 보임 → 다중 silver 측정이 회귀 감지에 필수.

### 3. 변환률 차이가 데이터셋 호환성 지표

- GSD 98.2% (lossless) → mecab과 같은 annotation 도구 추측
- Kaist 71.8% (lossy) → 다른 annotation scheme

## 검증

- `cargo test --workspace --exclude mecab-ko-ffi --lib`: all pass / 0 fail
- `cargo clippy --workspace --all-targets -- -D warnings`: clean
- `test_ud_gsd_dual_metric`: PASS (strict morph 67.4%)
- 기존 4-gate: 모두 동일 (평가 데이터 추가만)

## 변경 파일

- `tools/convert_ud_gsd.py` (신규)
- `data/raw/ud_gsd/ko_gsd-ud-{test,dev}.conllu` (downloaded)
- `data/eval/ud_gsd_{test,dev}.tsv` (converted, 971 + 933 sentences)
- `rust/crates/mecab-ko-core/tests/accuracy_eval.rs`: `test_ud_gsd_dual_metric` 추가 (~80줄)
- `docs/research/accuracy/2026-05-20_sprint143_ud_gsd.md` (신규)
- `PLAN.md`, `PROGRESS.md` 갱신

## Sprint 144 후보

- A: accuracy-gate CI에 UD GSD 추가 (4 → 5 gate)
- B [메인]: Full CRF Retrain (Track E)
- C: NIKL Modu 추가
- D: 다른 mecab 결합 토큰 패턴 (Sprint 141 연장)
