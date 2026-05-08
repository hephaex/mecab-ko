//! # Lazy Entry Loading — entries.bin v3 (MKE3)
//!
//! v3 upgrades `feature_len` from `u16` to `u32`, removing the 65535-byte
//! feature-string limit present in v2.
//!
//! ## Format
//!
//! ```text
//! [Header — 24 bytes]
//!   magic:        [u8; 4]  = "MKE3"
//!   version:      u32 (LE) = 3
//!   count:        u32 (LE)
//!   flags:        u16 (LE)  bit 0 = FEATURE_U32 (always set)
//!   reserved:     u16 (LE) = 0
//!   index_offset: u64 (LE)
//!
//! [Entry Records — variable length]
//!   left_id:      u16 (LE)
//!   right_id:     u16 (LE)
//!   cost:         i16 (LE)
//!   surface_len:  u16 (LE)
//!   feature_len:  u32 (LE)   ← upgraded from u16
//!   surface:      [u8; surface_len]
//!   feature:      [u8; feature_len]
//!
//! [Index Table — count × 8 bytes]
//!   offset_N: u64 (LE)
//! ```

use std::io::{Read, Seek, SeekFrom};
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use memmap2::Mmap;

use crate::dictionary::DictEntry;
use crate::error::{DictError, Result};

/// Magic bytes identifying an MKE3 file.
pub const ENTRIES_V3_MAGIC: &[u8; 4] = b"MKE3";
/// Format version stored in the header.
pub const ENTRIES_V3_VERSION: u32 = 3;
/// Size of the MKE3 file header in bytes.
pub const HEADER_V3_SIZE: usize = 24;

/// Header flag bit: `feature_len` field is `u32`.  Always set in v3.
pub const FEATURE_U32: u16 = 1;

const DEFAULT_CACHE_SIZE: usize = 10_000;

// SAFETY: DEFAULT_CACHE_SIZE = 10_000 > 0.
const DEFAULT_CACHE_SIZE_NZ: NonZeroUsize = {
    match NonZeroUsize::new(DEFAULT_CACHE_SIZE) {
        Some(n) => n,
        None => panic!("DEFAULT_CACHE_SIZE must be > 0"),
    }
};

/// Detected entries.bin format version.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntriesFormat {
    /// `MKED` — legacy v1
    V1,
    /// `MKE2` — v2 with u16 `feature_len`
    V2,
    /// `MKE3` — v3 with u32 `feature_len`
    V3,
}

/// Read the first 4 bytes of a file and return the format.
///
/// # Errors
///
/// Returns `DictError::Io` if the file cannot be read, or
/// `DictError::Format` if the magic bytes are unrecognised.
pub fn detect_entries_format<P: AsRef<Path>>(path: P) -> Result<EntriesFormat> {
    use std::io::Read as _;
    let mut file = std::fs::File::open(path.as_ref()).map_err(DictError::Io)?;
    let mut magic = [0u8; 4];
    file.read_exact(&mut magic)
        .map_err(|e| DictError::Format(format!("cannot read magic: {e}")))?;
    match &magic {
        b"MKE3" => Ok(EntriesFormat::V3),
        b"MKE2" => Ok(EntriesFormat::V2),
        b"MKED" => Ok(EntriesFormat::V1),
        _ => Err(DictError::Format(format!(
            "unknown magic bytes: {magic:?}"
        ))),
    }
}

/// Lazy-loading entry store backed by an MKE3 memory-mapped file.
pub struct LazyEntriesV3 {
    #[allow(dead_code)]
    path: PathBuf,
    mmap: Mmap,
    count: u32,
    index_offset: u64,
    flags: u16,
    cache: RwLock<lru::LruCache<u32, Arc<DictEntry>>>,
}

impl LazyEntriesV3 {
    /// Open an MKE3 file and memory-map it.
    ///
    /// # Errors
    ///
    /// Returns `DictError::Format` for invalid headers and
    /// `DictError::Io` for I/O failures.
    #[allow(unsafe_code)]
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let file = std::fs::File::open(&path).map_err(DictError::Io)?;
        // SAFETY: The file is opened read-only; the mmap is immutable for
        // the lifetime of this struct, and no writes occur through it.
        let mmap = unsafe { Mmap::map(&file).map_err(DictError::Io)? };

        if mmap.len() < HEADER_V3_SIZE {
            return Err(DictError::Format("MKE3: file too small".into()));
        }

