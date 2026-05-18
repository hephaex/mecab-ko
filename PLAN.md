# PLAN — mecab-ko Sprint 137 (next)

> 마지막 업데이트: 2026-05-19

## 완료: Sprint 136 — VA/VV lenient + ㄹ불규칙 normalize + CRF 인프라 조사

### Sprint 136 P3 (완료): VA↔VV lenient 동치
- `TAG_EQUIVALENCE_GROUPS_PRACTICAL`에 `["VA","VV"]` 추가
- 측정: practical morph 70.5→71.6% (+1.1pp), practical eo 22.7→23.5% (+0.8pp)
- Conservative는 VA/VV 구분 유지 (정밀 평가용)

### Sprint 136 P3a (완료): ㄹ불규칙 normalize
- `normalize_endings`에 9개 ㄹ불규칙 동사 단방향 패턴 추가
- 따르아→따라, 모르아→몰라, 다르아→달라, 부르어→불러, 흐르어→흘러, 오르아→올라, 자르아→잘라, 누르어→눌러, 고르아→골라
- 측정: surface canonical_lenient 95.4→95.5% (+0.1pp)
- 명시 목록만 (자동 음절 분해 시 false positive 방지)
- 단위 테스트: overcorrect 방지 검증 (푸르러/길러 비매칭)

### Sprint 136 P1 (완료): CRF retrain 인프라 조사
- 보고서: `docs/research/accuracy/2026-05-19_sprint136_crf_retrain_infra.md`
- mecab-ko-dic: feature.def(33줄) + matrix.def(10M) + model.def + left-id(2,693)/right-id(3,822)
- legacy/src에 mecab-cost-train / mecab-dict-gen 빌드 산출물 존재
- Rust 측: `DenseMatrix::from_def_file` 경로 → dict-builder 재실행으로 custom matrix 가능

### Sprint 136 baseline (CI 자동 감시)

| Mode | Sprint 135 | Sprint 136 |
|------|-----------|-----------|
| morph strict | 66.8% | 66.8% |
| morph practical | 70.5% | **71.6%** |
| per-eojeol practical | 22.7% | **23.5%** |
| surface canonical_lenient | 95.4% | **95.5%** |
| sample.tsv Token | 100.0% | 100.0% |
| sample.tsv Sentence | 99.9% | 99.9% |

## 다음 스프린트: Sprint 137 (미정 — 사용자 선택)

> Sprint 136 P1 CRF 조사 기반 권고: Track A 우선, Track C 병행 가능, Track B는 escalation

### 후보 Track A: Connection Cost 부분 조정 실험 [권장 시작]

**목표**: matrix.def에서 problematic 5-10개 (left_id, right_id) 쌍의 cost만 수동 조정. CRF 전 단계 가벼운 실험.

**단계**:
1. KLUE DP per-eojeol 오류 중 SPLIT_DIFFERENT(10.2%)를 (left_id, right_id) 쌍 분포로 매핑
2. 상위 5-10 problematic 쌍 식별 (예: MAG↔NNG, VV↔NNG)
3. matrix.def 수동 수정 → dict-builder 재실행
4. KLUE DP morph + sample.tsv 양쪽 무회귀 검증

**예상 효과**: +0.3 ~ +1.0pp
**리스크**: 낮음 (개별 쌍 격리)
**비용**: 분석 1 sprint + 실험 1 sprint

### 후보 Track C: Inflect.csv ㄹ불규칙 활용형 정적 추가

**목표**: ㄹ불규칙 동사 활용형(따라/몰라/달라 등)을 Inflect.csv에 정적 추가하여 mecab이 직접 인식하도록.
**효과**: SURFACE_MISMATCH 흡수 +0.1-0.3pp + normalize_endings rule 단순화 가능
**리스크**: 낮음
**비용**: 1 sprint

### 후보 Track B (escalation): Full CRF Retrain

**조건**: Track A 효과 < +0.3pp이거나 stakeholder가 더 큰 lift 요구할 때
**비용**: 3-5 sprint (학습 데이터 준비 + mecab-cost-train + 회귀 검증)
**리스크**: 높음 (코퍼스 라이선스, binary 호환성, 학습 split)

### 후보 P2: Noisy data 추가 (Sprint 131부터 deferred)

데이터셋 선정 조사 (KoBEST, KorQuAD, NIA, SNS). 라이선스 + 도메인 매칭 평가.
평가 다양화 + 잠재적 학습 데이터.

### 후보 P4: borderline NNG↔NNP normalization layer

Sprint 132에서 보류된 5 entries (보스턴/외무부). 별도 normalization layer 검토.
호스트 NNP(73건)는 KLUE domain bias 확인 후 결정.

### 검증 기준 (모든 후보 공통)

- `cargo test --workspace --exclude mecab-ko-ffi` 전체 pass
- `cargo clippy --workspace --all-targets --exclude mecab-ko-ffi -- -D warnings` clean
- accuracy-gate CI 통과 (sample.tsv 99.9%+ / KLUE DP morph 60%/eo 15% / surface_only strict 50%/canon 80%)
