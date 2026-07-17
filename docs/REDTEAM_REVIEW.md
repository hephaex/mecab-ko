# mecab-ko Redteam Review (v0.7.2)

> 정적 분석(코드 읽기·grep·구조 분석) + `cargo build`/`cargo audit`/`cargo clippy` 실행 기반.
> 비목표: 신규 PoC/fuzz, 소스 수정, 외부 실공격. 모든 소견은 file:line 증거 인용.
> 검토일: 2026-06-30 · 도구: cargo 1.95.0, cargo-audit 0.22.0, clippy 0.1.95.

## 요약 (심각도별 집계)

| 심각도 | 건수 | 컴포넌트 |
|--------|------|----------|
| Critical | 0 | — |
| High | 3 | 구조/공급망(quinn-proto), 형태소엔진(memmap2 unsound), 형태소엔진(정수 오버플로우) |
| Medium | 5 | rkyv unsound, anyhow unsound, JNI setDictionaryPath, 캐시 키 충돌, deny.toml advisories 미설정 |
| Low | 4 | unmaintained 크레이트 2종, CSV 수동 split, SIMD nightly-only, dict-sync 네트워크 노출 |

빌드: `cargo build --workspace` **성공** (12.6s). clippy: **경고 0**. unsafe는 `#![deny(unsafe_code)]` 하에 명시적 `#[allow]` 지점(mmap 4곳, JNI 7곳, SIMD 2곳)에만 존재 — 설계 규율은 양호.

---

## 1. 구조/아키텍처 (공급망·FFI 경계·unsafe 규율)

### [HIGH] H-1 quinn-proto 원격 메모리 고갈 (RUSTSEC-2026-0185)
- **증거**: `cargo audit` → `quinn-proto 0.11.14` (CVSS 7.5). 의존 경로: `mecab-ko-dict-sync → reqwest 0.12.28 → quinn 0.11.9 → quinn-proto`.
- **공격면**: dict-sync는 원격 사전 동기화로 네트워크에 노출됨. 비순차 스트림 재조립의 무제한 버퍼링으로 원격 메모리 고갈(DoS).
- **수정 방향**: `quinn-proto >= 0.11.15`로 업그레이드. `cargo update -p quinn-proto`. dict-sync가 선택적 기능이면 default-features에서 제외 검토.

### [MEDIUM] M-1 `deny.toml`의 `[advisories]` 섹션 비어 있음
- **증거**: `rust/deny.toml:6` — `[advisories]` 헤더만 있고 `vulnerability`/`unmaintained`/`yanked`/`unsound` 처리 레벨 미지정.
- **영향**: cargo-deny가 RUSTSEC 권고를 deny로 강제하지 않음. CI가 `cargo deny check`만 돌리면 위 High/unsound가 빌드를 막지 못하고 통과될 수 있음.
- **수정 방향**: `[advisories]`에 `unmaintained = "warn"`, `yanked = "deny"`, 그리고 알려진 unsound에 대한 명시적 `ignore` 목록(만료일 주석 포함)을 추가. 또는 의존성 업그레이드로 권고 자체를 제거.

### [LOW] L-1 unmaintained 전이 의존성 2종
- **증거**: `paste 1.0.15` (RUSTSEC-2024-0436, `tikv-jemalloc-ctl → mecab-ko-profiler`), `proc-macro-error2 2.0.1` (RUSTSEC-2026-0173, `tabled → mecab-ko-profiler`).
- **영향**: profiler 전용(런타임 핵심 아님). 보안 영향 낮으나 장기 유지보수 리스크.
- **수정 방향**: tabled/jemalloc 대체 또는 상위 버전 추적. profiler를 optional feature로 격리.

### [정보] 양호 신호
- `rust/Cargo.toml:51,66,90` — bincode(RUSTSEC-2025-0141), rkyv(RUSTSEC-2026-0001), pyo3(RUSTSEC-2025-0020) 대응을 주석으로 추적 중.
- `rust/deny.toml:31` — `wildcards = "deny"` (와일드카드 버전 금지). 공급망 위생 양호.
- FFI 크레이트(python/wasm/node)는 workspace에서 의도적으로 제외(`Cargo.toml:26-30`)되어 플랫폼별 빌드 격리.

---

## 2. 토크나이저 (API·분석 모드·캐싱·입력 처리)

