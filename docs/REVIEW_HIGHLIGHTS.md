# mecab-ko 리뷰 하이라이트
> 리뷰 날짜: 2026-06-25 | 버전: v0.7.2 | Opus 아키텍처 리뷰 판정: **WATCH / COMMENT**
> **후속 조치 완료: 2026-07-17** — 아래 🔴 버그 항목은 전부 수정되어 커밋됨
> (`56a71b5` Opus 리뷰 픽스, `1564c0d` dict 헤더 오버플로우, `995473c` crossbeam-epoch).
> 이 문서는 당시 리뷰 기록이며, 현재 상태는 PROGRESS.md와 docs/REDTEAM_REVIEW.md 후속 조치 표 참조.
> 전체 리뷰: `~/.gjc/memory/project_mecab_ko.md`

---

## 🔴 실제 버그 — Opus 발견 (MEDIUM 리스크)

### 1. `clamp_oob_cost` 계약 누락 — N-best + SIMD (public API 영향)

**근본 원인**: `DEFAULT_OOB_CONNECTION_COST = 10_000` 클램핑이 `nbest.rs:360`과 `simd.rs:275` (scalar fallback)에서 **누락됨**.  
→ 같은 OOV 노드가 1-best에서는 cost 10,000으로 허용되지만 N-best/SIMD fallback에서는 `i32::MAX` (prohibited)로 처리됨  
→ `nbest[0] ≠ 1-best` 불일치 발생 — **shipped public API**

**수정 방향**: `ConnectionCost::cost` 계약 내부에서 clamp 강제화 (호출지점 분산 구조가 근본 원인)

```rust
// nbest.rs:360 — 현재
let conn_cost = matrix.connection_cost(right_id, left_id);  // i32::MAX 가능

// 수정
let conn_cost = clamp_oob_cost(matrix.connection_cost(right_id, left_id));
```

**검증**: OOV 문장으로 `1-best == nbest[0]` property test 추가

---

### 2. `LazyEntries V3` 실패 시 silent eager fallback (`dictionary.rs:308-316`)

**현상**: rkyv alignment/validation 실패 시 tracing::warn 없이 eager fallback → ~34MB 메모리 목표 무력화 + 손상 dict 디버깅 불가

**수정**:
```rust
// dictionary.rs:308-316 에 추가
tracing::warn!(?error, ?path, "LazyV3 load failed, falling back to eager (memory target voided)");
```

---

### 3. JNI extern 함수에 `catch_unwind` 부재 (`jni.rs`)

**현상**: release build의 `panic=abort` 설정 + FFI 경계 panic → JVM 프로세스 강제 abort  
**수정**: `std::panic::catch_unwind`로 모든 `extern "C" fn` 진입점 래핑

---

### 4. Facade wildcard re-export → semver 표면 비대

`mecab-ko/src/lib.rs`에서 `pub use mecab_ko_dict::*` + `pool/memory/evaluate` 노출  
→ 내부 구현 item들이 public API로 승격, v1.0 stability 위험

---

### 5. SIMD 경로 clamp 비일관 (`simd.rs:182-189`, `:275`)

nodes < 8일 때 scalar fallback 실행, OOB clamp 없음 → scalar Viterbi와 발산  
(nightly 전용 + CI `continue-on-error`라 현재 미shipped이나 활성 시 비결정 결과)

---

## ⚡ 즉시 확인 필요 (인프라)

| 항목 | 기한 | 조치 |
|------|------|------|
| **GITHUB_TOKEN PAT 만료** | 2026-09-07 | CT118 `/opt/baram/.env` PAT 갱신 |
| **refs/tags/main ambiguous** | 수시 | `git push origin :refs/tags/main` |

---

## 💪 강점 (Opus 인정)

1. **명확한 crate 경계 + trait 추상화** — `EntryStore` trait + `impl_lazy_store!` 매크로, `blanket impl<T: Matrix> ConnectionCost`
2. **메모리 전략** — LazyEntries v3(rkyv zero-copy)+mmap+compact_str → ~34MB(-77%)
3. **보안 위생** — `deny(unsafe_code)` + `cargo-deny` + RUSTSEC 전수 대응 문서화
4. **수치 안정성** — `i16` cell + `i32::MAX` OOB sentinel + `saturating_add_chain`
5. **JNI 설계** — monotonic i64 handle + `RwLock<HashMap<i64,Arc<Mutex>>>` registry (raw pointer 미노출)

