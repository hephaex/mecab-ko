# Sprint 174 — Sprint Cycle 공식 종료 + 유지보수 모드 선언

> **결정**: Sprint 122~173 (52 sprints) 누적 자동 진행 가능 영역 모두 소진. 유지보수 모드 진입. 다음 메이저 작업 (NIKL Modu 다운로드, Sejong 입수, 사용자 명시 신규 영역)까지 sprint-run 정지.

---

## 1. Sprint Cycle 총결산 (Sprint 122 → 173)

### 정확도 lift 누적

| Metric | Sprint 122 baseline | Sprint 173 현재 | Δ |
|--------|-------------------|---------------|---|
| sample.tsv | 100%/99.9% | 100%/99.9% | — (보존) |
| **KLUE morph practical** | ~65.8% | **72.1%** | **+6.3pp** |
| KLUE eojeol practical | ~5000 | 5327 | +327 |
| **KLUE surface canonical_lenient** | ~89% | **95.6%** | **+6pp** |
| UD Kaist morph practical | — | 68.6% | (new silver, S139) |
| UD GSD morph practical | — | 71.8% | (new silver, S143) |

### 인프라/품질 누적

| 영역 | Δ |
|------|---|
| accuracy_eval.rs 줄 수 | 4963 → 2406 (**-51%**) |
| WASM tests | 5 → 11 (+120%) |
| Docs 정리 | 28+ 파일 archive |
| CI gate | sample.tsv → 5-gate (KLUE/surface/UD×2) |
| 5-gate CI | MSRV/coverage/continue-on-error 정리 |
| 성능 baseline | v0.7.2 5/9 benches 측정 |

### Sprint 분류 (52 sprints, ~1.5 개월)

| 유형 | 건수 | 효과 |
|------|------|------|
| Lift sprints | 11 | +6.3pp 누적 |
| Infrastructure | 12 | 5-gate CI, 3 silver, helper, CI 강화 |
| Rollback (viterbi/CRF) | 4 | Sprint 138/145/155/167 |
| 비이슈 확인 | 3 | mecab dict 처리 검증 |
| 정리 | 8 | -2557 줄, 28 docs, CI 정리 |
| 진단 only | 6 | 성능 benches, 바인딩 인벤토리 |
| 외부 인프라 | 1 | NIKL Modu 인프라 준비 |
| Track B (실패) | 5 | 빌드 + 학습 + 변환 + 회귀 (S164~168) |
| 종합 | 2 | Sprint cycle 종합 정리 |

---

## 2. 검증된 영역 매트릭스 (확정)

### ✅ 안전 영역 (효과 검증)

1. **PRACTICAL 동치 그룹** (`TAG_EQUIVALENCE_GROUPS_PRACTICAL`)
   - NNB/NNG, VA/VV, VV/XSV, MAG/MAJ
   - Conservative 정밀 보존 + Practical downstream

2. **Surface normalization** (`normalize_endings`)
   - 하았/하어, 이습니다, 르 불규칙, ㄷ 불규칙, 명시 어구
   - 누적 +6pp

3. **Splitter rule (제한적)**
   - VCP+ETM/EC, VCP+EP, VA+ETM multi-syllable
   - mecab dict 못 처리 패턴만

4. **Silver dataset 통합**
   - UD Korean-Kaist, UD Korean-GSD
   - 5-gate CI 완성

### ❌ 위험 영역 (회귀 확인)

| Sprint | 시도 | 회귀 |
|--------|------|------|
| 138 | matrix.def cost | -0.9pp sample.tsv |
| 145 | multi-syllable VV+ETM | -1 sentence |
| 155 | dict cost=-5000 NNP | -0.2pp KLUE |
| **167** | **CRF retrain (POS only)** | **-37.8pp** |

→ viterbi/CRF mechanism 직접 변경 = cascade 회귀 위험 매우 큼.

### ⏸ 비이슈 영역 (mecab dict 처리)

빈도 분석으로 미처리 같지만 실제로는 mecab decomposition fallback이 처리:
- ETM+ETM "라는" 33건 (S148)
- XSA+ETM 38건 (S153)
- EP+ETM/XSV+ETM/VX+EP/XSA+EP 218건 (S154)

총 빈도 833건 중 실효 작업 24건만 (2.9%).

---

## 3. 메타 학습 종합

### 3.1 mecab dict의 압도적 강력함

