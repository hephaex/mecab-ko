//! # Lazy Entry Loading
//!
//! 메모리 효율을 위한 지연 로딩 엔트리 시스템입니다.
//! 엔트리를 필요할 때만 디스크에서 읽어옵니다.
//!
//! ## entries.bin v2 포맷
//!
//! ```text
//! [Header]
//!   magic: [u8; 4] = "MKE2"
//!   version: u32 = 2
//!   count: u32
//!   index_offset: u64 (인덱스 테이블 시작 위치)
//!
//! [Entry Data] (압축 가능)
//!   entry_0: [left_id:u16][right_id:u16][cost:i16][surface_len:u16][feature_len:u16][surface][feature]
//!   entry_1: ...
//!   ...
//!
//! [Index Table]
//!   offset_0: u64 (entry_0의 파일 내 위치)
//!   offset_1: u64
//!   ...
//! ```

use std::collections::HashMap;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use memmap2::Mmap;

use crate::dictionary::DictEntry;
use crate::error::{DictError, Result};

/// entries.bin v2 매직 넘버
const LAZY_ENTRIES_MAGIC: &[u8; 4] = b"MKE2";
/// entries.bin v2 버전
const LAZY_ENTRIES_VERSION: u32 = 2;
/// 헤더 크기 바이트
const HEADER_SIZE: usize = 20;

/// LRU 캐시 기본 크기 (핫스팟 엔트리 캐싱)
const DEFAULT_CACHE_SIZE: usize = 10000;

/// 지연 로딩 엔트리 저장소
///
/// 엔트리를 필요할 때만 디스크에서 읽어옵니다.
/// 자주 사용되는 엔트리는 LRU 캐시에 저장됩니다.
pub struct LazyEntries {
    /// 파일 경로 (디버깅용)
    #[allow(dead_code)]
    path: PathBuf,
    /// 메모리 맵 (읽기 전용)
    mmap: Mmap,
    /// 엔트리 수
    count: u32,
    /// 인덱스 테이블 오프셋
    index_offset: u64,
    /// LRU 캐시 (인덱스 -> 엔트리)
    cache: RwLock<LruCache>,
}

/// 간단한 LRU 캐시 구현
struct LruCache {
    /// 캐시 맵
    entries: HashMap<u32, Arc<DictEntry>>,
    /// 최대 크기
    max_size: usize,
    /// 접근 순서 (가장 최근 접근이 뒤로)
    access_order: Vec<u32>,
}

impl LruCache {
    fn new(max_size: usize) -> Self {
        Self {
            entries: HashMap::with_capacity(max_size),
            max_size,
            access_order: Vec::with_capacity(max_size),
        }
    }

    fn get(&mut self, index: u32) -> Option<Arc<DictEntry>> {
        if let Some(entry) = self.entries.get(&index) {
            // 접근 순서 업데이트
            self.access_order.retain(|&i| i != index);
            self.access_order.push(index);
            Some(Arc::clone(entry))
        } else {
            None
        }
    }

    fn insert(&mut self, index: u32, entry: DictEntry) -> Arc<DictEntry> {
        // 캐시가 가득 찼으면 가장 오래된 항목 제거
        if self.entries.len() >= self.max_size && !self.access_order.is_empty() {
            let oldest = self.access_order.remove(0);
            self.entries.remove(&oldest);
        }

        let arc_entry = Arc::new(entry);
        self.entries.insert(index, Arc::clone(&arc_entry));
        self.access_order.push(index);
        arc_entry
    }

    fn len(&self) -> usize {
        self.entries.len()
    }

    fn clear(&mut self) {
        self.entries.clear();
        self.access_order.clear();
    }
}

impl LazyEntries {
    /// entries.bin v2 파일에서 로드
    ///
    /// # Errors
    ///
    /// - 파일 형식이 잘못된 경우
    /// - 파일 읽기 실패한 경우
    ///
    /// # Safety
    ///
    /// Uses memory-mapped I/O which requires unsafe.
    /// The safety is ensured by memmap2 crate's implementation.
    #[allow(unsafe_code)]
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let file = std::fs::File::open(&path).map_err(DictError::Io)?;
        // SAFETY: memmap2::Mmap handles the memory mapping safely.
        // The file is opened read-only and the mmap is immutable.
        let mmap = unsafe { Mmap::map(&file).map_err(DictError::Io)? };

        // 헤더 검증
        if mmap.len() < HEADER_SIZE {
            return Err(DictError::Format("entries.bin v2: file too small".into()));
        }

        let mut cursor = std::io::Cursor::new(&mmap[..]);

