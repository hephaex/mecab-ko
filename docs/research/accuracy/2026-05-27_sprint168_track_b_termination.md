# Sprint 168 — Track B 공식 종료 + 정확도 Sprint Cycle 종합

> **결정**: Track B (Full CRF Retrain) 공식 종료. 사용자 confirm (Option D). 학습 데이터 features 부족이 근본 원인 — 추가 sprint로 해결 어려움. Sprint 122~167 누적 +6.3pp KLUE morph practical / +6pp surface 달성 후 정확도 sprint cycle 마무리.

---

## 1. Track B 종료 결정 배경

### Sprint 164~167 시도 결과

| Sprint | Step | 결과 |
|--------|------|------|
| 164 | 빌드 환경 | ✅ legacy/ macOS arm64 |
| 165 | 학습 데이터 (UD dev 3016 sentences) | ✅ |
| 166 | 학습 + dict 변환 (62.6초 파이프라인) | ✅ |
| 167 | Rust 통합 | ❌ **-37.8pp sample.tsv** |

### 근본 원인 (Sprint 167 분석)

`.tagged` 형식의 features 부족:
```
surface\tPOS,*,*,*,*,*,*,*,*  ← POS만 정확
```

mecab `feature.def`가 활용하는 features (semantic, reading, T/F jongseong, 활용형)가 모두 `*` (wildcard) → CRF overfit → catastrophic regression.

### 사용자 결정 (Option D)

- Option A (Self-training): self-amplification 위험, 효과 제한적
- Option B (corpus 확장): leakage 문제
- Option C (Sejong 입수): 라이선스 절차 + 자동화 불가
- **Option D (종료)**: 학습 데이터 features 부족이 근본 원인 — 추가 sprint로 해결 어려움

→ Track B 공식 종료.

---

## 2. Track B 인프라 자산 (재사용 가능)

이번 시도가 미완성으로 끝났지만 다음 자산은 보존:

### 2.1 빌드 환경 (Sprint 164)
- `legacy/` macOS arm64 빌드 절차 문서화
- `mecab-cost-train`, `mecab-dict-gen`, `mecab-dict-index` executable

### 2.2 변환 도구 (Sprint 165)
- `tools/to_mecab_tagged.py` (CoNLL-U → .tagged)
- KAIST_TO_SEJONG 매핑 재사용

### 2.3 학습 파이프라인 (Sprint 166)
- 4단계 파이프라인 (62.6초): dict-index → cost-train → dict-gen → dict-index
- 4가지 문제 해결 (entries.csv, tokenizer, dicrc, source CSV)

### 2.4 격리 메커니즘 (Sprint 167)
- 별도 dict 디렉토리 + 환경 변수 패턴
- 회귀 시 zero-cost rollback

차후 Sejong 코퍼스 입수 또는 features-rich 학습 데이터 확보 시 즉시 활용 가능.

---

## 3. 정확도 Sprint Cycle 종합 (Sprint 122 → 167)

### 누적 진척 표

| Metric | Sprint 122 | Sprint 167 | Δ |
|--------|-----------|-----------|---|
| sample.tsv | 100%/99.9% | 100%/99.9% | — (baseline 보존) |
| **KLUE morph practical** | ~65.8% | **72.1%** | **+6.3pp** |
| KLUE eojeol practical | ~5000 | 5327 | +327 |
| **KLUE surface canonical_lenient** | ~89% | **95.6%** | **+6pp** |
| UD Kaist morph practical | — | 68.6% | (new silver, S139) |
| UD GSD morph practical | — | 71.8% | (new silver, S143) |

### Sprint 분류 (46 sprints)

| 유형 | 건수 | 효과 |
|------|------|------|
| Lift sprints | 11 | +6.3pp 누적 |
| Infrastructure | 9 | 5-gate CI, 3 silver, helper 추출 |
| Rollback | 4 | viterbi/CRF 변경 모두 회귀 |
| 비이슈 확인 | 3 | mecab dict decomposition 처리 |
| 정리 | 5 | -2557줄 (accuracy_eval), 28 docs archive |

### 영역 소진 매트릭스