        let mut cur = std::io::Cursor::new(&mmap[..]);

        let mut magic = [0u8; 4];
        cur.read_exact(&mut magic)
            .map_err(|e| DictError::Format(format!("MKE3: cannot read magic: {e}")))?;
        if &magic != ENTRIES_V3_MAGIC {
            return Err(DictError::Format(
                "MKE3: invalid magic (expected MKE3)".into(),
            ));
        }

        let version = cur
            .read_u32::<LittleEndian>()
            .map_err(|e| DictError::Format(format!("MKE3: cannot read version: {e}")))?;
        if version != ENTRIES_V3_VERSION {
            return Err(DictError::Format(format!(
                "MKE3: unsupported version {version}"
            )));
        }

        let count = cur
            .read_u32::<LittleEndian>()
            .map_err(|e| DictError::Format(format!("MKE3: cannot read count: {e}")))?;

        let flags = cur
            .read_u16::<LittleEndian>()
            .map_err(|e| DictError::Format(format!("MKE3: cannot read flags: {e}")))?;

        // reserved u16 — skip
        cur.read_u16::<LittleEndian>()
            .map_err(|e| DictError::Format(format!("MKE3: cannot read reserved: {e}")))?;

        let index_offset = cur
            .read_u64::<LittleEndian>()
            .map_err(|e| DictError::Format(format!("MKE3: cannot read index_offset: {e}")))?;

        let expected_index_end = index_offset + u64::from(count) * 8;
        if expected_index_end > mmap.len() as u64 {
            return Err(DictError::Format(format!(
                "MKE3: index table extends beyond file (offset={index_offset}, count={count}, file_len={})",
                mmap.len()
            )));
        }

