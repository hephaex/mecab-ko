# Sprint 138 — Tier A matrix.def 실험 (실패 → rollback)

> **결론**: matrix.def Tier A 수동 cost 조정은 sample.tsv baseline 회귀 회피 불가. 완전 rollback. Sprint 139는 다른 접근 필요.

---

## 실험 설정

### 환경
- 기준 데이터셋: sample.tsv (1,100문장, 99.9% baseline) + KLUE DP val (1,995문장)
- 사전: mecab-ko-dic-2.1.1-20180720 + verb-inflections + klue-domain user-dict
- 도구: 신규 `cargo run --release --example matrix_def_to_bin` (matrix.def → matrix.bin.zst 단독 변환, dict-builder CSV 버그 우회)

### P2 사전 검토 (skip)

ㄹ불규칙 활용형이 이미 Inflect.csv에 모두 존재함을 확인:
- 따라/달라/몰라/불러/흘러/올라/잘라/눌러/골라 — 모두 entries 있음
- mecab이 분해를 선호하는 것은 connection cost 문제 (P1 영역)
- P2 작업 불요

---

## P1 실험 1 — 5쌍 모두 적용

### 조정값 (matrix.def 형식: right_id left_id cost)

| Pair | Before | After | Δ |
|------|--------|-------|---|
| (3534, 0) NNG-T → BOS/EOS | -1504 | -1204 | +300 |
| (3533, 0) NNG-F → BOS/EOS | -1504 | -1204 | +300 |
| (0, 1780) BOS/EOS → NNG | -1133 | -833 | +300 |
| (3534, 1780) NNG-T → NNG | 269 | 769 | +500 |
| (3533, 1780) NNG-F → NNG | 269 | 769 | +500 |

### 결과

| 메트릭 | Before | After | Δ | 평가 |
|--------|--------|-------|---|------|
| sample.tsv Token | **100.0%** | **99.1%** | **-0.9pp** | ❌ baseline 위반 |
| sample.tsv Sentence | 99.9% | (회귀 동반) | - | ❌ |
| KLUE morph strict | 66.8% | 66.9% | +0.1pp | 미미 |
| KLUE eo strict | 20.7% | 21.0% | +0.3pp | 미미 |
| KLUE morph practical | 71.6% | 71.6% | 0 | 영향 없음 |
| KLUE eo practical | 23.5% | 23.7% | +0.2pp | 미미 |

**판정**: ❌ FAIL. sample.tsv -0.9pp 회귀로 baseline assertion 위반.

---

## P1 실험 2 — 부분 적용 (NNG+NNG 2쌍만)

BOS/EOS 경계가 sample.tsv 회귀 원인이라 추정, 어절 내부 NNG+NNG 복합어 쌍만 적용:

| Pair | Before | After |
|------|--------|-------|
| (3534, 1780) NNG-T → NNG | 269 | 769 |
| (3533, 1780) NNG-F → NNG | 269 | 769 |

### 결과

| 메트릭 | Before | After | Δ | 평가 |
|--------|--------|-------|---|------|
| sample.tsv Token | **100.0%** | 99.9% | -0.1pp | ⚠️ 미미 회귀 |
| sample.tsv Sentence | **99.9%** | **99.7%** | **-0.2pp** | ❌ baseline 위반 (>= 99.8%) |
| sample.tsv POS | 99.9% | 99.9% | 0 | OK |

**판정**: ❌ FAIL. NNG+NNG cost 조정만으로도 sample.tsv 2개 문장 추가 회귀.

---

## P1 Final Rollback

- matrix.def → s137-baseline (원본)
- matrix.bin.zst → 재변환으로 원본 복구
- `test_full_accuracy_evaluation`: Token 100.0% / Sentence 99.9% / POS 99.9% 확인
- s137-baseline 백업 파일 삭제 (불필요)

---

## 핵심 학습 포인트

### 1. matrix.def 수동 조정은 너무 거친 도구

**왜 중요한가**:
한 (right_id, left_id) 쌍의 cost는 그 두 context를 가진 모든 노드 조합에 적용됨. 어절 중간 split을 회피하려는 의도가 어절 경계 처리에도 영향. 따라서 "이 어절 케이스만 조정"은 cost 단독으로 불가능.

