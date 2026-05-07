# mecab-ko v2 바이너리 사전 포맷 코드 분석

> **문서 버전**: 1.0
> **작성일**: 2026-05-07
> **분석 대상**: `crates/mecab-ko-dict` (Rust 구현체)
> **선행 문서**: `docs/dictionary-format-v2.md` (MeCab 원본 포맷 분석)

이 문서는 mecab-ko Rust 구현체의 v2 바이너리 사전 포맷을 소스 코드 수준에서 분석한 기술 설계 문서입니다. 각 바이너리 파일의 정확한 바이트 레이아웃, 구현상의 특성, 한계, 그리고 v3 개선 기회를 정리합니다.

---

## 1. 개요 (v2 포맷 요약)

v2 사전은 세 개의 핵심 바이너리 파일로 구성됩니다.

| 파일 | 역할 | 구현 모듈 |
|------|------|-----------|
| `entries.bin` (v2 = `MKE2`) | 사전 엔트리 저장소 (lazy loading 지원) | `lazy_entries.rs` |
| `sys.dic` | Double-Array Trie (형태소 검색) | `trie.rs` (yada 크레이트 래핑) |
| `matrix.bin` | 연접 비용 행렬 | `matrix/mod.rs` |

모든 수치 필드는 **Little-Endian**으로 저장됩니다. 압축은 선택적 Cargo feature (`zstd`)로 제어되며, 기본 활성화 상태입니다.

사전 로딩 경로는 두 가지입니다.

- `SystemDictionary::load()` — entries.bin v1/v2, matrix.bin, sys.dic 통합 로딩
- `MmapDictionary::load()` — loader.rs 경유, 고유 entries.bin 형식(매직 없음) 또는 CSV 폴백

---

## 2. entries.bin 분석

### 2.1 v1 포맷 (MKED, 레거시)

v1은 순차적 선형 구조입니다. 임의 접근이 불가능하며, 전체 로드 후 사용합니다.

```
[Header: 12 bytes]
  magic       : [u8; 4] = b"MKED"
  version     : u32 (LE) = 1
  count       : u32 (LE)

[Entry Data: 가변 길이, count 개]
  left_id     : u16 (LE)
  right_id    : u16 (LE)
  cost        : i16 (LE)
  surface_len : u16 (LE)
  feature_len : u16 (LE)
  surface     : [u8; surface_len]  (UTF-8)
  feature     : [u8; feature_len]  (UTF-8)
```

`dictionary.rs`의 `load_entries_bin()` 및 `save_entries_bin()`가 이 형식을 생성/읽습니다.

### 2.2 v2 포맷 (MKE2, 현행)

v2는 파일 끝에 오프셋 인덱스 테이블을 추가하여 임의 접근(O(1))을 가능하게 합니다.

```
[Header: 20 bytes]
  magic        : [u8; 4] = b"MKE2"
  version      : u32 (LE) = 2
  count        : u32 (LE)
  index_offset : u64 (LE)   <- 인덱스 테이블 시작 위치

[Entry Data: 가변 길이, count 개, 순차 배치]
  left_id      : u16 (LE)
  right_id     : u16 (LE)
  cost         : i16 (LE)
  surface_len  : u16 (LE)
  feature_len  : u16 (LE)
  surface      : [u8; surface_len]  (UTF-8)
  feature      : [u8; feature_len]  (UTF-8)

[Index Table: count * 8 bytes, 파일 끝]
  offset_0     : u64 (LE)   <- entry_0의 파일 내 절대 오프셋
  offset_1     : u64 (LE)
  ...
  offset_{count-1} : u64 (LE)
```

각 엔트리 레코드의 고정 헤더 크기: `2 + 2 + 2 + 2 + 2 = 10 bytes`.
가변 부분: `surface_len + feature_len` bytes.

인덱스 조회 공식:
```
index_table_pos = index_offset + (entry_index * 8)
entry_pos = mmap[index_table_pos .. index_table_pos + 8] as u64 (LE)
```

