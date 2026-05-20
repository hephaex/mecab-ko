# PLAN — mecab-ko Sprint 151 (next)

> 마지막 업데이트: 2026-05-21

## 완료: Sprint 150 A — VA+ETM multi-syllable

### 결과

| Metric | Before | After | Δ |
|--------|--------|-------|---|
| sample.tsv | 100.0%/99.9% | 무회귀 | — |
| **KLUE morph strict** | 66.5% | **66.9%** | **+0.4pp** |
| KLUE surface strict | 87.7% | 87.8% | +0.1pp |
| UD Kaist morph practical | 68.3% | 68.4% | +0.1pp |

### 핵심

- raw VA+ETM 542건 → ending_rules가 이미 95.6% 처리
- 미처리 24건 (빠른/나쁜/예쁜/느린): multi-syllable ㄴ jongseong split 추가
- VV+ETM은 Sprint 145 회귀로 제외, VA만 안전

### 보고서

`docs/research/accuracy/2026-05-21_sprint150_va_etm_multisyllable.md`

## 다음 스프린트: Sprint 151 (미정 — 사용자 선택)

### 후보 C: accuracy_eval.rs setup helper 추출 (잔여 P0)

남은 P0 정리:
- 반복되는 tokenizer setup 코드 (~22번 반복, ~30라인씩)
- `fn make_tokenizer(project_root: &Path) -> Tokenizer`
- `fn make_project_root() -> PathBuf`
- 약 ~1000줄 감소 가능

**비용**: 0.3 sprint, 낮은 위험

### 후보 D: Node/WASM CI 강화

e2e-ffi-tests.yml의 continue-on-error 제거:
- nodejs-e2e 빌드 단계 (L166, L175)
- wasm-e2e job (L177, L180 — job level)
- 실제 빌드 동작 먼저 확인 필요

**비용**: 0.2 sprint, 낮은 위험

### 후보 E: XSA+ETM 38건 분석

Sprint 145 분석에서 XSA+ETM 38건: 스러운, 스런, 로운
- "한가스러운" → 한가/NNG + 스럽/XSA + 운/ETM (ㅂ 불규칙 XSA)
- 패턴: ㅂ 불규칙 XSA (어렵, 무겁 등과 동일)

복잡 (XSA는 NNG에 붙는 접미사). 보수적 접근 필요.

**비용**: 0.5 sprint, 중간 위험

### 후보 B [메인]: Full CRF Retrain (Track B)

3-5 sprint 장기 작업. 학습 데이터 + mecab-cost-train.
잠재 lift +1~5pp (가장 큰 single improvement).

## 검증 기준

- `cargo test --workspace --exclude mecab-ko-ffi` 전체 pass
- `cargo clippy --workspace --all-targets --exclude mecab-ko-ffi -- -D warnings` clean
- **5-gate CI 통과** (sample.tsv / KLUE morph / surface_only / UD Kaist / UD GSD)
- sample.tsv baseline 100%/99.9% **회귀 금지**
