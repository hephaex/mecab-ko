# PROGRESS — mecab-ko Sprint 174 (Sprint Cycle 종료 + 유지보수 모드)

> 마지막 업데이트: 2026-05-27

## Sprint 174 — Cycle 종료 + 유지보수 모드 선언

| Task | 상태 | 결과 |
|------|------|------|
| S174-1: NIKL Modu 7번째 체크 | ⏸ 미다운로드 | cycle 종료 진행 |
| S174-2: Sprint cycle 총결산 작성 | ✅ 완료 | sprint174_cycle_termination.md |
| S174-3: 유지보수 모드 선언 | ✅ 완료 | sprint-run 정지 (트리거 조건 충족 시 재개) |

## 핵심: 유지보수 모드 진입

자동 진행 가능 영역 모두 소진. 사용자 결정 또는 외부 의존 작업 필요.

### sprint-run 재개 트리거

1. **NIKL Modu 다운로드 완료** — `./tools/nikl_modu_setup.sh <json>` 실행
2. **Sejong 코퍼스 입수** — Track B 재시도 (인프라 즉시 활용)
3. **사용자 명시 신규 영역** — 특정 기능/버그/바인딩

## Sprint Cycle 총결산 (Sprint 122 → 173)

### 정확도 누적 진척

| Metric | Baseline | 현재 | Δ |
|--------|---------|------|---|
| sample.tsv | 100%/99.9% | 100%/99.9% | — (보존) |
| **KLUE morph practical** | ~65.8% | **72.1%** | **+6.3pp** |
| KLUE eojeol practical | ~5000 | 5327 | +327 |
| **KLUE surface canonical_lenient** | ~89% | **95.6%** | **+6pp** |
| UD Kaist morph practical | — | 68.6% | (new silver) |
| UD GSD morph practical | — | 71.8% | (new silver) |

### 인프라 누적 진척

| 영역 | Δ |
|------|---|
| accuracy_eval.rs | 4963 → 2406 줄 (-51%) |
| WASM tests | 5 → 11 (+120%) |
| Docs archive | 28+ 파일 정리 |
| CI gate | sample.tsv → 5-gate |
| 성능 baseline | v0.7.2 5/9 benches 측정 |

### Sprint 분류 (52 sprints, ~1.5 개월)

| 유형 | 건수 |
|------|------|
| Lift sprints | 11 |
| Infrastructure | 12 |
| Rollback | 4 |
| 비이슈 확인 | 3 |
| 정리 | 8 |
| 진단 only | 6 |
| 외부 인프라 | 1 |
| Track B (실패) | 5 |
| 종합 정리 | 2 |

## 검증된 영역 매트릭스 (확정)

### ✅ 안전 영역
- PRACTICAL 동치 그룹 (NNB/NNG, VA/VV, VV/XSV, MAG/MAJ)
- Surface normalization (하았/이습니다/르/ㄷ 불규칙 + 명시 어구)
- Splitter rule (제한적, mecab dict 미처리만)
- Silver dataset 통합

### ❌ 위험 영역 (회귀 4회)
- matrix.def cost (S138)
- multi-syllable VV+ETM (S145)
- dict cost=-5000 (S155)
- CRF retrain (POS only) (S167)

→ viterbi/CRF 직접 변경 = cascade 회귀 매우 큼.

### ⏸ 비이슈 (mecab dict 처리)
- ETM+ETM, XSA+ETM, EP+ETM 등 833건 빈도 → 실효 24건만 (2.9%)

## 메타 학습

1. mecab dict의 압도적 강력함 (decomposition fallback)
2. 빈도 ≠ 실효 lift (변환 후 측정 필수)
3. 3 silver 일관 lift = 진짜 효과
4. Rollback 신속화 (Sprint 138 정책)
5. 격리의 가치 (별도 dict + 환경 변수)
6. 진단 데이터 재활용 (다른 접근 가능)
7. 규칙 5 자동 트랙 선택 (전문가도 측정으로 검증)
8. 작은 corpus + sparse features = overfit 함정

