# PLAN — mecab-ko Sprint 142 (next)

> 마지막 업데이트: 2026-05-20

## 완료: Sprint 141 A — VCP+ETM/EC splitter fix

### 가설 검증 (초기 가설 폐기)
XSN/VCP practical 동치는 무효. 실제 문제는 mecab의 결합 토큰 출력.

### 구현
- `splitter.rs`에 VCP+ETM 패턴 추가: `인` → `이/VCP + ㄴ/ETM`, `일` → `이/VCP + ㄹ/ETM`, `라는` → `이/VCP + 라는/ETM`
- VCP+EC 패턴: `라`/`며`/`라서`/`라고`/`라며`/`라면`/`라야`/`라든지` → `이/VCP + X/EC`
- 4 단위 테스트 (overcorrect 방지 포함)

### 측정 결과
- UD morph strict: 66.3% → 66.4% (+0.1pp)
- KLUE: 무변경 (VCP+ETM 패턴 KLUE에 적게 등장)
- sample.tsv: 100.0%/99.9% 무회귀
- UD eojeol -2건 미미 회귀 (-0.01pp, 무시 수준)

### 보고서
`docs/research/accuracy/2026-05-20_sprint141_vcp_split_fix.md`

## 다음 스프린트: Sprint 142 (미정 — 사용자 선택)

### 후보 A: 다른 mecab 결합 토큰 패턴 조사

**근거**: VCP+ETM/EC 분리가 효과 있었고 안전했음. 다른 패턴도 동일 접근.

**가능한 패턴**:
- NNG+VCP+EC (예: "이고" → mecab 한 토큰)
- VV+EP+EF (예: "왔다", "갔다" 결합 표기)
- 추가 결합 패턴 분석 필요

**작업**:
1. mecab 출력 vs KLUE/UD gold 분해 비교
2. 빈도 분석 (KLUE + UD 양쪽)
3. 안전한 분리 규칙 추가
4. 단위 테스트 + 회귀 검증

**비용**: 0.5-1 sprint
**위험**: 낮음 (Sprint 141 패턴 확립됨)

### 후보 B: dict-builder CSV 버그 수정 (Track D 선행)

Sprint 138 미해결. Full CRF retrain 진입 전 필요.

**비용**: 0.5-1 sprint
**위험**: 낮음

### 후보 C: NIKL Modu 또는 OpenKorPOS 추가

추가 silver 평가 데이터셋 통합. 도메인 다양성 강화.

**비용**: 0.5-1 sprint
**위험**: 낮음 (UD Kaist 통합 패턴 재사용 가능)

### 후보 D: UD Korean-GSD 통합

UD Kaist와 같은 변환기 (`convert_ud_kaist.py`) 재사용 가능.
6,339 sentences, CC BY-SA 4.0.
도메인은 Google news 등 — 또 다른 silver source.

**비용**: 0.5 sprint (스크립트 재사용)
**위험**: 낮음

### 후보 E: Full CRF Retrain (escalation, B 선행 필요)

3-5 sprint 대규모 작업. 학습 코퍼스 라이선스 + binary 호환성 리스크.

## 백로그

- P4 (borderline NNG↔NNP): Sprint 132 보류 5 entries
- accuracy-gate CI 추가 게이트 (UD Kaist eojeol gate 등)

## 검증 기준 (모든 후보 공통)

- `cargo test --workspace --exclude mecab-ko-ffi` 전체 pass
- `cargo clippy --workspace --all-targets --exclude mecab-ko-ffi -- -D warnings` clean
- 4-gate CI 통과 (sample.tsv / KLUE morph / surface_only / UD Kaist)
