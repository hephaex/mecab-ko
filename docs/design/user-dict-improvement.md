# mecab-ko 사용자 사전 개선 설계

> 현재 CSV 기반 사용자 사전의 Hot Reload v2 안정화, Domain Overlay API 확장, 포맷 표준화를 통해 운영 품질을 높이기 위한 단계적 개선 계획

---

## 1. 현재 상태

### 1.1 사용자 사전 포맷 (CSV)

현재 `user_dict.rs`는 두 가지 CSV 포맷을 지원합니다.

**기본 포맷** (`표면형,품사,비용,읽기`):
```csv
# 주석 라인
형태소분석,NNG,-1000,형태소분석
딥러닝,NNG,-500,
챗GPT,NNP,-1000,챗지피티
```

**확장 포맷** (`표면형,품사,비용,읽기,left_id,right_id`):
```csv
카카오톡,NNP,-1000,카카오톡,1234,5678
```

e2e 테스트 픽스처(`legacy/tests/e2e/fixtures/user_dict.csv`)는 MeCab 원본 7-필드 포맷을 사용하고 있어 현재 파서와 불일치가 있습니다.

```csv
# Format: surface,left_id,right_id,cost,pos,...
카카오톡,0,0,0,NNP,*,*,*
```

이 불일치는 레거시 파이프라인과의 호환성 문제를 야기합니다.

### 1.2 로딩 방식

| 경로 | 방식 | 특징 |
|------|------|------|
| `UserDictionary::load_from_csv()` | 런타임 로드 | 파일 전체 파싱, Trie 캐시 지연 빌드 |
| `UserDictionary::load_from_str()` | 런타임 로드 | 인메모리 파싱 |
| `UserDictionaryBuilder::load_csv()` | 런타임 로드 | 빌더 패턴 체인 |
| `HotReloadDictionary` | 런타임 교체 | `RwLock<VersionedDictionary>` + `FileWatcher` |
| `HotReloadDictV2` | 런타임 교체 | `ArcSwap<DictionarySnapshot>` + `DomainStack` |

빌드 타임 사전 컴파일(`dict-build` CI)은 시스템 사전(`sys.dic`, `entries.bin`)에만 적용됩니다. 사용자 사전은 항상 런타임에 로드됩니다.

### 1.3 제한 사항

1. **Trie 미통합**: `UserDictionary`의 `common_prefix_search()`는 Trie를 사용하지 않고 전체 엔트리를 선형 스캔합니다(`O(n)`). `build_trie()`를 호출해도 `common_prefix_search()`는 여전히 선형 탐색을 합니다.

2. **컨텍스트 ID 고정**: `UserEntry::new()`는 `left_id: 0, right_id: 0`으로 고정합니다. 시스템 사전의 문맥 연접 비용 행렬(`matrix.bin`)과 연결되지 않아 연접 비용이 0으로 취급됩니다.

3. **포맷 불일치**: `user_dict.rs`의 기본 포맷(`표면형,품사,비용,읽기`)과 MeCab 원본 포맷(`표면형,left_id,right_id,비용,품사,...`) 사이에 필드 순서가 다릅니다.

4. **Hot Reload v1과 v2의 미연결**: `FileWatcher`는 `HotReloadDictionary`(v1)에만 연결되어 있습니다. `HotReloadDictV2`는 `ArcSwap` 기반의 우수한 성능을 제공하지만 파일 감시와 연결되지 않았습니다.

5. **DomainStack 검색 성능**: `DomainStack::common_prefix_search()`는 각 도메인의 `UserDictionary::common_prefix_search()`를 순차 호출합니다. 도메인 수가 많아지면 O(도메인 수 × 엔트리 수)입니다.

---

## 2. Hot Reload v2

### 2.1 현재 Hot Reload 구현 상태 분석

**v1 (`hot_reload.rs`)**:
```
HotReloadDictionary {
    state: RwLock<VersionedDictionary>
    version_history: VecDeque<VersionedDictionary>
    file_watcher: Option<FileWatcher>
}
```
- `RwLock` 기반: 읽기도 lock을 잡아야 하므로 고빈도 읽기에서 경합 발생
- `FileWatcher`와 직접 연결됨
- `DeltaUpdate`로 증분 업데이트 지원
- 롤백: 버전 히스토리 최대 10개 보관