### [MEDIUM] M-2 토큰 캐시 키 충돌 시 원문 미검증
- **증거**: `rust/crates/mecab-ko-core/src/cache.rs:211` `make_key`는 `DefaultHasher`(SipHash) u64만 반환. `get`(cache.rs:218)과 `get_or_insert`는 **키만으로 조회하고 원본 텍스트를 저장/비교하지 않음**.
- **공격면**: 64비트 해시 충돌(생일 문제상 실무 확률 극히 낮으나 0 아님)이 발생하면 입력 A의 캐시가 입력 B에 반환 → **조용한 오분석**. redteam 관점: 의도적 충돌 입력으로 검색 결과 오염 가능.
- **수정 방향**: `CacheEntry`에 원문 문자열(또는 길이+prefix)을 저장하고 hit 시 동일성 확인. 또는 128비트 해시 사용. 성능 민감 시 충돌 검증을 디버그 빌드에서만 활성화하는 것은 부적절(런타임 정확성 문제이므로 항상 검증 권장).

### [LOW] L-2 입력 처리 — 분석 모드/경계 입력
- **증거**: README 노출 API(`tokenize`/`wakati`/`nouns`/`verbs`/`lemmas`)는 빈 문자열·거대 입력에 대한 명시적 상한이 코드에서 보이지 않음. (캐시는 `max_entries` LRU로 제한되나 단일 입력 길이 상한은 별도 확인 필요.)
- **수정 방향**: 공개 API에 입력 길이 상한 옵션 또는 문서화된 권고 추가. 매우 긴 입력에 대한 lattice 메모리 사용 프로파일 측정.

---

## 3. 형태소분석 엔진 (Viterbi·Trie·Matrix·사전 바이너리)

### [HIGH] H-2 memmap2 unsound — unchecked pointer offset (RUSTSEC-2026-0186)
- **증거**: `cargo audit` → `memmap2 0.9.10` unsound. mecab-ko 전체 mmap 로딩의 기반: `trie/mmap.rs:27`, `matrix/mmap.rs:46`, `lazy_entries.rs:93`, `lazy_entries_v3.rs:118` 모두 `unsafe { Mmap::map(&file) }`.
- **공격면**: 사전 파일 로딩 경로 전체가 영향. SAFETY 주석(`trie/mmap.rs:25-26`, `lazy_entries_v3.rs:117-118`)이 "외부에서 파일을 변조/truncate하지 않아야 함"을 **호출자 책임으로 전가**하나, 신뢰되지 않은 사전 파일(예: 다운로드된 도메인 사전)에는 이 가정이 깨질 수 있음.
- **수정 방향**: `memmap2`를 패치 버전으로 업그레이드. mmap된 파일을 읽기 전용/불변으로 강제하고, 가능하면 사전 파일에 무결성 해시(헤더 체크섬)를 추가해 변조 탐지.

### [HIGH] H-3 사전 헤더 검증의 정수 오버플로우 (경계 검증 우회)
- **증거**:
  - `rust/crates/mecab-ko-dict/src/lazy_entries_v3.rs:161` — `let expected_index_end = index_offset + u64::from(count) * 8;` (unchecked `+`)
  - `rust/crates/mecab-ko-dict/src/lazy_entries_v3.rs:281` — `let table_pos = self.index_offset + u64::from(index) * 8;` (unchecked)
  - `rust/crates/mecab-ko-dict/src/lazy_entries.rs:184` — `let index_pos = self.index_offset + (u64::from(index) * 8);` (unchecked)
- **공격면**: 악의적 MKE3/사전 파일이 거대한 `index_offset`(u64)을 담으면 `index_offset + count*8`이 wrapping → `expected_index_end > mmap.len()` 검증을 **우회**. 이후 `entry_offset`/`load_entry_from_mmap`의 OOB 읽기로 이어질 수 있음.
  - debug 빌드: overflow panic → DoS.
  - release 빌드(`Cargo.toml:119` `panic = "abort"`, overflow-checks 기본 off): silent wrap → 경계 검증 무력화.
- **완화 요소**: `load_entry_from_mmap`(lazy_entries_v3.rs:327)의 `surface_len + feature_len > remaining` 검증은 `saturating_sub` 사용 — 2차 방어선 존재. 그러나 1차 헤더 검증이 우회되면 잘못된 offset으로 진입 가능.
- **수정 방향**: 모든 헤더/인덱스 산술을 `checked_add`/`checked_mul`로 교체하고 overflow 시 `DictError::Format` 반환. 예: `index_offset.checked_add(u64::from(count).checked_mul(8)?)?`. release 프로파일에 `overflow-checks = true` 추가도 검토(성능 영향 측정 후).

