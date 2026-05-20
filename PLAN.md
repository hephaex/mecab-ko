# PLAN — mecab-ko Sprint 152 (next)

> 마지막 업데이트: 2026-05-21

## 완료: Sprint 151 C — setup helper 추출

### 결과

- accuracy_eval.rs: 2969 → 2406줄 (**-19%**)
- 3개 helper 함수: `project_root`, `dict_path`, `make_tokenizer`
- 24개 함수에서 30라인 boilerplate → 2라인으로 축소
- 누적 효과 (Sprint 148~151): 4963 → 2406줄 (**-51%**)

## 완료된 P0 정리 작업

| Sprint | 항목 | 결과 |
|--------|------|------|
| 149 | MSRV 1.80 CI gate | 추가 |
| 149 | coverage --fail-under 60 | 추가 |
| 149 | placeholder 테스트 삭제 | 2 파일 |
| 149 | accuracy_eval 진단 함수 24개 삭제 | 4963→2800 |
| 149 | multi-syllable rollback guard | 3 테스트 |
| 151 C | setup helper 추출 | 2800→2406 |

## 다음 스프린트: Sprint 152 (미정 — 사용자 선택)

### 후보 D: Node/WASM CI 강화 (잔여 P0)

`.github/workflows/e2e-ffi-tests.yml`의 `continue-on-error: true` 분석/제거:
- nodejs-e2e 빌드 단계 (L166, L175): hard-fail로 전환 검토
- wasm-e2e job (L177): job-level continue-on-error 분석
- 실제 빌드 동작 먼저 확인 필요

**비용**: 0.2 sprint, 낮은 위험

### 후보 E: XSA+ETM 38건 분석

Sprint 145 빈도 분석: XSA+ETM 38건 (스러운, 스런, 로운)
- "한가스러운" → 한가/NNG + 스럽/XSA + 운/ETM (ㅂ 불규칙 XSA)
- 패턴: ㅂ 불규칙 (어렵, 무겁과 동일 메커니즘)
- 진단 → ending_rules 처리 여부 확인 후 결정

**비용**: 0.5 sprint, 중간 위험

### 후보 B [메인]: Full CRF Retrain (Track B)

3-5 sprint 장기 작업. 학습 데이터 + mecab-cost-train.
잠재 lift +1~5pp (가장 큰 single improvement).

## 검증 기준

- `cargo test --workspace --exclude mecab-ko-ffi` 전체 pass
- `cargo clippy --workspace --all-targets --exclude mecab-ko-ffi -- -D warnings` clean
- **5-gate CI 통과** (sample.tsv / KLUE morph / surface_only / UD Kaist / UD GSD)
- sample.tsv baseline 100%/99.9% **회귀 금지**
