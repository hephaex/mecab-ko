# PLAN — mecab-ko Sprint 129

> 마지막 업데이트: 2026-05-16

## 현재 스프린트: Sprint 129 P3 — 진짜 분석 오류 디버그 (carryover)

### 배경

Sprint 126 P1에서 NNG/NNP/NNB confusion 809건을 분류:
- NNB↔NNG counter words: 158건 (convention)
- SL↔NNP foreign abbr: 52건 (convention)
- **NNG/NNP: 242건 (real error)**
- **MAG/NNG: 95건 (real error)**
- **VV/NNG: 43건 (real error)**

Sprint 127 P1에서 GOLD_SINGLE_PRED_MULTI 553건(2.5%) 추가 식별 — mecab이 단일 단어를 분할.

진단 추정(~85-90%) 대비 KLUE DP morpheme 65.8%는 14pp 부족. 이중 25%+는 진짜 오류로 추정.
Sprint 129 P3는 이 real error 380건+553건을 surface 단위로 추출·분류하여 처방을 정한다.

### 목표

각 confusion pattern을 surface 단위 frequency 정렬로 추출하고, 처방별 분류:
- **사전 추가**: 누락 NNP 추가로 해결 가능 (cost=-5000 패턴)
- **cost 조정**: 사전에 있지만 분할이 우선 — entry cost 또는 cost factor 조정
- **CRF retrain**: context 의존, 정적 사전으론 불가
- **convention 차이**: KLUE 라벨링 규약 차이 (무시)

### 작업 목록

- [ ] **S129P3-01** (implement): MAG↔NNG, VV↔NNG confusion analysis 확장
  - 기존 `test_klue_dp_nng_nnp_analysis`와 동일 패턴
  - target_tags 확장 또는 별도 test로 분리
  - frequency 정렬된 surface 리스트 + 샘플 문장

- [ ] **S129P3-02** (implement): GOLD_SINGLE_PRED_MULTI surface frequency
  - 553건 surface를 frequency 정렬
  - 어떤 단일 단어가 어떻게 분할되는지 (예: "한국전자통신연구원" → "한국/NNP + 전자/NNG + 통신/NNG + 연구원/NNG")
  - 각 surface에 대한 mecab 분할 결과 캡처

- [ ] **S129P3-03** (test): 분석 실행
  - `cargo test --release -- --ignored --nocapture test_klue_dp_*_analysis`
  - 출력을 `/tmp/sprint129_analysis_*.txt`에 저장

- [ ] **S129P3-04** (analysis): 처방 분류
  - 각 confusion pattern을 (a)/(b)/(c)/(d)로 분류
  - 정량 추정: 각 처방으로 흡수 가능한 morpheme accuracy lift
  - 사전 추가 후보 surface 리스트 (별도 파일 가능)

- [ ] **S129P3-05** (docs): 보고서 작성
  - `docs/research/accuracy/2026-05-16_klue_dp_real_errors.md`
  - 카테고리별 분포 표, 샘플, 처방 분류, lift 추정
  - Sprint 130 권고 (어느 처방을 우선 적용할지)

- [ ] **S129P3-06** (commit + push + memory)
  - 검증 (cargo test + clippy)
  - commit: `feat(eval): Sprint 129 P3 - real error analysis + fix categorization`
  - memory update (project_sprint_status.md)
  - push (Sprint 128 b496663, 7f26cd3 함께)

### 검증 기준

- `cargo test --workspace --exclude mecab-ko-ffi` 전체 pass
- `cargo clippy --workspace --all-targets --exclude mecab-ko-ffi -- -D warnings` clean
- 보고서가 380+553건의 surface 분포를 정량적으로 보고
- 처방별 분류 명확 (사전 추가 후보 N건, cost 조정 N건, CRF N건, convention N건)

### 보류 (Sprint 130+)

- P1: `eojeol_surface_only` metric (검색/인덱싱 use case)
- P2: 종결어미 normalization 확장 (이ㅂ니다 ↔ 이습니다)
- P4: Noisy data 추가 + CI integration (HF auto-download + KLUE DP 3-mode gate)
