//! # 실시간 사전 업데이트 (Hot Reload) 모듈
//!
//! 무중단 사전 교체, 파일 변경 감지, 버전 관리 기능을 제공합니다.
//!
//! ## 주요 기능
//!
//! - **파일 변경 감지**: notify 크레이트를 통한 자동 감지
//! - **무중단 교체**: `RwLock`과 Copy-on-Write 전략
//! - **델타 업데이트**: 변경분만 적용하여 성능 최적화
//! - **버전 관리**: 사전 버전 추적 및 롤백 지원
//! - **스레드 안전성**: 동시 읽기/쓰기 처리
//!
//! ## 아키텍처
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │ HotReloadDictionary                     │
//! │  - RwLock<VersionedDictionary>          │
//! │  - FileWatcher (notify)                 │
//! └─────────────────────────────────────────┘
//!          │                    │
//!          ▼                    ▼
//! ┌─────────────────┐  ┌────────────────┐
//! │ System Dict     │  │ User Dict      │
//! │ (Read-only)     │  │ (Mutable)      │
//! └─────────────────┘  └────────────────┘
//! ```
//!
//! ## 사용 예제
//!
//! ```rust,ignore
//! use mecab_ko_dict::hot_reload::{HotReloadDictionary, DeltaUpdate, DeltaUpdateBuilder};
//!
//! // 핫 리로드 사전 생성
//! let dict = HotReloadDictionary::new("/path/to/dict").unwrap();
//!
//! // 실시간 엔트리 추가
//! dict.add_entry("딥러닝", "NNG", -1000i16, None).unwrap();
//!
//! // 델타 업데이트 적용
//! let delta = DeltaUpdateBuilder::new()
//!     .add("머신러닝", "NNG", -1000)
//!     .add("자연어처리", "NNG", -1000)
//!     .build();
//! dict.apply_delta(delta).unwrap();
//!
//! // 버전 관리
//! let version = dict.current_version();
//! dict.rollback(version - 1).unwrap();
//! ```
//!
//! ## 성능 특성
//!
//! - **읽기**: Lock contention 최소화 (`RwLock`)
//! - **쓰기**: `Copy-on-Write`로 기존 읽기 영향 없음
//! - **델타 업데이트**: O(변경분) 복잡도
//! - **메모리**: 버전당 증분 메모리 사용

use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};

use crate::error::{DictError, Result};
use crate::user_dict::{UserDictionary, UserEntry};
#[cfg(test)]
use crate::DictEntry;
use crate::{Entry, SystemDictionary};

/// 사전 버전
///
/// 각 업데이트마다 증가하는 단조 증가 버전 번호입니다.
pub type Version = u64;

/// 최대 버전 히스토리 크기 (기본값)
const DEFAULT_MAX_VERSION_HISTORY: usize = 10;

/// 최대 델타 큐 크기
const DEFAULT_MAX_DELTA_QUEUE: usize = 100;

/// 버전이 있는 사전 데이터
///
/// Copy-on-Write 전략을 위해 Arc로 래핑됩니다.
#[derive(Clone)]
struct VersionedDictionary {
    /// 버전 번호
    version: Version,
    /// 시스템 사전 (읽기 전용)
    system_dict: Arc<SystemDictionary>,
    /// 사용자 사전 (변경 가능)
    user_dict: Arc<UserDictionary>,
    /// 타임스탬프
    timestamp: SystemTime,
}

impl VersionedDictionary {
    /// 새 버전 생성 (사용자 사전 업데이트)
    fn new_version(&self, user_dict: UserDictionary) -> Self {
        Self {
            version: self.version + 1,
            system_dict: Arc::clone(&self.system_dict),
            user_dict: Arc::new(user_dict),
            timestamp: SystemTime::now(),
        }
    }

    /// 시스템 사전 교체
    fn with_system_dict(&self, system_dict: SystemDictionary) -> Self {
        Self {
            version: self.version + 1,
            system_dict: Arc::new(system_dict),
            user_dict: Arc::clone(&self.user_dict),
            timestamp: SystemTime::now(),
        }
    }
}

