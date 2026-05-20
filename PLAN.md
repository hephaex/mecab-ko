# PLAN — mecab-ko Sprint 150 (next)

> 마지막 업데이트: 2026-05-21

## 완료: Sprint 149 — P0 정리 스프린트

### 결과

| 항목 | 전 | 후 |
|------|---|---|
| accuracy_eval.rs 줄 수 | 4963 | 2800 (-43%) |
| placeholder 테스트 파일 | 2개 (33 함수, 0 assertion) | 삭제 |
| MSRV CI 검증 | 없음 | msrv job (1.80.0) |
| coverage floor | 없음 | --fail-under 60 |
| splitter 단위 테스트 | 8개 | 11개 (+rollback guard) |
| 전체 lib 테스트 | 399 pass | 402 pass |

### 커밋: `ea5cfa9`

## 다음 스프린트: Sprint 150 (미정 — 사용자 선택)

### 후보 A: VA+ETM 542건 처리

형용사 활용형 분리. VV+ETM과 동일 패턴 (1-syllable 기준):
- "큰" → 크/VA + ㄴ/ETM (이미 구현됨)
- "어려운" (2-syllable) → VA+ETM multisyllable은 이미 guard로 제외

**실제 평가 영향 먼저 측정 필요.** VA+ETM 542건이 이미 1-syllable 경로로 처리되는지 확인.

**비용**: 0.5 sprint (분석 + 가능하면 추가 패턴)

### 후보 B [메인]: Full CRF Retrain (Track B)

3-5 sprint. 학습 데이터 준비 + mecab-cost-train 실행.
- Sprint 136에서 인프라 조사 완료
- 잠재 lift: +1~5pp (가장 큰 single improvement)

### 후보 C: accuracy_eval.rs setup helper 추출

setup boilerplate 추출로 추가 ~1000줄 감소. 빌드 시간 단축.
- `fn make_tokenizer(project_root: &Path) -> Tokenizer`
- `fn make_project_root() -> PathBuf`

**비용**: 0.3 sprint (정밀 리팩토링)

### 후보 D: Node/WASM CI 강화

e2e-ffi-tests.yml에서 Node/WASM 빌드 잡 `continue-on-error: false`로 전환.
실제 빌드 여부 확인 필요.

**비용**: 0.2 sprint

## 검증 기준

- `cargo test --workspace --exclude mecab-ko-ffi` 전체 pass
- `cargo clippy --workspace --all-targets --exclude mecab-ko-ffi -- -D warnings` clean
- **5-gate CI 통과** (sample.tsv / KLUE morph / surface_only / UD Kaist / UD GSD)
- sample.tsv baseline 100%/99.9% **회귀 금지**