### 2.3 Lazy Loading 메커니즘

`LazyEntries` 구조체(`lazy_entries.rs`)가 v2 포맷의 핵심입니다.

```
LazyEntries {
    path: PathBuf           // 디버깅용
    mmap: Mmap              // memmap2 기반 읽기 전용 메모리 맵
    count: u32              // 엔트리 수
    index_offset: u64       // 인덱스 테이블 위치
    cache: RwLock<LruCache> // LRU 캐시 (기본 10,000 항목)
}
```

조회 흐름:

1. `get(index)` 호출
2. `RwLock<LruCache>` write lock 획득 후 캐시 확인
3. 캐시 히트: `Arc<DictEntry>` 즉시 반환
4. 캐시 미스: `get_entry_offset(index)` → 인덱스 테이블에서 오프셋 읽기 → `load_entry_from_disk(index)` → `mmap` 슬라이스에서 역직렬화 → 캐시 삽입

mmap은 파일 오픈 시 즉시 매핑되므로 OS 페이지 캐시를 통해 실제 I/O가 지연됩니다.

### 2.4 LRU 캐시 구현

`LruCache`는 `HashMap<u32, Arc<DictEntry>>` + `Vec<u32>` 접근 순서 벡터로 구현된 단순 LRU입니다.

```
LruCache {
    entries     : HashMap<u32, Arc<DictEntry>>
    max_size    : usize                         // 기본 10,000
    access_order: Vec<u32>                      // 인덱스 순서 (최근 = 뒤)
}
```

eviction 시 `access_order.remove(0)`으로 가장 오래된 항목 제거합니다. `Vec::remove(0)`은 O(n) 연산이므로 핫스팟 시나리오에서 성능 병목이 될 수 있습니다.

주의: `get()`에서 캐시 확인과 캐시 삽입이 모두 write lock을 요구합니다. 읽기 경합 시 병목이 발생할 수 있습니다.

### 2.5 EntryStore 추상화

`EntryStore` trait(`entry_store.rs`)이 Eager/Lazy 모드를 추상화합니다.

```
EagerStore: Vec<Arc<DictEntry>>    <- 전체 메모리 로드
LazyStore:  LazyEntries            <- 필요 시 디스크 읽기
```

`SystemDictionary`는 `Arc<dyn EntryStore>`를 보유하며, `LoadOptions`로 런타임에 선택합니다.

```
LoadOptions::default()           -> use_lazy_entries: true  (LazyStore)
LoadOptions::speed_optimized()   -> use_lazy_entries: false (EagerStore)
LoadOptions::memory_optimized()  -> use_mmap_matrix: true + use_lazy_entries: true
```

### 2.6 v1 → v2 마이그레이션

`lazy_entries.rs`의 `migrate_entries_v1_to_v2()` 함수가 v1(MKED) 파일을 읽어 v2(MKE2)로 저장합니다. 전체 엔트리를 메모리에 로드한 뒤 `save_entries()` 호출 방식입니다.

---

## 3. sys.dic 분석 (Trie)

### 3.1 구조

`sys.dic`은 yada 크레이트의 `DoubleArray` 직렬화 형식을 그대로 사용합니다. mecab-ko-dict는 이를 래핑하는 `Trie` 구조체만 제공하며, 파일 내부 바이트 포맷은 yada 크레이트가 정의합니다.

```
Trie<'a> {
    da: DoubleArray<Cow<'a, [u8]>>
}
```

`Cow<'a, [u8]>` 덕분에 두 가지 소유권 모드를 지원합니다.
- `Cow::Borrowed` — 외부 슬라이스를 빌려 사용 (mmap 슬라이스 등)
- `Cow::Owned` — Vec을 직접 소유 (파일에서 읽은 후)