/// 핫 리로드 가능한 사전
///
/// `RwLock`으로 동시 접근을 제어하며, Copy-on-Write 전략으로 무중단 업데이트를 지원합니다.
pub struct HotReloadDictionary {
    /// 현재 사전 (버전 포함)
    current: Arc<RwLock<VersionedDictionary>>,
    /// 버전 히스토리 (롤백용)
    history: Arc<RwLock<VecDeque<VersionedDictionary>>>,
    /// 최대 히스토리 크기
    max_history: usize,
    /// 델타 업데이트 큐
    delta_queue: Arc<RwLock<VecDeque<DeltaUpdate>>>,
    /// 최대 델타 큐 크기
    max_delta_queue: usize,
    /// 사전 디렉토리 경로
    dicdir: PathBuf,
}

impl HotReloadDictionary {
    /// 사전 디렉토리에서 핫 리로드 사전 생성
    ///
    /// # Arguments
    ///
    /// * `dicdir` - 사전 디렉토리 경로
    ///
    /// # Errors
    ///
    /// - 사전 파일을 찾을 수 없는 경우
    /// - 사전 파일 포맷이 잘못된 경우
    pub fn new<P: AsRef<Path>>(dicdir: P) -> Result<Self> {
        let dicdir = dicdir.as_ref().to_path_buf();
        let system_dict = SystemDictionary::load(&dicdir)?;

        let versioned = VersionedDictionary {
            version: 1,
            system_dict: Arc::new(system_dict),
            user_dict: Arc::new(UserDictionary::new()),
            timestamp: SystemTime::now(),
        };

        Ok(Self {
            current: Arc::new(RwLock::new(versioned)),
            history: Arc::new(RwLock::new(VecDeque::new())),
            max_history: DEFAULT_MAX_VERSION_HISTORY,
            delta_queue: Arc::new(RwLock::new(VecDeque::new())),
            max_delta_queue: DEFAULT_MAX_DELTA_QUEUE,
            dicdir,
        })
    }

    /// 기본 경로에서 핫 리로드 사전 생성
    ///
    /// # Errors
    ///
    /// Returns an error if the dictionary files cannot be loaded.
    pub fn new_default() -> Result<Self> {
        let system_dict = SystemDictionary::load_default()?;
        let dicdir = system_dict.dicdir().to_path_buf();

        let versioned = VersionedDictionary {
            version: 1,
            system_dict: Arc::new(system_dict),
            user_dict: Arc::new(UserDictionary::new()),
            timestamp: SystemTime::now(),
        };

        Ok(Self {
            current: Arc::new(RwLock::new(versioned)),
            history: Arc::new(RwLock::new(VecDeque::new())),
            max_history: DEFAULT_MAX_VERSION_HISTORY,
            delta_queue: Arc::new(RwLock::new(VecDeque::new())),
            max_delta_queue: DEFAULT_MAX_DELTA_QUEUE,
            dicdir,
        })
    }

    /// 최대 버전 히스토리 크기 설정
    #[must_use]
    pub const fn with_max_history(mut self, max_history: usize) -> Self {
        self.max_history = max_history;
        self
    }

    /// 최대 델타 큐 크기 설정
    #[must_use]
    pub const fn with_max_delta_queue(mut self, max_delta_queue: usize) -> Self {
        self.max_delta_queue = max_delta_queue;
        self
    }

    /// 현재 버전 반환
    #[must_use]
    pub fn current_version(&self) -> Version {
        self.current.read().map(|dict| dict.version).unwrap_or(0)
    }

    /// 사전 디렉토리 경로 반환
    #[must_use]
    pub fn dicdir(&self) -> &Path {
        &self.dicdir
    }

