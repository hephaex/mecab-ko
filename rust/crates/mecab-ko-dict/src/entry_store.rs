//! # Entry Store Abstraction
//!
//! 사전 엔트리 저장소 추상화 레이어입니다.
//! Eager 로드와 Lazy 로드를 모두 지원합니다.
//!
//! ## 구현체
//!
//! - `EagerStore`: 전체 엔트리를 메모리에 로드 (기존 방식)
//! - `LazyStore`: 필요 시 디스크에서 로드 (메모리 최적화)

use std::sync::Arc;

use crate::dictionary::DictEntry;
use crate::error::{DictError, Result};
use crate::lazy_entries::LazyEntries;
use crate::lazy_entries_v3::LazyEntriesV3;

/// 사전 엔트리 저장소 인터페이스
///
/// Eager/Lazy 로드 모드를 추상화합니다.
pub trait EntryStore: Send + Sync {
    /// 인덱스로 엔트리 조회
    ///
    /// # Errors
    ///
    /// - 인덱스가 범위를 벗어난 경우
    /// - Lazy 모드에서 디스크 읽기 실패 시
    fn get(&self, index: u32) -> Result<Arc<DictEntry>>;

    /// 인덱스에서 시작하여 같은 surface를 가진 연속된 엔트리 반환
    ///
    /// # Errors
    ///
    /// - 인덱스가 범위를 벗어난 경우
    /// - Lazy 모드에서 디스크 읽기 실패 시
    fn get_entries_at(&self, first_index: u32, surface: &str) -> Result<Vec<Arc<DictEntry>>>;

    /// 엔트리 수 반환
    fn len(&self) -> usize;

    /// 비어있는지 확인
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Eager 로드 저장소
///
/// 전체 엔트리를 메모리에 로드합니다.
/// 빠른 접근이 필요하고 메모리가 충분할 때 사용합니다.
pub struct EagerStore {
    /// 전체 엔트리 (Arc로 래핑하여 공유)
    entries: Vec<Arc<DictEntry>>,
}

impl EagerStore {
    /// 새 Eager 저장소 생성
    ///
    /// `DictEntry` 벡터를 `Arc`로 래핑합니다.
    #[must_use]
    pub fn new(entries: Vec<DictEntry>) -> Self {
        Self {
            entries: entries.into_iter().map(Arc::new).collect(),
        }
    }

    /// Arc 벡터로부터 직접 생성
    #[must_use]
    pub const fn from_arc_vec(entries: Vec<Arc<DictEntry>>) -> Self {
        Self { entries }
    }

    /// 내부 엔트리 참조 반환 (테스트용)
    #[cfg(test)]
    #[must_use]
    pub fn entries(&self) -> &[Arc<DictEntry>] {
        &self.entries
    }
}

impl EntryStore for EagerStore {
    fn get(&self, index: u32) -> Result<Arc<DictEntry>> {
        self.entries.get(index as usize).cloned().ok_or_else(|| {
            DictError::Format(format!(
                "entry index out of bounds: {} >= {}",
                index,
                self.entries.len()
            ))
        })
    }

    fn get_entries_at(&self, first_index: u32, surface: &str) -> Result<Vec<Arc<DictEntry>>> {
        let start = first_index as usize;
        let results: Vec<Arc<DictEntry>> = self
            .entries
            .get(start..)
            .unwrap_or(&[])
            .iter()
            .take_while(|e| e.surface == surface)
            .cloned()
            .collect();
        Ok(results)
    }

    fn len(&self) -> usize {
        self.entries.len()
    }
}

/// Lazy 로드 저장소
///
/// 엔트리를 필요할 때만 디스크에서 읽어옵니다.
/// 메모리 사용량을 줄이고 싶을 때 사용합니다.
pub struct LazyStore {
    /// 지연 로딩 엔트리
    lazy_entries: LazyEntries,
}

impl LazyStore {
    /// 새 Lazy 저장소 생성
    #[must_use]
    pub const fn new(lazy_entries: LazyEntries) -> Self {
        Self { lazy_entries }
    }

    /// 캐시된 엔트리 수 반환
    #[must_use]
    pub fn cached_count(&self) -> usize {
        self.lazy_entries.cached_count()
    }

    /// 캐시 크기 설정
    pub fn set_cache_size(&self, size: usize) {
        self.lazy_entries.set_cache_size(size);
    }

    /// 캐시 초기화
    pub fn clear_cache(&self) {
        self.lazy_entries.clear_cache();
    }
}

impl EntryStore for LazyStore {
    fn get(&self, index: u32) -> Result<Arc<DictEntry>> {
        self.lazy_entries.get(index)
    }

    fn get_entries_at(&self, first_index: u32, surface: &str) -> Result<Vec<Arc<DictEntry>>> {
        self.lazy_entries.get_entries_at(first_index, surface)
    }

    fn len(&self) -> usize {
        self.lazy_entries.len()
    }
}

/// Lazy 로드 저장소 (v3 포맷)
///
/// MKE3 v3 포맷의 엔트리를 필요할 때만 디스크에서 읽어옵니다.
pub struct LazyStoreV3 {
    lazy_entries: LazyEntriesV3,
}

impl LazyStoreV3 {
    /// 새 v3 Lazy 저장소 생성
    #[must_use]
    pub const fn new(lazy_entries: LazyEntriesV3) -> Self {
        Self { lazy_entries }
    }

