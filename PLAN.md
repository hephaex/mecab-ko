# PLAN — mecab-ko Sprint 141 (next)

> 마지막 업데이트: 2026-05-19

## 완료: Sprint 140 — UD Kaist 분석 + CI 게이트 (A + C)

### A: UD Kaist SPLIT_DIFFERENT pair 분석
- `test_ud_kaist_split_diff_connection_pairs` 신규
- 1,755 SPLIT_DIFFERENT, 479 unique pairs, 3,134 occurrences
- KLUE vs UD 비교: NNG 5쌍 도메인 독립적 (Sprint 138 회귀 결론 재확인) + UD 특화 XSN(적) 패턴 발견

### C: accuracy-gate CI에 UD Kaist 추가
- 4번째 게이트: morph strict ≥ 60% floor
- PR comment 4번째 섹션 (UD Korean-Kaist Silver)
- Sprint 138 회귀 같은 cost 조정 변화를 두 도메인 동시 감지

### convert_ud_kaist.py 수정
- text reconstruct from token forms (eojeol/morpheme alignment 보장)
- 이전 변환은 UD CoNLL-U metadata text 사용 → punctuation 분리 차이로 misalignment

### 보고서
`docs/research/accuracy/2026-05-19_sprint140_ud_kaist_pair_analysis.md`

## 다음 스프린트: Sprint 141 (미정 — 사용자 선택)

### 후보 A: XSN(적) practical 동치 검토

**근거**: UD Kaist 특화 92건 (KLUE 27건). XSN-T(적) → VCP(인) 패턴.
"X적인" 분해가 학술 문체에서 일반적. KLUE에는 적게 등장.

**작업**:
1. XSN/NNG 동치 추가 시도 (`TAG_EQUIVALENCE_GROUPS_PRACTICAL`)
2. UD lift + KLUE 회귀 측정
3. lift > 회귀이면 commit

**비용**: 0.5 sprint
**위험**: 낮음

### 후보 B: UD Kaist 특화 cost 조정 실험

**근거**: XSN/NNG 관련 cost 조정. KLUE보다 회귀 위험 낮을 가능성.

**작업**:
1. matrix.def에서 (3777, 2240), (3533, 2609), (3534, 2609) 쌍 cost 조정
2. 4-gate (sample.tsv + KLUE + surface + UD) 동시 검증
3. 조건: 4 gate 모두 무회귀 + UD lift > 0

**비용**: 0.5-1 sprint
**위험**: 중간 (cost 조정은 항상 회귀 잠재)

### 후보 C: dict-builder CSV 버그 수정 (Track D 선행)

Sprint 138에서 발견된 Symbol.csv/entries.csv 쉼표 surface 처리 버그.
**Full CRF retrain (Track D) 진입 전 필요**.

**비용**: 0.5-1 sprint
**위험**: 낮음

### 후보 D: Full CRF Retrain (escalation, 선행 C 필요)

학습 데이터 (Sejong + KLUE + UD Kaist train) + mecab-cost-train → matrix.def 재생성.
3-5 sprint, 학습 데이터 라이선스 + binary 호환성 리스크.

### 후보 E: NIKL Modu 평가 추가 (manual download)

Korean Language Institute의 morphological corpus. Academic only license.
- 변환 스크립트 + 평가 통합 (단, redistribute 불가, 로컬-only)
- 비용 0.5-1 sprint, 위험 낮음

## 백로그

- P4 (borderline NNG↔NNP): Sprint 132 보류 5 entries
- UD Korean-GSD (CC BY-SA 4.0, 6,339 sentences): UD Kaist와 같은 변환기 재사용 가능

## 검증 기준 (모든 후보 공통)

- `cargo test --workspace --exclude mecab-ko-ffi` 전체 pass
- `cargo clippy --workspace --all-targets --exclude mecab-ko-ffi -- -D warnings` clean
- 4-gate CI 통과 (sample.tsv 99.9%+ / KLUE morph 60%/eo 15% / surface_only strict 50%/canon 80% / UD Kaist morph 60%+)
