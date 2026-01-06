# 바이너리 사전 포맷 v3.0 설계 명세서

> **문서 버전**: 3.0
> **작성일**: 2026-01-05
> **이슈**: DIC-010 바이너리 사전 포맷 v3.0 설계
> **상태**: Draft

---

## 목차

1. [개요](#1-개요)
2. [설계 목표](#2-설계-목표)
3. [파일 구조](#3-파일-구조)
4. [공통 헤더 구조](#4-공통-헤더-구조)
5. [Double-Array Trie 바이너리 포맷](#5-double-array-trie-바이너리-포맷)
6. [Connection Matrix 바이너리 포맷](#6-connection-matrix-바이너리-포맷)
7. [엔트리 데이터 포맷](#7-엔트리-데이터-포맷)
8. [압축 지원](#8-압축-지원)
9. [메모리 매핑 지원](#9-메모리-매핑-지원)
10. [버전 관리 메커니즘](#10-버전-관리-메커니즘)
11. [사용자 사전 포맷](#11-사용자-사전-포맷)
12. [구현 참조](#12-구현-참조)

---

## 1. 개요

### 1.1 문서 목적

이 문서는 MeCab-Ko Rust 구현을 위한 바이너리 사전 포맷 v3.0을 정의합니다. 기존 MeCab 바이너리 포맷과의 차이점을 명시하고, Rust 에코시스템에 최적화된 새로운 포맷을 설계합니다.

### 1.2 기존 포맷과의 비교

| 항목 | MeCab 기존 포맷 | v3.0 신규 포맷 |
|------|----------------|----------------|
| Trie 구현 | Darts (C++) | yada (Rust) |
| 직렬화 | 커스텀 바이너리 | bincode + 옵션 |
| 압축 | 미지원 | Zstd 지원 |
| 메모리 매핑 | 부분 지원 | 완전 지원 (memmap2) |
| 바이트 순서 | 플랫폼 의존 | Little-Endian 고정 |
| 버전 관리 | 단순 버전 번호 | Semantic Versioning + 호환성 체크 |

### 1.3 관련 코드 파일

- `/home/mare/mecab-ko/rust/crates/mecab-ko-dict/src/trie.rs` - Double-Array Trie 구현
- `/home/mare/mecab-ko/rust/crates/mecab-ko-dict/src/matrix.rs` - Connection Matrix 구현
- `/home/mare/mecab-ko/rust/crates/mecab-ko-dict/src/user_dict.rs` - 사용자 사전 구현
- `/home/mare/mecab-ko/rust/crates/mecab-ko-dict/src/lib.rs` - 라이브러리 인터페이스

---

## 2. 설계 목표

### 2.1 핵심 목표

1. **빠른 로딩**: 메모리 매핑을 통한 지연 로딩 지원
2. **효율적 압축**: Zstd 압축으로 디스크 공간 절약 (30-50% 크기 감소 목표)
3. **메모리 효율성**: Zero-copy 접근 가능한 구조
4. **하위 호환성**: 버전 업그레이드 시 graceful degradation
5. **크로스 플랫폼**: Little-Endian 고정으로 플랫폼 독립성 보장

### 2.2 비기능 요구사항

| 요구사항 | 목표 |
|----------|------|
| 사전 로딩 시간 (mmap) | < 10ms |
| 사전 로딩 시간 (전체) | < 100ms |
| 압축률 (Zstd level 3) | 30-50% |
| 메모리 오버헤드 | < 5% |

---

## 3. 파일 구조

### 3.1 사전 디렉토리 레이아웃

```
dict/
├── mecab-ko-dict.meta       # 메타데이터 (JSON)
├── sys.dic                  # 시스템 사전 (Trie + 엔트리)
├── sys.dic.zst              # 압축된 시스템 사전 (선택)
├── matrix.bin               # 연접 비용 행렬
├── matrix.bin.zst           # 압축된 연접 비용 행렬 (선택)
├── char.bin                 # 문자 카테고리 정의
├── unk.bin                  # 미등록어 정의
└── user/                    # 사용자 사전 디렉토리
    ├── user.dic             # 사용자 사전
    └── *.csv                # 사용자 사전 CSV 원본
```

### 3.2 파일별 역할

| 파일 | 용도 | 필수 | 압축 가능 |
|------|------|------|-----------|
| `mecab-ko-dict.meta` | 사전 메타정보 | O | X |
| `sys.dic` | 시스템 사전 | O | O |
| `matrix.bin` | 연접 비용 행렬 | O | O |
| `char.bin` | 문자 카테고리 | O | X |
| `unk.bin` | 미등록어 정의 | O | X |
| `user.dic` | 사용자 사전 | X | O |

---

## 4. 공통 헤더 구조

### 4.1 파일 헤더 (모든 바이너리 파일 공통)

```
┌────────────────────────────────────────────────────┐
│                   File Header (64 bytes)            │
├────────────────────────────────────────────────────┤
│ Offset │ Size │ Type    │ Field          │ Value   │
├────────┼──────┼─────────┼────────────────┼─────────┤
│ 0x00   │ 4    │ [u8; 4] │ magic          │ "MKOD" │
│ 0x04   │ 1    │ u8      │ version_major  │ 3       │
│ 0x05   │ 1    │ u8      │ version_minor  │ 0       │
│ 0x06   │ 1    │ u8      │ version_patch  │ 0       │
│ 0x07   │ 1    │ u8      │ flags          │ 비트맵   │
│ 0x08   │ 4    │ u32     │ file_type      │ 파일 타입│
│ 0x0C   │ 4    │ u32     │ checksum       │ CRC32   │
│ 0x10   │ 8    │ u64     │ data_size      │ 데이터크기│
│ 0x18   │ 8    │ u64     │ entry_count    │ 엔트리 수│
│ 0x20   │ 32   │ [u8;32] │ reserved       │ 예약     │
└────────────────────────────────────────────────────┘
```

### 4.2 Magic Number

```
"MKOD" = MeCab-Ko Dictionary (0x4D 0x4B 0x4F 0x44)
```

### 4.3 Flags 비트맵

```
Bit 0: 압축 여부 (0: 미압축, 1: 압축)
Bit 1: 압축 알고리즘 (0: Zstd, 1: 예약)
Bit 2: Endianness (0: Little, 1: Big) - 현재 항상 0
Bit 3: 메모리 맵 최적화 (0: 미적용, 1: 적용)
Bit 4-7: 예약
```

### 4.4 File Type 코드

| 코드 | 타입 | 설명 |
|------|------|------|
| 0x01 | SYS_DICT | 시스템 사전 |
| 0x02 | USER_DICT | 사용자 사전 |
| 0x03 | MATRIX | 연접 비용 행렬 |
| 0x04 | CHAR_DEF | 문자 카테고리 |
| 0x05 | UNK_DEF | 미등록어 정의 |

### 4.5 Rust 구조체 정의

```rust
/// 파일 헤더 (64바이트)
#[repr(C, packed)]
pub struct FileHeader {
    /// 매직 넘버 "MKOD"
    pub magic: [u8; 4],
    /// 버전 (major.minor.patch)
    pub version_major: u8,
    pub version_minor: u8,
    pub version_patch: u8,
    /// 플래그 비트맵
    pub flags: u8,
    /// 파일 타입
    pub file_type: u32,
    /// 체크섬 (CRC32)
    pub checksum: u32,
    /// 데이터 크기 (헤더 제외)
    pub data_size: u64,
    /// 엔트리 수
    pub entry_count: u64,
    /// 예약 공간
    pub reserved: [u8; 32],
}
```

---

## 5. Double-Array Trie 바이너리 포맷

### 5.1 개요

Double-Array Trie는 `yada` 라이브러리를 사용하여 구현됩니다. 이 섹션은 yada가 생성하는 바이너리 포맷과 이를 사전 파일에 통합하는 방법을 정의합니다.

### 5.2 yada 라이브러리 바이너리 포맷

yada 라이브러리는 다음과 같은 내부 바이너리 구조를 사용합니다:

```
┌──────────────────────────────────────────────────────────┐
│                 yada Double-Array 구조                    │
├──────────────────────────────────────────────────────────┤
│ [u32 array]                                               │
│ ├── base[0], check[0]  - interleaved                     │
│ ├── base[1], check[1]                                    │
│ ├── ...                                                   │
│ └── base[n], check[n]                                    │
└──────────────────────────────────────────────────────────┘
```

### 5.3 Trie 파일 구조 (sys.dic)

```
┌────────────────────────────────────────────────────────────┐
│                    sys.dic 파일 구조                        │
├────────────────────────────────────────────────────────────┤
│ [File Header]           64 bytes                           │
├────────────────────────────────────────────────────────────┤
│ [Trie Section Header]   16 bytes                           │
│ ├── trie_size: u64      Trie 바이트 크기                    │
│ └── entry_offset: u64   엔트리 섹션 오프셋                   │
├────────────────────────────────────────────────────────────┤
│ [Trie Data]             Variable                           │
│ └── yada Double-Array 바이너리                              │
├────────────────────────────────────────────────────────────┤
│ [Entry Section Header]  16 bytes                           │
│ ├── entry_count: u64    엔트리 수                           │
│ └── feature_offset: u64 피처 문자열 오프셋                   │
├────────────────────────────────────────────────────────────┤
│ [Entry Array]           Variable                           │
│ └── Entry[] (고정 크기)                                     │
├────────────────────────────────────────────────────────────┤
│ [Feature Strings]       Variable                           │
│ └── NULL 종료 UTF-8 문자열들                                │
└────────────────────────────────────────────────────────────┘
```

### 5.4 Trie 빌드 및 로드 API

현재 구현된 API (`trie.rs` 기반):

```rust
/// Trie 빌드
pub fn build(entries: &[(&str, u32)]) -> Result<Vec<u8>> {
    // 정렬된 키-값 쌍에서 Trie 빌드
    DoubleArrayBuilder::build(&keyset)
}

/// Trie 로드 (메모리에서)
pub fn new(bytes: &'a [u8]) -> Self {
    Self {
        da: DoubleArray::new(Cow::Borrowed(bytes)),
    }
}

/// 파일에서 로드
pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Trie<'static>> {
    let bytes = std::fs::read(path)?;
    Ok(Self::from_vec(bytes))
}

/// 압축 파일에서 로드 (Zstd)
pub fn from_compressed_file<P: AsRef<Path>>(path: P) -> Result<Trie<'static>> {
    let file = std::fs::File::open(path)?;
    let mut decoder = zstd::Decoder::new(file)?;
    let mut bytes = Vec::new();
    decoder.read_to_end(&mut bytes)?;
    Ok(Self::from_vec(bytes))
}
```

### 5.5 Trie 검색 API

```rust
/// 정확히 일치하는 키 검색
pub fn exact_match(&self, key: &str) -> Option<u32>;

/// 공통 접두사 검색 (형태소 분석의 핵심)
pub fn common_prefix_search<'b>(&'b self, text: &'b str)
    -> impl Iterator<Item = (u32, usize)> + 'b;

/// 특정 위치에서 공통 접두사 검색
pub fn common_prefix_search_at(&self, text: &str, start_byte: usize)
    -> Vec<(u32, usize)>;
```

---

## 6. Connection Matrix 바이너리 포맷

### 6.1 개요

연접 비용 행렬은 형태소 간 연결 비용을 저장합니다. v3.0에서는 세 가지 저장 전략을 지원합니다:

1. **DenseMatrix**: 모든 값을 메모리에 저장 (기본)
2. **SparseMatrix**: 희소 행렬 최적화
3. **MmapMatrix**: 메모리 맵 기반 접근

### 6.2 Dense Matrix 바이너리 포맷

```
┌────────────────────────────────────────────────────────────┐
│                   matrix.bin 파일 구조                      │
├────────────────────────────────────────────────────────────┤
│ [File Header]           64 bytes (생략 가능)                │
├────────────────────────────────────────────────────────────┤
│ [Matrix Header]         4 bytes                            │
│ ├── lsize: u16          좌문맥 크기 (Little-Endian)         │
│ └── rsize: u16          우문맥 크기 (Little-Endian)         │
├────────────────────────────────────────────────────────────┤
│ [Cost Array]            lsize * rsize * 2 bytes            │
│ └── costs[i]: i16       연접 비용 (Little-Endian)           │
│     인덱스: right_id + lsize * left_id                      │
└────────────────────────────────────────────────────────────┘
```

### 6.3 비용 배열 접근

```rust
/// 비용 조회 (row-major order)
fn get(&self, right_id: u16, left_id: u16) -> i32 {
    let index = right_id as usize + self.lsize * left_id as usize;
    self.costs[index] as i32
}
```

### 6.4 현재 구현 (`matrix.rs`)

```rust
/// 밀집 연접 비용 행렬
pub struct DenseMatrix {
    lsize: usize,
    rsize: usize,
    costs: Vec<i16>,
}

/// 바이너리 직렬화
pub fn to_bin_bytes(&self) -> Vec<u8> {
    let mut buf = Vec::with_capacity(MATRIX_HEADER_SIZE + self.costs.len() * 2);
    buf.write_u16::<LittleEndian>(self.lsize as u16).ok();
    buf.write_u16::<LittleEndian>(self.rsize as u16).ok();
    for &cost in &self.costs {
        buf.write_i16::<LittleEndian>(cost).ok();
    }
    buf
}

/// 바이너리 역직렬화
pub fn from_bin_bytes(data: &[u8]) -> Result<Self> {
    let mut cursor = io::Cursor::new(data);
    let lsize = cursor.read_u16::<LittleEndian>()? as usize;
    let rsize = cursor.read_u16::<LittleEndian>()? as usize;
    // ... costs 읽기
}
```

### 6.5 메모리 맵 행렬

```rust
/// 메모리 맵 연접 비용 행렬
pub struct MmapMatrix {
    lsize: usize,
    rsize: usize,
    mmap: memmap2::Mmap,
}

impl MmapMatrix {
    /// 파일에서 메모리 맵으로 로드
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = std::fs::File::open(path)?;
        let mmap = unsafe { memmap2::Mmap::map(&file)? };
        // 헤더 읽기...
    }

    /// 비용 조회 (직접 mmap 접근)
    fn get(&self, right_id: u16, left_id: u16) -> i32 {
        let offset = MATRIX_HEADER_SIZE
            + (right_id as usize + self.lsize * left_id as usize) * 2;
        let bytes = [self.mmap[offset], self.mmap[offset + 1]];
        i16::from_le_bytes(bytes) as i32
    }
}
```

### 6.6 희소 행렬 (선택적 최적화)

희소 행렬은 대부분의 값이 기본값인 경우 메모리를 절약합니다:

```rust
pub struct SparseMatrix {
    lsize: usize,
    rsize: usize,
    default_cost: i16,
    entries: HashMap<usize, i16>,  // 비기본 값만 저장
}
```

**희소도 기준**: 기본값이 아닌 엔트리가 10% 미만일 때 권장

### 6.7 Matrix Trait

모든 행렬 구현이 따라야 하는 인터페이스:

```rust
pub trait Matrix {
    /// 연접 비용 조회
    fn get(&self, right_id: u16, left_id: u16) -> i32;

    /// 좌문맥 크기
    fn left_size(&self) -> usize;

    /// 우문맥 크기
    fn right_size(&self) -> usize;

    /// 전체 엔트리 수
    fn entry_count(&self) -> usize {
        self.left_size() * self.right_size()
    }
}
```

### 6.8 비용 값 범위

| 범위 | 의미 |
|------|------|
| i16::MIN (-32768) | 매우 자연스러운 연결 |
| 음수 | 자연스러운 연결 (선호) |
| 0 | 중립 |
| 양수 | 부자연스러운 연결 (비선호) |
| i16::MAX (32767) | 연결 불가능 |

---

## 7. 엔트리 데이터 포맷

### 7.1 사전 엔트리 구조

```rust
/// 사전 엔트리 (고정 크기 16바이트)
#[repr(C, packed)]
pub struct BinaryEntry {
    /// 좌문맥 ID
    pub left_id: u16,
    /// 우문맥 ID
    pub right_id: u16,
    /// 단어 비용
    pub cost: i16,
    /// 플래그 (종성 유무 등)
    pub flags: u16,
    /// 피처 문자열 오프셋
    pub feature_offset: u32,
    /// 피처 문자열 길이
    pub feature_length: u16,
    /// 예약
    pub reserved: u16,
}
```

### 7.2 플래그 비트맵

```
Bit 0: 종성 유무 (0: F, 1: T)
Bit 1-2: 타입 (00: 일반, 01: Compound, 10: Inflect, 11: 예약)
Bit 3-15: 예약
```

### 7.3 피처 문자열 포맷

```
품사,의미분류,종성,읽기,타입,첫품사,끝품사,분석결과
```

예시:
```
NNG,인사,T,안녕,*,*,*,*
VV+ETM,*,T,위한,Inflect,VV,ETM,위하/VV/*+ᆫ/ETM/*
```

---

## 8. 압축 지원

### 8.1 지원 압축 알고리즘

| 알고리즘 | 플래그 값 | 압축률 | 속도 | 권장 용도 |
|----------|-----------|--------|------|-----------|
| Zstd | 0 | 매우 높음 | 빠름 | 기본 권장 |
| 미압축 | - | 없음 | 최고 | 메모리 맵 |

### 8.2 Zstd 압축 설정

```rust
/// 압축 레벨 가이드라인
/// Level 1-3: 빠른 압축 (배포용)
/// Level 6-9: 균형 (일반용)
/// Level 19-22: 최대 압축 (아카이브용)

const DEFAULT_COMPRESSION_LEVEL: i32 = 3;

/// 압축 저장
pub fn save_to_compressed_file<P: AsRef<Path>>(
    bytes: &[u8],
    path: P,
    level: i32,
) -> Result<()> {
    let file = std::fs::File::create(path)?;
    let mut encoder = zstd::Encoder::new(file, level)?;
    encoder.write_all(bytes)?;
    encoder.finish()?;
    Ok(())
}
```

### 8.3 압축 파일 확장자 규칙

| 원본 파일 | 압축 파일 |
|-----------|-----------|
| `sys.dic` | `sys.dic.zst` |
| `matrix.bin` | `matrix.bin.zst` |
| `user.dic` | `user.dic.zst` |

### 8.4 로딩 전략

```
압축 파일 존재 확인
    ↓
[압축 파일 있음] → Zstd 압축 해제 → 메모리 로드
    ↓
[압축 파일 없음] → 원본 파일 로드
    ↓
[메모리 맵 요청] → Mmap 생성 (압축 미지원)
```

---

## 9. 메모리 매핑 지원

### 9.1 개요

메모리 매핑(mmap)은 대용량 사전을 효율적으로 로드하는 방법입니다. 파일을 가상 메모리에 매핑하여 실제 접근 시에만 물리 메모리를 사용합니다.

### 9.2 memmap2 라이브러리 사용

```rust
use memmap2::Mmap;

/// 메모리 맵 생성
pub fn create_mmap(path: &Path) -> Result<Mmap> {
    let file = std::fs::File::open(path)?;
    // SAFETY: 파일이 외부에서 수정되지 않는다고 가정
    let mmap = unsafe { Mmap::map(&file)? };
    Ok(mmap)
}
```

### 9.3 메모리 맵 최적화 파일 구조

메모리 맵을 효율적으로 사용하려면 다음 조건을 만족해야 합니다:

1. **정렬**: 데이터 섹션이 페이지 경계(4KB)에 정렬
2. **미압축**: 압축된 파일은 mmap 불가
3. **고정 크기**: 가변 길이 데이터는 별도 섹션에 분리

### 9.4 MmapMatrix 구현 예시

```rust
impl MmapMatrix {
    /// 비용 오프셋 계산
    #[inline]
    fn offset(&self, right_id: u16, left_id: u16) -> usize {
        MATRIX_HEADER_SIZE
            + (right_id as usize + self.lsize * left_id as usize) * 2
    }

    /// 비용 조회 (zero-copy)
    fn get(&self, right_id: u16, left_id: u16) -> i32 {
        let offset = self.offset(right_id, left_id);
        if offset + 2 <= self.mmap.len() {
            let bytes = [self.mmap[offset], self.mmap[offset + 1]];
            i16::from_le_bytes(bytes) as i32
        } else {
            INVALID_CONNECTION_COST
        }
    }
}
```

### 9.5 사용 권장 사항

| 시나리오 | 권장 로딩 방식 |
|----------|----------------|
| 서버 (다중 프로세스) | MmapMatrix (메모리 공유) |
| CLI (단일 실행) | DenseMatrix (전체 로드) |
| 임베디드 (메모리 제한) | MmapMatrix + 압축 해제 캐시 |
| WebAssembly | DenseMatrix (mmap 미지원) |

---

## 10. 버전 관리 메커니즘

### 10.1 Semantic Versioning

```
Major.Minor.Patch
  │     │     │
  │     │     └── 버그 수정 (하위 호환)
  │     └────── 기능 추가 (하위 호환)
  └──────────── 호환성 변경 (하위 비호환 가능)
```

### 10.2 호환성 규칙

| 버전 변경 | 읽기 가능 | 쓰기 가능 |
|-----------|-----------|-----------|
| Major 증가 | X (마이그레이션 필요) | X |
| Minor 증가 | O (기존 필드만) | O (새 필드 무시) |
| Patch 증가 | O | O |

### 10.3 버전 검증 코드

```rust
const CURRENT_VERSION: (u8, u8, u8) = (3, 0, 0);

pub fn validate_version(header: &FileHeader) -> Result<()> {
    let file_version = (
        header.version_major,
        header.version_minor,
        header.version_patch,
    );

    // Major 버전 불일치 시 에러
    if file_version.0 != CURRENT_VERSION.0 {
        return Err(DictError::Version {
            expected: format!("{}.x.x", CURRENT_VERSION.0),
            found: format!("{}.{}.{}",
                file_version.0, file_version.1, file_version.2),
        });
    }

    // Minor 버전이 더 높으면 경고
    if file_version.1 > CURRENT_VERSION.1 {
        log::warn!("Dictionary version is newer than library");
    }

    Ok(())
}
```

### 10.4 마이그레이션 도구

버전 업그레이드가 필요한 경우를 위한 CLI 도구:

```bash
# v2.x → v3.0 마이그레이션
mecab-ko-dict migrate --input old_dict/ --output new_dict/ --target-version 3.0
```

---

## 11. 사용자 사전 포맷

### 11.1 CSV 포맷 (입력)

```csv
# 사용자 정의 사전
# 표면형,품사,비용,읽기
형태소분석,NNG,-1000,형태소분석
딥러닝,NNG,-500,
챗GPT,NNP,-1000,챗지피티
```

### 11.2 바이너리 포맷 (컴파일 후)

사용자 사전도 시스템 사전과 동일한 구조를 사용합니다:

```
┌────────────────────────────────────────┐
│ [File Header]    file_type = USER_DICT │
├────────────────────────────────────────┤
│ [Trie Data]                            │
├────────────────────────────────────────┤
│ [Entry Array]                          │
├────────────────────────────────────────┤
│ [Feature Strings]                      │
└────────────────────────────────────────┘
```

### 11.3 UserDictionary 구현 (`user_dict.rs`)

```rust
/// 사용자 사전 엔트리
pub struct UserEntry {
    pub surface: String,
    pub left_id: u16,
    pub right_id: u16,
    pub cost: i16,
    pub pos: String,
    pub reading: Option<String>,
    pub lemma: Option<String>,
}

/// 사용자 사전
pub struct UserDictionary {
    entries: Vec<UserEntry>,
    surface_map: HashMap<String, Vec<usize>>,
    trie_cache: Option<Vec<u8>>,
    default_cost: i16,  // 기본값: -1000
}

impl UserDictionary {
    /// CSV에서 로드
    pub fn load_from_csv<P: AsRef<Path>>(&mut self, path: P) -> Result<&mut Self>;

    /// Trie 빌드
    pub fn build_trie(&mut self) -> Result<&[u8]>;

    /// 검색
    pub fn lookup(&self, surface: &str) -> Vec<&UserEntry>;
}
```

### 11.4 사용자 사전 우선순위

```
검색 순서:
1. 사용자 사전 (낮은 비용 우선)
2. 시스템 사전

비용 가이드라인:
- -2000 ~ -1000: 최우선 (신조어, 고유명사)
- -999 ~ -500: 높은 우선순위
- -499 ~ 0: 일반 우선순위
- 양수: 낮은 우선순위
```

---

## 12. 구현 참조

### 12.1 의존성 (Cargo.toml)

```toml
[dependencies]
yada = "0.5"           # Double-Array Trie
byteorder = "1.5"      # 바이트 순서 처리
memmap2 = "0.9"        # 메모리 매핑
zstd = "0.13"          # 압축
```

### 12.2 전체 로딩 예시

```rust
use mecab_ko_dict::{
    Trie, DenseMatrix, MmapMatrix, MatrixLoader, ConnectionMatrix
};

// 시스템 사전 로드
let trie = if use_mmap {
    Trie::from_file("dict/sys.dic")?
} else if compressed {
    Trie::from_compressed_file("dict/sys.dic.zst")?
} else {
    Trie::from_file("dict/sys.dic")?
};

// 연접 비용 행렬 로드
let matrix: ConnectionMatrix = if use_mmap {
    ConnectionMatrix::from_mmap_file("dict/matrix.bin")?
} else {
    ConnectionMatrix::load("dict/matrix.bin")?  // 자동 포맷 감지
};

// 비용 조회
let cost = matrix.get(right_id, left_id);
```

### 12.3 파일 목록 및 크기 예상

| 파일 | 미압축 크기 | Zstd 압축 | 압축률 |
|------|-------------|-----------|--------|
| sys.dic | ~50 MB | ~25 MB | 50% |
| matrix.bin | ~6 MB | ~2 MB | 67% |
| char.bin | ~130 KB | N/A | - |
| unk.bin | ~10 KB | N/A | - |
| **합계** | ~56 MB | ~27 MB | 52% |

---

## 부록

### A. 체크섬 계산

```rust
use crc32fast::Hasher;

pub fn calculate_checksum(data: &[u8]) -> u32 {
    let mut hasher = Hasher::new();
    hasher.update(data);
    hasher.finalize()
}
```

### B. 에러 타입

```rust
/// 사전 에러 타입
pub enum DictError {
    /// IO 에러
    Io(std::io::Error),

    /// 포맷 에러
    Format(String),

    /// 버전 불일치
    Version { expected: String, found: String },

    /// 체크섬 불일치
    Checksum { expected: u32, found: u32 },
}
```

### C. 향후 확장 계획

1. **v3.1**: 증분 업데이트 지원
2. **v3.2**: 원격 사전 로딩 (HTTP)
3. **v4.0**: FST(Finite State Transducer) 기반 Trie

---

## 참고 자료

- [yada 라이브러리](https://crates.io/crates/yada) - Double-Array Trie
- [memmap2](https://crates.io/crates/memmap2) - 메모리 매핑
- [zstd](https://crates.io/crates/zstd) - Zstandard 압축
- [MeCab-Ko-Dic 포맷 v2](./dictionary-format-v2.md) - 기존 포맷 분석
- [Lindera 분석](./lindera-analysis.md) - 참조 구현 분석

---

**문서 버전**: 3.0
**최종 수정**: 2026-01-05
