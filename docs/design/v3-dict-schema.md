# mecab-ko v3 바이너리 사전 포맷 설계

> v2의 LRU O(n) eviction, sys.dic 전체 메모리 로드, matrix.bin 헤더 부재 세 가지 핵심 문제를 해결하는 v3 바이너리 사전 포맷 설계

---

**문서 버전**: 1.0
**작성일**: 2026-05-08
**선행 문서**: `docs/design/v2-dict-code-analysis.md`

---

## 1. 설계 목표

### 1.1 성능

- entries.bin LRU eviction을 O(n) → O(1)로 개선
- sys.dic mmap 경로 추가로 초기 로딩 비용 제거
- LRU 읽기 경로에서 write lock 경합 제거 (read path 분리)

### 1.2 메모리 효율

- sys.dic mmap으로 프로세스 간 물리 메모리 공유 (OS page cache 활용)
- entries.bin feature 문자열 `u32` 길이 확장으로 복합어 오류 방지
- matrix.bin MmapMatrix 경로 기본화 (DenseMatrix Vec 복사 제거)

### 1.3 안전성 및 하위 호환성

- matrix.bin에 magic + version 추가로 파일 손상 조기 감지
- entries.bin MKE3 포맷은 MKE2와 병존 — 감지 로직으로 자동 선택
- v1(MKED) / v2(MKE2) 파일은 v3 로더에서 계속 읽기 가능

---

## 2. entries.bin v3 (MKE3)

### 2.1 헤더 레이아웃

```
[Header: 24 bytes]
  magic         : [u8; 4]  = b"MKE3"
  version       : u32 (LE) = 3
  count         : u32 (LE)         -- 총 엔트리 수
  flags         : u16 (LE)         -- 비트 플래그 (아래 참조)
  reserved      : u16 (LE) = 0
  index_offset  : u64 (LE)         -- 인덱스 테이블 파일 내 절대 오프셋
```

flags 비트 정의:

| 비트 | 의미 |
|------|------|
| 0    | feature 길이 필드가 u32 (0 = u16 호환, 1 = u32 확장) |
| 1    | 예약 |
| 2–15 | 예약 (0으로 설정) |

v3 구현체는 flags bit 0 = 1로 항상 기록한다. v2 하위 호환 읽기 시 bit 0 = 0 처리.

### 2.2 엔트리 레코드 레이아웃

```
[Entry Record: 가변 길이]
  left_id     : u16 (LE)                -- 좌문맥 ID
  right_id    : u16 (LE)                -- 우문맥 ID
  cost        : i16 (LE)                -- 단어 비용
  surface_len : u16 (LE)                -- surface 바이트 길이 (최대 65,535)
  feature_len : u32 (LE)                -- feature 바이트 길이 (v3 확장: 최대 4 GB)
  surface     : [u8; surface_len]       -- UTF-8
  feature     : [u8; feature_len]       -- UTF-8 (쉼표 구분 품사 정보)
```

고정 헤더 크기: `2 + 2 + 2 + 2 + 4 = 12 bytes` (v2의 10 bytes에서 +2).

### 2.3 인덱스 테이블

파일 끝에 배치. 구조는 v2와 동일하나 헤더의 `index_offset` 필드로 위치를 확인한다.

```
[Index Table: count * 8 bytes]
  offset_0    : u64 (LE)   -- entry_0의 파일 내 절대 오프셋
  offset_1    : u64 (LE)
  ...
  offset_{count-1} : u64 (LE)
```

인덱스 조회 공식 (v2와 동일):

```
index_table_pos = index_offset + (entry_index * 8)
entry_pos       = bytes[index_table_pos..+8] as u64 (LE)
```

### 2.4 Rust 타입 정의

```rust
/// entries.bin v3 헤더
#[repr(C)]
struct EntriesV3Header {
    magic:        [u8; 4],   // b"MKE3"
    version:      u32,       // 3 (LE)
    count:        u32,       // LE
    flags:        u16,       // LE
    reserved:     u16,       // 0
    index_offset: u64,       // LE
}

const ENTRIES_V3_HEADER_SIZE: usize = 24;

bitflags::bitflags! {
    struct EntriesFlags: u16 {
        const FEATURE_U32 = 0b0000_0001;
    }
}
```

### 2.5 LRU 캐시 개선: `lru` 크레이트 도입