현재 `Trie::from_file()`은 `std::fs::read()`로 전체 파일을 `Vec<u8>`에 올린 뒤 `Cow::Owned`로 사용합니다. mmap 기반 `Cow::Borrowed` 경로는 구현되어 있지 않습니다.

### 3.2 Double-Array Trie 알고리즘

yada 크레이트는 **LOUDS-based Double-Array Trie**를 구현합니다. 각 노드는 `base`와 `check` 두 배열로 표현되며, 바이트 단위로 트라이를 탐색합니다.

한국어 텍스트는 UTF-8로 처리됩니다. 한글 한 음절은 UTF-8에서 3바이트이므로, 음절 단위 형태소는 Trie 깊이가 글자 수의 3배입니다.

### 3.3 검색 API

```
Trie::exact_match(key: &str) -> Option<u32>
    입력 키와 정확히 일치하는 엔트리의 u32 값(= entries.bin 인덱스) 반환

Trie::common_prefix_search(text: &str) -> impl Iterator<Item = (u32, usize)>
    텍스트의 모든 접두사 매칭 열거
    반환값: (entries.bin 인덱스, 매칭된 바이트 길이)

Trie::common_prefix_search_at(text: &str, start_byte: usize) -> Vec<(u32, usize)>
    특정 바이트 위치에서 시작하는 접두사 매칭
    반환값: (entries.bin 인덱스, 매칭 끝 바이트 위치)
```

형태소 분석기는 `common_prefix_search_at`을 텍스트 각 위치마다 호출하여 후보 형태소를 수집합니다.

### 3.4 TrieBuilder

```
TrieBuilder::build(entries: &[(&str, u32)]) -> Result<Vec<u8>>
```

입력은 **반드시 바이트 순으로 정렬**되어야 합니다. 정렬 없이 사용하려면 `build_unsorted()`를 사용합니다. 내부적으로 `entries.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()))` 후 `build()` 호출합니다.

빌드 결과인 `Vec<u8>`을 `save_to_file()` 또는 `save_to_compressed_file(level: i32)`로 저장합니다.

### 3.5 압축 지원

`zstd` feature 활성화 시:
- `Trie::from_compressed_file()` — `zstd::Decoder`로 스트리밍 압축 해제 후 `from_vec()`
- `TrieBuilder::save_to_compressed_file(bytes, path, level)` — `zstd::Encoder` 스트리밍 압축

압축 해제 결과는 항상 `Vec<u8>`에 올라가며, mmap은 사용하지 않습니다.

### 3.6 EntryIndex / PrefixMatch 보조 타입

```
EntryIndex(u32)     -- Trie 조회값을 타입으로 래핑
PrefixMatch {
    index:       EntryIndex
    byte_length: usize
    start_byte:  usize
    end_byte:    usize
}
```

`DictionarySearcher<'a, E>` — Trie + 엔트리 배열을 결합한 검색 헬퍼 (테스트/유틸리티 용도).

---

## 4. matrix.bin 분석

### 4.1 바이너리 포맷

```
[Header: 4 bytes]
  lsize : u16 (LE)   <- 좌문맥 ID 개수
  rsize : u16 (LE)   <- 우문맥 ID 개수

[Cost Data: lsize * rsize * 2 bytes]
  costs : [i16 (LE); lsize * rsize]
```

접근 인덱스 공식:
```
index = right_id + lsize * left_id
cost  = costs[index]   (i16, range -32768 ~ 32767)
```

이 공식은 row-major 배치에서 `right_id`가 열(column), `left_id`가 행(row)에 해당합니다.

헤더 크기 상수: `MATRIX_HEADER_SIZE = 4`.

### 4.2 매트릭스 구현체

세 가지 구현체가 `Matrix` trait으로 통일됩니다.

```
Matrix trait {
    fn get(&self, right_id: u16, left_id: u16) -> i32
    fn left_size(&self) -> usize
    fn right_size(&self) -> usize
    fn entry_count(&self) -> usize    // default: left_size * right_size
}
```