**v2 (`hot_reload_v2.rs`)**:
```
HotReloadDictV2 {
    current: ArcSwap<DictionarySnapshot>
    write_state: Mutex<WriteState>
}
```
- `ArcSwap` 기반: 읽기는 원자적 포인터 로드만으로 완료 (wait-free)
- 쓰기는 `Mutex<WriteState>`로 직렬화
- `DomainStack`을 기본 단위로 사용
- `FileWatcher`와 미연결 — 파일 감시 → 자동 재로드 루프 없음

### 2.2 v2 안정화 목표

목표: 파일 감시 → 사전 재빌드 → `ArcSwap` atomic swap 의 완전한 파이프라인 구성

**파이프라인 흐름**:
```
파일 변경 감지 (notify)
    │
    ▼  (debounce 300ms)
CSV 파싱 + UserDictionary 빌드
    │
    ▼  (백그라운드 스레드)
DomainStack 교체 준비
    │
    ▼
HotReloadDictV2::update() → ArcSwap 교체
    │
    ▼
기존 Arc<DictionarySnapshot> 자동 해제
```

**구현 방안**:

```rust
pub struct WatchedHotReload {
    inner: Arc<HotReloadDictV2>,
    _watcher: RecommendedWatcher,
}

impl WatchedHotReload {
    pub fn new(
        initial: DomainStack,
        watch_paths: &[(DomainId, PathBuf)],
    ) -> Result<Self> {
        let inner = Arc::new(HotReloadDictV2::new(initial));
        let inner_clone = Arc::clone(&inner);

        let (tx, rx) = crossbeam_channel::bounded(16);
        let mut watcher = notify::recommended_watcher(move |ev| {
            let _ = tx.try_send(ev);
        })?;

        for (_, path) in watch_paths {
            watcher.watch(path, RecursiveMode::NonRecursive)?;
        }

        // 백그라운드 reload 스레드
        std::thread::spawn(move || {
            Self::reload_loop(rx, Arc::clone(&inner_clone), /* domain_paths */);
        });

        Ok(Self { inner, _watcher: watcher })
    }
}
```

### 2.3 스레드 안전성: RwLock vs ArcSwap

| 특성 | v1 (RwLock) | v2 (ArcSwap) |
|------|-------------|--------------|
| 읽기 비용 | lock 획득 필요 | 원자적 포인터 로드 |
| 쓰기 비용 | write lock (독점) | Mutex + store |
| 읽기 경합 | 있음 | 없음 (wait-free) |
| 롤백 | VecDeque 히스토리 | VecDeque 히스토리 |
| 추천 용도 | 단순 구성 | 고빈도 읽기 환경 |

`ArcSwap::load()`는 `Guard<Arc<T>>`를 반환하며 Guard 유효 기간 동안 이전 스냅샷이 메모리에서 해제되지 않습니다. 짧은 스코프 내에서 Guard를 사용하고 즉시 drop하는 것이 메모리 효율에 유리합니다.

### 2.4 성능 영향 최소화

- CSV 파싱과 `UserDictionary` 빌드는 백그라운드 스레드에서 수행합니다. ArcSwap 교체는 파싱 완료 후 단 한 번의 원자 연산입니다.
- `WatchConfig::debounce_ms`(기본 300ms)로 연속 파일 변경 시 중복 재로드를 억제합니다.
- Trie 재빌드(`UserDictionary::build_trie()`)가 가장 비용이 큰 작업입니다. 엔트리 수가 10,000을 초과하면 백그라운드 스레드 우선순위를 낮추거나 증분 Trie 업데이트를 검토합니다.

---

## 3. Domain Overlay API

### 3.1 개념

기본 시스템 사전 위에 도메인별 사용자 사전을 레이어로 겹칩니다. 이미 `domain.rs`에 `DomainStack`이 구현되어 있으나 Tokenizer에 직접 노출되는 편의 API가 없습니다.

```
검색 우선순위:
  overlay (priority 0, 예: medical)
      │
  overlay (priority 1, 예: finance)
      │
  user_dict (기본 사용자 사전)
      │
  system_dict (시스템 사전 Trie)
```

### 3.2 현재 DomainStack API

`domain.rs`의 현재 공개 API:

