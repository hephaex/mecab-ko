# v0.7.0 마이그레이션 가이드

이 문서는 mecab-ko v0.6.x에서 v0.7.0으로 업그레이드할 때 필요한 변경 사항을 설명합니다.

## 주요 변경 사항

### 1. 사전 로딩 기본값 변경

v0.7.0부터 `LoadOptions::default()`는 **LazyEntries 모드**를 기본으로 사용합니다.

| 버전 | 기본 로딩 모드 | 메모리 사용량 |
|------|----------------|---------------|
| v0.6.x | Eager (전체 로드) | ~150MB |
| v0.7.0 | Lazy (지연 로드) | ~130MB |

**기존 동작 유지 방법:**

```rust
use mecab_ko_dict::dictionary::{LoadOptions, SystemDictionary};

// v0.7.0에서 Eager 모드 사용 (v0.6.x 동작과 동일)
let dict = SystemDictionary::load_with_options(
    "/path/to/dict",
    LoadOptions::speed_optimized(),
)?;

// 또는
let dict = SystemDictionary::load_with_options(
    "/path/to/dict",
    LoadOptions::eager(),
)?;
```

### 2. SystemDictionary API 변경

#### `entries()` 메서드 제거

v0.6.x:
```rust
let dict = SystemDictionary::load_default()?;
let entries = dict.entries(); // Vec<DictEntry> 참조
println!("Entry count: {}", entries.len());
```

v0.7.0:
```rust
let dict = SystemDictionary::load_default()?;
// entries() 대신 entry_count() 사용
println!("Entry count: {}", dict.entry_count());

// 개별 엔트리 접근은 get_entry() 사용
if let Ok(entry) = dict.get_entry(0) {
    println!("First entry: {}", entry.surface);
}
```

#### `get_entry()` 반환 타입 변경

v0.6.x:
```rust
// Option<&DictEntry> 반환
if let Some(entry) = dict.get_entry(index) {
    println!("{}", entry.surface);
}
```

v0.7.0:
```rust
// Result<Arc<DictEntry>> 반환
match dict.get_entry(index) {
    Ok(entry) => println!("{}", entry.surface),
    Err(e) => eprintln!("Error: {}", e),
}
```

**변경 이유:**
- Lazy 모드에서는 디스크 I/O가 발생할 수 있어 에러 처리 필요
- `Arc<DictEntry>`를 반환하여 스레드 간 공유 용이

#### `common_prefix_search()` 반환 타입 변경

v0.6.x:
```rust
let results: Vec<(&DictEntry, usize)> = dict.common_prefix_search("한국어");
for (entry, len) in results {
    println!("{}: {}", entry.surface, len);
}
```

v0.7.0:
```rust
let results: Result<Vec<(Arc<DictEntry>, usize)>> = dict.common_prefix_search("한국어");
for (entry, len) in results? {
    println!("{}: {}", entry.surface, len);
}
```

### 3. LoadOptions 구조

```rust
pub struct LoadOptions {
    /// Matrix에 mmap 사용 (멀티프로세스 메모리 공유)
    pub use_mmap_matrix: bool,

    /// entries에 lazy loading 사용 (메모리 절약)
    pub use_lazy_entries: bool,

    /// lazy entries 캐시 크기 (기본: 10000)
    pub lazy_cache_size: Option<usize>,
}
```

#### 사전 정의된 옵션

| 메서드 | use_mmap_matrix | use_lazy_entries | 용도 |
|--------|-----------------|------------------|------|
| `default()` | false | **true** | 메모리 절약 (권장) |
| `speed_optimized()` | false | false | 최대 속도 |
| `memory_optimized()` | true | true | 최소 메모리 |
| `eager()` | false | false | v0.6.x 호환 |

### 4. 마이그레이션 체크리스트

- [ ] `dict.entries()` → `dict.entry_count()` 변경
- [ ] `dict.entries()[i]` → `dict.get_entry(i)?` 변경
- [ ] `get_entry()` 결과에 `?` 또는 `match` 추가
- [ ] `common_prefix_search()` 결과에 `?` 또는 `unwrap_or_default()` 추가
- [ ] Eager 모드가 필요하면 `LoadOptions::speed_optimized()` 명시
- [ ] entries.bin이 v2 포맷(MKE2)인지 확인 (v1(MKED)도 호환됨)