경계 밖 접근은 `INVALID_CONNECTION_COST = i32::MAX`를 반환합니다.

#### DenseMatrix

전체 비용 배열을 `Vec<i16>`에 보유합니다. `from_bin_bytes()` 내에서 전체 파일을 읽고 i16 배열로 파싱합니다.

```
DenseMatrix {
    lsize : usize
    rsize : usize
    costs : Vec<i16>    // heap 할당, lsize * rsize 원소
}
```

메모리 사용량: `lsize * rsize * 2` bytes + Vec 오버헤드.
mecab-ko-dic 실제 크기 기준(약 2800 x 2800): ~15.7 MB.

#### MmapMatrix

`memmap2::Mmap`으로 파일을 매핑하며, `get()` 호출 시 `i16::from_le_bytes` 변환을 수행합니다.

```
MmapMatrix {
    lsize : usize
    rsize : usize
    mmap  : memmap2::Mmap
}
```

```
offset(right_id, left_id) = MATRIX_HEADER_SIZE + (right_id + lsize * left_id) * 2
bytes = [mmap[offset], mmap[offset + 1]]
cost  = i16::from_le_bytes(bytes) as i32
```

파일 크기 엄격 검증: `mmap.len() != MATRIX_HEADER_SIZE + lsize * rsize * 2`이면 오류 반환.

여러 프로세스가 같은 `matrix.bin`을 공유하면 OS가 물리 메모리를 공유합니다. `LoadOptions::memory_optimized()` 설정 시 활성화됩니다.

#### SparseMatrix

`HashMap<usize, i16>` — 기본값과 다른 항목만 저장합니다. 직렬화/역직렬화 지원 없음(변환 유틸리티 용도).

```
SparseMatrix {
    lsize        : usize
    rsize        : usize
    default_cost : i16
    entries      : HashMap<usize, i16>
}
```

sparsity() = 1 - (저장된 항목 수 / 전체 항목 수).

#### ConnectionMatrix 열거형

```
enum ConnectionMatrix {
    Dense(DenseMatrix)
    Sparse(SparseMatrix)
    Mmap(MmapMatrix)
}
```

`Matrix` trait을 구현하며, `SystemDictionary`는 이 열거형을 보유합니다.

### 4.3 포맷 자동 감지 (MatrixLoader)

`MatrixLoader::load(path)` — 파일 확장자로 포맷 결정:
- `.def` → 텍스트 파싱 (matrix.def)
- `.zst` 또는 `.bin.zst` → zstd 압축 해제 후 바이너리 파싱
- `.bin` → 바이너리 파싱
- 기타 → 바이너리 시도 후 텍스트 폴백

### 4.4 텍스트 포맷 (matrix.def)

```
<lsize> <rsize>
<right_id> <left_id> <cost>
...
```

초기값은 `i16::MAX`(연결 불가). 명시된 엔트리만 덮어씁니다. 주석(`#`)과 빈 줄 건너뜁니다.

### 4.5 SIMD 최적화 (선택 feature)

`matrix/simd.rs` — `simd` feature 활성화 시 `SimdMatrix` 제공. Rust nightly `portable_simd`를 사용하며, `lib.rs`에서 `#![cfg_attr(feature = "simd", feature(portable_simd))]`로 선언합니다.

---

## 5. v2 한계 및 v3 개선 기회

### 5.1 entries.bin — LRU 캐시 성능 문제

**한계**: `LruCache`의 `access_order: Vec<u32>` 구조에서 eviction 시 `Vec::remove(0)`은 O(n) 연산입니다. 캐시 크기 10,000에서는 무시 가능하지만, 크기를 늘릴수록 eviction 비용이 선형 증가합니다.

**한계**: `get()` 메서드가 캐시 확인을 위해 write lock을 획득합니다(`RwLock::write()`). 읽기 위주 워크로드에서 read lock으로 확인하고 miss 시에만 write lock을 잡는 double-checked 패턴이 더 효율적입니다.