```rust
// 도메인 추가/교체
stack.add_domain(DomainId("medical".into()), 0, Arc::new(dict), None);

// 도메인 제거
stack.remove_domain(&DomainId("medical".into()));

// 우선순위 순 목록
stack.list_domains(); // Vec<(DomainId, u8, usize)>

// 검색 (우선순위 순)
stack.lookup("표면형");           // 정확 매칭
stack.common_prefix_search("텍스트"); // 접두사 매칭
```

### 3.3 Tokenizer 레벨 편의 API 스케치

Tokenizer에서 도메인을 직접 조작하는 API를 추가합니다:

```rust
impl Tokenizer {
    /// 도메인 오버레이 추가 (priority: 낮을수록 높은 우선순위)
    pub fn add_overlay(
        &self,
        name: &str,
        dict: UserDictionary,
        priority: u8,
    ) -> Result<()>;

    /// CSV 파일로 도메인 오버레이 추가
    pub fn add_overlay_from_csv(
        &self,
        name: &str,
        path: &Path,
        priority: u8,
    ) -> Result<()>;

    /// 도메인 오버레이 제거
    pub fn remove_overlay(&self, name: &str) -> Result<()>;

    /// 등록된 도메인 목록 반환
    pub fn list_overlays(&self) -> Vec<(String, u8, usize)>;
}
```

사용 예시:
```rust
let tokenizer = Tokenizer::new(dict)?;

// 의료 도메인 사전 추가
tokenizer.add_overlay_from_csv("medical", "dicts/medical.csv", 0)?;

// 금융 도메인 사전 추가
tokenizer.add_overlay_from_csv("finance", "dicts/finance.csv", 1)?;

// 분석
let tokens = tokenizer.analyze("코스피가 상승했다")?;

// 의료 도메인만 제거
tokenizer.remove_overlay("medical")?;
```

### 3.4 우선순위 체계

```
priority 0  (가장 높음): 도메인 오버레이 (용도 특화 사전)
priority 1..127        : 추가 도메인 오버레이
priority 128           : 기본 사용자 사전 (user_dict)
priority 255           : 시스템 사전 (system_dict, 읽기 전용)
```

같은 priority를 가진 도메인은 삽입 순서가 우선합니다(`sort_by_key`의 stable sort 보장).

### 3.5 구현 방안: 별도 Trie vs 통합 검색

**방안 A: 각 도메인이 독립 Trie를 보유** (현재 구조 확장)

장점: 도메인별 독립 업데이트, 메모리 격리
단점: 도메인이 많을수록 검색 비용 O(도메인 수) 증가

```rust
// DomainStack::common_prefix_search()가 각 도메인 Trie를 순차 검색
domains.iter().flat_map(|d| d.dictionary.common_prefix_search(text))
```

**방안 B: 통합 Trie (도메인 ID를 값으로 인코딩)**

장점: O(1) 도메인 수에 무관한 검색 비용
단점: 도메인 추가/제거 시 Trie 재빌드 필요

권장: **방안 A**를 Phase 3 MVP로 먼저 구현합니다. 도메인이 10개 미만인 실운영 환경에서는 성능 차이가 미미합니다. 도메인 수가 수십 개를 초과하는 요구사항이 생기면 방안 B로 전환합니다.

---

## 4. 사용자 사전 포맷 표준화

### 4.1 현재 포맷 비교

| 포맷 | 필드 순서 | 사용처 |
|------|-----------|--------|
| mecab-ko 기본 | `표면형,품사,비용,읽기` | `user_dict.rs` `load_from_csv()` |
| mecab-ko 확장 | `표면형,품사,비용,읽기,left_id,right_id` | `user_dict.rs` `parse_csv_line()` |
| MeCab 원본 | `표면형,left_id,right_id,비용,품사,...` | `e2e/fixtures/user_dict.csv` |

e2e 픽스처의 MeCab 원본 포맷은 `parse_csv_line()`에서 2번째 필드를 품사로 읽으므로 `left_id`(0)가 품사로 해석됩니다. 이는 묵시적 버그입니다.

### 4.2 간소화 제안: 품사 자동 매핑

사용자가 컨텍스트 ID를 직접 지정하는 경우는 매우 드뭅니다. ID 자동 매핑을 기본으로 합니다.