### [LOW] L-3 SIMD unsafe — nightly 전용, fallback 동치 검증됨
- **증거**: `rust/crates/mecab-ko-core/src/viterbi/simd.rs:21` `#![allow(unsafe_code)]`, `matrix/simd.rs:20`. 모듈 주석(simd.rs:5-9)에 따르면 `--features simd`(nightly) 없이는 비활성, scalar fallback과 property 테스트로 동치 검증.
- **영향**: 안정 릴리스 기본 빌드에서 비활성이므로 노출 낮음.
- **수정 방향**: 현 규율 유지. SIMD/scalar 동치 property 테스트를 CI 필수로 유지.

---

## 4. 사용자 사전 (CSV 파싱·검증·주입)

### [LOW] L-4 CSV 수동 `split(',')` — 따옴표/이스케이프 미처리
- **증거**: `rust/crates/mecab-ko-dict/src/user_dict.rs:445` `parse_csv_line`의 `line.split(',').collect()`. workspace는 `csv = "1.3"` 크레이트를 의존성으로 두지만(`Cargo.toml:70`) user_dict 파싱은 수동 split 사용.
- **공격면**: feature/reading 필드에 콤마가 포함되면 필드 경계가 깨져 오파싱. cost/left_id/right_id는 `parse::<i16>/u16`로 검증되어(user_dict.rs:463,478) 주입은 차단되나, surface/feature 오염 가능.
- **완화 요소**: 라인 단위 처리, `#` 주석/빈 줄 skip(user_dict.rs:430), 필드 수·빈 값 검증 존재(user_dict.rs:448,455). 숫자 필드는 엄격 파싱.
- **수정 방향**: 이미 의존 중인 `csv` 크레이트로 파싱 일원화하여 따옴표/이스케이프를 정규 처리.

### [정보] 양호 신호
- `user_dict.rs:401,425` — `load_from_csv`/`load_from_str`는 라이브러리 API로 경로/내용을 호출자 책임으로 받음(정상). 숫자 필드 파싱이 엄격해 명령 주입·정수 주입 차단.
- POS 태그 화이트리스트(`VALID_POS_TAGS`, user_dict.rs:35)와 `is_valid_pos_tag` 검증 존재.
- mecab-ko-dict-validator 크레이트가 사전 검증을 별도 제공(rules.rs/validator.rs).

---

## 5. 확장 (OpenSearch/ES 플러그인 JNI·Python/Node/WASM 바인딩)

### [MEDIUM] M-3 JNI `setDictionaryPath` 경로 검증 부재
- **증거**: `rust/crates/mecab-ko-elasticsearch/src/jni.rs:333` `set_dictionary_path_impl` — Java에서 받은 문자열을 검증·정규화 없이 `*DICTIONARY_PATH.write() = path_str` 로 저장.
- **공격면**: 신뢰되지 않은 호출자가 임의 경로를 설정 → 이후 사전 로딩이 그 경로를 사용하면 경로 트래버설/임의 파일 mmap. ES 플러그인은 보통 신뢰된 클러스터 설정으로 호출되나, 다중 테넌트 환경에서는 권한 경계 우회 소지.
- **완화 요소**: handle 레지스트리 설계는 견고 — raw pointer 미노출, `AtomicI64` 핸들 + `RwLock<HashMap>`(jni.rs:57-65), 모든 JNI 진입점이 `catch_unwind`로 패닉을 Java 예외로 변환(jni.rs:318-330)하여 unwind-over-FFI UB 차단. 이건 모범적.
- **연결 증거**: `docs/integrations/examples/elasticsearch-index-settings.json:19` — `mecab_ko_with_userdict` 토크나이저가 `"user_dict_path": "/usr/share/elasticsearch/config/user-dict.csv"`를 인덱스 설정에서 직접 받음. 사전 경로 주입이 추상적 위험이 아니라 공개 설정 surface로 실재함. ES 인덱스 settings를 제어할 수 있는 주체가 임의 사전 경로를 지정 가능.
- **수정 방향**: 경로를 정규화(`canonicalize`)하고 허용된 사전 루트 디렉토리(allowlist) 하위인지 검증. 설정 가능한 base dir 도입.

### [MEDIUM] M-4 rkyv unsound — InlineVec/SerVec use-after-free (RUSTSEC-2026-0122)
- **증거**: `cargo audit` → `rkyv 0.8.15` unsound. 경로: `mecab-ko-dict-builder → rkyv`. (zero-copy 직렬화에 사용, `Cargo.toml:67`.)
- **공격면**: 빌드 도구 경로. 신뢰되지 않은 rkyv 데이터 역직렬화 시 패닉 안전성 결여로 UAF.
- **수정 방향**: rkyv 패치 버전 추적/업그레이드. dict-builder가 신뢰된 입력만 처리하도록 보장.