**v3 제안**: `indexmap` 또는 `lru` 크레이트 도입으로 O(1) eviction 보장. 또는 `DashMap` + 별도 eviction 큐로 락 경합 최소화.

### 5.2 entries.bin — 압축 미지원

**한계**: entries.bin v2는 압축을 지원하지 않습니다. 압축 버전(`.zst`)에 대한 `LazyEntries` 로딩 경로가 없습니다.

**v3 제안**: 섹션 단위 압축(zstd frame per block) 또는 파일 전체 압축 후 index table을 압축 외부에 두는 방식. 후자가 랜덤 접근 친화적입니다.

### 5.3 entries.bin — surface 길이 제한

**한계**: `surface_len: u16` 최대 65,535 bytes. 현실적으로 문제 없으나, feature 문자열에도 같은 제한(`feature_len: u16`)이 적용됩니다. 복합어의 feature 문자열이 긴 경우 빌드 시 오류가 발생합니다(`"feature too long"`).

**v3 제안**: `feature_len: u32`로 확장하거나, feature 문자열을 별도 string pool로 분리.

### 5.4 sys.dic — 전체 메모리 로드

**한계**: `Trie::from_file()`은 `std::fs::read()`로 전체 trie 바이트를 `Vec<u8>`에 로드합니다. 대용량 사전에서 시작 시간과 메모리 사용량이 증가합니다.

**v3 제안**: mmap 기반 `Trie::from_mmap_file()` 경로 추가. `Cow::Borrowed(&mmap[..])` 패턴으로 파일을 그대로 참조합니다.

### 5.5 sys.dic — yada 크레이트 의존

**한계**: Trie 내부 포맷이 yada 크레이트에 완전히 종속됩니다. yada의 포맷 변경이 하위 호환을 깨뜨릴 수 있으며, 커스텀 최적화(예: 한글 전용 노드 압축)가 불가능합니다.

**v3 제안**: 자체 Double-Array 구현 또는 fst 크레이트(`fst::Map`) 전환 검토. fst는 이미 의존성에 포함되어 있습니다(`Cargo.toml: fst.workspace = true`).

### 5.6 matrix.bin — 매직 넘버 / 버전 없음

**한계**: matrix.bin 헤더에 매직 넘버와 버전이 없습니다. 파일 크기 검증(`mmap.len() != header + data`)만으로 형식을 확인합니다.

```
현재: [lsize: u16][rsize: u16][costs...]
```

파일 손상이나 잘못된 파일 경로 지정 시 크기 우연 일치로 오류 없이 잘못된 데이터를 반환할 수 있습니다.

**v3 제안**:
```
v3: [magic: 4][version: u32][lsize: u32][rsize: u32][costs...]
```
`lsize`, `rsize`도 u32로 확장하면 65,535 이상의 문맥 ID를 지원합니다.

### 5.7 엔디안 고정 (이식성)

**현재 상태**: 모든 수치 필드가 `byteorder::LittleEndian`으로 명시적 인코딩됩니다. 크로스 컴파일 및 빅엔디안 아키텍처에서도 정확히 동작합니다.

**v3 고려**: 네이티브 엔디안 변환을 위해 `byteorder` 대신 `u16::to_le_bytes()` / `u16::from_le_bytes()` 직접 사용으로 의존성 제거를 검토할 수 있습니다.

### 5.8 MmapDictionary 고유 entries.bin 포맷 혼재

**한계**: `loader.rs`의 `MmapDictionary`는 `SystemDictionary`의 entries.bin(MKED/MKE2)과 다른 고유 바이너리 형식을 사용합니다(매직 없이 count:u32만). 두 형식이 혼재하여 `parse_entries_binary()`에서 매직을 확인하고 CSV 폴백을 강제합니다.

**v3 제안**: `MmapDictionary`를 `SystemDictionary`로 통합하거나, entries.bin 형식을 단일화.

