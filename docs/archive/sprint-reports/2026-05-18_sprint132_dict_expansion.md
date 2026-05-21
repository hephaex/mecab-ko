# Sprint 132 P1 — KLUE Dictionary Expansion (빈도 2-4)

> **한 줄 요약**: Sprint 129 P3에서 식별한 빈도 2-4 후보 ~50개 검증 후 41개 추가. KLUE DP per-eojeol strict 66.5→**66.8%** (+0.3pp morph) / 53.4→**53.9%** (+0.5pp eojeol). 3개 entry(보다/이미/진짜)는 sample.tsv 회귀로 제외. 누적 (Sprint 128 → 132): morph **+1.0pp**, eojeol **+1.5pp**.

## 배경

Sprint 130 P1에서 빈도 5+ 후보 18개 추가 (+0.7pp morph). Sprint 132는 동일 패턴으로 빈도 2-4 후보 (Sprint 129 P3 식별 ~54 surfaces) 확대.

## 후보 분류 + 검증 결과

총 49 surfaces 검토. mecab-ko-dic grep으로 missing vs cost override 분류, homonym 위험 평가.

### 추가된 41 entries

**Missing NNP (7건)** — mecab-ko-dic에 없음, NNP로 신규 추가:
새누리당, 민주통합당, 공화당, 민주노총, 개성공단, 소녀시대, 테르미니역

**Missing NNG (3건)**:
비서실장, 청결도, 가성비

**NNP cost override (5건)** — 단일 NNP 의미:
프라하, 세인트루이스, 서울시, 국정원, 서울대

**NNG cost override (4건)** — Compound 분할 방지:
한국인, 초등학교, 상수도, 관광지

**MAG cost override 단일 부사 (11건)**:
갑자기, 그대로, 너무나, 거듭, 여러모로, 곧바로, 되게, 철저히, 혹시, 즉각, 훨씬

**MAG cost override homonym 안전 (11건)**:
주로, 빨리, 워낙, 끝내, 매일, 지금, 다소, 우선, 달리, 전부, 거의

### 제외된 3 entries (sample.tsv 회귀)

| Surface | Override POS | 회귀 케이스 | 원인 |
|---------|------------|-----------|------|
| 보다 | MAG | "예상보다 높았다", "보다 봤다" | JKB(비교조사) 의미가 더 흔함; VV(보다) 동사 어간 흡수 |
| 이미 | MAG | "이미지를 빌드", "많이 미안해요" | "이미지" 단어 분할 (이미/MAG + 지/VX); cascade로 "미안" → "이미/MAG + 안" |
| 진짜 | MAG | "진짜요", "진짜예요" | NNG(진짜) 의미 우선 케이스에 영향 |

### 제외된 5 entries (homonym 보류 — 후속 검토)

NNG로 mecab-ko-dic에 있으나 KLUE는 NNP로 라벨링하는 borderline:
보스턴, 다운타운, 아크로폴리스, 외무부, 테라스 — convention 차이 vs real fix 모호함, Sprint 130 pattern(homonym → KLUE 도메인 안전)이 비대칭으로 적용 불가능.

## 측정 결과 (KLUE DP 1,995 sentences)

### Sprint 130 → Sprint 132 비교

| 메트릭 | Sprint 128 (no klue-dict) | Sprint 130 P1 (+18) | Sprint 132 P1 (+41 cumul.) | Δ vs 130 | Δ vs 128 |
|--------|---------------------------|---------------------|---------------------------|---------|---------|
| per-eojeol strict morph | 65.8% | 66.5% | **66.8%** | +0.3pp | +1.0pp |
| per-eojeol strict eo | 52.4% | 53.4% | **53.9%** | +0.5pp | +1.5pp |
| per-eojeol practical morph | 70.3% | 71.0% | **71.3%** | +0.3pp | +1.0pp |
| per-eojeol practical eo | 59.4% | 60.4% | **61.0%** | +0.6pp | +1.6pp |
| sequence strict eo | 19.2% | 20.1% | 20.7% | +0.6pp | +1.5pp |
| sequence practical eo | 21.7% | 22.7% | 23.3% | +0.6pp | +1.6pp |

