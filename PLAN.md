# PLAN — mecab-ko Sprint 170 (next)

> 마지막 업데이트: 2026-05-27

## 완료: Sprint 169 — B-1: WASM 바인딩 테스트 확장

### 결과

- WASM 인라인 테스트 5 → **11개** (+6 신규)
- 추가 영역: nouns/wakati/empty/positions/optional/json_format
- 빌드 검증 정상 (wasm32 target)

## Sprint 170 후보 — B-1 계속 또는 다른 영역

### B-1 계속

#### Python (mecab-ko-python)
- tests/ 3개 → 확장 검토
- 가장 성숙한 바인딩 (BND-003 시리즈 문서)
- API surface 진단 후 우선순위

#### Node (mecab-ko-node)
- index.test.ts 320줄 (이미 풍부)
- 보완 영역 식별 필요
- pre-built binary 검증

### 다른 영역

#### B-2: 성능 진단 sprint
- mecab-ko-profiler 활용
- 핫스팟 식별
- 측정 + 보고

#### 유지보수 모드
- 정확도 sprint 종료
- 버그/의존성만

### 외부 의존 (사용자 작업)

- NIKL Modu 다운로드 → 측정
- Sejong 코퍼스 입수 → Track B 재시도

## 결정 프로세스

규칙 5 자동 채택 — B-1 계속 (Python 또는 Node) 또는 B-2 (성능) 시도.
다음 sprint-run 시 자동 결정.

## 검증 기준

- `cargo test --workspace --exclude mecab-ko-ffi` 전체 pass
- `cargo clippy --workspace --all-targets --exclude mecab-ko-ffi -- -D warnings` clean
- **5-gate CI 통과**
- sample.tsv baseline 100%/99.9% **회귀 금지**