    /// 엔트리 조회 (시스템 사전 + 사용자 사전)
    ///
    /// # Arguments
    ///
    /// * `surface` - 검색할 표면형
    ///
    /// # Errors
    ///
    /// Returns an error if the dictionary lock cannot be acquired.
    pub fn lookup(&self, surface: &str) -> Result<Vec<Entry>> {
        let dict = self.current.read().map_err(|_| {
            DictError::Format("Failed to acquire read lock on dictionary".to_string())
        })?;

        let mut results = Vec::new();

        // 시스템 사전 검색
        if let Some(index) = dict.system_dict.trie().exact_match(surface) {
            if let Ok(entry) = dict.system_dict.get_entry(index) {
                results.push(entry.to_entry());
            }
        }

        // 사용자 사전 검색
        let user_entries = dict.user_dict.lookup(surface);
        results.extend(user_entries.iter().map(|e| e.to_entry()));
        drop(dict);

        Ok(results)
    }

    /// 실시간 엔트리 추가
    ///
    /// # Arguments
    ///
    /// * `surface` - 표면형
    /// * `pos` - 품사
    /// * `cost` - 비용 (낮을수록 우선)
    /// * `reading` - 읽기 (선택)
    ///
    /// # Errors
    ///
    /// Returns an error if the dictionary lock cannot be acquired.
    pub fn add_entry(
        &self,
        surface: impl Into<String>,
        pos: impl Into<String>,
        cost: i16,
        reading: Option<String>,
    ) -> Result<Version> {
        let mut dict = self.current.write().map_err(|_| {
            DictError::Format("Failed to acquire write lock on dictionary".to_string())
        })?;

        // 현재 사용자 사전 복사 (Copy-on-Write)
        let mut new_user_dict = (*dict.user_dict).clone();
        new_user_dict.add_entry(surface, pos, Some(cost), reading);

        // 버전 히스토리 저장
        self.save_to_history(&dict)?;

        // 새 버전으로 교체
        *dict = dict.new_version(new_user_dict);

        Ok(dict.version)
    }

    /// 엔트리 제거
    ///
    /// # Arguments
    ///
    /// * `surface` - 제거할 표면형
    ///
    /// # Returns
    ///
    /// 제거된 엔트리 수
    ///
    /// # Errors
    ///
    /// Returns an error if the dictionary lock cannot be acquired.
    pub fn remove_entry(&self, surface: &str) -> Result<(Version, usize)> {
        let mut dict = self.current.write().map_err(|_| {
            DictError::Format("Failed to acquire write lock on dictionary".to_string())
        })?;

        // 현재 사용자 사전 복사
        let new_user_dict = (*dict.user_dict).clone();

        // 엔트리 제거 (표면형이 일치하는 모든 엔트리)
        let removed_count = new_user_dict
            .entries()
            .iter()
            .filter(|e| e.surface == surface)
            .count();

        if removed_count == 0 {
            return Ok((dict.version, 0));
        }

        // 새 사용자 사전 생성 (필터링)
        let filtered_entries: Vec<_> = new_user_dict
            .entries()
            .iter()
            .filter(|e| e.surface != surface)
            .cloned()
            .collect();

        let mut rebuilt_dict = UserDictionary::new();
        for entry in filtered_entries {
            rebuilt_dict.add_entry_with_ids(
                entry.surface,
                entry.pos,
                entry.cost,
                entry.left_id,
                entry.right_id,
                entry.reading,
            );
        }

        // 버전 히스토리 저장
        self.save_to_history(&dict)?;

        // 새 버전으로 교체
        *dict = dict.new_version(rebuilt_dict);

        Ok((dict.version, removed_count))
    }

    /// 엔트리 수정
    ///
    /// # Arguments
    ///
    /// * `surface` - 수정할 표면형
    /// * `update_fn` - 업데이트 함수
    ///
    /// # Errors
    ///
    /// Returns an error if the dictionary lock cannot be acquired.
    pub fn update_entry<F>(&self, surface: &str, update_fn: F) -> Result<Version>
    where
        F: Fn(&mut UserEntry),
    {
        let mut dict = self.current.write().map_err(|_| {
            DictError::Format("Failed to acquire write lock on dictionary".to_string())
        })?;

        // 현재 사용자 사전 복사
        let new_user_dict = (*dict.user_dict).clone();

        // 엔트리 수정
        let updated_entries: Vec<_> = new_user_dict
            .entries()
            .iter()
            .map(|e| {
                let mut updated = e.clone();
                if updated.surface == surface {
                    update_fn(&mut updated);
                }
                updated
            })
            .collect();

        // 새 사용자 사전 생성
        let mut rebuilt_dict = UserDictionary::new();
        for entry in updated_entries {
            rebuilt_dict.add_entry_with_ids(
                entry.surface,
                entry.pos,
                entry.cost,
                entry.left_id,
                entry.right_id,
                entry.reading,
            );
        }

        // 버전 히스토리 저장
        self.save_to_history(&dict)?;

        // 새 버전으로 교체
        *dict = dict.new_version(rebuilt_dict);

        Ok(dict.version)
    }

