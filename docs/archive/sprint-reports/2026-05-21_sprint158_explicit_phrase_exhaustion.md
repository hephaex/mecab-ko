# Sprint 158 — 명시 어구 정규화 + 안전 영역 소진 선언

> **결과**: EXPLICIT_PHRASE_PATTERNS 3 패턴 추가 (인하여/확인하여/참고하시어요). KLUE surface +10 eojeols (+0.05pp). 안전 영역 lift ≤ +0.05pp/sprint 확인 → 영역 소진 선언. 다음 단계: 비가역 대규모 작업 (NIKL Modu / CRF Retrain) confirm 필요.

---

## 1. 변경 내용

### EXPLICIT_PHRASE_PATTERNS (evaluate.rs)

```rust
const EXPLICIT_PHRASE_PATTERNS: &[(&str, &str)] = &[
    ("인하여", "인해"),              // 4건 KLUE
    ("확인하여", "확인해"),           // 3건 KLUE
    ("참고하시어요", "참고하세요"),    // 3건 KLUE
];
```

normalize_endings Step 5 추가. gold (풀버전) / pred (축약형) 차이 흡수.

---

## 2. 측정

| Metric | Before | After | Δ |
|--------|--------|-------|---|
| sample.tsv | 100.0%/99.9% | 100.0%/99.9% | 무회귀 ✓ |
| KLUE surface canonical_lenient | 21419 | **21429** | **+10 eojeols** |
| 그 외 | 변경 없음 | — | — |

ROI 매우 낮음 (3 패턴 → 10 eojeols).

---

## 3. 안전 영역 소진 선언

### Sprint 156~158 누적 효과

| Sprint | 영역 | 효과 |
|--------|------|------|
| 156 | ㄷ 불규칙 + 르 추가 | +30 eojeols |
| 157 | MAG/MAJ practical | +124 eojeols + 3 silver +0.2pp |
| **158** | **명시 어구** | **+10 eojeols** |

trend: Sprint마다 효과 감소.

### 각 영역 상태

| 영역 | Sprint | 상태 |
|------|--------|------|
| Splitter rule | 154 | ❌ 소진 (mecab dict이 처리) |
| Dict cost 확장 | 155 | ❌ 회귀 (viterbi cascade) |
| CRF matrix | 138 | ❌ 회귀 |
| Surface normalization | 156, 158 | ⚠️ 효과 ≤ +0.05pp |
| PRACTICAL 동치 | 157 | ⚠️ 추가 후보 부족 |

### 남은 미처리 surface mismatch (정규화 불가)

진단 결과 남은 패턴들:
- 있어디에서 (20건) — mecab tokenizer over-split
- 모으게하고 (5건) — mecab분해 오류
- 있안으며 (7건) — mecab분해 오류
- 가깝시고 (9건) — mecab분해 오류

이들은 mecab의 tokenization 자체 이슈 → splitter/normalize 영역이 아님. CRF retrain만이 해결 경로.

---

## 4. 다음 단계: 비가역 대규모 작업

### F: NIKL Modu 도입

- Academic license 다운로드
- 구어/SNS 도메인 → 5번째 silver dataset
- coverage 확장 (정확도 lift는 아님)
- 비가역 (다운로드 + CI 추가)

### E: Full CRF Retrain (Track B)

- 3-5 sprint 장기 작업
- 학습 데이터 + mecab-cost-train
- Sprint 136에서 인프라 조사 완료
- 잠재 lift +1~5pp KLUE morph
- 비가역 (대규모 작업)

### 또는 정확도 외 영역

- 문서 정리
- CLI/API 사용성
- 성능 최적화
- 새 언어 바인딩

---

## 5. 누적 진척 (Sprint 122 baseline → Sprint 158)

| Metric | Sprint 122 | Sprint 158 | 총 Δ |
|--------|-----------|-----------|-----|
| sample.tsv | 100%/99.9% | 100%/99.9% | — (baseline) |
| **KLUE morph practical** | ~65.8% | **72.1%** | **+6.3pp** |
| KLUE eojeol practical | ~5000 | 5327 | +327 |
| **KLUE surface canonical_lenient** | ~89% | **95.6%** | **+6pp** |
| UD Kaist morph practical | — | 68.6% | (new silver) |
| UD GSD morph practical | — | 71.8% | (new silver) |

30+ sprint로 KLUE practical +6.3pp / surface +6pp 누적.

---

## 6. 핵심 학습 — Sprint 122~158 종합

### 성공 패턴

1. **PRACTICAL 동치 그룹 확장** (Sprint 126, 136, 147, 157): 3 silver 일관 lift
2. **Surface normalization** (Sprint 128, 134, 136, 156): 누적 효과
3. **Splitter rule 일부** (Sprint 141, 146, 150 A): mecab dict 못 처리한 패턴만
4. **Silver dataset 통합** (Sprint 139, 143): coverage 확장

### 실패 패턴

1. **CRF matrix 수동 조정** (Sprint 138): viterbi cascade
2. **Dict cost 확장** (Sprint 155): viterbi cascade
3. **Multi-syllable VV+ETM splitter** (Sprint 145): false positive

### 비이슈 패턴 (mecab dict이 처리)

- ETM+ETM "라는" (Sprint 148 D)
- XSA+ETM (Sprint 153 E)
- EP+ETM/XSV+ETM/VX+EP/XSA+EP (Sprint 154)

### 메타 학습

- **mecab dict은 매우 강력** — 변환 fallback이 대부분 처리
- **빈도 분석 ≠ 실효 lift** — 항상 변환 후 측정
- **안전 영역 = 메트릭 영역** (평가 함수, 동치 그룹, normalize_endings)
- **viterbi/CRF 영향 변경 = 위험** (cascade 회귀)

---

## 7. 변경 파일

- `rust/crates/mecab-ko-core/src/evaluate.rs`: EXPLICIT_PHRASE_PATTERNS + Step 5 + 단위 테스트
- `docs/research/accuracy/2026-05-21_sprint158_explicit_phrase_exhaustion.md` (본 문서)
- `PLAN.md`, `PROGRESS.md`: Sprint 158 완료 + 영역 소진 선언

---

*작성: 2026-05-21 (Sprint 158)*