        Ok(Self {
            path,
            mmap,
            count,
            index_offset,
            flags,
            cache: RwLock::new(lru::LruCache::new(DEFAULT_CACHE_SIZE_NZ)),
        })
    }

    /// Total number of entries in the file.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.count as usize
    }

    /// Returns `true` if there are no entries.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Number of entries currently held in the LRU cache.
    #[must_use]
    pub fn cached_count(&self) -> usize {
        self.cache.read().map(|c| c.len()).unwrap_or(0)
    }

    /// Resize the LRU cache (minimum 1).
    pub fn set_cache_size(&self, size: usize) {
        if let Ok(mut cache) = self.cache.write() {
            cache.resize(NonZeroUsize::new(size).unwrap_or(NonZeroUsize::new(1).unwrap()));
        }
    }

    /// Evict all cached entries.
    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.cache.write() {
            cache.clear();
        }
    }

    /// Flags field from the header.
    #[must_use]
    pub const fn flags(&self) -> u16 {
        self.flags
    }

    /// Look up entry `index`, returning a shared `Arc`.
    ///
    /// Cache check uses `peek` (no LRU promotion) to avoid a write lock on
    /// every hit; the write lock is taken only on a miss.
    ///
    /// # Errors
    ///
    /// Returns `DictError::Format` when `index` is out of bounds or the
    /// on-disk record is corrupt.
    pub fn get(&self, index: u32) -> Result<Arc<DictEntry>> {
        {
            let cache = self
                .cache
                .read()
                .map_err(|_| DictError::Format("MKE3: cache lock poisoned".into()))?;
            if let Some(entry) = cache.peek(&index) {
                return Ok(Arc::clone(entry));
            }
        }

        let entry = Arc::new(self.load_entry_from_mmap(index)?);
        {
            let mut cache = self
                .cache
                .write()
                .map_err(|_| DictError::Format("MKE3: cache lock poisoned".into()))?;
            if let Some(existing) = cache.get(&index) {
                return Ok(Arc::clone(existing));
            }
            cache.put(index, Arc::clone(&entry));
        }
        Ok(entry)
    }

    /// Collect all consecutive entries starting at `first_index` that share `surface`.
    ///
    /// Iterates forward from `first_index` until either the end of the file or
    /// an entry whose surface differs from `surface`.
    ///
    /// # Errors
    ///
    /// Returns `DictError::Format` if any individual entry cannot be loaded.
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

    fn entry_offset(&self, index: u32) -> Result<u64> {
        if index >= self.count {
            return Err(DictError::Format(format!(
                "MKE3: index {index} out of bounds (count={})",
                self.count
            )));
        }
        let table_pos = self.index_offset + u64::from(index) * 8;
        let mmap_len = u64::try_from(self.mmap.len())
            .map_err(|_| DictError::Format("MKE3: mmap length overflow".into()))?;
        if table_pos + 8 > mmap_len {
            return Err(DictError::Format(format!(
                "MKE3: index table overflow at position {table_pos}"
            )));
        }
        let pos = usize::try_from(table_pos)
            .map_err(|_| DictError::Format("MKE3: table position overflow".into()))?;
        let mut cur = std::io::Cursor::new(&self.mmap[pos..]);
        cur.read_u64::<LittleEndian>()
            .map_err(|e| DictError::Format(format!("MKE3: cannot read entry offset: {e}")))
    }

    fn load_entry_from_mmap(&self, index: u32) -> Result<DictEntry> {
        let offset = self.entry_offset(index)?;
        let offset_usize = usize::try_from(offset)
            .map_err(|_| DictError::Format("MKE3: offset overflow".into()))?;
        if offset_usize >= self.mmap.len() {
            return Err(DictError::Format(format!(
                "MKE3: entry {index} offset {offset} out of mmap bounds"
            )));
        }

        let mut cur = std::io::Cursor::new(&self.mmap[offset_usize..]);

        let left_id = cur
            .read_u16::<LittleEndian>()
            .map_err(|e| DictError::Format(format!("MKE3 entry {index} left_id: {e}")))?;
        let right_id = cur
            .read_u16::<LittleEndian>()
            .map_err(|e| DictError::Format(format!("MKE3 entry {index} right_id: {e}")))?;
        let cost = cur
            .read_i16::<LittleEndian>()
            .map_err(|e| DictError::Format(format!("MKE3 entry {index} cost: {e}")))?;
        let surface_len = cur
            .read_u16::<LittleEndian>()
            .map_err(|e| DictError::Format(format!("MKE3 entry {index} surface_len: {e}")))?
            as usize;
        let feature_len = cur
            .read_u32::<LittleEndian>()
            .map_err(|e| DictError::Format(format!("MKE3 entry {index} feature_len: {e}")))?
            as usize;

        let record_header = 2 + 2 + 2 + 2 + 4;
        let remaining = self.mmap.len().saturating_sub(offset_usize + record_header);
        if surface_len + feature_len > remaining {
            return Err(DictError::Format(format!(
                "MKE3 entry {index}: surface_len({surface_len}) + feature_len({feature_len}) exceeds remaining bytes({remaining})"
            )));
        }

        let mut surface_bytes = vec![0u8; surface_len];
        cur.read_exact(&mut surface_bytes)
            .map_err(|e| DictError::Format(format!("MKE3 entry {index} surface: {e}")))?;
        let surface = String::from_utf8(surface_bytes)
            .map_err(|e| DictError::Format(format!("MKE3 entry {index} surface utf8: {e}")))?;

        let mut feature_bytes = vec![0u8; feature_len];
        cur.read_exact(&mut feature_bytes)
            .map_err(|e| DictError::Format(format!("MKE3 entry {index} feature: {e}")))?;
        let feature = String::from_utf8(feature_bytes)
            .map_err(|e| DictError::Format(format!("MKE3 entry {index} feature utf8: {e}")))?;

        Ok(DictEntry {
            surface,
            left_id,
            right_id,
            cost,
            feature,
        })
    }
}