## Track B 자산 보존

Sejong 코퍼스 입수 시 즉시 재활용:
- legacy/ macOS arm64 빌드 (S164)
- tools/to_mecab_tagged.py (S165)
- 4단계 학습 파이프라인 62.6초 (S166)
- 격리 메커니즘 (S167)

## 검증

- `cargo test --workspace --exclude mecab-ko-ffi --lib`: 717 pass (2026-06-25 기준, +6 from 711)
- 5-gate sample.tsv: 100.0%/99.9% (baseline 유지)
- 성능 회귀 없음 (v0.3.0 → v0.7.2)

## 변경 파일

- `docs/research/accuracy/2026-05-27_sprint174_cycle_termination.md` (신규)
- `PLAN.md`, `PROGRESS.md` 유지보수 모드 갱신

## Sprint 175+ (트리거 조건 발생 시 재개)

자동 sprint-run 정지 상태. 다음 명시 작업 발생 시 즉시 재개 가능.

## Sprint 175 — REDTEAM_REVIEW.md 후속 조치 (2026-07-02)

`docs/REDTEAM_REVIEW.md` (2026-06-30 정적 분석 + cargo audit/clippy) 우선순위 로드맵 중
자동 진행 가능한 항목 처리. `docs/REVIEW_HIGHLIGHTS.md`의 clamp_oob_cost / silent
eager fallback / JNI catch_unwind 항목은 재검토 결과 이미 코드에 반영되어 있음을 확인
(리뷰 문서가 stale — 코드가 리뷰보다 최신).

| Task | 상태 | 결과 |
|------|------|------|
| H-3: 사전 헤더 정수 오버플로우 | ✅ 완료 | `lazy_entries_v3.rs`(2곳)·`lazy_entries.rs`(1곳) `checked_add`/`checked_mul`로 교체, `index_pos+8`/`table_pos+8` 오버플로우도 방어 |
| M-1: `deny.toml` `[advisories]` 미설정 | ✅ 완료 | vulnerability/yanked 기본 deny 유지, `unmaintained`/`unsound` scope="workspace", 이미 추적 중인 RUSTSEC ID 6건에 근거 주석 포함 `ignore` 등록 |
| 신규 발견: quick-xml 0.36.2 취약점 (RUSTSEC-2026-0195/0194) | ✅ 완료 | `mecab-ko-dict-sync`의 직접 의존성을 0.41로 상향 (REDTEAM 검토일 이후 신규 advisory) |
| H-1 quinn-proto / H-2 memmap2 / M-4 rkyv / M-5 anyhow | ⏸ 보류 | 패치 버전 upstream 미출시 또는 실제 도달 불가 경로(quinn-proto는 mockito 개발 의존성, http3 미사용) — `deny.toml`에 근거와 함께 ignore 등록 |
| M-3 JNI 경로 검증 / M-2 캐시 키 충돌 | 미착수 | 다음 세션 후보 |

### 검증
- `cargo build --workspace`: 성공
- `cargo test --workspace --exclude mecab-ko-ffi --lib`: 717 pass (회귀 없음)
- `cargo clippy --workspace`: 경고 0
- `cargo deny check`: advisories/bans/licenses/sources ok

### 변경 파일
- `rust/crates/mecab-ko-dict/src/lazy_entries_v3.rs`
- `rust/crates/mecab-ko-dict/src/lazy_entries.rs`
- `rust/crates/mecab-ko-dict-sync/Cargo.toml`
- `rust/deny.toml`

이 작업은 PLAN.md 유지보수 모드의 "사용자 명시 신규 영역" 트리거(레드팀 리뷰 후속 조치)로
진행됨. NIKL Modu/Sejong 코퍼스 트리거는 여전히 미충족 — 정확도 sprint는 계속 정지 상태.