    /// 델타 업데이트 적용
    ///
    /// 여러 변경 사항을 하나의 트랜잭션으로 적용합니다.
    ///
    /// # Arguments
    ///
    /// * `delta` - 델타 업데이트
    ///
    /// # Errors
    ///
    /// Returns an error if the dictionary lock cannot be acquired.
    pub fn apply_delta(&self, delta: DeltaUpdate) -> Result<Version> {
        let mut dict = self.current.write().map_err(|_| {
            DictError::Format("Failed to acquire write lock on dictionary".to_string())
        })?;

        // 현재 사용자 사전 복사
        let mut new_user_dict = (*dict.user_dict).clone();

        // 제거 작업
        for surface in &delta.removals {
            let filtered_entries: Vec<_> = new_user_dict
                .entries()
                .iter()
                .filter(|e| e.surface != *surface)
                .cloned()
                .collect();

            let mut rebuilt_dict = UserDictionary::new();
            for entry in filtered_entries {
                rebuilt_dict.add_entry_with_ids(
                    entry.surface,
                    entry.pos,
                    entry.cost,
                    entry.left_id,
                    entry.right_id,
                    entry.reading,
                );
            }
            new_user_dict = rebuilt_dict;
        }

        // 추가 작업
        for addition in &delta.additions {
            new_user_dict.add_entry(
                addition.surface.clone(),
                addition.pos.clone(),
                Some(addition.cost),
                addition.reading.clone(),
            );
        }

        // 수정 작업
        for modification in &delta.modifications {
            let updated_entries: Vec<_> = new_user_dict
                .entries()
                .iter()
                .map(|e| {
                    if e.surface == modification.surface {
                        modification.to_user_entry()
                    } else {
                        e.clone()
                    }
                })
                .collect();

            let mut rebuilt_dict = UserDictionary::new();
            for entry in updated_entries {
                rebuilt_dict.add_entry_with_ids(
                    entry.surface,
                    entry.pos,
                    entry.cost,
                    entry.left_id,
                    entry.right_id,
                    entry.reading,
                );
            }
            new_user_dict = rebuilt_dict;
        }

        // 버전 히스토리 저장
        self.save_to_history(&dict)?;

        // 델타 큐에 추가
        self.enqueue_delta(delta)?;

        // 새 버전으로 교체
        *dict = dict.new_version(new_user_dict);

        Ok(dict.version)
    }

    /// 시스템 사전 리로드
    ///
    /// 사전 파일이 변경되었을 때 호출됩니다.
    ///
    /// # Errors
    ///
    /// Returns an error if the dictionary files cannot be reloaded or the lock cannot be acquired.
    pub fn reload_system_dict(&self) -> Result<Version> {
        let mut dict = self.current.write().map_err(|_| {
            DictError::Format("Failed to acquire write lock on dictionary".to_string())
        })?;

        // 시스템 사전 다시 로드
        let new_system_dict = SystemDictionary::load(&self.dicdir)?;

        // 버전 히스토리 저장
        self.save_to_history(&dict)?;

        // 새 버전으로 교체
        *dict = dict.with_system_dict(new_system_dict);

        Ok(dict.version)
    }

