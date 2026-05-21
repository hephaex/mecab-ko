# Sprint 154 — 빈도 기반 작업 영역 소진 선언

> **결과**: Sprint 145 빈도 분석의 4개 후보 (EP+ETM/XSV+ETM/VX+EP/XSA+EP) 218건 모두 100% 처리됨. 빈도 기반 splitter rule 작업 영역 소진. 남은 mismatch (KLUE 2237/22404 ≈ 10%)는 CRF/dict cost 레벨 작업 필요.

---

## 1. 통합 진단 (4 후보 동시 측정)

Sprint 148 D, 153 E 교훈 (전문가도 틀릴 수 있음, 빈도 ≠ 실측) 반영하여 4개 후보를 한 번에 측정.

### 측정 결과

| 패턴 | Raw | Split OK | 미처리 |
|------|-----|----------|--------|
| EP+ETM | 86 | 86 (100%) | 0 |
| XSV+ETM | 72 | 72 (100%) | 0 |
| VX+EP | 25 | 25 (100%) | 0 |
| XSA+EP | 35 | 35 (100%) | 0 |
| **합계** | **218** | **218 (100%)** | **0** |

### 실제 split 예시

```
EP+ETM:
  던 → 시/EP + 던/ETM
  실 → 시/EP + ㄹ/ETM
  으실 → 으시/EP + ㄹ/ETM

XSV+ETM:
  헌 → 허/XSV + ㄴ/ETM
  시킬 → 시키/XSV + ㄹ/ETM

VX+EP:
  했 → 하/VX + 았/EP
  못했 → 못하/VX + 았/EP
  왔 → 오/VX + 았/EP

XSA+EP:
  스러웠 → 스럽/XSA + 었/EP   (ㅂ 불규칙 stem)
  했 → 하/XSA + 았/EP
```

모두 mecab dict decomposition fallback이 정확히 처리.

---

## 2. 빈도 분석 한계 명확화

Sprint 145 빈도 분석 (compound POS 빈도) → splitter rule 후보 식별 시도. 결과:

| Sprint | 후보 | 실제 미처리 | 결과 |
|--------|------|----------|------|
| 147 | VV/XSV (lift) | — | practical 동치 (실효 lift) |
| 148 D | ETM+ETM 33 | 0 | 비이슈 (splitter 중복 태그 규칙) |
| 150 A | VA+ETM 542 | 24 | 부분 lift (+0.4pp KLUE strict) |
| 153 E | XSA+ETM 38 | 0 | 비이슈 (converter decomp) |
| **154** | **EP+ETM/XSV+ETM/VX+EP/XSA+EP 218** | **0** | **비이슈** |

**결론**: 빈도 분석 기반 splitter rule 작업 영역 **소진**.

### 왜 그런가?

mecab-ko-dic이 매우 풍부한 `decomposition` features를 제공:
- ㅂ 불규칙 (스럽 → 스러운)
- ㄹ 불규칙 (만들 → 만든)
- ㅎ 불규칙 (노랗 → 노란)
- 시제/존칭 결합 (시/EP + 던/ETM)

`SejongConverter::convert_token` (converter.rs L162-187)이 dict features를 ending_rules보다 먼저 시도하므로, 대부분의 compound POS 패턴이 이미 정확히 분리됨.

빈도 분석은 mecab `raw output`만 측정 → 사전 분해 정보 무시 → 과대 추정.

---

## 3. 진짜 mismatch — CRF 레벨 작업

`test_klue_dp_split_diff_connection_pairs` 측정:
- SPLIT_DIFFERENT eojeols: **2237 / 22404 (~10%)**
- Top connection pairs:

```
(RID=3534, LID=0)   298건  NNG → BOS/EOS    공정성|을, 돌|입, 천명|을
(RID=0, LID=1780)   264건  BOS/EOS → NNG    지|검장, 주|의, 지|옥
(RID=3533, LID=0)   196건  NNG → BOS/EOS    대|한, 위|한, 여|주
(RID=5, LID=1794)   166건  EF → SF          다|., 빠집니다|., 보인다|.
(RID=0, LID=0)      162건  BOS/EOS pair     한|다면, 나|갈
(RID=3561, LID=1780) 134건 SH → NNG         100|여명, 1|천명
```

이 mismatch들은 splitter rule이 아닌 **CRF/matrix cost / dict 확장** 영역.

### 가능한 작업 영역

1. **dict 확장 (Sprint 130, 132 패턴)**: 미처리 NNG/NNP를 user dict로 추가
   - 예: "검장", "공정성을" → 사전 등록 또는 토큰화 비용 조정
2. **CRF Track A** (Sprint 137 분석 활용): 안전한 connection pair 수동 조정
   - Sprint 138 시도 → 회귀 발생 → rollback
3. **Full CRF Retrain** (Track B): 3-5 sprint 비가역 대규모
4. **새 silver dataset**: 구어/SNS 도메인 확장 (NIKL Modu)

---

## 4. 핵심 학습 포인트

### 4.1 빈도 분석의 진정한 가치

빈도 분석 ≠ 작업 후보. 빈도는 단지 **잠재 영역 식별** 도구.

올바른 워크플로우:
1. 빈도 분석 → 후보 식별
2. **변환 후 진단** (splitter + converter 모두)
3. 실제 미처리 ≥ threshold → 작업
4. 그렇지 않으면 → 비이슈, 다음 후보

### 4.2 mecab dict의 위력

mecab-ko-dic 2.1.1의 `decomposition` features가 ending_rules/splitter 코드를 사실상 대체. ㅂ/ㄹ/ㅎ 불규칙 모두 dict이 처리.

→ 사전 자체가 가장 큰 자산. dict 확장 (Sprint 130, 132)이 더 효과적인 lift 경로일 가능성.

### 4.3 통합 진단의 효율성

이전: 후보당 sprint 1회 (148 D, 153 E)
현재: 후보 4개를 1 sprint로 통합 진단

→ 빠른 영역 소진 확인 가능. 후보가 많을 때 통합 진단 우선.

### 4.4 작업 영역 전환 신호

splitter rule 영역 소진 → 다음 단계로 전환:
- 빈도 기반 작업 중단
- CRF/dict 레벨 작업 시작
- 또는 다른 정확도 영역 (surface normalization, 사전 확장)

---

## 5. Sprint 155 방향 (제안)

### 안전한 후보

#### A. dict 확장 — Sprint 130/132 패턴 재방문
- 도메인 NNG/NNP 추가 (KLUE/UD에서 미처리 단어)
- 빈도 측정 + 효과 검증 가능
- 위험: 낮음, lift: 가능

#### B. test_klue_dp_real_error_analysis 활용
- 기존 진단 함수로 실제 오분류 패턴 분석
- 빈도 기반보다 실제 오류 기반 작업

#### C. surface normalization 확장 (Sprint 134 패턴)
- normalize_endings 추가 후보
- canonical 평가 lift 가능

### 비권고

- splitter rule 추가: 영역 소진 확인
- 빈도 기반 후보 추가: 동일 실패 예상

### 비가역 대규모 (사용자 confirm 필요)

- Full CRF Retrain (Track B): 3-5 sprint
- 새 silver dataset (NIKL Modu): 라이선스 + 다운로드 필요

---

## 6. 변경 파일

- `rust/crates/mecab-ko-core/tests/accuracy_eval.rs`: `test_sprint154_unified_diagnosis` 추가
- `docs/research/accuracy/2026-05-21_sprint154_frequency_exhausted.md` (본 문서)
- `PLAN.md`, `PROGRESS.md` 갱신

---

*작성: 2026-05-21 (Sprint 154)*