### 5. 성능 특성

#### Eager 모드 (LoadOptions::speed_optimized())
- **초기화**: 느림 (모든 엔트리 로드)
- **조회**: 빠름 (메모리에서 즉시 접근)
- **메모리**: 높음 (~150MB)
- **용도**: 실시간 서비스, 대량 처리

#### Lazy 모드 (LoadOptions::default())
- **초기화**: 빠름 (인덱스만 로드)
- **첫 조회**: 느림 (디스크에서 로드)
- **이후 조회**: 빠름 (LRU 캐시)
- **메모리**: 낮음 (~130MB)
- **용도**: 배치 처리, 메모리 제한 환경

### 6. 코드 예제

#### 기본 사용 (v0.7.0 방식)

```rust
use mecab_ko::Tokenizer;

// 기본값은 Lazy 모드
let mut tokenizer = Tokenizer::new()?;
let tokens = tokenizer.tokenize("한국어 형태소 분석");

for token in tokens {
    println!("{}: {}", token.text(), token.pos());
}
```

#### 고성능 서버 (Eager 모드)

```rust
use mecab_ko_dict::dictionary::{LoadOptions, SystemDictionary};
use mecab_ko_core::tokenizer::Tokenizer;

// 서버 시작 시 전체 로드
let dict = SystemDictionary::load_with_options(
    "/path/to/dict",
    LoadOptions::speed_optimized(),
)?;

// 요청 처리 시 빠른 응답
let mut tokenizer = Tokenizer::from_dictionary(dict);
```

#### 메모리 제한 환경

```rust
use mecab_ko_dict::dictionary::{LoadOptions, SystemDictionary};

// mmap + lazy 로드로 메모리 최소화
let dict = SystemDictionary::load_with_options(
    "/path/to/dict",
    LoadOptions::memory_optimized(),
)?;
```

### 7. entries.bin v2 변환

Lazy 모드의 메모리 절감 효과를 얻으려면 entries.bin이 v2 (MKE2) 포맷이어야 합니다.

#### 포맷 확인

```bash
mecab-ko-dict-builder info --dict /path/to/dict
```

출력 예시:
```
entries.bin format: v1 (MKED) - Eager loading only
  → Run 'convert' command for memory optimization
```

#### v2 변환

```bash
# 변환 (기존 파일 백업됨)
mecab-ko-dict-builder convert --dict /path/to/dict --verbose

# 검증
mecab-ko-dict-builder info --dict /path/to/dict
```

#### 변환 효과

| 지표 | v1 (Eager) | v2 (Lazy) | 개선율 |
|------|------------|-----------|--------|
| 메모리 | 291 MB | 66 MB | **-77%** |
| 로드 시간 | 806 ms | 41 ms | **-95%** |
| 조회 속도 | 3.7 ms/10K | 22 ms/10K | -6x |

> **참고**: v1 포맷에서는 Lazy 모드가 자동으로 EagerStore로 폴백됩니다.

### 8. 문제 해결

#### "entries.bin: invalid magic number" 에러

이 에러는 entries.bin 파일이 인식되지 않는 형식일 때 발생합니다.

**해결 방법:**
1. v0.7.0은 v1 (MKED) 및 v2 (MKE2) 포맷 모두 지원합니다.
2. 사전을 다시 빌드하거나 최신 mecab-ko-dic을 설치하세요.

#### Lazy 모드에서 성능 저하

LRU 캐시가 작으면 디스크 I/O가 자주 발생합니다.

**해결 방법:**
```rust
let opts = LoadOptions {
    use_lazy_entries: true,
    lazy_cache_size: Some(50000), // 캐시 크기 증가
    ..Default::default()
};
```

---

## 버전 히스토리

| 버전 | 날짜 | 주요 변경 |
|------|------|-----------|
| v0.7.0 | 2026-03-21 | LazyEntries 기본 활성화, EntryStore 추상화 |
| v0.6.0 | 2026-03-19 | Eager 로딩 기본, LazyEntries 옵션 |

---

문의사항은 GitHub Issues에 등록해 주세요: https://github.com/user/mecab-ko/issues