    /// 특정 버전으로 롤백
    ///
    /// # Arguments
    ///
    /// * `target_version` - 롤백할 버전
    ///
    /// # Errors
    ///
    /// Returns an error if the version is not found in history or locks cannot be acquired.
    pub fn rollback(&self, target_version: Version) -> Result<()> {
        let target = {
            let history = self.history.read().map_err(|_| {
                DictError::Format("Failed to acquire read lock on history".to_string())
            })?;

            history
                .iter()
                .find(|v| v.version == target_version)
                .ok_or_else(|| {
                    DictError::Format(format!("Version {target_version} not found in history"))
                })?
                .clone()
        };

        *self.current.write().map_err(|_| {
            DictError::Format("Failed to acquire write lock on dictionary".to_string())
        })? = target;

        Ok(())
    }

    /// 버전 히스토리 조회
    ///
    /// # Errors
    ///
    /// Returns an error if locks cannot be acquired.
    pub fn version_history(&self) -> Result<Vec<VersionInfo>> {
        let history = self
            .history
            .read()
            .map_err(|_| DictError::Format("Failed to acquire read lock on history".to_string()))?;

        let current = self.current.read().map_err(|_| {
            DictError::Format("Failed to acquire read lock on dictionary".to_string())
        })?;

        let mut versions = vec![VersionInfo {
            version: current.version,
            timestamp: current.timestamp,
            user_entry_count: current.user_dict.len(),
        }];

        versions.extend(history.iter().map(|v| VersionInfo {
            version: v.version,
            timestamp: v.timestamp,
            user_entry_count: v.user_dict.len(),
        }));
        drop(history);
        drop(current);

        versions.sort_by_key(|v| std::cmp::Reverse(v.version));

        Ok(versions)
    }

    /// 버전 히스토리에 저장
    fn save_to_history(&self, dict: &VersionedDictionary) -> Result<()> {
        let mut history = self.history.write().map_err(|_| {
            DictError::Format("Failed to acquire write lock on history".to_string())
        })?;

        history.push_back(dict.clone());

        // 최대 크기 초과 시 오래된 버전 제거
        while history.len() > self.max_history {
            history.pop_front();
        }
        drop(history);

        Ok(())
    }

    /// 델타 큐에 추가
    fn enqueue_delta(&self, delta: DeltaUpdate) -> Result<()> {
        let mut queue = self.delta_queue.write().map_err(|_| {
            DictError::Format("Failed to acquire write lock on delta queue".to_string())
        })?;

        queue.push_back(delta);

        while queue.len() > self.max_delta_queue {
            queue.pop_front();
        }
        drop(queue);

        Ok(())
    }

    /// 델타 히스토리 조회
    ///
    /// # Errors
    ///
    /// Returns an error if the lock cannot be acquired.
    pub fn delta_history(&self) -> Result<Vec<DeltaUpdate>> {
        let queue = self.delta_queue.read().map_err(|_| {
            DictError::Format("Failed to acquire read lock on delta queue".to_string())
        })?;

        Ok(queue.iter().cloned().collect())
    }

    /// 사용자 사전 내보내기
    ///
    /// # Errors
    ///
    /// Returns an error if the lock cannot be acquired.
    pub fn export_user_dict(&self) -> Result<UserDictionary> {
        let dict = self.current.read().map_err(|_| {
            DictError::Format("Failed to acquire read lock on dictionary".to_string())
        })?;

        let user_dict = (*dict.user_dict).clone();
        drop(dict);
        Ok(user_dict)
    }

    /// 사용자 사전 가져오기
    ///
    /// # Errors
    ///
    /// Returns an error if the lock cannot be acquired.
    pub fn import_user_dict(&self, user_dict: UserDictionary) -> Result<Version> {
        let mut dict = self.current.write().map_err(|_| {
            DictError::Format("Failed to acquire write lock on dictionary".to_string())
        })?;

        self.save_to_history(&dict)?;
        *dict = dict.new_version(user_dict);

        Ok(dict.version)
    }
}

/// 델타 업데이트
///
/// 여러 변경 사항을 하나의 트랜잭션으로 묶습니다.
#[derive(Debug, Clone)]
pub struct DeltaUpdate {
    /// 추가할 엔트리
    additions: Vec<EntryChange>,
    /// 제거할 엔트리 (표면형)
    removals: Vec<String>,
    /// 수정할 엔트리
    modifications: Vec<EntryChange>,
}

