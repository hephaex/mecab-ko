# PLAN — mecab-ko Sprint 143 (next)

> 마지막 업데이트: 2026-05-20

## 완료: Sprint 142 B — dict-builder CSV unquoted comma surface 수정

### 문제
- entries.csv 단일 행 (`,,1792,3558,...` — surface=",", unquoted) 으로 dict-builder 전체 실패
- Sprint 138에서 발견. 이번 sprint에서 해결.

### 수정
- `src/lib.rs:parse_csv_content`: record count 보정 (13 fields + record[0]/[1] empty → surface=",", field shift)
- 2 신규 단위 테스트 (unquoted/quoted surface 모두 검증)

### Round-trip 검증
- dict-builder 전체 mecab-ko-dic 재빌드 성공 (1.63M entries, 77초)
- 4-gate 무회귀 (sample.tsv / KLUE / surface-only / UD Kaist 모두 동일)

### 효과
**Sprint 138 차단 원인 해결 → Track E (Full CRF Retrain) 진입 가능**.

### 보고서
`docs/research/accuracy/2026-05-20_sprint142_dict_builder_csv_fix.md`

## 다음 스프린트: Sprint 143 (미정 — 사용자 선택)

### 후보 B [메인]: Full CRF Retrain (Track E)

**선행 조건 충족**: Sprint 142로 dict-builder 정상 작동.

**작업 (3-5 sprint 예상)**:
1. 학습 코퍼스 준비 (Sejong + KLUE train + UD Kaist train)
2. `legacy/src/mecab-cost-train` (C++) 빌드 + 실행
3. 새 `model.def` → `matrix.def` + `left/right-id.def` 재생성
4. `cargo run --bin mecab-ko-dict-builder` 재실행 → binary
5. 4-gate 회귀 검증 + lift 측정

**리스크**:
- 학습 코퍼스 라이선스 (Sejong 비공개, KLUE+UD CC BY-SA 4.0)
- left/right-id.def 변경 시 기존 binary 호환성 깨질 수 있음
- 학습 시간 (수 시간)
- 정확도 향상 잠재 +1pp 이상 (지금까지 시도와 차원 다른 결과)

### 후보 A: 다른 mecab 결합 토큰 패턴 조사

NNG+VCP+EC, VV+EP+EF 등. Sprint 141 패턴 확장. 비용 0.5-1 sprint, 위험 낮음.

### 후보 C: UD Korean-GSD 통합 (변환기 재사용)

CC BY-SA 4.0, 6,339 sentences. `convert_ud_kaist.py` 재사용 가능. 비용 0.5 sprint.

### 후보 D: NIKL Modu 수동 다운로드 + 평가

Academic license, 로컬 only. 변환 스크립트 + 평가 통합. 비용 0.5-1 sprint.

## 백로그

- P4 (borderline NNG↔NNP): Sprint 132 보류 5 entries
- accuracy-gate CI 추가 게이트 (UD Kaist eojeol 등)

## 검증 기준 (모든 후보 공통)

- `cargo test --workspace --exclude mecab-ko-ffi` 전체 pass
- `cargo clippy --workspace --all-targets --exclude mecab-ko-ffi -- -D warnings` clean
- 4-gate CI 통과 (sample.tsv / KLUE morph / surface_only / UD Kaist)