/// Write `entries` as an MKE3 file at `path`.
///
/// # Errors
///
/// Returns `DictError::Io` on write failure, or `DictError::Format` when
/// the entry count or surface length exceeds supported limits.
pub fn save_entries_v3<P: AsRef<Path>>(entries: &[DictEntry], path: P) -> Result<()> {
    use std::io::Write;

    let path = path.as_ref();
    let mut file = std::fs::File::create(path).map_err(DictError::Io)?;

    let count = u32::try_from(entries.len())
        .map_err(|_| DictError::Format("MKE3: too many entries".into()))?;

    // Header — index_offset written as 0 placeholder, patched at the end.
    file.write_all(ENTRIES_V3_MAGIC).map_err(DictError::Io)?;
    file.write_u32::<LittleEndian>(ENTRIES_V3_VERSION)
        .map_err(DictError::Io)?;
    file.write_u32::<LittleEndian>(count)
        .map_err(DictError::Io)?;
    file.write_u16::<LittleEndian>(FEATURE_U32)
        .map_err(DictError::Io)?;
    file.write_u16::<LittleEndian>(0) // reserved
        .map_err(DictError::Io)?;
    file.write_u64::<LittleEndian>(0) // placeholder
        .map_err(DictError::Io)?;

    let mut offsets: Vec<u64> = Vec::with_capacity(entries.len());

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
            .map_err(|_| DictError::Format("MKE3: surface too long".into()))?;
        file.write_u16::<LittleEndian>(surface_len)
            .map_err(DictError::Io)?;

        let feature_bytes = entry.feature.as_bytes();
        let feature_len = u32::try_from(feature_bytes.len())
            .map_err(|_| DictError::Format("MKE3: feature too long".into()))?;
        file.write_u32::<LittleEndian>(feature_len)
            .map_err(DictError::Io)?;

        file.write_all(surface_bytes).map_err(DictError::Io)?;
        file.write_all(feature_bytes).map_err(DictError::Io)?;
    }

    // Index table
    let index_offset = file.stream_position().map_err(DictError::Io)?;
    for offset in offsets {
        file.write_u64::<LittleEndian>(offset)
            .map_err(DictError::Io)?;
    }

    // Patch index_offset in header (starts at byte 16: 4+4+4+2+2 = 16)
    file.seek(SeekFrom::Start(16)).map_err(DictError::Io)?;
    file.write_u64::<LittleEndian>(index_offset)
        .map_err(DictError::Io)?;

    Ok(())
}