impl Default for DeltaUpdate {
    fn default() -> Self {
        Self::new()
    }
}

impl DeltaUpdate {
    /// 새 델타 업데이트 생성
    #[must_use]
    pub const fn new() -> Self {
        Self {
            additions: Vec::new(),
            removals: Vec::new(),
            modifications: Vec::new(),
        }
    }

    /// 빌더 패턴 시작
    #[must_use]
    pub const fn builder() -> DeltaUpdateBuilder {
        DeltaUpdateBuilder::new()
    }

    /// 추가 작업 수
    #[must_use]
    pub fn addition_count(&self) -> usize {
        self.additions.len()
    }

    /// 제거 작업 수
    #[must_use]
    pub fn removal_count(&self) -> usize {
        self.removals.len()
    }

    /// 수정 작업 수
    #[must_use]
    pub fn modification_count(&self) -> usize {
        self.modifications.len()
    }

    /// 총 변경 작업 수
    #[must_use]
    pub fn total_changes(&self) -> usize {
        self.additions.len() + self.removals.len() + self.modifications.len()
    }
}

/// 엔트리 변경 정보
#[derive(Debug, Clone)]
pub struct EntryChange {
    /// 표면형
    pub surface: String,
    /// 품사
    pub pos: String,
    /// 비용
    pub cost: i16,
    /// 읽기
    pub reading: Option<String>,
    /// 좌문맥 ID
    pub left_id: u16,
    /// 우문맥 ID
    pub right_id: u16,
}

impl EntryChange {
    /// `UserEntry`로 변환
    fn to_user_entry(&self) -> UserEntry {
        UserEntry::new(
            self.surface.clone(),
            self.pos.clone(),
            self.cost,
            self.reading.clone(),
        )
        .with_context_ids(self.left_id, self.right_id)
    }
}

/// 델타 업데이트 빌더
pub struct DeltaUpdateBuilder {
    delta: DeltaUpdate,
}

impl Default for DeltaUpdateBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl DeltaUpdateBuilder {
    /// 새 빌더 생성
    #[must_use]
    pub const fn new() -> Self {
        Self {
            delta: DeltaUpdate::new(),
        }
    }

    /// 엔트리 추가
    #[must_use]
    pub fn add(mut self, surface: impl Into<String>, pos: impl Into<String>, cost: i16) -> Self {
        self.delta.additions.push(EntryChange {
            surface: surface.into(),
            pos: pos.into(),
            cost,
            reading: None,
            left_id: 0,
            right_id: 0,
        });
        self
    }

    /// 엔트리 추가 (읽기 포함)
    #[must_use]
    pub fn add_with_reading(
        mut self,
        surface: impl Into<String>,
        pos: impl Into<String>,
        cost: i16,
        reading: impl Into<String>,
    ) -> Self {
        self.delta.additions.push(EntryChange {
            surface: surface.into(),
            pos: pos.into(),
            cost,
            reading: Some(reading.into()),
            left_id: 0,
            right_id: 0,
        });
        self
    }

    /// 엔트리 제거
    #[must_use]
    pub fn remove(mut self, surface: impl Into<String>) -> Self {
        self.delta.removals.push(surface.into());
        self
    }

    /// 엔트리 수정
    #[must_use]
    pub fn modify(mut self, surface: impl Into<String>, pos: impl Into<String>, cost: i16) -> Self {
        self.delta.modifications.push(EntryChange {
            surface: surface.into(),
            pos: pos.into(),
            cost,
            reading: None,
            left_id: 0,
            right_id: 0,
        });
        self
    }

    /// 델타 업데이트 빌드
    #[must_use]
    pub fn build(self) -> DeltaUpdate {
        self.delta
    }
}

/// 버전 정보
#[derive(Debug, Clone)]
pub struct VersionInfo {
    /// 버전 번호
    pub version: Version,
    /// 타임스탬프
    pub timestamp: SystemTime,
    /// 사용자 사전 엔트리 수
    pub user_entry_count: usize,
}