        // 매직 넘버 검증
        let mut magic = [0u8; 4];
        cursor.read_exact(&mut magic).map_err(|e| {
            DictError::Format(format!("entries.bin v2: failed to read magic: {e}"))
        })?;
        if &magic != LAZY_ENTRIES_MAGIC {
            return Err(DictError::Format(
                "entries.bin v2: invalid magic number (expected MKE2)".into(),
            ));
        }

        // 버전 검증
        let version = cursor.read_u32::<LittleEndian>().map_err(|e| {
            DictError::Format(format!("entries.bin v2: failed to read version: {e}"))
        })?;
        if version != LAZY_ENTRIES_VERSION {
            return Err(DictError::Format(format!(
                "entries.bin v2: unsupported version {version}"
            )));
        }

        // 엔트리 수
        let count = cursor.read_u32::<LittleEndian>().map_err(|e| {
            DictError::Format(format!("entries.bin v2: failed to read count: {e}"))
        })?;

        // 인덱스 테이블 오프셋
        let index_offset = cursor.read_u64::<LittleEndian>().map_err(|e| {
            DictError::Format(format!("entries.bin v2: failed to read index_offset: {e}"))
        })?;

        Ok(Self {
            path,
            mmap,
            count,
            index_offset,
            cache: RwLock::new(LruCache::new(DEFAULT_CACHE_SIZE)),
        })
    }

    /// 엔트리 수 반환
    #[must_use]
    pub const fn len(&self) -> usize {
        self.count as usize
    }

    /// 비어있는지 확인
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// 캐시된 엔트리 수 반환
    #[must_use]
    pub fn cached_count(&self) -> usize {
        self.cache.read().map(|c| c.len()).unwrap_or(0)
    }

    /// 캐시 크기 설정
    pub fn set_cache_size(&self, size: usize) {
        if let Ok(mut cache) = self.cache.write() {
            cache.max_size = size;
        }
    }

    /// 캐시 초기화
    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.cache.write() {
            cache.clear();
        }
    }

    /// 인덱스로 엔트리 오프셋 조회
    fn get_entry_offset(&self, index: u32) -> Result<u64> {
        if index >= self.count {
            return Err(DictError::Format(format!(
                "entry index out of bounds: {index} >= {}",
                self.count
            )));
        }

        // 인덱스 테이블에서 오프셋 읽기
        let index_pos = self.index_offset + (u64::from(index) * 8);
        let mmap_len = u64::try_from(self.mmap.len())
            .map_err(|_| DictError::Format("mmap length overflow".into()))?;
        if index_pos + 8 > mmap_len {
            return Err(DictError::Format(format!(
                "index table overflow at position {index_pos}"
            )));
        }

        let pos = usize::try_from(index_pos)
            .map_err(|_| DictError::Format("index position overflow".into()))?;
        let mut cursor = std::io::Cursor::new(&self.mmap[pos..]);
        let offset = cursor
            .read_u64::<LittleEndian>()
            .map_err(|e| DictError::Format(format!("failed to read entry offset: {e}")))?;

        Ok(offset)
    }

    /// 인덱스로 엔트리 로드
    ///
    /// 캐시에 있으면 캐시에서 반환, 없으면 디스크에서 읽어 캐시에 저장
    ///
    /// # Errors
    ///
    /// - 인덱스가 범위를 벗어난 경우
    /// - 엔트리 읽기 실패한 경우
    pub fn get(&self, index: u32) -> Result<Arc<DictEntry>> {
        // 1. 캐시 확인
        {
            let mut cache = self
                .cache
                .write()
                .map_err(|_| DictError::Format("cache lock poisoned".into()))?;
            if let Some(entry) = cache.get(index) {
                return Ok(entry);
            }
        }

        // 2. 디스크에서 읽기
        let entry = self.load_entry_from_disk(index)?;

        // 3. 캐시에 저장
        let mut cache = self
            .cache
            .write()
            .map_err(|_| DictError::Format("cache lock poisoned".into()))?;
        Ok(cache.insert(index, entry))
    }

    /// 디스크에서 엔트리 로드
    fn load_entry_from_disk(&self, index: u32) -> Result<DictEntry> {
        let offset = self.get_entry_offset(index)?;

        let offset_usize = usize::try_from(offset)
            .map_err(|_| DictError::Format("offset overflow".into()))?;

        if offset_usize >= self.mmap.len() {
            return Err(DictError::Format(format!(
                "entry offset out of bounds: {offset}"
            )));
        }

        let mut cursor = std::io::Cursor::new(&self.mmap[offset_usize..]);

        let left_id = cursor
            .read_u16::<LittleEndian>()
            .map_err(|e| DictError::Format(format!("entry {index} left_id: {e}")))?;
        let right_id = cursor
            .read_u16::<LittleEndian>()
            .map_err(|e| DictError::Format(format!("entry {index} right_id: {e}")))?;
        let cost = cursor
            .read_i16::<LittleEndian>()
            .map_err(|e| DictError::Format(format!("entry {index} cost: {e}")))?;
        let surface_len = cursor
            .read_u16::<LittleEndian>()
            .map_err(|e| DictError::Format(format!("entry {index} surface_len: {e}")))?
            as usize;
        let feature_len = cursor
            .read_u16::<LittleEndian>()
            .map_err(|e| DictError::Format(format!("entry {index} feature_len: {e}")))?
            as usize;

        let mut surface_bytes = vec![0u8; surface_len];
        cursor
            .read_exact(&mut surface_bytes)
            .map_err(|e| DictError::Format(format!("entry {index} surface: {e}")))?;
        let surface = String::from_utf8(surface_bytes)
            .map_err(|e| DictError::Format(format!("entry {index} surface utf8: {e}")))?;

        let mut feature_bytes = vec![0u8; feature_len];
        cursor
            .read_exact(&mut feature_bytes)
            .map_err(|e| DictError::Format(format!("entry {index} feature: {e}")))?;
        let feature = String::from_utf8(feature_bytes)
            .map_err(|e| DictError::Format(format!("entry {index} feature utf8: {e}")))?;

        Ok(DictEntry {
            surface,
            left_id,
            right_id,
            cost,
            feature,
        })
    }

    /// 인덱스에서 시작하여 같은 surface를 가진 연속된 엔트리 반환
    ///
    /// # Errors
    ///
    /// - 인덱스가 범위를 벗어난 경우
    /// - 엔트리 읽기 실패한 경우
    pub fn get_entries_at(&self, first_index: u32, surface: &str) -> Result<Vec<Arc<DictEntry>>> {
        let mut results = Vec::new();
        let mut index = first_index;

        while index < self.count {
            let entry = self.get(index)?;
            if entry.surface == surface {
                results.push(entry);
                index += 1;
            } else {
                break;
            }
        }

        Ok(results)
    }

    /// 모든 엔트리를 Vec으로 로드 (마이그레이션/변환용)
    ///
    /// 주의: 대용량 사전에서는 메모리를 많이 사용합니다.
    ///
    /// # Errors
    ///
    /// - 엔트리 읽기 실패한 경우
    pub fn load_all(&self) -> Result<Vec<DictEntry>> {
        let mut entries = Vec::with_capacity(self.count as usize);
        for i in 0..self.count {
            let entry = self.load_entry_from_disk(i)?;
            entries.push(entry);
        }
        Ok(entries)
    }

    /// `DictEntry` 벡터를 entries.bin v2로 저장
    ///
    /// # Errors
    ///
    /// - 파일 쓰기 실패 시 에러
    pub fn save_entries<P: AsRef<Path>>(entries: &[DictEntry], path: P) -> Result<()> {
        use std::io::Write;

        let path = path.as_ref();
        let mut file = std::fs::File::create(path).map_err(DictError::Io)?;

        let count = u32::try_from(entries.len())
            .map_err(|_| DictError::Format("too many entries".into()))?;

        // 1. 임시 헤더 작성 (index_offset은 나중에 업데이트)
        file.write_all(LAZY_ENTRIES_MAGIC).map_err(DictError::Io)?;
        file.write_u32::<LittleEndian>(LAZY_ENTRIES_VERSION)
            .map_err(DictError::Io)?;
        file.write_u32::<LittleEndian>(count)
            .map_err(DictError::Io)?;
        file.write_u64::<LittleEndian>(0) // placeholder
            .map_err(DictError::Io)?;

        // 2. 엔트리 데이터 작성, 오프셋 기록
        let mut offsets = Vec::with_capacity(entries.len());

        for entry in entries {
            let offset = file.stream_position().map_err(DictError::Io)?;
            offsets.push(offset);

            file.write_u16::<LittleEndian>(entry.left_id)
                .map_err(DictError::Io)?;
            file.write_u16::<LittleEndian>(entry.right_id)
                .map_err(DictError::Io)?;
            file.write_i16::<LittleEndian>(entry.cost)
                .map_err(DictError::Io)?;

            let surface_bytes = entry.surface.as_bytes();
            let surface_len = u16::try_from(surface_bytes.len())
                .map_err(|_| DictError::Format("surface too long".into()))?;
            file.write_u16::<LittleEndian>(surface_len)
                .map_err(DictError::Io)?;

            let feature_bytes = entry.feature.as_bytes();
            let feature_len = u16::try_from(feature_bytes.len())
                .map_err(|_| DictError::Format("feature too long".into()))?;
            file.write_u16::<LittleEndian>(feature_len)
                .map_err(DictError::Io)?;

            file.write_all(surface_bytes).map_err(DictError::Io)?;
            file.write_all(feature_bytes).map_err(DictError::Io)?;
        }

        // 3. 인덱스 테이블 작성
        let index_offset = file.stream_position().map_err(DictError::Io)?;

        for offset in offsets {
            file.write_u64::<LittleEndian>(offset)
                .map_err(DictError::Io)?;
        }

        // 4. 헤더의 index_offset 업데이트
        file.seek(SeekFrom::Start(12)).map_err(DictError::Io)?;
        file.write_u64::<LittleEndian>(index_offset)
            .map_err(DictError::Io)?;

        Ok(())
    }
}

