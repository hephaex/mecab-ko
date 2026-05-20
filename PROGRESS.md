# PROGRESS — mecab-ko Sprint 149 (P0 Cleanup)

> 마지막 업데이트: 2026-05-21

## Sprint 149 — P0 정리 스프린트

| Task | 상태 | 결과 |
|------|------|------|
| S149-01: MSRV 1.80 CI gate 추가 | ✅ 완료 | ci.yml msrv job 추가 |
| S149-02: coverage floor 설정 | ✅ 완료 | tarpaulin --fail-under 60 |
| S149-03: placeholder 테스트 파일 삭제 | ✅ 완료 | integration_kiwi.rs, integration_performance.rs 삭제 |
| S149-04: accuracy_eval.rs 진단 함수 24개 삭제 | ✅ 완료 | 4963→2800줄 (-43%) |
| S149-05: multi-syllable VV+ETM rollback guard 테스트 | ✅ 완료 | 3개 신규 테스트 (스프린트 145 가드) |

## 변경 내용

### ci.yml: MSRV 1.80 게이트

```yaml
msrv:
  name: MSRV Check (Rust 1.80)
  steps:
    - uses: ./.github/actions/rust-setup
      with: { toolchain: 1.80.0 }
    - run: cargo check --workspace --lib
```

Cargo.toml에 선언된 `rust-version = "1.80"`이 CI에서 실제 검증됨.

### ci.yml: coverage floor

```yaml
cargo tarpaulin ... --fail-under 60
```

사용자 룰의 80% 목표 향해 60% floor로 시작 (이전: floor 없음).

### placeholder 테스트 삭제

- `integration_performance.rs` — 21개 테스트, 모두 `println!("...placeholder")` + 0 assertion
- `integration_kiwi.rs` — 12개 테스트, `assert!(true)` 1개만 존재

### accuracy_eval.rs 정리

삭제된 sprint-specific 진단 함수 24개:
test_ec_error_analysis, test_etm_error_analysis, test_ef_error_analysis,
test_etn_error_analysis, test_xsv_error_analysis, test_vcp_error_analysis,
test_nng_error_analysis, test_xpn_error_analysis, test_nnb_error_analysis,
test_ec_sample_errors, test_jks_sample_errors, test_mag_sample_errors,
test_ef_sample_errors, test_vx_sample_errors, test_xsv_sample_errors,
test_vx_pattern_debug, test_ep_error_analysis, test_ef_error_cases_detailed,
test_vcp_sample_errors, test_vv_sample_errors, test_xsv_debug_sentences,
test_specific_sentence_debug, test_ep_sample_errors, test_list_all_mismatches

유지된 5-gate 함수:
test_accuracy_gate ★, test_klue_dp_dual_metric ★, test_klue_dp_eojeol_surface_only ★,
test_ud_kaist_dual_metric ★, test_ud_gsd_dual_metric ★

### splitter.rs 신규 테스트

- `test_split_morpheme_vv_etm_single_syllable_rieul` — "올" → 오/VV + ㄹ/ETM
- `test_split_morpheme_va_etm_single_syllable_nieun` — "큰" → 크/VA + ㄴ/ETM
- `test_split_morpheme_vv_etm_multisyllable_no_jamo_split` — Sprint 145 rollback guard

## 검증

- `cargo test --workspace --exclude mecab-ko-ffi --lib`: **402 passed / 0 failed** (399+3)
- `cargo clippy --workspace --all-targets -- -D warnings`: clean
- 5-gate sample.tsv: PASSED (100.0%/99.9%)
- placeholder 파일 삭제 후 mecab-ko 패키지 테스트: 모두 통과

## Sprint 150 후보

- A: VA+ETM 542건 처리 (형용사 활용 분리)
- B: Full CRF Retrain (Track B)
- C: accuracy_eval.rs setup helper 함수 추출 (추가 정리)
- D: Node/WASM continue-on-error 제거 (빌드 단계)