**표준 포맷 v2**:
```
표면형,품사[,비용[,읽기]]
```

- 표면형: 필수
- 품사: 필수 (세종 태그 또는 자동 추정 `_auto`)
- 비용: 선택 (기본값: -1000)
- 읽기: 선택

품사 `_auto` 지정 시 `estimate_pos(surface)`를 사용합니다.

```csv
# 표준 포맷 v2 예시
딥러닝,NNG
형태소분석,NNG,-1500
챗GPT,NNP,-1000,챗지피티
메타버스,_auto
```

left_id, right_id는 품사 태그로부터 시스템 사전의 문맥 테이블을 참조하여 자동 매핑합니다. (Phase 2에서 구현)

### 4.3 TOML/YAML 포맷 대안 검토

| 포맷 | 장점 | 단점 |
|------|------|------|
| CSV | 단순, MeCab 호환, 라인 기반 편집 쉬움 | 필드 순서 혼동, 메타데이터 표현 어려움 |
| TOML | 구조화, 주석 지원, 타입 명시 | 파싱 의존성 추가, 대용량 비효율 |
| YAML | 읽기 쉬움, 중첩 구조 | 파싱 비용, 들여쓰기 오류 취약 |

결론: CSV를 유지합니다. 도메인 메타데이터(도메인 이름, 우선순위, 설명)는 별도 TOML 매니페스트 파일로 분리합니다.

```toml
# domain.toml
[domains.medical]
priority = 0
dict_path = "dicts/medical.csv"
description = "의료 전문 용어"

[domains.finance]
priority = 1
dict_path = "dicts/finance.csv"
description = "금융 용어"
```

### 4.4 하위 호환성

기존 `표면형,품사,비용,읽기` 포맷은 v2에서도 완전히 동작합니다. 필드 수 기반으로 포맷을 자동 판별합니다.

```rust
fn detect_format(parts: &[&str]) -> CsvFormat {
    match parts.len() {
        2..=4 => CsvFormat::Standard,        // 표면형,품사[,비용[,읽기]]
        5..=6 => CsvFormat::Extended,        // 표면형,품사,비용,읽기,left_id,right_id
        7..=8 => CsvFormat::MeCabOriginal,   // MeCab 7-필드 (e2e 픽스처)
        _ => CsvFormat::Unknown,
    }
}
```

MeCab 원본 포맷 자동 감지는 Phase 1에서 구현하여 e2e 픽스처 호환성을 확보합니다.

---

## 5. 구현 우선순위

### Phase 1: 포맷 간소화 + 검증 강화 (Sprint 103-104)

목표: 기존 코드 품질 향상, 버그 수정

- [ ] `parse_csv_line()` 에 MeCab 원본 포맷 자동 감지 추가
- [ ] `_auto` 품사 키워드 지원 (`estimate_pos()` 위임)
- [ ] `ValidationResult`에 포맷 오류 상세 메시지 개선
- [ ] `common_prefix_search()`를 Trie 기반으로 전환 (빌드된 경우에만)
- [ ] e2e 픽스처 포맷 이슈 해결

예상 작업량: 3-4 days

### Phase 2: Hot Reload v2 안정화 (Sprint 105-106)

목표: `HotReloadDictV2` + `FileWatcher` 통합

- [ ] `WatchedHotReload` 구조체 구현 (`HotReloadDictV2` + `FileWatcher` 결합)
- [ ] 백그라운드 reload 스레드 + panic 처리
- [ ] `domain.toml` 매니페스트 파서 (`toml` crate 의존성)
- [ ] reload 성공/실패 콜백 API
- [ ] 통합 테스트: 파일 변경 → 자동 재로드 검증

예상 작업량: 5-7 days

### Phase 3: Domain Overlay MVP (Sprint 107-108)

목표: Tokenizer 레벨 편의 API, 운영 사용 가능

- [ ] `Tokenizer::add_overlay()` / `remove_overlay()` API
- [ ] `Tokenizer::add_overlay_from_csv()` 구현
- [ ] `WatchedHotReload`와 도메인 생명주기 통합
- [ ] 도메인 우선순위 충돌 경고 (`ValidationResult` 확장)
- [ ] 벤치마크: 도메인 0개 vs 5개 vs 10개 검색 성능 비교