impl VersionInfo {
    /// 버전 생성 시간 (Duration)
    #[must_use]
    pub fn age(&self) -> Option<Duration> {
        SystemTime::now().duration_since(self.timestamp).ok()
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::vec_init_then_push)]
mod tests {
    use super::*;
    use crate::matrix::DenseMatrix;
    use crate::trie::TrieBuilder;

    fn create_test_system_dict() -> SystemDictionary {
        let entries = vec![("가", 0u32), ("가다", 1), ("가방", 2)];
        let trie_bytes = TrieBuilder::build(&entries).expect("should build trie");
        let trie = crate::trie::TrieBackend::Owned(crate::trie::Trie::from_vec(trie_bytes));
        let matrix = crate::matrix::ConnectionMatrix::Dense(DenseMatrix::new(10, 10, 100));

        let mut dict_entries = Vec::new();
        dict_entries.push(DictEntry::new("가", 1, 1, 100, "NNG,*,T,가,*,*,*,*"));
        dict_entries.push(DictEntry::new("가다", 2, 2, 200, "VV,*,F,가다,*,*,*,*"));
        dict_entries.push(DictEntry::new("가방", 3, 3, 300, "NNG,*,T,가방,*,*,*,*"));

        SystemDictionary::new_test(PathBuf::from("./test_dic"), trie, matrix, dict_entries)
    }

