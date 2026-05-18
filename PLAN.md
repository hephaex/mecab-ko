# PLAN — mecab-ko Sprint 133 (next)

> 마지막 업데이트: 2026-05-18

## 완료: Sprint 132 P1 — KLUE Dictionary Expansion (빈도 2-4)

- klue-domain.csv 18 → **59 entries** (+41 from Sprint 132, 3개 회귀로 제외)
- KLUE DP per-eojeol strict: morph **66.5 → 66.8%** (+0.3pp), eo **53.4 → 53.9%** (+0.5pp)
- 누적 (Sprint 128 → 132): morph **+1.0pp**, eojeol **+1.5pp**
- Sample.tsv 100%/99.9% 무회귀 (CI 게이트가 3개 culprit 즉시 차단)
- 보고서: docs/research/accuracy/2026-05-18_sprint132_dict_expansion.md

### 핵심 학습

빈도 기반 dict 추가 천장 근접 — Sprint 130 (빈도 5+) +0.039pp/entry vs Sprint 132 (빈도 2-4) +0.007pp/entry. 5.4× 효율 차이. 빈도 1회 long tail은 더 낮을 것. 다음 트랙은 다른 접근 필요.

## 다음 스프린트: Sprint 133 (미정 — 사용자 선택)

### 후보 P1: Noisy data 추가

Sprint 131 P4에서 deferred. 데이터셋 선정 조사 (KoBEST, KorQuAD, NIA, SNS). 라이선스 + 도메인 매칭 평가. 조사-only sprint 또는 1개 데이터셋 즉시 통합.

### 후보 P2: eojeol_surface_only metric

검색/인덱싱 use case 전용. Surface concat이 canonical match면 정답. `evaluate_dataset_eojeol_surface_only(tokenizer, dataset, surface_eq)`. Sprint 127 slice-lenient ceiling 87.7%와 연계.

### 후보 P3: 종결어미 normalization 확장

`normalize_endings`에 이ㅂ니다↔이습니다 매핑 추가. Sprint 128 surface lenient +0pp였던 lift 재측정.

### 후보 P4: CRF retrain 인프라 조사

Sprint 129 P3에서 식별한 ~400 cases context-dep 오류. Long-term investment, research-only sprint. Sprint 132에서 도달한 dict 천장 너머 lift의 유일한 경로.

### 후보 P5: 보스턴/외무부 등 borderline NNG↔NNP 해결

Sprint 132에서 보류된 5 entries (보스턴/다운타운/아크로폴리스/외무부/테라스). KLUE convention 차이로 dict override는 회귀 위험. 별도 normalization layer 또는 도메인별 alias 검토.

### 검증 기준 (모든 후보 공통)

- `cargo test --workspace --exclude mecab-ko-ffi` 전체 pass
- `cargo clippy --workspace --all-targets --exclude mecab-ko-ffi -- -D warnings` clean
- accuracy-gate CI 통과 (sample.tsv 99.9%+ / KLUE DP floors)