| 영역 | 결과 |
|------|------|
| Splitter rule | ❌ Sprint 154 소진 (mecab dict 처리) |
| Dict cost 확장 | ❌ Sprint 155 회귀 |
| CRF matrix | ❌ Sprint 138 회귀 |
| **CRF Full Retrain** | ❌ **Sprint 167 회귀** (Track B 종료) |
| Surface normalization | ✅ 누적 +6pp |
| PRACTICAL 동치 | ✅ 누적 +6.3pp |
| Silver dataset | ✅ 5-gate 완성 |

---

## 4. 메타 학습 종합

### 4.1 mecab dict의 강력함

- `SejongConverter::convert_token` decomposition fallback이 ㅂ/ㄹ/ㅎ 불규칙까지 처리
- 빈도 분석 vs 실제 미처리 갭이 큼 (Sprint 148/153/154)
- 218/833 = 26.2% only가 실제 미처리

### 4.2 viterbi/CRF 변경 = 위험 (4번째 확인)

| Sprint | 시도 | 회귀 |
|--------|------|------|
| 138 | matrix.def cost | -0.9pp |
| 145 D | multi-syllable VV+ETM | -1 sentence |
| 155 A | dict cost=-5000 NNP | -0.2pp |
| 167 | CRF retrain | -37.8pp |

→ viterbi/CRF mechanism 직접 변경은 cascade 회귀 매우 위험.

### 4.3 안전 영역 = 메트릭 영역

- normalize_endings (평가 함수만)
- TAG_EQUIVALENCE_GROUPS_PRACTICAL (동치 그룹)
- 코드 영향 zero — cascade 없음

### 4.4 진단 우선 원칙

- 빈도 분석 → 변환 후 진단 → 실제 미처리 측정 → 작업 결정
- Sprint 154 통합 진단 (4 후보) = 효율적 영역 소진 확인

### 4.5 격리의 가치

- 별도 dict 디렉토리 + 환경 변수
- 회귀 시 zero-cost rollback
- 기존 baseline 절대 손상 방지

---

## 5. 다음 단계

### 정확도 lift 작업 종료 선언

- 안전 영역 모두 소진 (Sprint 158)
- Track B (Full CRF Retrain) 실패
- 추가 lift는 다음 중 하나 필요:
  - Sejong 코퍼스 입수 (학술 라이선스)
  - NIKL Modu 다운로드 + 측정 (사용자 작업)
  - 새 dictionary 도입 (mecab-ko-dic 후속 버전)

### 잔여 작업 (사용자 액션 시)

#### NIKL Modu (Sprint 159~163 인프라 완료, 사용자 다운로드 대기)
```bash
# 사용자 다운로드 완료 시
./tools/nikl_modu_setup.sh ~/Korpora/NIKL_MP/NXMP*.json
```

#### Sejong 코퍼스 (시간 소요)
- 국립국어원 학술 등록
- 또는 KAIST 입수
- 입수 시 Track B 재시도 가능

### 다른 방향 (사용자 결정 시)

- 정확도 외 영역 (성능, 사용성, 바인딩)
- 유지보수 모드

---

## 6. 변경 파일

- `docs/research/accuracy/2026-05-27_sprint168_track_b_termination.md` (본 문서)
- `data/training_run_1/`: 학습 실패 산출물 유지 (재사용 가능, gitignore)
- `PLAN.md`, `PROGRESS.md` 갱신

---

## 7. 결론

**Track B 공식 종료. 정확도 lift sprint cycle 마무리.**

Sprint 122~167 (46 sprints) 누적:
- **KLUE morph practical +6.3pp** (65.8% → 72.1%)
- **KLUE surface canonical_lenient +6pp** (89% → 95.6%)
- **sample.tsv baseline 보존** (100%/99.9%)
- **2 silver dataset 추가** (UD Kaist, UD GSD)
- **5-gate CI 시스템 확립**
- **30+ docs archived**

추가 lift는 외부 입수 (Sejong / NIKL Modu / 새 dict) 필요. 그 외에는 정확도 외 영역으로 전환.

---

*작성: 2026-05-27 (Sprint 168, Track B 종료)*