/// Migrate an entries.bin file from v2 (MKE2) format to v3 (MKE3).
///
/// Reads all entries from the v2 file, then writes them in v3 format.
///
/// # Errors
///
/// Returns an error if the source cannot be read or the destination
/// cannot be written.
pub fn migrate_v2_to_v3<P: AsRef<Path>, Q: AsRef<Path>>(
    v2_path: P,
    v3_path: Q,
) -> Result<usize> {
    use crate::lazy_entries::LazyEntries;

    let v2 = LazyEntries::from_file(v2_path)?;
    let count = v2.len();
    let entries = v2.load_all()?;
    save_entries_v3(&entries, v3_path)?;
    Ok(count)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]

    use super::*;
    use tempfile::tempdir;

    fn sample_entries() -> Vec<DictEntry> {
        vec![
            DictEntry::new("안녕", 1, 1, 100, "NNG,*,T,안녕,*,*,*,*"),
            DictEntry::new("하세요", 2, 2, 50, "VV,*,F,하세요,*,*,*,*"),
            DictEntry::new("감사", 3, 3, 80, "NNG,*,F,감사,*,*,*,*"),
            DictEntry::new("합니다", 4, 4, -10, "XSV,*,F,합니다,*,*,*,*"),
            DictEntry::new("가", 5, 5, 200, "JKS,*,F,가,*,*,*,*"),
        ]
    }

    #[test]
    fn test_v3_roundtrip() {
        let entries = sample_entries();
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("entries_v3.bin");

        save_entries_v3(&entries, &path).expect("save");

        let lazy = LazyEntriesV3::from_file(&path).expect("load");
        assert_eq!(lazy.len(), 5);
        assert!(!lazy.is_empty());
        assert_eq!(lazy.flags() & FEATURE_U32, FEATURE_U32);

        for (i, expected) in entries.iter().enumerate() {
            let got = lazy.get(i as u32).expect("get");
            assert_eq!(got.surface, expected.surface, "surface[{i}]");
            assert_eq!(got.left_id, expected.left_id, "left_id[{i}]");
            assert_eq!(got.right_id, expected.right_id, "right_id[{i}]");
            assert_eq!(got.cost, expected.cost, "cost[{i}]");
            assert_eq!(got.feature, expected.feature, "feature[{i}]");
        }

        assert!(lazy.get(5).is_err());
    }

    #[test]
    fn test_v3_large_feature() {
        // v2's u16 cap is 65535; this exceeds it.
        let large_feature = "X".repeat(70_000);
        let entry = DictEntry::new("테스트", 10, 10, 0, &large_feature);

        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("large_feature.bin");

        save_entries_v3(&[entry], &path).expect("save large feature");

        let lazy = LazyEntriesV3::from_file(&path).expect("load");
        assert_eq!(lazy.len(), 1);

        let got = lazy.get(0).expect("get");
        assert_eq!(got.surface, "테스트");
        assert_eq!(got.feature.len(), 70_000);
        assert!(got.feature.chars().all(|c| c == 'X'));
    }

    #[test]
    fn test_detect_format() {
        let dir = tempdir().expect("tempdir");

        let v3_path = dir.path().join("v3.bin");
        save_entries_v3(&sample_entries(), &v3_path).expect("save v3");
        assert_eq!(
            detect_entries_format(&v3_path).expect("detect v3"),
            EntriesFormat::V3
        );

        // Write a fake v2 header to check detection.
        let v2_path = dir.path().join("v2.bin");
        {
            use std::io::Write;
            let mut f = std::fs::File::create(&v2_path).expect("create v2 file");
            f.write_all(b"MKE2").expect("write magic");
        }
        assert_eq!(
            detect_entries_format(&v2_path).expect("detect v2"),
            EntriesFormat::V2
        );

        // Write a fake v1 header.
        let v1_path = dir.path().join("v1.bin");
        {
            use std::io::Write;
            let mut f = std::fs::File::create(&v1_path).expect("create v1 file");
            f.write_all(b"MKED").expect("write magic");
        }
        assert_eq!(
            detect_entries_format(&v1_path).expect("detect v1"),
            EntriesFormat::V1
        );

        // Unknown magic → error.
        let unk_path = dir.path().join("unk.bin");
        {
            use std::io::Write;
            let mut f = std::fs::File::create(&unk_path).expect("create unk file");
            f.write_all(b"????").expect("write magic");
        }
        assert!(detect_entries_format(&unk_path).is_err());
    }

    #[test]
    fn test_get_entries_at() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("entries_v3.bin");

        let entries = vec![
            DictEntry::new("가", 1, 1, 100, "NNG"),
            DictEntry::new("가", 2, 2, 50, "JKS"),
            DictEntry::new("나", 3, 3, 200, "NP"),
        ];
        save_entries_v3(&entries, &path).expect("save");

        let lazy = LazyEntriesV3::from_file(&path).expect("load");

        let results = lazy.get_entries_at(0, "가").expect("get_entries_at");
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].feature, "NNG");
        assert_eq!(results[1].feature, "JKS");

        let results = lazy.get_entries_at(2, "나").expect("get_entries_at");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].surface, "나");

        let results = lazy.get_entries_at(0, "다").expect("get_entries_at");
        assert!(results.is_empty());
    }

    #[test]
    fn test_v3_cache() {
        let entries = sample_entries();
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("cache_test.bin");

        save_entries_v3(&entries, &path).expect("save");

        let lazy = LazyEntriesV3::from_file(&path).expect("load");

        assert_eq!(lazy.cached_count(), 0);

        let _ = lazy.get(0).expect("get 0");
        assert_eq!(lazy.cached_count(), 1);

        let _ = lazy.get(0).expect("get 0 again");
        assert_eq!(lazy.cached_count(), 1, "no duplicate on repeated get");

        let _ = lazy.get(1).expect("get 1");
        assert_eq!(lazy.cached_count(), 2);

        lazy.clear_cache();
        assert_eq!(lazy.cached_count(), 0);

        // set_cache_size to 1 and verify LRU eviction.
        lazy.set_cache_size(1);
        let _ = lazy.get(0).expect("get 0");
        let _ = lazy.get(1).expect("get 1");
        assert_eq!(lazy.cached_count(), 1);
    }

    #[test]
    fn test_migrate_v2_to_v3() {
        use crate::lazy_entries::LazyEntries;

        let entries = vec![
            DictEntry::new("가", 1, 1, 100, "NNG"),
            DictEntry::new("가", 2, 2, 50, "JKS"),
            DictEntry::new("나", 3, 3, 200, "NP"),
        ];

        let dir = tempdir().expect("tempdir");
        let v2_path = dir.path().join("entries_v2.bin");
        let v3_path = dir.path().join("entries_v3.bin");

        LazyEntries::save_entries(&entries, &v2_path).expect("save v2");

        let count = migrate_v2_to_v3(&v2_path, &v3_path).expect("migrate");
        assert_eq!(count, 3);

        assert_eq!(
            detect_entries_format(&v3_path).expect("detect"),
            EntriesFormat::V3
        );

        let v3 = LazyEntriesV3::from_file(&v3_path).expect("load v3");
        assert_eq!(v3.len(), 3);

        let e0 = v3.get(0).expect("get 0");
        assert_eq!(e0.surface, "가");
        assert_eq!(e0.left_id, 1);
        assert_eq!(e0.feature, "NNG");

        let e2 = v3.get(2).expect("get 2");
        assert_eq!(e2.surface, "나");
        assert_eq!(e2.feature, "NP");
    }
}
