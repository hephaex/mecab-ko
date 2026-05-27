# PLAN — mecab-ko Sprint 171+ (자동 진행 가능 영역 거의 소진)

> 마지막 업데이트: 2026-05-27

## 완료: Sprint 170 — B-1 바인딩 영역 종합 진단

### 결과

- Python tests 30+, Node tests 31, WASM tests 11 = **총 72+ 테스트**
- 3 바인딩 모두 충분히 검증됨
- B-1 영역 추가 작업의 marginal value 낮음

## 자동 진행 가능 영역 종합 매트릭스

| 영역 | 상태 | 마지막 sprint |
|------|------|------|
| 정확도 lift (안전) | 영역 소진 | S158 |
| 정확도 lift (위험 viterbi/CRF) | 4회 회귀 → 종료 | S167 |
| WASM tests | 11개 (Sprint 169) | S169 |
| Python tests | 30+ (이미 풍부) | (기존) |
| Node tests | 31개 (이미 풍부) | (기존) |
| Docs 정리 | 28+ 파일 archive | S162 |
| CI 정리 | MSRV/coverage/continue-on-error | S152 |
| Setup helper 추출 | -51% lines | S151 |
| 진단 함수 정리 | -43% lines | S149 |

**거의 모든 안전·자동 작업 완료.**

## Sprint 171+ 옵션 (사용자 결정 필요)

### 옵션 1: B-2 성능 진단 sprint
- mecab-ko-profiler 활용
- 핫스팟 식별 (Viterbi/dict lookup)
- 측정 + 보고만 (구현은 별도)
- 비교적 안전, 자동 가능

### 옵션 2: 유지보수 모드 (권고)
- sprint cycle 공식 종료
- 버그 픽스, 의존성 업데이트만
- 다음 메이저 작업 (Sejong 입수, NIKL Modu 다운로드, 사용자 요청) 대기

### 옵션 3: 외부 입수 대기
- NIKL Modu (`tools/nikl_modu_setup.sh` 준비됨)
- Sejong 코퍼스 (Track B 재시도)

### 옵션 4: 사용자 명시 신규 영역
- 특정 기능 추가
- 특정 버그 픽스
- 사용자 우선순위 명시 시 진행

## Sprint 122 → 170 종합 누적

| Metric | Baseline | 현재 |
|--------|---------|------|
| sample.tsv | 100%/99.9% | 100%/99.9% (보존) |
| **KLUE morph practical** | ~65.8% | **72.1%** (+6.3pp) |
| **KLUE surface canonical_lenient** | ~89% | **95.6%** (+6pp) |
| UD Kaist morph practical | — | 68.6% |
| UD GSD morph practical | — | 71.8% |
| accuracy_eval.rs 줄 수 | 4963 | 2406 (-51%) |
| WASM tests | 5 | 11 (+120%) |

## 검증 기준 (모든 옵션 공통)

- `cargo test --workspace --exclude mecab-ko-ffi` 전체 pass
- `cargo clippy --workspace --all-targets --exclude mecab-ko-ffi -- -D warnings` clean
- **5-gate CI 통과**
- sample.tsv baseline 100%/99.9% **회귀 금지**
