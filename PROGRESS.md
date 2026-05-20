# PROGRESS — mecab-ko Sprint 146 (명시 surface 안전 패턴)

> 마지막 업데이트: 2026-05-20

## Sprint 146 A — VCP+EP "였" 분리 (NP+JX skip)

| Task | 상태 | 비고 |
|------|------|------|
| S146-A1: NP+JX 명시 surface 분리 | ✅ 완료 (skip 결정) | mecab이 "그는"/"이는" 이미 분리; 결합 "난"/"게다가"는 KLUE에 없음 |
| S146-A2: VCP+EP "였" 분리 | ✅ 완료 | "였" → "이/VCP + 었/EP" 명시 surface 처리 |
| S146-A3: 5-gate 검증 | ✅ 완료 | 5-gate 무회귀, 실측 lift 없음 (형태론 정확성만) |

## 핵심 발견

### NP+JX skip 사유

**mecab CLI 직접 확인**:
- "그는" → 그/NP + 는/JX (이미 분리)
- "이는" → 이/NP + 는/JX (이미 분리)
- "저는" → 저/NP + 는/JX (이미 분리)
- "난" → 난/NP+JX (결합, contraction)
- "게다가" → 게다가/NP+JX (결합)

**KLUE gold 확인**: "난"/"게다가" NP+JX 결합 surface 등장 안 함. 모두 분리된 morpheme.

→ mecab 분리 출력은 이미 KLUE/UD와 일치. 결합 출력(contraction)은 KLUE에 없음 → 분리 시도 시 false morpheme 추가. **skip**.

### VCP+EP "였" 분리 추가

`splitter.rs`에 추가:
```rust
if pos == "VCP+EP" && surface == "였" {
    return vec![("이".to_string(), "VCP"), ("었".to_string(), "EP")];
}
```

**단위 테스트 2개**:
- test_split_morpheme_vcp_ep_yeoss
- test_split_morpheme_vcp_ep_other_surface_no_split

### 측정 결과 (모든 메트릭 동일)

| 메트릭 | Before | After |
|--------|--------|-------|
| sample.tsv Token / Sentence | 100.0% / 99.9% | 동일 |
| KLUE morph / eo strict | 66.8% / 20.7% | 동일 |
| KLUE practical | 71.6% / 23.5% | 동일 |
| Surface canonical_lenient | 95.5% | 동일 |
| UD Kaist morph | 66.3% | 동일 |
| UD GSD morph | 67.4% | 동일 |

### 실측 lift 0 원인

Sprint 145 분석의 VCP+EP 101건 ("였")은 `Token.pos` 문자열에 `+` 포함된 raw mecab feature. 실측 평가는 SejongConverter 처리 후 비교 → 분석 단계 raw count가 실측 lift와 직접 비례하지 않음. mecab CLI 출력에서도 "였/EP" 단독으로 보임 → mecab 실제 token 분해 path가 다를 가능성.

### 유지 결정 (형태론적 정확성)

- 형태론적으로 정확 (KLUE/UD gold와 일치)
- 단위 테스트로 정확성 보장
- 향후 mecab dict 업데이트 대비
- 회귀 0

## 핵심 학습 포인트

### 1. 분석 빈도 ≠ 실측 영향

Sprint 145 빈도 분석은 raw feature 기반. 실측은 SejongConverter 후. 두 단계가 일치하지 않을 수 있음 → 실측 검증 필수.

### 2. mecab CLI 출력이 절대적 기준

분석 테스트의 통계보다 CLI 직접 확인 + KLUE gold 비교가 정확.

### 3. mecab이 이미 분리하는 패턴은 skip

mecab 분리 출력 = KLUE gold이면 추가 작업 불필요.

### 4. 형태론적 정확성도 commit 가치

실측 lift 0이라도 정확한 코드는 downstream 일관성 향상.

## 측정값 (변경 없음)

| 메트릭 | Sprint 145 | Sprint 146 |
|--------|-----------|-----------|
| 모든 5-gate | 동일 | 동일 |

## 검증

- `cargo test --workspace --exclude mecab-ko-ffi --lib`: **398 passed / 0 failed** (396 + 2 신규)
- `cargo clippy --workspace --all-targets -- -D warnings`: clean
- `test_full_accuracy_evaluation`: PASS
- `test_klue_dp_*` / `test_ud_*`: PASS (변화 없음)

## 변경 파일

- `rust/crates/mecab-ko-core/src/sejong/splitter.rs`: VCP+EP "였" 분리 + 2 단위 테스트
- `docs/research/accuracy/2026-05-20_sprint146_explicit_surface_splits.md` (신규)
- `PLAN.md`, `PROGRESS.md` 갱신

## Sprint 147 후보

- A: 추가 안전 패턴 (XSV+EP/EC 명시 surface)
- B [메인]: Full CRF Retrain (Track E)
- C: NIKL Modu 수동 다운로드
- D: VV+EP 명시 동사 분리