**현재 문제 (v2)**:
- `LruCache.access_order: Vec<u32>` — eviction 시 `Vec::remove(0)`은 O(n)
- `get()` 진입 시 write lock 획득 — 읽기 위주 워크로드에서 불필요한 직렬화

**v3 해결책**:

```rust
use lru::LruCache;  // lru = "0.16" (Cargo.lock 확인: 0.16.3)
use std::num::NonZeroUsize;
use std::sync::{Arc, RwLock};

pub struct LazyEntriesV3 {
    path:         PathBuf,
    mmap:         memmap2::Mmap,
    count:        u32,
    index_offset: u64,
    flags:        u16,
    // lru::LruCache는 내부적으로 LinkedHashMap 기반 — O(1) get/insert/evict
    cache:        RwLock<LruCache<u32, Arc<DictEntry>>>,
}
```

읽기 경로 분리 (double-checked locking 패턴):

```rust
pub fn get(&self, index: u32) -> Result<Arc<DictEntry>> {
    // 1. read lock으로 캐시 확인
    // lru::LruCache::peek()는 LRU 순서를 변경하지 않으므로 read lock에서 안전
    {
        let cache = self.cache.read()
            .map_err(|_| DictError::Format("cache lock poisoned".into()))?;
        if let Some(entry) = cache.peek(&index) {
            return Ok(Arc::clone(entry));
        }
    }

    // 2. 디스크에서 읽기 (락 없이)
    let entry = self.load_entry_from_disk(index)?;

    // 3. write lock으로 캐시 삽입
    // lru::LruCache::put()은 O(1) — insert + evict 모두
    let mut cache = self.cache.write()
        .map_err(|_| DictError::Format("cache lock poisoned".into()))?;
    let arc = Arc::new(entry);
    cache.put(index, Arc::clone(&arc));
    Ok(arc)
}
```

> 주의: `peek()`를 사용하면 LRU 순서가 갱신되지 않는다. 읽기 경합이 많은 핫스팟 환경에서는 이 트레이드오프가 허용 가능하다. 순서 갱신이 필요하다면 read lock을 포기하고 write lock 단일 경로를 유지한다.

`Cargo.toml` 변경:

```toml
# workspace Cargo.toml [workspace.dependencies]
lru = "0.16"

# crates/mecab-ko-dict/Cargo.toml
lru = { workspace = true }
```

### 2.6 v2 → v3 마이그레이션

```rust
/// entries.bin v2(MKE2) → v3(MKE3) 마이그레이션
pub fn migrate_entries_v2_to_v3<P: AsRef<Path>>(
    v2_path: P,
    v3_path: P,
) -> Result<()> {
    // 1. v2 LazyEntries로 전체 로드
    let v2 = LazyEntries::from_file(v2_path.as_ref())?;
    let entries = v2.load_all()?;

    // 2. v3 포맷으로 저장
    LazyEntriesV3::save_entries(&entries, v3_path.as_ref())?;

    Ok(())
}
```

로더 자동 감지 순서:

```rust
fn detect_entries_format(path: &Path) -> Result<EntriesFormat> {
    let mut buf = [0u8; 4];
    File::open(path)?.read_exact(&mut buf)?;
    match &buf {
        b"MKE3" => Ok(EntriesFormat::V3),
        b"MKE2" => Ok(EntriesFormat::V2),
        b"MKED" => Ok(EntriesFormat::V1),
        _       => Err(DictError::Format("unknown entries.bin magic".into())),
    }
}
```

---

## 3. sys.dic v3

### 3.1 현재 상태 (v2)

`Trie::from_file()`은 `std::fs::read()`로 전체 파일을 `Vec<u8>`에 올린 뒤 `Cow::Owned`로 yada에 전달한다. 대용량 사전(mecab-ko-dic 기준 약 30 MB)에서 초기 로딩에 40–100 ms가 소요된다.

### 3.2 mmap 경로 추가