**적용 원칙**:
matrix.def 수동 조정 전에 (a) 해당 쌍이 sample.tsv에서 얼마나 자주 나타나는지, (b) 어절 내부 vs 경계 분포가 어떤지 사전 분석. 사전 분석 없이 cost 조정은 회귀 위험 높음. 이상적으로는 학습 데이터 기반 CRF retrain만이 trade-off 자동 해결.

### 2. dict-builder CSV 파싱 버그 (별도 이슈)

**현상**: `cargo run --bin mecab-ko-dict-builder` 실행 시 "Invalid left_id at line 4" 에러. entries.csv / Symbol.csv의 13-field 행(쉼표가 surface로 포함된 경우)에서 fail.

**우회**: 신규 `matrix_def_to_bin` example로 matrix.def → matrix.bin.zst 단독 변환.

**향후 작업**: dict-builder의 CSV escape/quote 처리 버그 수정 (별도 이슈로 트래킹 권장).

### 3. KLUE 일부 향상이 sample.tsv 회귀를 정당화하지 않음

**왜 중요한가**:
sample.tsv 99.9% baseline은 Sprint 122에서 어렵게 달성. 회귀는 quality regression. KLUE +0.3pp는 정확도 측면 의미 있으나 baseline 위반 trade-off 부적합. 두 데이터셋의 측정 우선순위가 다름.

**적용 원칙**:
matrix.def 변경 결정 기준: "sample.tsv baseline 무회귀 AND KLUE +lift" 둘 다 만족 시에만 진행. 어느 하나라도 회귀하면 rollback.

---

## Sprint 139 권고 — 다음 단계

### Track B (escalation): Full CRF Retrain

**이유**: cost 조정의 sample.tsv 회귀 문제는 학습 데이터 부재 때문. CRF는 학습 데이터 분포에서 자동 최적화 → trade-off 자동 해결.

**비용**: 3-5 sprint
- 학습 코퍼스 준비 (Sejong + KLUE DP train + sample 도메인)
- mecab-cost-train 실행 (C++, legacy/src)
- model.def → matrix.def + left/right-id.def 재생성
- dict-builder CSV 버그 수정 (선행 필요)
- 전체 회귀 검증

**리스크**:
- 학습 코퍼스 라이선스
- left/right-id.def 변경 시 사전 binary 호환성
- dict-builder CSV 버그 우회/수정 필요

### Track A 대안: 더 세분화된 cost 분석

**아이디어**: 어절 내부에서만 적용되는 cost 분석 (예: BOS 이후 N>=2 노드 이전의 context는 어절 내부)

**문제**: mecab의 matrix는 (left_id, right_id) 차원만 가짐 — 위치 정보 없음. 즉 어절 내부 vs 경계 cost 구분 불가. 이 한계 때문에 Track A는 본질적으로 한계 있음.

### Track C 대안: dict-builder CSV 버그 수정 (선행 작업)

**효과**: Track B/A 다음 실험을 위한 인프라 정리.
**비용**: 0.5-1 sprint
**우선순위**: 중간 (Track B 진입 전 필요)

---

## 인프라 결과물

**신규 도구**: `rust/crates/mecab-ko-dict/examples/matrix_def_to_bin.rs`
- `cargo run --release --example matrix_def_to_bin -p mecab-ko-dict -- <in.def> <out.bin.zst>`
- dict-builder 우회. matrix.def 텍스트만 변경 시 사용.
- Sprint 139+ Track B 검증에도 활용 예정.

---

## 측정값 (변경 없음 — rollback)

| 메트릭 | Sprint 137 | Sprint 138 | 비고 |
|--------|-----------|-----------|------|
| morph strict | 66.8% | 66.8% | rollback 후 baseline 동일 |
| morph practical | 71.6% | 71.6% | |
| per-eojeol practical | 23.5% | 23.5% | |
| surface canonical_lenient | 95.5% | 95.5% | |
| sample.tsv Token | 100.0% | 100.0% | |
| sample.tsv Sentence | 99.9% | 99.9% | |

---

*작성: 2026-05-19 (Sprint 138)*
