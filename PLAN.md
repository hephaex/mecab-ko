# PLAN — mecab-ko Sprint 128

> 마지막 업데이트: 2026-05-11

## 현재 스프린트: Sprint 128 P2 — Surface normalization lenient

### 배경

Sprint 127 P1에서 SURFACE_MISMATCH가 12.3% (2,745건) 차지함을 측정. 샘플 분석:
- "인정하였다" (KLUE) vs "인정하았다" (mecab) — 어미 변환 표기 차이 (였 ↔ 았)
- "함께" (KLUE) vs "하ㅁ께" (mecab) — 모음 분해 차이 (NFC vs NFD)
- "통하여" (KLUE) vs "통하어" (mecab) — 어 ↔ 여 표기

이 차이는 morpheme 표기 convention의 차이로, 의미 손실 없이 흡수 가능.
NFC compose만으로 상당 부분 흡수 가능 추정 + 어미 변환 동치 추가 검토.

### 목표

surface 비교에 normalization을 주입할 수 있게 하고, KLUE DP에서 측정.
SURFACE_MISMATCH 12.3% 중 얼마를 흡수하는지 정량화.

### 작업 목록

- [ ] **S128P2-01** (analysis): SURFACE_MISMATCH 케이스 NFC compose 효과 측정
  - 본 sprint 127의 SURFACE_MISMATCH 2,745건 대상
  - 양쪽 surface를 NFC로 compose 후 비교
  - 추가로 흡수되는 비율 보고

- [ ] **S128P2-02** (analysis): 추가 normalization 후보 검토
  - 어미 변환 동치 (였 ↔ 았, 어 ↔ 여)
  - NFC 후에도 남는 mismatch 패턴 분석

- [ ] **S128P2-03** (implement): `SurfaceMatchFn` 패턴 도입
  - `pub type SurfaceMatchFn = fn(&str, &str) -> bool`
  - `surface_eq_strict`: `a == b`
  - `surface_eq_nfc`: NFC compose 후 비교
  - PosMatchFn과 동일 패턴

- [ ] **S128P2-04** (implement): `evaluate_dataset_dual_with_match`에 surface_eq 주입
  - 기존 `evaluate_dataset_dual_with_pos_match` 확장
  - 추가 함수 또는 매개변수 확장
  - 모든 기존 API 보존

- [ ] **S128P2-05** (test): KLUE DP에서 surface lenient 측정
  - strict / lenient (conservative+practical) / surface lenient 비교
  - morpheme + eojeol 양쪽
  - 회귀 테스트 floor 갱신

- [ ] **S128P2-06** (docs): 보고서 작성
  - `docs/research/accuracy/2026-05-11_klue_dp_surface_lenient.md`
  - NFC + 추가 normalization의 lift 측정
  - Sprint 129 권고

- [ ] **S128P2-07** (commit + memory)

### 검증 기준

- `cargo test --workspace --exclude mecab-ko-ffi` 전체 pass
- `cargo clippy --workspace --all-targets --exclude mecab-ko-ffi -- -D warnings` clean
- 기존 evaluate API 모두 보존 (function pointer 주입 패턴 일관성)
- 측정 결과가 SURFACE_MISMATCH 흡수량을 명시
