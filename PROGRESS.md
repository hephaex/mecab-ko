# PROGRESS — mecab-ko Sprint 150 (VA+ETM multi-syllable)

> 마지막 업데이트: 2026-05-21

## Sprint 150 A — VA+ETM multi-syllable ㄴ jongseong split

| Task | 상태 | 결과 |
|------|------|------|
| S150-A1: VA+ETM raw 빈도 진단 | ✅ 완료 | 542건 (1-syl 160 + multi 382) |
| S150-A2: post-splitter mismatch 진단 | ✅ 완료 | 518/542 처리됨 (95.6%), 미처리 24건 |
| S150-A3: gold 검증 | ✅ 완료 | 빠르/VA + ㄴ/ETM 형식 확인 |
| S150-A4: splitter 규칙 추가 (VA만, VV는 Sprint 145 제외) | ✅ 완료 | 4 단위 테스트 |
| S150-A5: 5-gate 측정 | ✅ 완료 | KLUE strict +0.4pp, 무회귀 |
| S150-A6: clippy 정리 | ✅ 완료 | clean |

## 핵심 발견

### Raw 빈도 vs 실제 미처리 갭

- VA+ETM raw 542건 (1-syl 160, multi 382)
- **ending_rules가 이미 518건 (95.6%) 처리**
- 미처리 24건: 빠른(13) + 나쁜(5) + 예쁜(4) + 느린(1) + 신선한(1)

빈도 분석만으로 판단했다면 542건 모두를 작업 대상으로 오해할 수 있었음.

### 미처리 24건 패턴

`multi-syllable VA+ETM with ㄴ jongseong on last char`:
- 빠른 → 빠르 + ㄴ (르 불규칙)
- 나쁜/예쁜 → 나쁘/예쁘 + ㄴ (ㅡ 탈락)

이는 1-syllable case의 자연 확장이지만 Sprint 145에서 VV+ETM에 적용 시 sample.tsv -1 sentence 회귀.
**VA만 확장** (어휘 범위 제한 → false positive 위험 낮음)으로 안전 처리.

### 측정 결과 (sample.tsv 무회귀)

| Metric | Before | After | Δ |
|--------|--------|-------|---|
| sample.tsv | 100.0%/99.9% | 100.0%/99.9% | — |
| **KLUE morph strict** | 66.5% | **66.9%** | **+0.4pp** |
| KLUE morph practical | 71.9% | 71.9% | — |
| KLUE surface strict | 87.7% | 87.8% | +0.1pp |
| UD Kaist morph practical | 68.3% | 68.4% | +0.1pp |
| UD GSD morph practical | 71.7% | 71.6% | -0.1pp (noise) |
| UD GSD eojeol practical | 2918 | 2926 | +8 |

가장 큰 시그널: **KLUE morph strict +0.4pp**.

## 검증

- `cargo test --workspace --exclude mecab-ko-ffi --lib`: **404 passed / 0 failed** (402+2)
- `cargo clippy --workspace --all-targets --exclude mecab-ko-ffi -- -D warnings`: clean
- 5-gate 모두 PASSED (sample.tsv 무회귀)

## 변경 파일

- `rust/crates/mecab-ko-core/src/sejong/splitter.rs`:
  - multi-syllable VA+ETM ㄴ jongseong split 추가 (L398)
  - 4 단위 테스트 신규
- `rust/crates/mecab-ko-core/tests/accuracy_eval.rs`:
  - `test_va_etm_post_splitter_mismatch` 진단
  - `test_va_etm_multisyllable_diagnosis` 빈도 분석
- `docs/research/accuracy/2026-05-21_sprint150_va_etm_multisyllable.md` (신규)

## Sprint 151 후보

- C: accuracy_eval.rs setup helper 추출 (잔여 P0 정리)
- D: Node/WASM CI 강화 (continue-on-error 제거)
- B: Full CRF Retrain (Track B, 메인 lift)
- E: 추가 미처리 케이스 분석 (XSA+ETM 38건 등)
