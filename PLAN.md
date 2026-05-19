# PLAN — mecab-ko Sprint 140 (next)

> 마지막 업데이트: 2026-05-19

## 완료: Sprint 139 P2 — UD Korean-Kaist Silver Baseline 통합

### 변환 결과
- `tools/convert_ud_kaist.py`: 50 KAIST XPOS → Sejong tag 매핑
- 3,124 sentences 변환 (test 1,638 + dev 1,486, 71.8% 변환률)
- 신규: `data/eval/ud_kaist_test.tsv`, `data/eval/ud_kaist_dev.tsv`
- 신규 테스트: `test_ud_kaist_dual_metric`

### Baseline 측정 (test split, 1,638 sentences)

| Metric | UD Kaist | KLUE DP |
|--------|---------|---------|
| Morph strict | 66.3% | 66.8% |
| Morph practical | 68.0% | 71.6% |
| Per-eojeol strict | 20.7% | 20.7% |
| Per-eojeol practical | 21.8% | 23.5% |

**핵심 발견**: morph strict 거의 동일 (mecab 일관). practical lift 차이는 KAIST jcc/JKC vs mecab JKS convention 차이.

### 보고서
`docs/research/accuracy/2026-05-19_sprint139_ud_kaist.md`

## 다음 스프린트: Sprint 140 (미정 — 사용자 선택)

### 후보 A: UD Kaist SPLIT_DIFFERENT 분석

**목적**: Sprint 137에서 KLUE DP에 적용한 connection pair 분석을 UD Kaist에도 적용 → 다른 도메인의 problematic pairs 발견.

**작업**:
1. `test_klue_dp_split_diff_connection_pairs` 변형으로 UD Kaist 분석
2. 두 데이터셋의 problematic pair 교집합/차집합 → 안전한 cost 조정 후보 식별
3. 도메인별 패턴 비교 보고서

**비용**: 0.5-1 sprint
**위험**: 낮음 (분석-only)

### 후보 B: JKC ↔ JKS practical 동치 검토

**목적**: UD Kaist에서 발견된 KAIST jcc(보격) ↔ mecab JKS(주격) convention 차이가 일반적인 lenient 흡수에 해당하는지 검토.

**작업**:
1. KLUE에서 JKC ↔ JKS 동치 추가 시 변화 측정 (KLUE 회귀 확인)
2. UD Kaist에서 lift 측정
3. 추가/제외 결정

**비용**: 0.5 sprint
**위험**: 중간 (KLUE에 negative effect 가능 — JKS/JKC는 의미 있는 case 차이)

### 후보 C: accuracy-gate CI에 UD Kaist 추가

**목적**: 현재 sample.tsv + KLUE DP 두 게이트 → UD Kaist 추가로 3 게이트. Sprint 138 같은 회귀를 더 강하게 감지.

**작업**:
1. `.github/workflows/accuracy-gate.yml`에 step 추가
2. floor 설정: morph strict ≥ 60% / per-eojeol strict ≥ 15%
3. PR comment 형식 확장

**비용**: 0.5 sprint
**위험**: 낮음

### 후보 D [메인 목표]: Full CRF Retrain (Track B)

**선행 조건**: Track C (dict-builder CSV 버그 수정) — Sprint 138 미해결

**작업**: 학습 데이터 (UD Kaist train + KLUE train + Sejong 일부) + mecab-cost-train → matrix.def + left/right-id.def 재생성.

**비용**: 3-5 sprint
**리스크**: 높음 (학습 코퍼스 라이선스, binary 호환성, dict-builder 의존)

### 후보 E [선행]: dict-builder CSV 버그 수정 (Track C)

Sprint 138에서 발견된 Symbol.csv/entries.csv의 쉼표 surface 처리 버그.
**Track B 진입 전 필요**.

**비용**: 0.5-1 sprint
**위험**: 낮음

## 백로그

- P4 (borderline NNG↔NNP): Sprint 132 보류 5 entries + 호스트 73건
- 추가 평가 데이터셋: UD Korean-GSD, OpenKorPOS (Sprint 123 보고서)

## 검증 기준 (모든 후보 공통)

- `cargo test --workspace --exclude mecab-ko-ffi` 전체 pass
- `cargo clippy --workspace --all-targets --exclude mecab-ko-ffi -- -D warnings` clean
- accuracy-gate CI 통과 (sample.tsv 99.9%+ / KLUE DP morph 60%/eo 15% / surface_only strict 50%/canon 80%)