/// entries.bin v1을 v2로 마이그레이션
///
/// v1 파일을 읽어서 v2 포맷으로 저장합니다.
/// v1 포맷: `[magic:4][version:u32][count:u32][entries...]`
///
/// # Errors
///
/// - 파일 읽기 실패 시 에러
/// - 파일 쓰기 실패 시 에러
pub fn migrate_entries_v1_to_v2<P: AsRef<Path>>(v1_path: P, v2_path: P) -> Result<()> {
    use byteorder::ReadBytesExt;

    let data = std::fs::read(v1_path.as_ref()).map_err(DictError::Io)?;
    let mut cursor = std::io::Cursor::new(&data);

    // v1 매직 넘버 검증 (MKED)
    let mut magic = [0u8; 4];
    cursor
        .read_exact(&mut magic)
        .map_err(|e| DictError::Format(format!("v1 magic: {e}")))?;
    if &magic != b"MKED" {
        return Err(DictError::Format(
            "entries.bin v1: invalid magic number".into(),
        ));
    }

    // v1 버전 검증
    let version = cursor
        .read_u32::<LittleEndian>()
        .map_err(|e| DictError::Format(format!("v1 version: {e}")))?;
    if version != 1 {
        return Err(DictError::Format(format!(
            "entries.bin v1: unsupported version {version}"
        )));
    }

    // 엔트리 수
    let count = cursor
        .read_u32::<LittleEndian>()
        .map_err(|e| DictError::Format(format!("v1 count: {e}")))?;

    let mut entries = Vec::with_capacity(count as usize);
    for i in 0..count {
        let left_id = cursor
            .read_u16::<LittleEndian>()
            .map_err(|e| DictError::Format(format!("v1 entry {i} left_id: {e}")))?;
        let right_id = cursor
            .read_u16::<LittleEndian>()
            .map_err(|e| DictError::Format(format!("v1 entry {i} right_id: {e}")))?;
        let cost = cursor
            .read_i16::<LittleEndian>()
            .map_err(|e| DictError::Format(format!("v1 entry {i} cost: {e}")))?;
        let surface_len = cursor
            .read_u16::<LittleEndian>()
            .map_err(|e| DictError::Format(format!("v1 entry {i} surface_len: {e}")))?
            as usize;
        let feature_len = cursor
            .read_u16::<LittleEndian>()
            .map_err(|e| DictError::Format(format!("v1 entry {i} feature_len: {e}")))?
            as usize;

        let mut surface_bytes = vec![0u8; surface_len];
        cursor
            .read_exact(&mut surface_bytes)
            .map_err(|e| DictError::Format(format!("v1 entry {i} surface: {e}")))?;
        let surface = String::from_utf8(surface_bytes)
            .map_err(|e| DictError::Format(format!("v1 entry {i} surface utf8: {e}")))?;

        let mut feature_bytes = vec![0u8; feature_len];
        cursor
            .read_exact(&mut feature_bytes)
            .map_err(|e| DictError::Format(format!("v1 entry {i} feature: {e}")))?;
        let feature = String::from_utf8(feature_bytes)
            .map_err(|e| DictError::Format(format!("v1 entry {i} feature utf8: {e}")))?;

        entries.push(DictEntry {
            surface,
            left_id,
            right_id,
            cost,
            feature,
        });
    }

    // v2로 저장
    LazyEntries::save_entries(&entries, v2_path)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_lazy_entries_roundtrip() {
        let entries = vec![
            DictEntry::new("안녕", 1, 1, 100, "NNG,*,T,안녕,*,*,*,*"),
            DictEntry::new("하세요", 2, 2, 50, "VV,*,F,하세요,*,*,*,*"),
            DictEntry::new("감사", 3, 3, 80, "NNG,*,F,감사,*,*,*,*"),
        ];

        let dir = tempdir().expect("create temp dir");
        let path = dir.path().join("entries.bin");

        // 저장
        LazyEntries::save_entries(&entries, &path).expect("save should work");

        // 로드
        let lazy = LazyEntries::from_file(&path).expect("load should work");

        assert_eq!(lazy.len(), 3);
        assert!(!lazy.is_empty());

        // 엔트리 조회
        let e0 = lazy.get(0).expect("get entry 0");
        assert_eq!(e0.surface, "안녕");
        assert_eq!(e0.left_id, 1);
        assert_eq!(e0.cost, 100);

        let e1 = lazy.get(1).expect("get entry 1");
        assert_eq!(e1.surface, "하세요");

        let e2 = lazy.get(2).expect("get entry 2");
        assert_eq!(e2.surface, "감사");

        // 범위 밖 인덱스
        assert!(lazy.get(100).is_err());
    }

    #[test]
    fn test_lazy_entries_cache() {
        let entries = vec![
            DictEntry::new("가", 1, 1, 100, "NNG"),
            DictEntry::new("나", 2, 2, 200, "NNG"),
        ];

        let dir = tempdir().expect("create temp dir");
        let path = dir.path().join("entries.bin");
        LazyEntries::save_entries(&entries, &path).expect("save");

        let lazy = LazyEntries::from_file(&path).expect("load");

        // 첫 조회 (캐시 미스)
        assert_eq!(lazy.cached_count(), 0);
        let _ = lazy.get(0).expect("get 0");
        assert_eq!(lazy.cached_count(), 1);

        // 두 번째 조회 (캐시 히트)
        let _ = lazy.get(0).expect("get 0 again");
        assert_eq!(lazy.cached_count(), 1);

        // 다른 엔트리 조회
        let _ = lazy.get(1).expect("get 1");
        assert_eq!(lazy.cached_count(), 2);

        // 캐시 초기화
        lazy.clear_cache();
        assert_eq!(lazy.cached_count(), 0);
    }

    #[test]
    fn test_get_entries_at() {
        let entries = vec![
            DictEntry::new("가", 1, 1, 100, "VV"),
            DictEntry::new("가", 2, 2, 50, "JKS"),
            DictEntry::new("나", 3, 3, 200, "NP"),
        ];

        let dir = tempdir().expect("create temp dir");
        let path = dir.path().join("entries.bin");
        LazyEntries::save_entries(&entries, &path).expect("save");

        let lazy = LazyEntries::from_file(&path).expect("load");

        // "가"로 시작하는 연속 엔트리
        let results = lazy.get_entries_at(0, "가").expect("get_entries_at");
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].feature, "VV");
        assert_eq!(results[1].feature, "JKS");

        // "나"
        let results = lazy.get_entries_at(2, "나").expect("get_entries_at");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_load_all() {
        let entries = vec![
            DictEntry::new("가", 1, 1, 100, "NNG"),
            DictEntry::new("나", 2, 2, 200, "NNG"),
            DictEntry::new("다", 3, 3, 300, "NNG"),
        ];

        let dir = tempdir().expect("create temp dir");
        let path = dir.path().join("entries.bin");
        LazyEntries::save_entries(&entries, &path).expect("save");

        let lazy = LazyEntries::from_file(&path).expect("load");
        let loaded = lazy.load_all().expect("load_all");

        assert_eq!(loaded.len(), 3);
        assert_eq!(loaded[0].surface, "가");
        assert_eq!(loaded[1].surface, "나");
        assert_eq!(loaded[2].surface, "다");
    }

    #[test]
    fn test_lru_cache_eviction() {
        let mut cache = LruCache::new(2);

        let e1 = DictEntry::new("가", 1, 1, 100, "");
        let e2 = DictEntry::new("나", 2, 2, 200, "");
        let e3 = DictEntry::new("다", 3, 3, 300, "");

        cache.insert(0, e1);
        cache.insert(1, e2);
        assert_eq!(cache.len(), 2);

        // 새 항목 추가 시 가장 오래된 것(0) 제거
        cache.insert(2, e3);
        assert_eq!(cache.len(), 2);
        assert!(cache.get(0).is_none()); // 제거됨
        assert!(cache.get(1).is_some());
        assert!(cache.get(2).is_some());
    }
}
