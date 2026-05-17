# PLAN — mecab-ko Sprint 132 (next)

> 마지막 업데이트: 2026-05-18

## 완료: Sprint 131 P4 — Accuracy Gate CI 인프라 수정 + 확장

### 변경

1. **기존 sample.tsv 게이트 수정** (조용히 깨져있던 상태)
   - `--ignored` 플래그 추가 (기존: ignored 테스트 스킵)
   - mecab-ko-dic 다운로드 + 빌드 단계 추가 (dict-build.yml 패턴 재사용)
   - `--exact test_accuracy_gate` 사용 (verified 테스트 제외, Sprint 122 baseline만)
   - `actions/cache` v4로 dict 캐싱 (rebuild 회피)

2. **KLUE DP 3-mode 게이트 신규 추가**
   - `test_klue_dp_dual_metric` + `test_klue_dp_dual_metric_lenient` 실행
   - Floor: morpheme ≥ 60%, eojeol ≥ 15% (기존 assertion 활용)
   - `--test-threads 1`로 출력 순서 결정성 확보

3. **PR comment 통합**
   - Sample.tsv (Token / Sentence) + KLUE DP (Strict morph/eo, Practical morph/eo)
   - Overall passed/failed 표시

### 검증 (로컬)
- regex 추출 모두 정확: STRICT_MORPH=66.5, STRICT_EO=20.1, PRAC_MORPH=71.0, PRAC_EO=22.7
- TOKEN=100.0, SENTENCE=99.9 (sample.tsv)
- actionlint shellcheck warnings 0건

## 다음 스프린트: Sprint 132 (미정 — 사용자 선택)

### 후보 P1: 빈도 2-4 surfaces 확대 (Sprint 130 후속)

Sprint 129 P3에서 식별한 빈도 2-4 후보 ~54 surfaces 검토.
- 안전 부분 (~30 surfaces): 갑자기, 그대로, 이미, 너무나, 비서실장, 초등학교, 상수도, 새누리당, 민주통합당 등
- Homonym 모호성 검토 필요 (~24 surfaces)

추정 lift: +0.3-0.5pp morpheme.

### 후보 P2: Noisy data 추가

Sprint 131 P4에서 deferred. KLUE 외 추가 데이터셋 (Twitter, SNS, 비표준 텍스트). 라이선스 + 데이터셋 선정 조사 우선.

### 후보 P3: eojeol_surface_only metric

검색/인덱싱 use case 전용. Surface concat이 canonical match면 정답.

### 후보 P4: 종결어미 normalization 확장

`normalize_endings`에 "이ㅂ니다" ↔ "이습니다" 매핑 추가.

### 후보 P5: CRF retrain 인프라 조사

장기 투자 (research-only). Sprint 132+ 실시.

### 검증 기준 (모든 후보 공통)

- `cargo test --workspace --exclude mecab-ko-ffi` 전체 pass
- `cargo clippy --workspace --all-targets --exclude mecab-ko-ffi -- -D warnings` clean
- accuracy-gate CI 통과 (sample.tsv 99.9%+ / KLUE DP floors)
