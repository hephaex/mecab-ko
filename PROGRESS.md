# PROGRESS — mecab-ko Sprint 173 (B-2: dict_loading + normalization)

> 마지막 업데이트: 2026-05-27

## Sprint 173 — B-2 계속 (dict_loading + normalization)

| Task | 상태 | 결과 |
|------|------|------|
| S173-1: dict_loading_bench 측정 | ✅ 완료 | 182 ns ~ 334 µs |
| S173-2: normalization_bench 측정 | ✅ 완료 | 1.1 ~ 27 µs |
| S173-3: PERFORMANCE_BASELINES.md 갱신 | ✅ 완료 | 2 섹션 추가 |

## 측정 결과

### Dictionary Loading

| 시나리오 | Time |
|---------|------|
| cached lookup (가장 빠름) | 182 ns |
| typical lookup | 2.1~3.4 µs |
| medium load | 91~111 µs |
| complex load | 268~334 µs |

dict lookup 매우 빠름 (cache hit 182 ns).

### Normalization

| 시나리오 | Time |
|---------|------|
| 가장 빠름 | 1.1 µs |
| 일반 | 4.2~9.0 µs |
| 복잡 | 15~27 µs |

Sprint 156 ㄷ 불규칙 + Sprint 158 명시 어구 추가 후 정상 범위 — **회귀 없음**.

## 누적 성능 측정 (Sprint 171~173)

| Bench | 측정 |
|-------|------|
| tokenizer_bench | ✅ (S171) |
| cold_start_bench | ✅ (S172) |
| viterbi_bench | ✅ (S172) |
| **dict_loading_bench** | **✅ (S173)** |
| **normalization_bench** | **✅ (S173)** |
| batch_bench | 미측정 |
| matrix_bench | 미측정 |
| memory_bench | 미측정 |
| trie_bench | 미측정 |

**5/9 benches 측정 완료**. 핵심 영역 모두 완료. 남은 4개 (batch/matrix/memory/trie) marginal value.

## 변경 파일

- `docs/PERFORMANCE_BASELINES.md`: dict_loading + normalization 섹션 추가
- `PLAN.md`, `PROGRESS.md` 갱신

## 검증

- `cargo test --workspace --exclude mecab-ko-ffi --lib`: 변경 없음 (411 pass)
- 5-gate sample.tsv: 영향 없음
- Criterion benches: 정상 출력

## Sprint 174 후보

### B-2 종료 권고

5/9 핵심 benches 측정 완료. 나머지 4개 (batch/matrix/memory/trie) 측정은 marginal value:
- 핵심 알고리즘 (viterbi, tokenizer, cold_start) 이미 측정
- 남은 영역은 derivative 또는 보조

**권고**: B-2 종료 → 유지보수 모드 또는 외부 입수 대기.

### 또는 B-2 심화 (memory profiling)

test-allocator feature 활성 시 실제 메모리 측정 가능. 단, 시간 소요.

### 외부 입수 대기

NIKL Modu / Sejong 다운로드 시 정확도 재개.
