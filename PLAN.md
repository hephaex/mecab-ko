# PLAN — mecab-ko Sprint 134 (next)

> 마지막 업데이트: 2026-05-18

## 완료: Sprint 133 P2 — Eojeol Surface-only Metric

- `EojeolSurfaceResult` + `evaluate_dataset_eojeol_surface_only[_with_match]` 추가
- KLUE DP 측정: strict **87.7%** / canonical **91.6%** / canonical_lenient **94.4%**
- Sprint 127 P1의 추정 ceiling 87.7%와 정확히 일치 (정식 API + 회귀 게이트화)
- 2 unit tests + 1 KLUE DP integration test
- Sample.tsv 100%/99.9% 무회귀
- 보고서: docs/research/accuracy/2026-05-18_sprint133_eojeol_surface_only.md

### 핵심 학습

Use case별 메트릭 분리의 가치 — 형태소 분석 메트릭(53.9%)과 검색 use case 메트릭(87.7%) 사이 34pp 차이. POS/split 무시의 trade-off가 명확하게 측정됨.

## 다음 스프린트: Sprint 134 (미정 — 사용자 선택)

### 후보 P1: Noisy data 추가

Sprint 131 P4에서 deferred. 데이터셋 선정 조사 (KoBEST, KorQuAD, NIA, SNS). 라이선스 + 도메인 매칭 평가. 조사-only sprint 또는 1개 데이터셋 즉시 통합.

### 후보 P3: 종결어미 normalization 확장

`normalize_endings`에 이ㅂ니다↔이습니다 매핑 추가. Sprint 128 surface lenient +0pp 재측정. Sprint 133에서 측정된 canonical_lenient absorption rate (+2.8pp)에 추가 lift 가능.

### 후보 P4: CRF retrain 인프라 조사

Sprint 129 P3에서 식별한 ~400 cases context-dep 오류. Long-term investment, research-only sprint. Sprint 132+에서 도달한 dict 천장 너머 lift의 유일한 경로.

### 후보 P5: 보스턴/외무부 등 borderline NNG↔NNP 해결

Sprint 132에서 보류된 5 entries. KLUE convention 차이로 dict override는 회귀 위험. 별도 normalization layer 또는 도메인별 alias 검토.

### 후보 P6: accuracy-gate.yml에 surface_only 추가

Sprint 133에서 만든 `test_klue_dp_eojeol_surface_only`를 CI 게이트로 추가. 다만 본 메트릭은 형태소 분석 변경에 둔감 — 기존 KLUE DP 3-mode 게이트로 충분할 수 있음.

### 검증 기준 (모든 후보 공통)

- `cargo test --workspace --exclude mecab-ko-ffi` 전체 pass
- `cargo clippy --workspace --all-targets --exclude mecab-ko-ffi -- -D warnings` clean
- accuracy-gate CI 통과 (sample.tsv 99.9%+ / KLUE DP floors)
