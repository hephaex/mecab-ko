# Sprint Learnings — mecab-ko Accuracy Improvement (Sprint 122~158)

> 30+ sprint의 핵심 학습. 상세 sprint 보고서는 `docs/archive/sprint-reports/`.

## 누적 지표

| Metric | Sprint 122 | Sprint 158 | Δ |
|--------|-----------|-----------|---|
| sample.tsv | 100%/99.9% | 100%/99.9% | — (baseline 보존) |
| **KLUE morph practical** | ~65.8% | **72.1%** | **+6.3pp** |
| **KLUE surface canonical_lenient** | ~89% | **95.6%** | **+6pp** |
| UD Kaist morph practical | — | 68.6% | (new silver) |
| UD GSD morph practical | — | 71.8% | (new silver) |

## 영역별 접근 정리

### ✅ 안전 영역 (효과 검증됨)

#### 1. PRACTICAL 동치 그룹 (`TAG_EQUIVALENCE_GROUPS_PRACTICAL`)

POS scheme convention 차이 흡수. Conservative 정밀 보존 + Practical downstream 활용.

| Sprint | 동치 추가 | 언어학적 배경 |
|--------|----------|--------------|
| 126 | NNB↔NNG | counter words ("일/월/명/씨") |
| 136 | VA↔VV | "있다" 형용사/동사 분류 |
| 147 | VV↔XSV | "하/되" 본동사/접사 분류 |
| 157 | MAG↔MAJ | 일반/접속 부사 분류 |

#### 2. Surface normalization (`normalize_endings`)

명시 stem 목록 (false positive 방지). 단방향 변환 (gold/pred 모두 정규화).

| Sprint | 패턴 | 효과 |
|--------|------|------|
| 128 | 하았/하어/하아 | +22.6% mismatch 흡수 |
| 134 | 이습니다, 르 불규칙 9 | +1.0pp surface_only |
| 136 | (Sprint 134와 연계) | |
| 156 | ㄷ 불규칙 9 + 르 1 | +30 eojeols |
| 158 | 명시 어구 3 | +10 eojeols |

#### 3. Splitter rule (제한적)

mecab dict이 처리 못한 패턴만. VV+ETM은 위험 (Sprint 145 회귀).

| Sprint | 패턴 |
|--------|------|
| 141 | VCP+ETM/EC (인, 일, 라) |
| 146 | VCP+EP "였" |
| 150 A | VA+ETM multi-syllable ㄴ jongseong |

#### 4. Silver dataset 통합

도메인 coverage 확장 + 동일 lift 검증.

| Sprint | Dataset |
|--------|---------|
| 139 | UD Korean-Kaist (1638 sentences) |
| 143 | UD Korean-GSD (971 sentences) |
| 159 F | NIKL Modu (인프라만, 수동 다운로드) |

### ❌ 위험 영역 (회귀 사례)

모든 시도가 **viterbi/CRF cascade 영향**.

| Sprint | 시도 | 결과 |
|--------|------|------|
| 138 | matrix.def 수동 cost 조정 | -0.9pp sample.tsv → rollback |
| 145 D | multi-syllable VV+ETM splitter | -1 sentence sample.tsv → rollback |
| 155 A | dict cost=-5000 NNP 9건 | -0.2pp KLUE → rollback |

**공통점**: viterbi path를 직접 변경 → 의도하지 않은 cascade.

### ⏸ 비이슈 영역 (mecab dict이 처리)

빈도 분석으로 "미처리 같지만" 실제로는 mecab decomposition fallback이 처리.

| Sprint | 패턴 | 실제 미처리 |
|--------|------|------------|
| 148 D | ETM+ETM "라는" 33건 | 0 (splitter 중복 태그) |
| 153 E | XSA+ETM 38건 | 0 (converter decomp) |
| 154 | 4 후보 (EP+ETM/XSV+ETM/VX+EP/XSA+EP) 218건 | 0 (모두 처리됨) |

→ 빈도 기반 splitter rule 영역 **소진 선언**.

## 메타 학습

### 1. mecab dict의 위력

`SejongConverter::convert_token` (converter.rs L162-187)의 decomposition fallback이
ending_rules보다 먼저 시도됨. 사전 features가 ㅂ/ㄹ/ㅎ 불규칙까지 처리.

### 2. 빈도 분석 ≠ 실효 lift

raw mecab POS 빈도는 작업 후보 식별만 가능. 실제 영향은 splitter+converter 변환 후 측정 필요.

**올바른 워크플로우**:
1. 빈도 분석 → 후보 식별
2. 변환 후 진단 (splitter + converter)
3. 실제 미처리 ≥ threshold → 작업
4. 그렇지 않으면 → 비이슈 문서화

### 3. 3 silver 일관 lift = 진짜 효과

도메인 독립 lift는 신뢰도 높음. 단일 도메인 anomaly는 false signal 가능.

| 예시 | Sprint | 결과 |
|------|--------|------|
| VV/XSV practical | 147 | 3 silver +0.2~0.4pp |
| MAG/MAJ practical | 157 | 3 silver +0.2pp |

### 4. Rollback 의사결정 신속화 (Sprint 138 정책)

sample.tsv 회귀 발견 즉시 rollback (분석은 rollback 후). 시간 절약.

### 5. 진단 데이터 재활용

같은 진단 데이터로 여러 접근 가능 (Sprint 155 진단 → A 실패 → G 성공).

## 워크플로우 정착

### Sprint 154 이후 (자동 트랙 선택, 규칙 5)

```
1. 전문가 리뷰 (rust-pro agent)
   ↓
2. Top 권고 자동 채택 (사용자 question 없음)
   ↓
3. 진단 우선 (raw → splitter → converter 변환 후)
   ↓
4. 실제 미처리 ≥ 10건이면 구현
   ↓
5. 5-gate 검증 (sample.tsv 무회귀 hard rule)
   ↓
6. 회귀 시 즉시 rollback
   ↓
7. 결과 문서화 + commit + push
```

## 영역 소진 후 다음 단계

Sprint 159 시점에서 안전 영역 거의 소진:
- splitter rule: 소진 (mecab dict이 처리)
- dict cost 확장: 회귀
- CRF matrix: 회귀
- surface normalization: ≤ +0.05pp/sprint
- PRACTICAL 동치: 추가 후보 부족

**남은 옵션** (비가역, 사용자 confirm 필요):
- NIKL Modu 도입 (Sprint 159 F 인프라 준비 완료)
- Full CRF Retrain (Track B, 3-5 sprint)
- 정확도 외 영역 (문서, CLI, 성능, 바인딩)

---

*작성: 2026-05-21 (Sprint 160) — 30+ sprint 종합 학습*