예상 작업량: 5-7 days

---

## 6. 리스크

### 6.1 컨텍스트 ID 자동 매핑의 정확도

**리스크**: `left_id: 0, right_id: 0` 고정은 연접 비용 행렬에서 0번 문맥 ID를 사용합니다. 0번이 해당 품사에 적합한 연접 패턴인지는 시스템 사전의 문맥 테이블 구성에 의존합니다. 잘못된 연접 비용은 분석 품질 저하로 이어집니다.

**완화**: Phase 2에서 품사 태그 → 대표 문맥 ID 매핑 테이블을 시스템 사전에서 추출하여 `pos_to_context_id.bin`으로 번들합니다.

### 6.2 Hot Reload 중 분석 요청 처리

**리스크**: CSV 파싱 + Trie 빌드 중에 분석 요청이 들어오면 이전 스냅샷을 계속 사용합니다. `ArcSwap::load()`의 Guard가 오래 유지되면 이전 스냅샷이 메모리에서 해제되지 않아 메모리 사용량이 일시적으로 2배가 됩니다.

**완화**: Guard 수명을 분석 함수 호출 스코프 내로 제한합니다. 장시간 분석 작업(배치 처리 등)은 Guard를 미리 해제하거나, 스냅샷 Arc를 명시적으로 클론하여 사용합니다.

### 6.3 DomainStack 불변성 보장

**리스크**: `HotReloadDictV2::update()` 클로저 내에서 이전 `DomainStack`을 참조하여 새 `DomainStack`을 생성합니다. 클로저가 패닉하면 `write_state` Mutex가 poisoned 상태가 됩니다. 이후 모든 쓰기 연산이 `expect("mutex poisoned")`으로 panic합니다.

**완화**: 클로저 내에서 panic이 발생하지 않도록 사전 검증을 수행합니다. CSV 파싱 오류는 클로저 외부에서 처리하고, 클로저는 이미 검증된 `UserDictionary`만 받습니다.

### 6.4 대용량 사전 재빌드 지연

**리스크**: 엔트리가 100,000개 이상인 도메인 사전의 Trie 재빌드는 100ms 이상 소요될 수 있습니다. 파일 변경이 짧은 시간 내에 반복되면 재빌드 큐가 누적됩니다.

**완화**: `crossbeam_channel::bounded(16)`으로 이벤트 큐를 제한합니다. reload 스레드가 바쁜 동안 도착한 이벤트는 드롭됩니다. reload 완료 후 마지막 파일 상태로 한 번 더 로드하는 "trailing reload" 패턴을 적용합니다.

### 6.5 포맷 자동 감지 오탐

**리스크**: 4-필드 CSV에서 3번째 필드가 숫자이면 Standard 포맷으로, MeCab 원본 7-필드와 구분이 어렵습니다. 필드 수가 4인 경우 2번째 필드가 유효한 품사 태그인지 확인하여 포맷을 결정합니다.

**완화**: 포맷 감지 로직에 `is_valid_pos_tag()` 검사를 포함합니다. 2번째 필드가 품사 태그가 아닌 경우(`"0"` 같은 숫자) MeCab 원본 포맷으로 판단합니다.

---

## 부록: 관련 파일 맵

| 파일 | 역할 |
|------|------|
| `crates/mecab-ko-dict/src/user_dict.rs` | `UserDictionary`, `UserEntry`, CSV 파서 |
| `crates/mecab-ko-dict/src/domain.rs` | `DomainStack`, `DomainDictionary`, `DomainId` |
| `crates/mecab-ko-dict/src/hot_reload.rs` | `HotReloadDictionary` v1 (RwLock 기반) |
| `crates/mecab-ko-dict/src/hot_reload_v2.rs` | `HotReloadDictV2` (ArcSwap 기반) |
| `crates/mecab-ko-dict/src/file_watcher.rs` | `FileWatcher`, `WatchConfig` (notify 래핑) |
| `legacy/tests/e2e/fixtures/user_dict.csv` | MeCab 원본 7-필드 포맷 (호환성 대상) |
| `docs/design/v2-dict-code-analysis.md` | v2 바이너리 사전 포맷 상세 분석 |

---

*작성: 2026-05-08*
