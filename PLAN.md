# PLAN — mecab-ko Sprint 147 (next)

> 마지막 업데이트: 2026-05-20

## 완료: Sprint 146 A — 명시 surface 안전 패턴 (VCP+EP "였")

### NP+JX skip 결정
- mecab CLI 확인: "그는"/"이는"/"저는" 이미 분해
- 결합 surface ("난"/"게다가")는 KLUE에 없음
- → 분리 시도 시 false morpheme 추가 위험

### VCP+EP "였" 분리 추가
- splitter.rs: `if pos == "VCP+EP" && surface == "였"` → split
- 단위 테스트 2개
- 실측 lift 0 (mecab raw feature ≠ SejongConverter 후 결과)
- 형태론적 정확성 + 회귀 0 → 유지

### 보고서
`docs/research/accuracy/2026-05-20_sprint146_explicit_surface_splits.md`

## 다음 스프린트: Sprint 147 (미정 — 사용자 선택)

### 후보 A: 추가 안전 패턴 (XSV+EP, XSV+EC)

XSV+EP 413건, XSV+EC 751건. 명시 surface 식별 + 분리 시도.

**가능 surface**:
- XSV+EP 413건: "했" (하/XSV + 었/EP), "됐" (되/XSV + 었/EP)
- XSV+EC 751건: "해" (하/XSV + 어/EC), "하고" (하/XSV + 고/EC)

**비용**: 0.5-1 sprint
**위험**: 낮음

### 후보 B [메인]: Full CRF Retrain (Track E)

3-5 sprint. 학습 데이터 (KLUE + UD Kaist + UD GSD train) + mecab-cost-train.

### 후보 C: NIKL Modu 수동 다운로드

Academic license. 구어/SNS 도메인 확장.

### 후보 D: VV+EP 명시 동사 분리

VV+EP 542건. 명시 동사 surface ("흘렸"/"버렸") → "VV + 었/EP".

## 백로그

- P4 (borderline NNG↔NNP): Sprint 132 보류 5 entries
- accuracy-gate CI에 UD Kaist/GSD eojeol gate

## 검증 기준

- `cargo test --workspace --exclude mecab-ko-ffi` 전체 pass
- `cargo clippy --workspace --all-targets --exclude mecab-ko-ffi -- -D warnings` clean
- **5-gate CI 통과** (sample.tsv / KLUE morph / surface_only / UD Kaist / UD GSD)
- sample.tsv baseline 100%/99.9% **회귀 금지**