### [MEDIUM] M-5 anyhow unsound — `Error::downcast_mut` (RUSTSEC-2026-0190)
- **증거**: `cargo audit` → `anyhow 1.0.102` unsound. 다수 크레이트가 전이 의존.
- **수정 방향**: `anyhow >= 1.0.103`(픽스 버전)으로 업그레이드.

### [LOW] L-5 dict-sync 네트워크 노출
- **증거**: `mecab-ko-dict-sync`가 `reqwest`/`mockito`로 원격 동기화(H-1의 quinn-proto 경로 부모). 네트워크 입력을 받는 유일 컴포넌트.
- **수정 방향**: TLS 검증·타임아웃·응답 크기 상한 확인. H-1 업그레이드와 함께 처리.

### [정보] 양호 신호
- JNI 7개 진입점 모두 `#[allow(unsafe_code)]`이지만 실제 `unsafe` 블록 없이 jni 크레이트 안전 래퍼만 사용, `catch_unwind`로 FFI 경계 패닉 차단(jni.rs). 핸들 기반 설계로 메모리 안전.

---

## 우선순위 로드맵

| 순위 | 항목 | 작업 | 근거 |
|------|------|------|------|
| 1 | H-3 정수 오버플로우 | `lazy_entries_v3.rs:161,281`·`lazy_entries.rs:184`를 `checked_add`/`checked_mul`로 교체 | 우리 코드 내부 결함, 즉시 수정 가능, 신뢰되지 않은 사전 파일 경계 검증의 정확성 직결 |
| 2 | H-1 quinn-proto | `cargo update -p quinn-proto`(>=0.11.15) | 네트워크 노출 High DoS, 단순 업그레이드 |
| 3 | H-2 memmap2 / M-4 rkyv / M-5 anyhow | 패치 버전 업그레이드 + 사전 파일 무결성 해시 검토 | unsound 3종, mmap은 엔진 전반 기반 |
| 4 | M-1 deny.toml `[advisories]` | 권고 처리 레벨 명시 + CI `cargo deny check` 강제 | 위 권고들이 CI를 통과하지 못하도록 게이트화 |
| 5 | M-3 JNI 경로 검증 | `setDictionaryPath` 경로 정규화 + allowlist | 다중 테넌트 경로 트래버설 차단 |
| 6 | M-2 캐시 키 충돌 | `CacheEntry`에 원문 저장 후 hit 동일성 검증 | 조용한 오분석 방지 |
| 7 | L-1·L-4·L-5 등 | unmaintained 대체, csv 크레이트로 파싱 일원화, dict-sync 하드닝 | 장기 위생 |

## 후속 조치 (2026-07-02, PROGRESS.md Sprint 175)

| 순위 | 항목 | 상태 |
|------|------|------|
| 1 | H-3 정수 오버플로우 | ✅ 완료 — `checked_add`/`checked_mul`로 교체, `+8` 오버플로우도 방어 |
| 2 | H-1 quinn-proto | ⏸ 보류 — mockito 개발 의존성 경로, reqwest http3 미사용(도달 불가), upstream 미출시. `deny.toml` ignore 등록 |
| 3 | H-2 memmap2 / M-4 rkyv / M-5 anyhow | ⏸ 보류 — 패치 버전 upstream 미출시. `deny.toml` ignore 등록(근거 주석 포함) |
| 4 | M-1 deny.toml `[advisories]` | ✅ 완료 — vulnerability/yanked deny 기본 유지, unmaintained/unsound scope=workspace |
| (신규) quick-xml RUSTSEC-2026-0194/0195 | ✅ 완료 | 직접 의존성 0.36→0.41 상향 (본 리뷰 검토일 이후 발견된 advisory) |
| 5 | M-3 JNI 경로 검증 | 미착수 |
| 6 | M-2 캐시 키 충돌 | 미착수 |
| 7 | L-1·L-4·L-5 등 | 미착수 |

## 검증 메모
- `cargo build --workspace`: 성공 (12.61s).
- `cargo clippy --workspace`: 경고 0.
- `cargo audit`: 1 vulnerability(H-1) + 5 warnings(memmap2/rkyv/anyhow unsound, paste/proc-macro-error2 unmaintained).
- 비목표 준수: 소스 미수정, 신규 PoC/fuzz 미작성, 외부 공격 미수행.