```rust
use memmap2::Mmap;
use std::borrow::Cow;

impl Trie<'static> {
    /// mmap 기반 로딩 — OS 페이지 캐시 활용, 프로세스 간 물리 메모리 공유
    ///
    /// # Safety
    /// memmap2::Mmap 내부 unsafe 사용. 파일이 로드 중 변경되지 않아야 한다.
    #[allow(unsafe_code)]
    pub fn from_mmap_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(path.as_ref()).map_err(DictError::Io)?;
        let mmap = unsafe { Mmap::map(&file).map_err(DictError::Io)? };

        // SAFETY: mmap의 수명을 'static으로 연장하지 않고,
        // Box<Mmap>으로 소유권을 갖고 슬라이스를 Cow::Borrowed에 전달.
        // TrieMmap 래퍼로 수명 관리.
        let da = yada::DoubleArray::new(Cow::Borrowed(&mmap[..]));
        // TODO: 수명 문제 해결을 위해 TrieMmap 구조체 도입 (3.3 참조)
        unimplemented!()
    }
}

/// mmap 수명 소유 래퍼
pub struct TrieMmap {
    /// Mmap을 먼저 선언 — drop 순서 보장 (da가 먼저 drop되어야 함)
    _mmap: Box<Mmap>,
    da:    yada::DoubleArray<Cow<'static, [u8]>>,
}
```

수명 문제 해결 접근 (`'static` transmute 대신 자기 참조 구조체 회피):

```rust
pub struct TrieMmap {
    mmap: Arc<Mmap>,
}

impl TrieMmap {
    pub fn exact_match(&self, key: &str) -> Option<u32> {
        // mmap 슬라이스를 매 호출마다 전달
        let da = yada::DoubleArray::new(Cow::Borrowed(&self.mmap[..]));
        da.exact_match_search(key.as_bytes()).map(|(v, _)| v)
    }

    pub fn common_prefix_search_at(
        &self,
        text: &str,
        start_byte: usize,
    ) -> Vec<(u32, usize)> {
        let da = yada::DoubleArray::new(Cow::Borrowed(&self.mmap[..]));
        da.common_prefix_search(&text.as_bytes()[start_byte..])
            .map(|(v, len)| (v, start_byte + len))
            .collect()
    }
}
```

> `yada::DoubleArray::new()`는 슬라이스 참조만 보유하고 복사하지 않는다. mmap 슬라이스가 유효한 동안 안전하다. `TrieMmap`이 `Arc<Mmap>`을 소유하므로 수명이 보장된다.

### 3.3 fallback 경로 유지

```rust
pub enum TrieBackend {
    /// Vec 로드 (현행 v2 동작, 압축 해제 포함)
    Owned(Trie<'static>),
    /// mmap 기반 (v3 신규)
    Mmap(TrieMmap),
}

impl TrieBackend {
    pub fn load<P: AsRef<Path>>(path: P, use_mmap: bool) -> Result<Self> {
        let path = path.as_ref();
        if use_mmap && !is_compressed(path) {
            // mmap은 압축 파일에 적용 불가 — .zst 확장자 제외
            TrieMmap::from_file(path).map(TrieBackend::Mmap)
        } else {
            // .zst 또는 use_mmap=false: 기존 Vec 로드 경로
            Trie::from_file_or_compressed(path).map(TrieBackend::Owned)
        }
    }
}
```

### 3.4 프로세스 간 공유 이점

같은 호스트의 여러 형태소 분석 프로세스(웹서버 worker 등)가 동일 `sys.dic`을 mmap하면, OS가 물리 페이지를 공유한다.

- mecab-ko-dic `sys.dic` 기준 약 30 MB
- 8개 프로세스 × 30 MB = 240 MB → mmap 공유 시 실제 물리 사용 ~30 MB

`LoadOptions`에 `use_mmap_trie: bool` 필드 추가:

```rust
impl LoadOptions {
    pub fn memory_optimized() -> Self {
        Self {
            use_lazy_entries: true,
            use_mmap_matrix:  true,
            use_mmap_trie:    true,   // v3 신규
            ..Default::default()
        }
    }
}
```

---

## 4. matrix.bin v3 (MKM3)

### 4.1 현재 문제 (v2)

```
현재 헤더: [lsize: u16][rsize: u16]  (4 bytes, magic 없음)
```

파일 크기만으로 유효성 검증. 파일 경로 오입력 또는 손상 시 조용한 오동작 가능성.

### 4.2 v3 헤더 레이아웃

```
[Header: 16 bytes]
  magic    : [u8; 4]  = b"MKM3"
  version  : u8       = 3
  flags    : u8                -- 비트 플래그
  reserved : u16      = 0
  lsize    : u32 (LE)          -- 좌문맥 ID 개수 (v2: u16 → v3: u32)
  rsize    : u32 (LE)          -- 우문맥 ID 개수 (v2: u16 → v3: u32)

[Cost Data: lsize * rsize * 2 bytes]
  costs    : [i16 (LE); lsize * rsize]
```

