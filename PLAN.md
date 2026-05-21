# PLAN — mecab-ko Sprint 160 (next, 사용자 작업 대기)

> 마지막 업데이트: 2026-05-21

## 완료: Sprint 159 F — NIKL Modu 인프라 준비

### 산출물

- `tools/convert_nikl_modu.py` — NIKL JSON → TSV 변환
- `accuracy_eval.rs::test_nikl_modu_dual_metric` — skip 패턴
- `docs/eval/nikl_modu_setup.md` — 다운로드/설정 가이드
- `.gitignore` — NIKL Modu pattern 추가

### 사용자 액션 필요

1. https://kli.korean.go.kr 학술 등록
2. NIKL Modu 형태분석 corpus 다운로드 (1-3일 승인 대기)
3. 변환 + 평가 실행

## Sprint 160 후보 (사용자 다운로드 진행 상황에 따라)

### 시나리오 A: NIKL Modu 다운로드 완료
- 측정 → POS mismatch 분석
- practical 동치 / normalize 추가 후보 발굴
- 도메인 차이 학습

### 시나리오 B: NIKL Modu 다운로드 대기 또는 보류
- 정확도 외 영역으로 전환:
  - **문서 정리**: docs/research 아카이브 (sprint 132~159 보고서 정리)
  - **CLI/API 사용성**: mecab-ko-cli 옵션 개선
  - **성능 최적화**: 프로파일링 + 핫스팟 식별
  - **새 언어 바인딩 강화**: Python/WASM/Node 통합

### 시나리오 C: CRF Retrain 결정
- Track B 시작 (3-5 sprint 장기)
- 학습 데이터 준비 + mecab-cost-train
- 잠재 lift +1~5pp KLUE morph

## 현재 정확도 지표 (Sprint 158 후)

| Metric | 현재 |
|--------|------|
| sample.tsv | 100.0%/99.9% |
| KLUE morph practical | 72.1% |
| KLUE surface canonical_lenient | 95.6% |
| UD Kaist morph practical | 68.6% |
| UD GSD morph practical | 71.8% |

## 검증 기준

- `cargo test --workspace --exclude mecab-ko-ffi` 전체 pass
- `cargo clippy --workspace --all-targets --exclude mecab-ko-ffi -- -D warnings` clean
- **5-gate CI 통과**
- sample.tsv baseline 100%/99.9% **회귀 금지**