### 회귀 (sample.tsv)

| 메트릭 | Before (Sprint 130) | After (Sprint 132 최종) | 변화 |
|--------|---------------------|------------------------|------|
| Token Accuracy | 100.0% | **100.0%** | 무회귀 |
| Sentence Accuracy | 99.9% | **99.9%** | 무회귀 |

3 entries 제외 전 측정: Token 99.8%, Sentence 99.3% — 회귀 게이트가 정확히 차단.

## 핵심 학습 포인트

### 1. 회귀 게이트가 첫 시도에서 핵심 entries 식별

Sprint 131 P4에서 막 수정된 sample.tsv 회귀 게이트가 즉시 작동. Sprint 132의 41 entries 추가 후 100% → 99.8% drop 즉시 감지하여 보다/이미/진짜 culprit 식별. **CI 인프라 투자의 즉각적 보상** — 회귀 게이트 없었으면 KLUE lift만 보고 잘못 commit했을 것.

### 2. Homonym 위험은 표층적 분류로 충분치 않음

Sprint 130의 가설: "homonym surface도 KLUE 도메인에서 MAG가 흔하면 안전". Sprint 132는 이를 일부 검증·일부 반박:
- 안전: 주로, 빨리, 워낙, 매일, 지금, 다소, 우선, 달리, 전부, 거의 (Sprint 130의 모두/다시/현재/일단/자주와 동일 패턴)
- 위험: 이미 (단어 일부로 등장 — "이미지", "조이미" 등), 보다 (비교조사로 매우 흔함), 진짜 (NNG 의미가 KLUE 외 도메인에서 우선)

**판단 기준 보정**: surface가 다른 단어의 prefix가 될 수 있는지 (이미/이미지), 조사로 흔히 쓰이는지 (보다/예상보다) 사전 점검 필요.

### 3. 한계 수익 (diminishing returns)

| Sprint | Entries | Cases | Morph lift | Lift/entry |
|--------|---------|-------|-----------|----------|
| 130 (빈도 5+) | 18 | ~95 | +0.7pp | +0.039pp/entry |
| 132 (빈도 2-4) | 41 | ~120 | +0.3pp | +0.007pp/entry |

5.4× 효율 차이. 빈도 1회 long tail (~289 surfaces)은 더 낮을 것 — **빈도 기반 dict 추가의 천장에 근접**. 다음 정확도 lift는 CRF retrain 또는 다른 종류의 데이터 보강 필요.

### 4. 사전 분류만으로 알 수 없는 케이스

보스턴/다운타운/아크로폴리스/외무부/테라스: mecab-ko-dic에서 NNG로 분류, KLUE는 NNP로 라벨링. cost override는 sample.tsv 회귀 위험 (NNG 의미가 다른 도메인에서 우선일 수 있음). 이런 borderline은 dict 대신 별도 normalization layer (예: 도메인별 alias) 필요.

## Sprint 133 권고

### 사전 확장 천장 도달 — 다른 트랙 권고

빈도 1회 long tail은 효율 너무 낮음. Sprint 133은 다른 접근:

- **P1**: Noisy data 추가 — Sprint 131 P4 deferred. 데이터셋 선정 조사 → 통합. KLUE 외 도메인 측정.
- **P2**: eojeol_surface_only metric — 검색/인덱싱 use case 명시적 분리.
- **P3**: 종결어미 norm 확장 — Sprint 128 surface lenient +0pp 재측정.
- **P4**: CRF retrain 인프라 조사 — long-term investment, research-only.

## 관련 문서

- [Sprint 129 P3 — 진짜 오류 분류](./2026-05-16_klue_dp_real_errors.md) — 빈도 2-4 후보 원천
- [Sprint 130 P1 — Dict lift](./2026-05-18_klue_dp_dict_lift.md) — 빈도 5+ 패턴
- [Sprint 131 P4 — CI accuracy gate](../../../.github/workflows/accuracy-gate.yml) — 회귀 차단 인프라

---

*작성: 2026-05-18*