---

## 6. 코드 맵

### 6.1 핵심 모듈

| 파일 경로 | 책임 |
|-----------|------|
| `crates/mecab-ko-dict/src/lazy_entries.rs` | entries.bin v2(MKE2) 읽기/쓰기, LRU 캐시, v1→v2 마이그레이션 |
| `crates/mecab-ko-dict/src/entry_store.rs` | `EntryStore` trait, `EagerStore`, `LazyStore` 추상화 |
| `crates/mecab-ko-dict/src/trie.rs` | yada 크레이트 래핑, `Trie`, `TrieBuilder`, `DictionarySearcher` |
| `crates/mecab-ko-dict/src/matrix/mod.rs` | matrix.bin 읽기/쓰기, `DenseMatrix`, `MmapMatrix`, `SparseMatrix`, `Matrix` trait |
| `crates/mecab-ko-dict/src/matrix/simd.rs` | SIMD 최적화 행렬 (`simd` feature) |
| `crates/mecab-ko-dict/src/dictionary.rs` | `SystemDictionary`, `DictEntry`, `LoadOptions`, `DictionaryLoader` |
| `crates/mecab-ko-dict/src/loader.rs` | `MmapDictionary`, `LazyDictionary`, `LoaderConfig` (독립 로더) |
| `crates/mecab-ko-dict/src/lib.rs` | 공개 API 재내보내기, `Dictionary` trait, `Entry`, 오류 타입 |

### 6.2 보조 모듈

| 파일 경로 | 책임 |
|-----------|------|
| `crates/mecab-ko-dict/src/user_dict.rs` | 사용자 사전, CSV 로딩, 빌더 패턴 |
| `crates/mecab-ko-dict/src/hot_reload.rs` | Delta 기반 핫 리로드 v1 |
| `crates/mecab-ko-dict/src/hot_reload_v2.rs` | ArcSwap 기반 wait-free 핫 리로드 (`hot-reload-v2` feature) |
| `crates/mecab-ko-dict/src/file_watcher.rs` | `notify` 크레이트 래핑, 파일 변경 감지 |
| `crates/mecab-ko-dict/src/string_pool.rs` | 문자열 중복 제거 풀 (`StringPool`, `ConcurrentStringPool`) |
| `crates/mecab-ko-dict/src/domain.rs` | 도메인 오버레이 사전 |

### 6.3 Cargo.toml 주요 의존성

| 크레이트 | 용도 |
|---------|------|
| `yada` | Double-Array Trie (sys.dic 형식 정의) |
| `memmap2` | 안전한 메모리 맵 I/O (`MmapMatrix`, `LazyEntries`) |
| `byteorder` | Little-Endian 읽기/쓰기 |
| `zstd` | 압축/해제 (optional, default on) |
| `fst` | FST 기반 검색 (의존성 포함, 현재 미사용) |
| `arc-swap` | Wait-free hot-reload (`hot-reload-v2` feature) |
| `compact_str` | 문자열 메모리 최적화 (`compact-strings` feature) |

---

## 선행 문서와의 관계

`docs/dictionary-format-v2.md` (v2.1, 2026-01-05)는 MeCab 원본 바이너리 포맷(sys.dic Darts 형식, 40바이트 헤더 등)을 분석하며, Rust 구현체의 파생 v2 포맷(`MKE2`, `MKED`)을 섹션 7에서 소개합니다.

본 문서는 그 분석을 소스 코드 수준으로 확장하며, 다음 항목을 추가합니다.

- entries.bin v2 인덱스 테이블 바이트 레이아웃의 정확한 계산식
- LRU 캐시 구현 상세 및 한계
- `EntryStore` trait 추상화 계층
- MmapMatrix 오프셋 계산 공식
- `MmapDictionary` 고유 포맷 혼재 문제
- v3 개선 제안 (압축, mmap trie, 매직 넘버, LRU 개선)

---

*작성: 2026-05-07*