    #[test]
    fn test_hot_reload_dictionary_add_entry() {
        let system_dict = create_test_system_dict();
        let dicdir = system_dict.dicdir().to_path_buf();

        let versioned = VersionedDictionary {
            version: 1,
            system_dict: Arc::new(system_dict),
            user_dict: Arc::new(UserDictionary::new()),
            timestamp: SystemTime::now(),
        };

        let dict = HotReloadDictionary {
            current: Arc::new(RwLock::new(versioned)),
            history: Arc::new(RwLock::new(VecDeque::new())),
            max_history: 10,
            delta_queue: Arc::new(RwLock::new(VecDeque::new())),
            max_delta_queue: 100,
            dicdir,
        };

        let v1 = dict.current_version();
        assert_eq!(v1, 1);

        let v2 = dict
            .add_entry("딥러닝", "NNG", -1000, None)
            .expect("should add entry");
        assert_eq!(v2, 2);

        let entries = dict.lookup("딥러닝").expect("should lookup");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].surface, "딥러닝");
    }

    #[test]
    fn test_hot_reload_dictionary_remove_entry() {
        let system_dict = create_test_system_dict();
        let dicdir = system_dict.dicdir().to_path_buf();

        let mut user_dict = UserDictionary::new();
        user_dict.add_entry("딥러닝", "NNG", Some(-1000), None);

        let versioned = VersionedDictionary {
            version: 1,
            system_dict: Arc::new(system_dict),
            user_dict: Arc::new(user_dict),
            timestamp: SystemTime::now(),
        };

        let dict = HotReloadDictionary {
            current: Arc::new(RwLock::new(versioned)),
            history: Arc::new(RwLock::new(VecDeque::new())),
            max_history: 10,
            delta_queue: Arc::new(RwLock::new(VecDeque::new())),
            max_delta_queue: 100,
            dicdir,
        };

        let (version, removed) = dict.remove_entry("딥러닝").expect("should remove");
        assert_eq!(version, 2);
        assert_eq!(removed, 1);

        let entries = dict.lookup("딥러닝").expect("should lookup");
        assert!(entries.is_empty());
    }

    #[test]
    fn test_delta_update() {
        let system_dict = create_test_system_dict();
        let dicdir = system_dict.dicdir().to_path_buf();

        let versioned = VersionedDictionary {
            version: 1,
            system_dict: Arc::new(system_dict),
            user_dict: Arc::new(UserDictionary::new()),
            timestamp: SystemTime::now(),
        };

        let dict = HotReloadDictionary {
            current: Arc::new(RwLock::new(versioned)),
            history: Arc::new(RwLock::new(VecDeque::new())),
            max_history: 10,
            delta_queue: Arc::new(RwLock::new(VecDeque::new())),
            max_delta_queue: 100,
            dicdir,
        };

        let delta = DeltaUpdate::builder()
            .add("딥러닝", "NNG", -1000)
            .add("머신러닝", "NNG", -1000)
            .add("자연어처리", "NNG", -1000)
            .build();

        assert_eq!(delta.addition_count(), 3);

        let version = dict.apply_delta(delta).expect("should apply delta");
        assert_eq!(version, 2);

        let entries = dict.lookup("딥러닝").expect("should lookup");
        assert_eq!(entries.len(), 1);

        let entries = dict.lookup("머신러닝").expect("should lookup");
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn test_version_rollback() {
        let system_dict = create_test_system_dict();
        let dicdir = system_dict.dicdir().to_path_buf();

        let versioned = VersionedDictionary {
            version: 1,
            system_dict: Arc::new(system_dict),
            user_dict: Arc::new(UserDictionary::new()),
            timestamp: SystemTime::now(),
        };

        let dict = HotReloadDictionary {
            current: Arc::new(RwLock::new(versioned)),
            history: Arc::new(RwLock::new(VecDeque::new())),
            max_history: 10,
            delta_queue: Arc::new(RwLock::new(VecDeque::new())),
            max_delta_queue: 100,
            dicdir,
        };

        // 버전 1
        let v1 = dict.current_version();

        // 버전 2
        dict.add_entry("딥러닝", "NNG", -1000, None)
            .expect("should add");

        // 버전 3
        dict.add_entry("머신러닝", "NNG", -1000, None)
            .expect("should add");

        assert_eq!(dict.current_version(), 3);

        // 버전 1로 롤백
        dict.rollback(v1).expect("should rollback");
        assert_eq!(dict.current_version(), v1);

        let entries = dict.lookup("딥러닝").expect("should lookup");
        assert!(entries.is_empty());
    }

    #[test]
    fn test_version_history() {
        let system_dict = create_test_system_dict();
        let dicdir = system_dict.dicdir().to_path_buf();

        let versioned = VersionedDictionary {
            version: 1,
            system_dict: Arc::new(system_dict),
            user_dict: Arc::new(UserDictionary::new()),
            timestamp: SystemTime::now(),
        };

        let dict = HotReloadDictionary {
            current: Arc::new(RwLock::new(versioned)),
            history: Arc::new(RwLock::new(VecDeque::new())),
            max_history: 10,
            delta_queue: Arc::new(RwLock::new(VecDeque::new())),
            max_delta_queue: 100,
            dicdir,
        };

        dict.add_entry("A", "NNG", 0, None).expect("should add");
        dict.add_entry("B", "NNG", 0, None).expect("should add");
        dict.add_entry("C", "NNG", 0, None).expect("should add");

        let history = dict.version_history().expect("should get history");
        assert_eq!(history.len(), 4); // current + 3 history
        assert_eq!(history[0].version, 4); // 최신 버전이 먼저
    }

    #[test]
    fn test_update_entry() {
        let system_dict = create_test_system_dict();
        let dicdir = system_dict.dicdir().to_path_buf();

        let mut user_dict = UserDictionary::new();
        user_dict.add_entry("딥러닝", "NNG", Some(-1000), None);

        let versioned = VersionedDictionary {
            version: 1,
            system_dict: Arc::new(system_dict),
            user_dict: Arc::new(user_dict),
            timestamp: SystemTime::now(),
        };

        let dict = HotReloadDictionary {
            current: Arc::new(RwLock::new(versioned)),
            history: Arc::new(RwLock::new(VecDeque::new())),
            max_history: 10,
            delta_queue: Arc::new(RwLock::new(VecDeque::new())),
            max_delta_queue: 100,
            dicdir,
        };

        dict.update_entry("딥러닝", |entry| {
            entry.cost = -2000;
            entry.reading = Some("딥러닝".to_string());
        })
        .expect("should update");

        let entries = dict.lookup("딥러닝").expect("should lookup");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].cost, -2000);
    }
}