flags 비트 정의:

| 비트 | 의미 |
|------|------|
| 0    | 체크섬 포함 (1 = 헤더 뒤 4 bytes CRC32) |
| 1    | 예약 |
| 2–7  | 예약 |

flags bit 0 = 0인 경우 체크섬 없음 (기본). 체크섬 활성화 시 헤더 총 크기 = 20 bytes.

```
[Optional Checksum: 4 bytes, flags bit 0 = 1일 때만]
  crc32    : u32 (LE)   -- Cost Data 전체에 대한 CRC32
```

### 4.3 Rust 타입 정의

```rust
const MATRIX_MAGIC_V3: &[u8; 4] = b"MKM3";
const MATRIX_HEADER_V3_SIZE: usize = 16;
const MATRIX_HEADER_V3_SIZE_WITH_CRC: usize = 20;

struct MatrixV3Header {
    magic:    [u8; 4],  // b"MKM3"
    version:  u8,       // 3
    flags:    u8,
    reserved: u16,      // 0
    lsize:    u32,      // LE
    rsize:    u32,      // LE
}

bitflags::bitflags! {
    struct MatrixFlags: u8 {
        const CHECKSUM = 0b0000_0001;
    }
}
```

### 4.4 접근 인덱스 공식 (변경 없음)

```
data_offset = MATRIX_HEADER_V3_SIZE (+ 4 if flags::CHECKSUM)
offset(right_id, left_id) = data_offset + (right_id + lsize * left_id) * 2
cost = i16::from_le_bytes([bytes[offset], bytes[offset + 1]]) as i32
```

`lsize`, `rsize`가 u32로 확장되었으므로 인덱스 계산을 usize로 캐스팅할 때 오버플로우 체크 필요:

```rust
fn cost_offset(lsize: u32, right_id: u16, left_id: u16) -> Option<usize> {
    let idx = (right_id as usize)
        .checked_add((lsize as usize).checked_mul(left_id as usize)?)?;
    let byte_off = idx.checked_mul(2)?;
    MATRIX_HEADER_V3_SIZE.checked_add(byte_off)
}
```

### 4.5 DenseMatrix / MmapMatrix / SparseMatrix 통합

`ConnectionMatrix` 열거형은 v2 구조를 유지하되, v3 헤더 파싱 경로 추가:

```rust
pub enum ConnectionMatrix {
    Dense(DenseMatrix),
    Sparse(SparseMatrix),
    Mmap(MmapMatrix),
}

impl ConnectionMatrix {
    /// v3 헤더를 읽어 DenseMatrix 또는 MmapMatrix 생성
    pub fn from_bin_v3<P: AsRef<Path>>(path: P, use_mmap: bool) -> Result<Self> {
        // 헤더 파싱
        let hdr = read_matrix_v3_header(path.as_ref())?;
        if use_mmap {
            MmapMatrix::from_v3(path, &hdr).map(ConnectionMatrix::Mmap)
        } else {
            DenseMatrix::from_v3(path, &hdr).map(ConnectionMatrix::Dense)
        }
    }
}
```

### 4.6 v2 → v3 변환 유틸리티

```rust
pub fn migrate_matrix_v2_to_v3<P: AsRef<Path>>(
    v2_path: P,
    v3_path: P,
    add_checksum: bool,
) -> Result<()> {
    let matrix = DenseMatrix::from_bin_bytes(
        &std::fs::read(v2_path.as_ref()).map_err(DictError::Io)?
    )?;
    write_matrix_v3(v3_path.as_ref(), &matrix, add_checksum)
}
```

---

## 5. 통합 사전 로더

### 5.1 `SystemDictionary::load()` v3 경로

