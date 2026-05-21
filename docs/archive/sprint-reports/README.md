# Sprint Reports Archive (Sprint 132~158)

이 디렉토리는 mecab-ko의 sprint 132~158 보고서 아카이브입니다.

각 보고서는 해당 sprint의 실험 결과 / 시도 / 학습을 자세히 기록합니다.

## 정리 기준 (Sprint 160, 2026-05-21)

`docs/research/accuracy/`에 누적된 sprint 보고서를 이 archive로 이동.
- 일반 분석 문서 (sprint-independent)는 `docs/research/accuracy/`에 유지
- sprint별 보고서는 이 archive로 분리

## 누적 진척 (Sprint 122 baseline → Sprint 158)

| Metric | Sprint 122 | Sprint 158 | Δ |
|--------|-----------|-----------|---|
| sample.tsv | 100%/99.9% | 100%/99.9% | — (baseline) |
| **KLUE morph practical** | ~65.8% | **72.1%** | **+6.3pp** |
| KLUE eojeol practical | ~5000 | 5327 | +327 |
| **KLUE surface canonical_lenient** | ~89% | **95.6%** | **+6pp** |
| UD Kaist morph practical | — | 68.6% | (new silver, Sprint 139) |
| UD GSD morph practical | — | 71.8% | (new silver, Sprint 143) |

## Sprint 분류

### Lift sprints (정확도 향상)

| Sprint | 효과 | 영역 |
|--------|------|------|
| 130 | +0.7pp morph (dict) | dict 확장 (KLUE domain) |
| 132 | +0.3pp morph | dict expansion 빈도 2-4 |
| 134 | +1.0pp surface_only | normalize_endings 확장 |
| 136 | normalize_endings + lenient 정리 | VA/VV practical + ㄹ불규칙 |
| 141 | (Sprint 141 보고서) | VCP+ETM/EC splitter |
| 146 | (Sprint 146 보고서) | VCP+EP "였" splitter |
| 147 | 3 silver +0.2~0.4pp practical | VV/XSV practical 동치 |
| 150 A | +0.4pp KLUE strict | VA+ETM multi-syllable |
| 156 | +0.1pp surface canonical_lenient | ㄷ 불규칙 + 르 추가 |
| 157 | 3 silver +0.2pp practical | MAG/MAJ practical 동치 |
| 158 | +0.05pp surface canonical_lenient | 명시 어구 정규화 |

### Infrastructure sprints

| Sprint | 효과 |
|--------|------|
| 133 | eojeol surface-only metric 도입 |
| 135 | surface_only CI gate 추가 (5-gate 완성) |
| 139 | UD Korean-Kaist silver baseline 통합 |
| 140 | UD Kaist CI gate (4-gate) + pair 분석 |
| 142 | dict-builder CSV unquoted comma 버그 픽스 |
| 143 | UD Korean-GSD silver baseline |
| 144 | UD GSD CI gate (5-gate 완성) |
| 149 | accuracy_eval 진단 함수 24개 정리 (-2163줄) |
| 151 C | accuracy_eval setup helper 추출 (-563줄) |
| 152 D | Node/WASM CI continue-on-error 정리 |
| 159 F | NIKL Modu 인프라 준비 (skip 패턴) |

### Rollback sprints (시도 → 회귀)

| Sprint | 시도 | 결과 |
|--------|------|------|
| 138 | matrix.def Tier A 수동 cost 조정 | -0.9pp sample.tsv → rollback |
| 145 D | multi-syllable VV+ETM split | -1 sentence sample.tsv → rollback |
| 155 A | dict cost=-5000 NNP 9건 추가 | -0.2pp KLUE → rollback |

### 분석 / 비이슈 sprints

| Sprint | 패턴 | 결론 |
|--------|------|------|
| 137 | connection cost pair 분석 | only |
| 145 | compound POS 빈도 분석 | only |
| 148 D | ETM+ETM "라는" 33건 | 비이슈 (splitter 중복 태그 규칙) |
| 153 E | XSA+ETM 38건 | 비이슈 (converter decomp fallback) |
| 154 | 4 후보 통합 진단 (218건) | 비이슈 (빈도 영역 소진 선언) |

## 핵심 학습 (Sprint 122~158 종합)

### 성공 패턴

1. **PRACTICAL 동치 그룹 확장** (Sprint 126, 136, 147, 157)
   - 3 silver 일관 lift = 진짜 convention 차이 흡수
   - Conservative 정밀 보존 + Practical downstream 활용

2. **Surface normalization** (Sprint 128, 134, 136, 156, 158)
   - 명시 stem 목록 (false positive 방지)
   - 누적 효과 (개별 작아도 합산 +6pp)

3. **Splitter rule (제한적)** (Sprint 141, 146, 150 A)
   - mecab dict이 처리 못한 패턴만 (대부분 처리됨)

4. **Silver dataset 통합** (Sprint 139, 143, 159 F)
   - 도메인 coverage 확장
   - 동일 lift 검증 (single anomaly 아님)

### 실패 패턴 (모두 viterbi/CRF 영향 변경)

1. **CRF matrix 수동 조정** (Sprint 138) → cascade 회귀
2. **Dict cost=-5000 추가** (Sprint 155) → cascade 회귀
3. **Multi-syllable VV+ETM splitter** (Sprint 145) → false positive

### 메타 학습

- **mecab dict이 매우 강력** — decomposition features가 ㅂ/ㄹ/ㅎ 불규칙까지 처리
- **빈도 분석 ≠ 실효 lift** — 항상 splitter+converter 변환 후 측정
- **안전 영역 = 메트릭/normalize_endings** (cascade 없음)
- **viterbi/CRF 영향 변경 = 위험** (Sprint 138/145/155 동일 패턴)
- **3 silver 일관 lift = 진짜 효과** (Sprint 147, 157)
- **빈도 4번 비이슈 = 영역 소진 신호** (Sprint 148, 153, 154)

## 보고서 목록 (sprint 순)

```
2026-05-18_sprint132_dict_expansion.md
2026-05-18_sprint133_eojeol_surface_only.md
2026-05-18_sprint134_normalize_endings_extension.md
2026-05-19_sprint136_crf_retrain_infra.md
2026-05-19_sprint137_connection_cost_analysis.md
2026-05-19_sprint138_tier_a_experiment.md
2026-05-19_sprint139_ud_kaist.md
2026-05-19_sprint140_ud_kaist_pair_analysis.md
2026-05-20_sprint141_vcp_split_fix.md
2026-05-20_sprint142_dict_builder_csv_fix.md
2026-05-20_sprint143_ud_gsd.md
2026-05-20_sprint144_ud_gsd_ci_gate.md
2026-05-20_sprint145_compound_pos_analysis.md
2026-05-20_sprint146_explicit_surface_splits.md
2026-05-20_sprint147_xsv_practical_equivalence.md
2026-05-20_sprint148_etm_etm_raneun_nonissue.md
2026-05-21_sprint150_va_etm_multisyllable.md
2026-05-21_sprint153_xsa_etm_nonissue.md
2026-05-21_sprint154_frequency_exhausted.md
2026-05-21_sprint155_dict_expansion_rollback.md
2026-05-21_sprint156_d_irregular_surface_normalization.md
2026-05-21_sprint157_mag_maj_practical.md
2026-05-21_sprint158_explicit_phrase_exhaustion.md
```

---

*Archive 생성: 2026-05-21 (Sprint 160)*
