# PLAN — mecab-ko Sprint 135 (next)

> 마지막 업데이트: 2026-05-18

## 완료: Sprint 134 P3 — Normalize Endings Extension

- `normalize_endings`에 2개 규칙 추가:
  - `이습니다 → 입니다` (다중-char, ~80 cases 흡수)
  - `하아 → 하여` (char-pair, ~12 cases 흡수)
- 3 새 unit tests + 분석 테스트 sync 갱신
- KLUE DP `canonical_lenient` surface-only **94.4% → 95.4%** (+1.0pp)
- SURFACE_MISMATCH 흡수율 **54.3% → 62.3%** (+220 cases)
- Sample.tsv 100%/99.9% 무회귀, all 40 ignored pass
- 보고서: docs/research/accuracy/2026-05-18_sprint134_normalize_endings_extension.md

### 핵심 학습

Normalize rule 추가가 dict 트랙(천장 도달)보다 lift 효율 높음 — 단 surface-only 메트릭 한정. Sprint 128에서 만든 인프라가 Sprint 133의 use case 분리 후 비로소 lift 측정 가능 — **메트릭이 잘못되면 좋은 작업도 +0pp로 보임**.

## 다음 스프린트: Sprint 135 (미정 — 사용자 선택)

### 후보 P1: Noisy data 추가 (Sprint 131부터 deferred)

데이터셋 선정 조사 (KoBEST, KorQuAD, NIA, SNS). 라이선스 + 도메인 매칭 평가. 조사-only sprint 또는 1개 데이터셋 즉시 통합.

### 후보 P3a: 추가 normalize 규칙 (Sprint 134 후속)

Sprint 134 후 remaining top 패턴 (총 1037 still mismatch):
- 따르아 → 따라 (18×): 동사 활용 contraction
- 것이 → 게 (12×): 대명사 contraction
- 앞서 → 앞서어 (12×): 어미 보존 (역방향)
- 갑니다 → 가이습니다 (5×): 1-syllable verb + 이습니다 (현재 룰 미흡수)

자동 norm 위험 있음 — 사례별 검토 후 안전 case만 추가.

### 후보 P4: CRF retrain 인프라 조사

Long-term investment, research-only. ~400 cases context-dep 오류 해결 경로.

### 후보 P5: borderline NNG↔NNP normalization layer

Sprint 132에서 보류된 5 entries (보스턴/외무부 등). 별도 normalization layer 검토.

### 후보 P6: accuracy-gate.yml에 surface_only 추가

Sprint 133의 `test_klue_dp_eojeol_surface_only`를 CI 게이트로 추가.

### 검증 기준 (모든 후보 공통)

- `cargo test --workspace --exclude mecab-ko-ffi` 전체 pass
- `cargo clippy --workspace --all-targets --exclude mecab-ko-ffi -- -D warnings` clean
- accuracy-gate CI 통과 (sample.tsv 99.9%+ / KLUE DP floors)