```rust
impl SystemDictionary {
    pub fn load<P: AsRef<Path>>(dir: P, opts: &LoadOptions) -> Result<Self> {
        let dir = dir.as_ref();

        // sys.dic — v3 mmap 경로 우선
        let trie = TrieBackend::load(
            dir.join("sys.dic"),
            opts.use_mmap_trie,
        )?;

        // entries.bin — 포맷 자동 감지 (MKE3 → MKE2 → MKED 순)
        let entries: Arc<dyn EntryStore> = match detect_entries_format(
            &dir.join("entries.bin")
        )? {
            EntriesFormat::V3 => Arc::new(LazyEntriesV3::from_file(
                dir.join("entries.bin"),
                opts.entries_cache_size,
            )?),
            EntriesFormat::V2 => {
                if opts.use_lazy_entries {
                    Arc::new(LazyEntries::from_file(dir.join("entries.bin"))?)
                } else {
                    Arc::new(EagerStore::from_file(dir.join("entries.bin"))?)
                }
            }
            EntriesFormat::V1 => {
                // v1은 EagerStore만 지원
                Arc::new(EagerStore::from_v1_file(dir.join("entries.bin"))?)
            }
        };

        // matrix.bin — v3 헤더 시도, 실패 시 v2 폴백
        let matrix = load_matrix_auto(
            &dir.join("matrix.bin"),
            opts.use_mmap_matrix,
        )?;

        Ok(Self { trie, entries, matrix })
    }
}

fn load_matrix_auto(path: &Path, use_mmap: bool) -> Result<ConnectionMatrix> {
    // 매직 넘버 확인
    let mut buf = [0u8; 4];
    File::open(path)?.read_exact(&mut buf)?;
    if &buf == b"MKM3" {
        ConnectionMatrix::from_bin_v3(path, use_mmap)
    } else {
        // v2: [lsize:u16][rsize:u16][...] — 매직 없음
        if use_mmap {
            MmapMatrix::from_file(path).map(ConnectionMatrix::Mmap)
        } else {
            let bytes = std::fs::read(path).map_err(DictError::Io)?;
            DenseMatrix::from_bin_bytes(&bytes).map(ConnectionMatrix::Dense)
        }
    }
}
```

### 5.2 `LoadOptions` v3 확장

```rust
pub struct LoadOptions {
    pub use_lazy_entries:   bool,    // 기존
    pub use_mmap_matrix:    bool,    // 기존
    pub use_mmap_trie:      bool,    // v3 신규: sys.dic mmap
    pub entries_cache_size: usize,   // v3 신규: LRU 캐시 크기 (기본 10,000)
}

impl Default for LoadOptions {
    fn default() -> Self {
        Self {
            use_lazy_entries:   true,
            use_mmap_matrix:    false,
            use_mmap_trie:      false,  // 기본 off (호환성)
            entries_cache_size: 10_000,
        }
    }
}

impl LoadOptions {
    pub fn memory_optimized() -> Self {
        Self {
            use_lazy_entries:   true,
            use_mmap_matrix:    true,
            use_mmap_trie:      true,
            entries_cache_size: 10_000,
        }
    }

    pub fn speed_optimized() -> Self {
        Self {
            use_lazy_entries:   false,
            use_mmap_matrix:    false,
            use_mmap_trie:      false,
            entries_cache_size: 0,
        }
    }
}
```

---

## 6. 구현 로드맵

### Phase 1: entries.bin LRU 교체 (lru 크레이트)

**목표**: `Vec::remove(0)` O(n) eviction → O(1)

작업 항목:
1. `Cargo.toml` workspace에 `lru = "0.16"` 추가 (이미 Cargo.lock에 존재)
2. `lazy_entries.rs`의 `LruCache` 구조체 제거
3. `use lru::LruCache` + `RwLock<LruCache<u32, Arc<DictEntry>>>` 교체
4. `get()` 메서드에 read-lock first 패턴 적용 (`peek()` 활용)
5. 기존 LRU eviction 테스트(`test_lru_cache_eviction`) 검증

**예상 공수**: 1일
**위험**: 낮음 — API 호환 유지, 동작 변경 없음

### Phase 2: matrix.bin 헤더 추가 (MKM3)

**목표**: magic + version으로 파일 손상 조기 감지

작업 항목:
1. `matrix/mod.rs`에 `MATRIX_MAGIC_V3`, `MatrixV3Header` 추가
2. `write_matrix_v3()` 구현 (v2 `DenseMatrix` 직렬화 함수와 병존)
3. `load_matrix_auto()` 로직 추가 (magic 확인 후 v3/v2 분기)
4. `MmapMatrix`의 오프셋 계산에서 헤더 크기 상수 교체
5. `migrate_matrix_v2_to_v3()` 유틸리티 구현
6. `dict-build` CLI에 `--upgrade-matrix` 옵션 추가

**예상 공수**: 2일
**위험**: 낮음 — v2 읽기 경로 유지, 쓰기만 v3

### Phase 3: sys.dic mmap 통합

**목표**: 초기 로딩 40–100 ms 제거, 프로세스 간 메모리 공유

