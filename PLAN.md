# PLAN — mecab-ko Sprint 131

> 마지막 업데이트: 2026-05-18

## 완료: Sprint 130 P1 — KLUE Domain Dictionary

- klue-domain.csv 18 entries (cost=-5000)
- KLUE DP morpheme **65.8% → 66.5%** (+0.7pp), per-eojeol **52.4% → 53.4%** (+1.0pp)
- Sample.tsv 100%/99.9% 무회귀
- 보고서: docs/research/accuracy/2026-05-18_klue_dp_dict_lift.md

## 다음 스프린트: Sprint 131 (미정 — 사용자 선택)

### 후보 P1: 빈도 2-4 surfaces 확대 (Sprint 130 후속)

Sprint 129 P3에서 식별한 빈도 2-4 후보 ~54 surfaces 검토.
- 안전 부분 (~30 surfaces): 갑자기, 그대로, 이미, 너무나, 비서실장, 초등학교, 상수도, 새누리당, 민주통합당 등
- Homonym 모호성 검토 필요 (~24 surfaces)

추정 lift: +0.3-0.5pp morpheme. Sprint 130 패턴 반복.

### 후보 P2: eojeol_surface_only metric

검색/인덱싱 use case 전용 메트릭. Surface concat이 canonical match면 정답 (POS/split 무시).
함수: `evaluate_dataset_eojeol_surface_only(tokenizer, dataset, surface_eq)`.

### 후보 P3: 종결어미 normalization 확장

`normalize_endings`에 "이ㅂ니다" ↔ "이습니다" 매핑 추가.
Sprint 128에서 +0pp였던 surface lenient의 추가 lift 측정.

### 후보 P4: Noisy data + CI integration

사용자 high priority 잔여. 노이지 데이터셋 추가 + HF auto-download + KLUE DP 3-mode CI gate.
인프라 작업.

### 후보 P5: CRF retrain 인프라 조사

Sprint 129 P3에서 식별한 ~400 cases context-dep 오류 해결을 위한 학습 인프라.
대규모 — 조사부터 시작 권고.

### 검증 기준 (모든 후보 공통)

- `cargo test --workspace --exclude mecab-ko-ffi` 전체 pass
- `cargo clippy --workspace --all-targets --exclude mecab-ko-ffi -- -D warnings` clean
- sample.tsv Token 100% / Sentence 99.9% 유지 (회귀 게이트)
