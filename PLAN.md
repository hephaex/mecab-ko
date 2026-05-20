# PLAN — mecab-ko Sprint 153 (next, 자동 결정)

> 마지막 업데이트: 2026-05-21

## 완료: Sprint 152 D — Node/WASM CI 강화

### 결과

- `e2e-ffi-tests.yml` 7개 continue-on-error 분석
- 2개 제거: nodejs-e2e Build (hard-gated 중복), wasm-e2e Run (잡 레벨 중복)
- 5개 정당화 유지 + 명시 코멘트 추가
- `agents.md` **규칙 5 추가**: 자동 트랙 선택 (전문가 리뷰 기반)

## 완료된 P0 정리 작업 (총 6개)

| Sprint | 항목 | 결과 |
|--------|------|------|
| 149 | MSRV 1.80 CI gate | 추가 |
| 149 | coverage --fail-under 60 | 추가 |
| 149 | placeholder 테스트 삭제 | 2 파일 |
| 149 | accuracy_eval 진단 함수 24개 삭제 | 4963→2800 |
| 149 | multi-syllable rollback guard | 3 테스트 |
| 151 C | setup helper 추출 | 2800→2406 |
| **152 D** | **Node/WASM CI continue-on-error 정리** | **2개 제거** |

**P0 정리 완료**. 남은 모든 작업은 정확도 lift 또는 인프라.

## Sprint 153 — 자동 결정 (전문가 리뷰 기반)

agents.md 규칙 5에 따라 도메인 전문가 에이전트가 후보 분석 후 Top 권고를 자동 채택.

### 현재 후보

#### E: XSA+ETM 38건 분석

Sprint 145 빈도: XSA+ETM 38건 (스러운, 스런, 로운)
- "한가스러운" → 한가/NNG + 스럽/XSA + 운/ETM (ㅂ 불규칙 XSA)
- Sprint 150 A 패턴 연장선
- ending_rules 처리 여부 진단 후 결정

**비용**: 0.5 sprint, 중간 위험

#### B [대규모]: Full CRF Retrain (Track B)

3-5 sprint 장기 작업 (학습 데이터 + mecab-cost-train)
- **규칙 5 예외**: 비가역적 대규모 작업 → 사전 confirm 필요
- 잠재 lift +1~5pp

#### F: 전문가 식별 신규

전문가 리뷰가 발견할 새 정확도 영역.

## 검증 기준

- `cargo test --workspace --exclude mecab-ko-ffi` 전체 pass
- `cargo clippy --workspace --all-targets --exclude mecab-ko-ffi -- -D warnings` clean
- **5-gate CI 통과** (sample.tsv / KLUE morph / surface_only / UD Kaist / UD GSD)
- sample.tsv baseline 100%/99.9% **회귀 금지**