---

## ⚠️ 아키텍처 리스크 요약

| 등급 | 항목 | 파일 |
|------|------|------|
| MEDIUM | clamp 계약 누락 (N-best) | `nbest.rs:360` |
| MEDIUM | SIMD clamp 비일관 | `simd.rs:275`, `:182` |
| MEDIUM | silent eager fallback | `dictionary.rs:308-316` |
| MEDIUM | facade wildcard re-export | `mecab-ko/src/lib.rs` |
| MEDIUM | JNI catch_unwind 부재 | `jni.rs` |
| LOW | SIMD CI 비게이트 | `.github/workflows/ci.yml` |
| LOW | full dict 215MB (목표 150MB) | LazyEntries 추가 최적화 필요 |
| LOW | deny.toml wildcards=allow | `deny.toml` |

---

## 📊 현재 지표 (검증됨)

| 지표 | 값 | 측정 조건 |
|------|-----|----------|
| KLUE morph practical | **72.1%** | 5,327 eojeol |
| KLUE surface canonical | **95.6%** | canonical_lenient |
| sample.tsv | **100%/99.9%** | CI gate (411 lib tests) |
| 토큰화 속도 | **~680K tok/sec** | mini-dict |
| Cold start | **90~100 µs** | Tokenizer::new() |
| Tokenizer 재사용 | **3.3 µs** | 30x faster than cold |
| 메모리 (mini-dict) | **~34MB** | LazyEntries v3 |
| 메모리 (full-dict) | **215MB** | 목표 150MB 미달 |

---

## 🚫 건드리지 말 것 (Danger Zones)

```
rust/crates/mecab-ko-core/src/viterbi/mod.rs    — 비용 모델 직접 수정 금지 (4회 회귀)
rust/crates/mecab-ko-dict/src/matrix/           — matrix.def 비용 변경 금지
rust/crates/mecab-ko-core/src/sejong/           — CRF 기반 변경 (회귀 확인)
rust/crates/mecab-ko-core/src/viterbi/simd.rs   — clamp 비일관 상태, 수정 전 property test 필수
```

**변경 전 반드시**: `cargo test --workspace --exclude mecab-ko-ffi --lib` + Silver 3개 전체 평가

---

## 🔑 인프라 요점

```yaml
서버:     CT118 /opt/mecab-ko
SSH키:    /root/.ssh/id_ed25519_mecab
Deploy:   GitHub deploy key ID 153966402
PAT:      /opt/baram/.env (만료 2026-09-07!)
스크립트: /opt/baram/weekly-dict-publish.sh
Cron:     30 13 * * 6 (토 22:30 KST)
최신PR:   #5 (2026-06-09, 6,333 NNP)
```

**인프라 작업 전 필수:**
```bash
cat ~/.gjc/memory/project_mecab_ko.md
cat ~/.gjc/memory/project_mecab_ko_dict_publish.md
```

---

## 📅 다음 액션 (우선순위 순) — 2026-07-17 갱신

| 우선순위 | 트리거 | 조치 | 상태 |
|----------|--------|------|------|
| **1 (버그)** | 즉시 | `nbest.rs` + `simd.rs` clamp 추가 + property test | ✅ 완료 (`56a71b5`) |
| **2 (관측성)** | 즉시 | `dictionary.rs` tracing::warn 추가 | ✅ 완료 (`56a71b5`) |
| **3 (안전)** | 즉시 | JNI `catch_unwind` 래핑 | ✅ 완료 (`56a71b5`) |
| **4 (API)** | v1.0 전 | facade wildcard re-export 화이트리스트화 | ✅ 완료 (`56a71b5`) |
| **5 (데이터)** | 외부 의존 | NIKL Modu 다운로드 → 정확도 sprint 재개 | ⏸ 대기 |
| **6 (인프라)** | 2026-09-07 | PAT 갱신 | ⏳ 미완 (7주 남음) |

---

*최종 갱신: 2026-07-17 (후속 조치 상태 반영) | 원 리뷰: 2026-06-25 Opus architect | /skill:review-project 자동 생성*
