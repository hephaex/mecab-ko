# PLAN — mecab-ko Sprint 149 (next)

> 마지막 업데이트: 2026-05-20

## 완료: Sprint 148 D — ETM+ETM "라는" 분석

### 발견

- mecab: `라는/ETM+ETM` (내부 복합 분석)
- gold: `라는/ETM` (단일 형태소)
- SejongConverter 중복 태그 규칙(splitter.rs L71-73)이 이미 정규화
- **0 mismatch** — 코드 변경 불필요

### 보고서

`PROGRESS.md` Sprint 148 섹션 참조

## 완료: Sprint 147 A — VV/XSV practical 동치 추가

### Lift (3 silver 모두)

- KLUE practical morph: 71.6% → 71.9% (+0.3pp)
- UD Kaist practical morph: 68.1% → 68.3% (+0.2pp)
- UD GSD practical morph: 71.3% → 71.7% (+0.4pp)
- sample.tsv 무회귀

## 다음 스프린트: Sprint 149 (미정 — 사용자 선택)

### 후보 A: VA+ETM 542건 분석

VV+ETM 5376건은 이미 처리 중. VA+ETM 542건은 형용사 활용형:
- "어려울" → 어렵/VA + ㄹ/ETM
- "바른" → 바르/VA + ㄴ/ETM
- "큰" → 크/VA + ㄴ/ETM

SejongConverter가 이미 처리하는지 확인 필요. 처리하지 않으면 분리 규칙 추가 대상.

**복잡도**: Sprint 141 VCP+ETM 패턴 유사 (irregular conjugation stem 식별 필요)
**비용**: 0.5-1 sprint
**위험**: 중간

### 후보 B [메인]: Full CRF Retrain (Track E)

3-5 sprint. 학습 데이터 + mecab-cost-train.

### 후보 C: NIKL Modu 수동 다운로드

Academic license, 구어/SNS 도메인 확장.

### 후보 E: 추가 practical 동치 후보 조사

Sprint 147 패턴 (POS scheme 차이) 연장선. 나머지 compound POS 패턴에서
additional conventional disagreement 탐색.

## 백로그

- P4 (borderline NNG↔NNP)
- accuracy-gate CI에 UD eojeol gate 추가

## 검증 기준

- `cargo test --workspace --exclude mecab-ko-ffi` 전체 pass
- `cargo clippy --workspace --all-targets --exclude mecab-ko-ffi -- -D warnings` clean
- **5-gate CI 통과** (sample.tsv / KLUE morph / surface_only / UD Kaist / UD GSD)
- sample.tsv baseline 100%/99.9% **회귀 금지**
