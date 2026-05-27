# PLAN — mecab-ko Sprint 172 (next)

> 마지막 업데이트: 2026-05-27

## 완료: Sprint 171 — B-2 성능 진단

### 결과

- mecab-profile CLI 빌드 + 동작 확인
- Criterion tokenizer_bench 정상 (평균 10µs/sentence, 7 MiB/s)
- 성능 인프라 평가 (9 benches + 4 profiler 모듈)

### 제약

- Memory tracking은 test-allocator feature 필요 (현재 0 B 표시)
- 측정은 mini-dict (full mecab-ko-dic 미설정)

## 누적 진척 (Sprint 122 → 171)

| 영역 | 결과 |
|------|------|
| 정확도 lift (S122~167) | +6.3pp KLUE, 종료 |
| WASM tests (S169) | 5 → 11 |
| 바인딩 진단 (S170) | 72+ 테스트 확인 |
| 성능 진단 (S171) | baseline 측정 |

## Sprint 172 후보

### B-2 계속: 전체 Criterion bench suite
- 9개 bench (batch, cold_start, dict_loading, matrix, memory, normalization, tokenizer, trie, viterbi)
- 시간 소요 (각 수 분)
- 결과를 PERFORMANCE_BASELINES.md에 누적 기록

### B-2 심화: memory profiling
- `cargo build --features test-allocator`
- dict_profiler / tokenizer_profiler 실제 측정
- 핫스팟 식별 보고

### 유지보수 모드
- Sprint cycle 종료 선언
- 버그/의존성만 대응

### 외부 입수 대기
- NIKL Modu / Sejong 입수 시 재개

## 결정 프로세스

규칙 5: 전문가 리뷰 자동 채택. 다음 sprint-run 시 진행.

## 검증 기준

- `cargo test --workspace --exclude mecab-ko-ffi` 전체 pass
- `cargo clippy --workspace --all-targets --exclude mecab-ko-ffi -- -D warnings` clean
- **5-gate CI 통과**
- sample.tsv baseline 100%/99.9% **회귀 금지**