    /// 캐시된 엔트리 수 반환
    #[must_use]
    pub fn cached_count(&self) -> usize {
        self.lazy_entries.cached_count()
    }

    /// 캐시 크기 설정
    pub fn set_cache_size(&self, size: usize) {
        self.lazy_entries.set_cache_size(size);
    }

    /// 캐시 초기화
    pub fn clear_cache(&self) {
        self.lazy_entries.clear_cache();
    }
}

impl EntryStore for LazyStoreV3 {
    fn get(&self, index: u32) -> Result<Arc<DictEntry>> {
        self.lazy_entries.get(index)
    }

    fn get_entries_at(&self, first_index: u32, surface: &str) -> Result<Vec<Arc<DictEntry>>> {
        self.lazy_entries.get_entries_at(first_index, surface)
    }

    fn len(&self) -> usize {
        self.lazy_entries.len()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::expect_used, clippy::unwrap_used)]

    use super::*;

    fn sample_entries() -> Vec<DictEntry> {
        vec![
            DictEntry::new("가", 1, 1, 100, "NNG"),
            DictEntry::new("가", 2, 2, 50, "JKS"),
            DictEntry::new("나", 3, 3, 200, "NP"),
        ]
    }

    #[test]
    fn test_eager_store_get() {
        let store = EagerStore::new(sample_entries());

        let entry = store.get(0).expect("should get entry 0");
        assert_eq!(entry.surface, "가");
        assert_eq!(entry.left_id, 1);

        let entry = store.get(1).expect("should get entry 1");
        assert_eq!(entry.surface, "가");
        assert_eq!(entry.left_id, 2);

        assert!(store.get(100).is_err());
    }

    #[test]
    fn test_eager_store_get_entries_at() {
        let store = EagerStore::new(sample_entries());

        // "가"로 시작하는 연속 엔트리 (인덱스 0, 1)
        let entries = store.get_entries_at(0, "가").expect("should get entries");
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].feature, "NNG");
        assert_eq!(entries[1].feature, "JKS");

        // "나" (인덱스 2)
        let entries = store.get_entries_at(2, "나").expect("should get entries");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].surface, "나");

        // 빈 결과
        let entries = store.get_entries_at(0, "다").expect("should get entries");
        assert!(entries.is_empty());
    }

    #[test]
    fn test_eager_store_len() {
        let store = EagerStore::new(sample_entries());
        assert_eq!(store.len(), 3);
        assert!(!store.is_empty());

        let empty_store = EagerStore::new(Vec::new());
        assert_eq!(empty_store.len(), 0);
        assert!(empty_store.is_empty());
    }

    #[test]
    fn test_lazy_store_roundtrip() {
        use tempfile::tempdir;

        let entries = sample_entries();
        let dir = tempdir().expect("create temp dir");
        let path = dir.path().join("entries.bin");

        // 저장
        LazyEntries::save_entries(&entries, &path).expect("save");

        // 로드
        let lazy = LazyEntries::from_file(&path).expect("load");
        let store = LazyStore::new(lazy);

        assert_eq!(store.len(), 3);

        let entry = store.get(0).expect("get 0");
        assert_eq!(entry.surface, "가");

        let entries = store.get_entries_at(0, "가").expect("get_entries_at");
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn test_lazy_store_cache() {
        use tempfile::tempdir;

        let entries = sample_entries();
        let dir = tempdir().expect("create temp dir");
        let path = dir.path().join("entries.bin");

        LazyEntries::save_entries(&entries, &path).expect("save");
        let lazy = LazyEntries::from_file(&path).expect("load");
        let store = LazyStore::new(lazy);

        assert_eq!(store.cached_count(), 0);

        let _ = store.get(0).expect("get 0");
        assert_eq!(store.cached_count(), 1);

        store.clear_cache();
        assert_eq!(store.cached_count(), 0);
    }

    #[test]
    fn test_lazy_store_v3_roundtrip() {
        use crate::lazy_entries_v3::{save_entries_v3, LazyEntriesV3};
        use tempfile::tempdir;

        let entries = sample_entries();
        let dir = tempdir().expect("create temp dir");
        let path = dir.path().join("entries_v3.bin");

        save_entries_v3(&entries, &path).expect("save");

        let lazy = LazyEntriesV3::from_file(&path).expect("load");
        let store = LazyStoreV3::new(lazy);

        assert_eq!(store.len(), 3);

        let entry = store.get(0).expect("get 0");
        assert_eq!(entry.surface, "가");

        let entries = store.get_entries_at(0, "가").expect("get_entries_at");
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn test_lazy_store_v3_cache() {
        use crate::lazy_entries_v3::{save_entries_v3, LazyEntriesV3};
        use tempfile::tempdir;

        let entries = sample_entries();
        let dir = tempdir().expect("create temp dir");
        let path = dir.path().join("entries_v3.bin");

        save_entries_v3(&entries, &path).expect("save");
        let lazy = LazyEntriesV3::from_file(&path).expect("load");
        let store = LazyStoreV3::new(lazy);

        assert_eq!(store.cached_count(), 0);
        let _ = store.get(0).expect("get 0");
        assert_eq!(store.cached_count(), 1);
        store.clear_cache();
        assert_eq!(store.cached_count(), 0);
    }
}