`SejongConverter::convert_token` decomposition fallback이 ending_rules보다 먼저 시도되어 사전 features 활용. ㅂ/ㄹ/ㅎ 불규칙 stem 복원까지 dict이 처리.

### 3.2 빈도 ≠ 실효 lift

raw mecab POS 빈도는 작업 후보 식별만 가능. **반드시 splitter+converter 변환 후 진단**.

올바른 워크플로우:
1. 빈도 분석 → 후보 식별
2. 변환 후 진단
3. 미처리 ≥ threshold → 작업
4. 그렇지 않으면 → 비이슈 문서화

### 3.3 3 silver 일관 lift = 진짜 효과

도메인 독립 lift는 신뢰도 높음. 단일 도메인 anomaly는 false signal 가능.

### 3.4 Rollback 신속화 (Sprint 138 정책)

sample.tsv 회귀 발견 즉시 rollback. 분석은 rollback 후. 시간 절약.

### 3.5 격리의 가치

별도 dict 디렉토리 + 환경 변수 = zero-cost rollback. 기존 baseline 절대 손상 방지.

### 3.6 진단 데이터 재활용

Sprint 155 진단 → A (dict 확장) 실패 → G (메트릭 동치) 성공. 같은 데이터로 여러 접근.

### 3.7 자동 트랙 선택 (규칙 5)

- 사용자 question 제거 → 빠른 진행
- 전문가 권고 활용 → 가설 명확
- **전문가도 틀릴 수 있음 → 항상 측정으로 검증** (S153 E)

### 3.8 작은 corpus + sparse features의 함정

F=0.99977 학습 적합도 = false signal (overfit). 실제 generalization 성능과 무관 (S167 CRF retrain catastrophic regression).

---

## 4. 유지보수 모드 진입

### 적용 범위

Sprint 174 이후:
- 자동 sprint-run 정지
- 버그/의존성 업데이트만 대응
- 다음 메이저 작업 명시 대기

### 트리거 조건 (sprint-run 재개)

다음 중 하나 발생 시:

1. **NIKL Modu 다운로드 완료** (사용자 작업)
   - `./tools/nikl_modu_setup.sh <json>` 실행
   - 정확도 측정 + 분석 sprint 재개

2. **Sejong 코퍼스 입수** (사용자 학술 작업)
   - Track B 재시도 (Sprint 164~167 인프라 즉시 활용)
   - features-rich 학습 → +1~5pp 잠재

3. **사용자 명시 신규 영역**
   - 특정 기능 추가
   - 특정 버그 픽스
   - 새 바인딩
   - 우선순위 명시 시 진행

### 진척 모니터링

- 5-gate CI 자동 실행 (PR마다)
- 회귀 발견 시 알림
- 의존성 업데이트는 `cargo update` periodic

---

## 5. Track B 자산 보존 (재사용 가능)

Sprint 164~167에서 구축한 인프라:

| 자산 | 위치 |
|------|------|
| legacy/ macOS arm64 빌드 | `legacy/src/.libs/` |
| mecab-cost-train 사용법 | docs/archive/sprint-reports/2026-05-19_sprint136_crf_retrain_infra.md |
| tools/to_mecab_tagged.py | CoNLL-U → .tagged 변환 |
| 4단계 학습 파이프라인 | 62.6초 (dict-index→cost-train→dict-gen→dict-index) |
| 격리 메커니즘 | 별도 dict 디렉토리 + 환경 변수 |

Sejong 코퍼스 입수 시 즉시 활용 가능.

---

## 6. 변경 파일

- `docs/research/accuracy/2026-05-27_sprint174_cycle_termination.md` (본 문서)
- `PLAN.md`, `PROGRESS.md` 유지보수 모드 갱신

---

## 7. 종합 결론

**Sprint 122 → 173 (52 sprints, ~1.5 개월) 정확도 작업 cycle 공식 종료**.

성과:
- **KLUE morph practical +6.3pp** (65.8% → 72.1%)
- **KLUE surface canonical_lenient +6pp** (89% → 95.6%)
- **sample.tsv baseline 보존** (100%/99.9%)
- **2 silver dataset 추가** + 5-gate CI 완성
- **accuracy_eval.rs -51% lines** + 30+ docs archive
- **성능 회귀 없음** (v0.3.0 → v0.7.2 비교)

남은 외부 의존 작업만 (NIKL Modu / Sejong / 사용자 명시) 대기.

---

*작성: 2026-05-27 (Sprint 174, 유지보수 모드 진입)*
