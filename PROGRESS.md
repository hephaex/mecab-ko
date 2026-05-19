# PROGRESS — mecab-ko Sprint 140 (UD Kaist 분석 + CI 게이트)

> 마지막 업데이트: 2026-05-19

## Sprint 140 — A (분석) + C (CI 게이트) 병행

| Task | 상태 | 비고 |
|------|------|------|
| S140-A: UD Kaist SPLIT_DIFFERENT 분석 | ✅ 완료 | `test_ud_kaist_split_diff_connection_pairs`. 1,755 SPLIT_DIFFERENT, 479 unique pairs |
| S140-C: accuracy-gate CI에 UD Kaist 추가 | ✅ 완료 | 4번째 게이트, morph strict ≥ 60% floor, PR comment 섹션 추가 |
| S140-A2: 분석 보고서 | ✅ 완료 | `docs/research/accuracy/2026-05-19_sprint140_ud_kaist_pair_analysis.md` |
| convert_ud_kaist.py 수정 | ✅ 완료 | text reconstruct from token forms (eojeol/morpheme 정렬 보장) |

## 핵심 발견

### 도메인 독립적 패턴 (KLUE + UD Kaist 공통)

| Pair | KLUE | UD | 의미 |
|------|------|-----|-----|
| (3534,0) NNG-T → BOS | 298 | 227 | 어절 중간 NNG-T로 종결 선호 |
| (3533,0) NNG-F → BOS | 196 | 257 | 어절 중간 NNG-F로 종결 선호 |
| (0,1780) BOS → NNG | 264 | 204 | 어절 중간 NNG 시작 선호 |
| (3534,1780) NNG-T → NNG | 109 | 104 | NNG+NNG 복합어 분해 |
| (3533,1780) NNG-F → NNG | 129 | 76 | NNG+NNG 복합어 분해 |

→ **Sprint 138 결론 재확인**: NNG cost 조정은 도메인 무관하게 어절 경계 영향 → sample.tsv 회귀 불가피.

### UD Kaist 특화 패턴 (학술/역사 텍스트)

| Pair | UD | KLUE | 의미 |
|------|-----|------|-----|
| (3777,2240) XSN(적) → VCP(인) | 92 | 27 | "X적인" — 학술 단정 |
| (3533,2609) NNG-F → XSN(적) | 60 | 14 | "역사적", "구체적" |
| (3534,2609) NNG-T → XSN(적) | 59 | 42 | "실질적", "전통적" |
| (0,1783) BOS → NNG(정적사태) | 68 | 36 | "한다", "있다" — 학술 문체 |

→ KLUE만으로는 학술 도메인의 XSN(적) 처리 이슈 보이지 않음. **도메인 다양화 가치 입증**.

### KLUE 특화 패턴 (현대 뉴스)

| Pair | KLUE | UD | 의미 |
|------|------|-----|------|
| (5,1794) EF → SF | 166 | 16 | 종결어미 + 마침표 |
| (3561,1780) SH → NNG | 134 | 28 | "100여명" — 수치 표현 |
| (3584,0) XR-T → BOS | 130 | 43 | "탁월한", "비롯한" |

## CI 게이트 (Sprint 140 C)

### 추가 step
- `.github/workflows/accuracy-gate.yml`: `Run UD Kaist silver gate`
- Floor: morph strict ≥ 60% (silver tolerance — test 내부는 40%, CI는 더 엄격)
- PR comment에 4번째 섹션 추가 (UD Korean-Kaist Silver)

### 4-gate 효과
- sample.tsv (1,100, baseline 100%/99.9%)
- KLUE DP morph (1,995, floor 60%/15%)
- KLUE DP surface-only (검색 use case, 50%/80%)
- **UD Kaist silver (1,638, floor 60%)** [신규]

Sprint 138 같은 cost 조정 회귀를 **두 도메인에서 동시 감지** 가능.

## 측정값 (변경 없음 — 분석 + CI 추가만)

| 메트릭 | Sprint 139 | Sprint 140 |
|--------|-----------|-----------|
| 모든 KLUE/UD/sample.tsv 메트릭 | 동일 | 동일 |

## 검증

- `cargo test --workspace --exclude mecab-ko-ffi --lib`: all pass / 0 fail
- `cargo clippy --workspace --all-targets -- -D warnings`: clean
- `test_ud_kaist_split_diff_connection_pairs`: PASS
- `test_ud_kaist_dual_metric`: PASS (CI 게이트용)
- actionlint: shellcheck SC2129 warnings는 기존 step과 동일 패턴 (style only)

## 변경 파일

- `rust/crates/mecab-ko-core/tests/accuracy_eval.rs`: `test_ud_kaist_split_diff_connection_pairs` 추가 (~135줄)
- `tools/convert_ud_kaist.py`: text reconstruct from token forms (eojeol/morpheme alignment 보장)
- `data/eval/ud_kaist_{test,dev}.tsv`: 재생성 (token-based text)
- `.github/workflows/accuracy-gate.yml`: `Run UD Kaist silver gate` step + PR comment 섹션
- `docs/research/accuracy/2026-05-19_sprint140_ud_kaist_pair_analysis.md` (신규)
- `PLAN.md`, `PROGRESS.md` 갱신

## Sprint 141 후보

- A: XSN(적) practical 동치 검토 (학술 텍스트 특화 패턴)
- B: UD Kaist NNG/XSN cost 조정 실험 (NNG보다 안전한 시작점)
- C: dict-builder CSV 버그 수정 (Track D 진입 선행)
- D: NIKL Modu 평가 추가 (manual download)
- E: Full CRF retrain (장기 escalation)
