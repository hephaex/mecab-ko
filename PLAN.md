# PLAN — mecab-ko Sprint 136 (next)

> 마지막 업데이트: 2026-05-18

## 완료: Sprint 135 P6 — Accuracy Gate Surface-only Integration

- `.github/workflows/accuracy-gate.yml`에 새 step 추가:
  - `test_klue_dp_eojeol_surface_only` 실행 (Sprint 133 P2)
  - strict / canonical / canonical_lenient 3개 값 추출
  - Floor 검증 (existing assertions: strict ≥ 50%, canonical ≥ 80%)
- PR comment에 검색 use case 메트릭 섹션 추가
- 의미 손실 명시 (형태소 분석이 아닌 색인 baseline)
- actionlint: 0 errors

### Sprint 134 baseline 측정 (CI에서 자동 감시)

| Mode | Floor | Current |
|------|-------|---------|
| strict | ≥ 50% | 87.7% |
| canonical | ≥ 80% | 91.6% |
| canonical_lenient | — | 95.4% |

## 다음 스프린트: Sprint 136 (미정 — 사용자 선택)

### 후보 P1: Noisy data 추가 (Sprint 131부터 deferred)

데이터셋 선정 조사 (KoBEST, KorQuAD, NIA, SNS). 라이선스 + 도메인 매칭 평가. 조사-only sprint 또는 1개 데이터셋 즉시 통합.

### 후보 P3a: 추가 normalize 규칙 (Sprint 134 후속)

Sprint 134 후 remaining top 패턴 (총 1037 still mismatch):
- 따르아 → 따라 (18×): 동사 활용 contraction
- 것이 → 게 (12×): 대명사 contraction
- 앞서 → 앞서어 (12×): 어미 보존 (역방향)
- 갑니다 → 가이습니다 (5×): 1-syllable verb + 이습니다 (현재 룰 미흡수)

자동 norm 위험 — 사례별 검토 후 안전 case만 추가.

### 후보 P4: CRF retrain 인프라 조사

Long-term investment, research-only. ~400 cases context-dep 오류 해결 경로.

### 후보 P5: borderline NNG↔NNP normalization layer

Sprint 132에서 보류된 5 entries (보스턴/외무부 등). 별도 normalization layer 검토.

### 검증 기준 (모든 후보 공통)

- `cargo test --workspace --exclude mecab-ko-ffi` 전체 pass
- `cargo clippy --workspace --all-targets --exclude mecab-ko-ffi -- -D warnings` clean
- accuracy-gate CI 통과 (sample.tsv 99.9%+ / KLUE DP morph 60%/eo 15% / surface_only strict 50%/canon 80%)
