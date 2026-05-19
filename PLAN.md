# PLAN — mecab-ko Sprint 139 (next)

> 마지막 업데이트: 2026-05-19

## 완료: Sprint 138 — Tier A matrix.def 실험 (실패 → rollback)

**P1 결과**: ❌ matrix.def 5쌍 cost +300~+500 조정 → sample.tsv Token -0.9pp 회귀. 부분 적용 (NNG+NNG 2쌍만) → sample.tsv Sentence -0.2pp 회귀. **둘 다 baseline 위반 → 완전 rollback**.

**P2 결과**: ⏭️ ㄹ불규칙 활용형 9개 모두 Inflect.csv에 이미 존재 (skip).

### Sprint 138 신규 인프라

- `rust/crates/mecab-ko-dict/examples/matrix_def_to_bin.rs`: matrix.def → matrix.bin.zst 단독 변환 도구 (dict-builder CSV 버그 우회)
- 보고서: `docs/research/accuracy/2026-05-19_sprint138_tier_a_experiment.md`

### Sprint 138 핵심 학습

1. matrix cost 조정은 어절 내부/경계 구분 불가 → 어떤 조정도 sample.tsv 회귀 위험
2. dict-builder CSV 파싱 버그 발견 (Symbol.csv/entries.csv의 쉼표 surface 행 → "Invalid left_id at line 4")
3. **다음 +1pp 이상 lift는 Full CRF Retrain만 가능** (학습 데이터 기반 trade-off 자동 해결)

## 다음 스프린트: Sprint 139 (미정 — 사용자 선택)

### Track C [선행 권장]: dict-builder CSV 버그 수정

**목적**: Track B 진입 전 인프라 정리. mecab-ko-dic CSV의 쉼표 surface 행을 dict-builder가 올바르게 파싱하도록 수정.

**작업**:
1. csv_parser 모듈에서 quote/escape 처리 확인
2. Symbol.csv/entries.csv 13-field 행 (surface가 쉼표) 처리 추가
3. mecab-ko-dic 재빌드 round-trip 검증 (baseline 회귀 0)

**비용**: 0.5-1 sprint
**리스크**: 낮음 (parser 수정만)

### Track B [메인 목표]: Full CRF Retrain

**목적**: matrix cost 조정의 sample.tsv 회귀 문제를 학습 데이터 기반으로 해결.

**작업**:
1. 학습 코퍼스 준비 (Sejong tagged + KLUE DP train + 추가 도메인)
2. `legacy/src/mecab-cost-train` (C++) 빌드 + 실행
3. 새 `model.def` → `matrix.def` + `left/right-id.def` 재생성
4. Rust dict-builder 재실행 → binary 재생성
5. 전체 회귀 검증 (sample.tsv + KLUE 4-mode)

**비용**: 3-5 sprint
**리스크**: 높음 (학습 데이터 라이선스, binary 호환성, dict-builder 의존)
**선행 조건**: Track C 완료 (CSV 버그 수정)

### Track A [보류]: 세분화 cost 분석

**상태**: 본질적 한계로 보류. mecab matrix는 (left_id, right_id) 차원만 가짐 — 위치 정보(어절 내부 vs 경계) 없음. cost 조정으로는 trade-off 자동 해결 불가.

### 백로그 (deferred)

- P2 (noisy data): KoBEST/KorQuAD/SNS 도메인 평가 추가
- P4 (borderline NNG↔NNP): Sprint 132 보류 5 entries + 호스트 73건

## 검증 기준 (모든 후보 공통)

- `cargo test --workspace --exclude mecab-ko-ffi` 전체 pass
- `cargo clippy --workspace --all-targets --exclude mecab-ko-ffi -- -D warnings` clean
- accuracy-gate CI 통과 (sample.tsv 99.9%+ / KLUE DP morph 60%/eo 15% / surface_only strict 50%/canon 80%)