작업 항목:
1. `TrieMmap` 구조체 구현 (`Arc<Mmap>` 소유, yada 호출 래핑)
2. `TrieBackend` 열거형으로 `Trie<'static>` / `TrieMmap` 통합
3. `LoadOptions.use_mmap_trie` 필드 추가
4. `SystemDictionary::load()`에서 `TrieBackend::load()` 호출
5. 압축 sys.dic (`.zst`) 에서는 mmap 비활성화 처리

**예상 공수**: 3일
**위험**: 중간 — yada API 수명 처리, `unsafe` 코드 포함

### Phase 4: entries.bin v3 (MKE3) + feature u32 확장

**목표**: feature_len u32 확장으로 복합어 빌드 오류 방지

작업 항목:
1. `EntriesV3Header`, `ENTRIES_V3_HEADER_SIZE` 상수 정의
2. `LazyEntriesV3::save_entries()` — feature_len u32 직렬화
3. `LazyEntriesV3::from_file()` — MKE3 헤더 파싱
4. `detect_entries_format()` — magic 기반 포맷 감지
5. `migrate_entries_v2_to_v3()` 구현
6. `SystemDictionary::load()`에 v3 분기 추가

**예상 공수**: 2일
**위험**: 낮음 — v2 경로와 완전히 독립적으로 구현 가능

### Phase 5: 통합 테스트 + 벤치마크

작업 항목:
1. v2 → v3 왕복 변환 통합 테스트
2. LRU 교체 전후 벤치마크 (`criterion`): 10k / 50k 엔트리 캐시
3. sys.dic mmap vs Vec 로딩 시간 비교 (sysinfo 기반 메모리 사용량 포함)
4. matrix.bin v3 헤더 손상 감지 테스트
5. `dict-build` e2e 테스트: v2 입력 → v3 출력 → 형태소 분석 정확도 검증

---

## 7. 리스크 및 대안

### 7.1 lru 크레이트 peek() 트레이드오프

`peek()`는 LRU 순서를 갱신하지 않아 hot item이 evict될 수 있다. 히트율이 중요한 환경이라면 단일 write lock 경로를 유지하고 대신 `DashMap` 기반 shard lock을 검토한다.

**대안**: `quick_cache` 크레이트 — shard 기반 read lock 지원, `lru` 크레이트 없이도 O(1) eviction 보장. 단, 추가 의존성 도입 비용.

### 7.2 sys.dic mmap 파일 변경 문제

mmap 중 파일이 변경되면 UB 발생 가능. mecab-ko-dict는 hot-reload 기능이 있으므로 reload 시 기존 `TrieMmap`을 완전히 drop한 후 새 인스턴스를 생성해야 한다.

**대응**: `ArcSwap<TrieBackend>`로 atomic 교체, 구 Mmap 참조 소멸 보장 (`Arc::strong_count` 확인 또는 drop 시점 지연).

### 7.3 matrix.bin u32 lsize/rsize 하위 호환

v3는 lsize/rsize를 u32로 확장하지만, 현재 mecab-ko-dic 기준(~2800)에서는 불필요하다. 단순화를 위해 lsize/rsize를 u16으로 유지하고 magic + version만 추가하는 최소 버전도 선택지다.

**결정**: 장기 확장성을 위해 u32 유지. v2 하위 호환 읽기는 magic 감지로 자동 분기하므로 실사용 영향 없음.

### 7.4 yada 크레이트 종속 심화

sys.dic mmap 경로가 yada API에 더 밀착된다. yada가 업데이트되어 `DoubleArray::new()` 시그니처가 변경되면 `TrieMmap` 구현을 수정해야 한다.

**대안**: fst 크레이트 전환 (이미 `Cargo.toml`에 `fst.workspace = true`로 존재). fst는 mmap-first 설계이므로 수명 처리가 더 깔끔하다. 단, sys.dic을 fst 포맷으로 재빌드해야 하므로 Phase 3 이후 별도 스프린트로 검토.

---

## 부록: 포맷 매직 넘버 요약

| 파일 | v1 magic | v2 magic | v3 magic |
|------|----------|----------|----------|
| entries.bin | `MKED` | `MKE2` | `MKE3` |
| matrix.bin  | 없음   | 없음    | `MKM3`  |
| sys.dic     | (yada 내부) | (yada 내부) | (yada 내부, mmap 경로 신규) |

모든 멀티바이트 수치 필드는 Little-Endian. 변경 없음.

---

*작성: 2026-05-08*
