//! # 파일 변경 감지 모듈
//!
//! `notify` 크레이트를 사용하여 사전 파일 변경을 감지하고
//! 자동으로 핫 리로드를 트리거합니다.
//!
//! ## 아키텍처
//!
//! ```text
//! ┌──────────────────────────────────────┐
//! │ FileWatcher                          │
//! │  - notify::RecommendedWatcher        │
//! │  - crossbeam::Receiver               │
//! └──────────────────────────────────────┘
//!          │
//!          ▼
//! ┌──────────────────────────────────────┐
//! │ File System Events                   │
//! │  - Create, Modify, Delete            │
//! └──────────────────────────────────────┘
//!          │
//!          ▼
//! ┌──────────────────────────────────────┐
//! │ HotReloadDictionary::reload()        │
//! └──────────────────────────────────────┘
//! ```
//!
//! ## 사용 예제
//!
//! ```rust,ignore
//! use mecab_ko_dict::file_watcher::{FileWatcher, WatchConfig};
//! use mecab_ko_dict::hot_reload::HotReloadDictionary;
//! use std::sync::Arc;
//!
//! let dict = Arc::new(HotReloadDictionary::new("/path/to/dict")?);
//! let config = WatchConfig::default().debounce_ms(500);
//!
//! let mut watcher = FileWatcher::new(dict.clone(), config)?;
//! watcher.start()?;
//!
//! // 파일 변경 감지 및 자동 리로드
//! // ...
//!
//! watcher.stop()?;
//! ```

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crossbeam_channel::{bounded, Receiver, Sender};
use notify::{
    event::{Event, EventKind},
    RecommendedWatcher, RecursiveMode, Watcher,
};

use crate::error::{DictError, Result};
use crate::hot_reload::HotReloadDictionary;

/// 파일 감시 설정
#[derive(Debug, Clone)]
pub struct WatchConfig {
    /// 디바운스 시간 (밀리초)
    pub debounce_ms: u64,
    /// 재귀 감시 여부
    pub recursive: bool,
    /// 감시할 파일 확장자
    pub watch_extensions: Vec<String>,
    /// 무시할 파일 패턴
    pub ignore_patterns: Vec<String>,
}

impl Default for WatchConfig {
    fn default() -> Self {
        Self {
            debounce_ms: 300,
            recursive: false,
            watch_extensions: vec![
                "dic".to_string(),
                "bin".to_string(),
                "def".to_string(),
                "csv".to_string(),
                "zst".to_string(),
            ],
            ignore_patterns: vec![".tmp".to_string(), ".swp".to_string(), "~".to_string()],
        }
    }
}

impl WatchConfig {
    /// 디바운스 시간 설정
    pub fn debounce_ms(mut self, ms: u64) -> Self {
        self.debounce_ms = ms;
        self
    }

    /// 재귀 감시 설정
    pub fn recursive(mut self, recursive: bool) -> Self {
        self.recursive = recursive;
        self
    }

    /// 감시할 파일 확장자 추가
    pub fn watch_extension(mut self, ext: impl Into<String>) -> Self {
        self.watch_extensions.push(ext.into());
        self
    }

    /// 무시할 파일 패턴 추가
    pub fn ignore_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.ignore_patterns.push(pattern.into());
        self
    }

    /// 파일이 감시 대상인지 확인
    fn should_watch(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();

        // 무시 패턴 확인
        for pattern in &self.ignore_patterns {
            if path_str.contains(pattern) {
                return false;
            }
        }

        // 확장자 확인
        if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy();
            return self.watch_extensions.iter().any(|e| e == &*ext_str);
        }

        false
    }
}

/// 파일 변경 이벤트
#[derive(Debug, Clone)]
pub enum FileEvent {
    /// 파일 생성
    Created(PathBuf),
    /// 파일 수정
    Modified(PathBuf),
    /// 파일 삭제
    Deleted(PathBuf),
    /// 파일 이름 변경
    Renamed {
        /// 이전 경로
        from: PathBuf,
        /// 새 경로
        to: PathBuf,
    },
}

/// 파일 감시자
pub struct FileWatcher {
    /// 사전 인스턴스
    dict: Arc<HotReloadDictionary>,
    /// 감시 설정
    config: WatchConfig,
    /// notify 감시자
    watcher: Option<RecommendedWatcher>,
    /// 이벤트 수신자
    event_rx: Option<Receiver<notify::Result<Event>>>,
    /// 종료 신호 송신자
    stop_tx: Option<Sender<()>>,
    /// 워커 스레드 핸들
    worker_handle: Option<thread::JoinHandle<()>>,
}

impl FileWatcher {
    /// 새 파일 감시자 생성
    ///
    /// # Arguments
    ///
    /// * `dict` - 핫 리로드 사전 인스턴스
    /// * `config` - 감시 설정
    pub fn new(dict: Arc<HotReloadDictionary>, config: WatchConfig) -> Result<Self> {
        Ok(Self {
            dict,
            config,
            watcher: None,
            event_rx: None,
            stop_tx: None,
            worker_handle: None,
        })
    }

    /// 기본 설정으로 파일 감시자 생성
    pub fn new_default(dict: Arc<HotReloadDictionary>) -> Result<Self> {
        Self::new(dict, WatchConfig::default())
    }

    /// 파일 감시 시작
    ///
    /// # Errors
    ///
    /// - 감시자 생성 실패
    /// - 디렉토리 접근 실패
    pub fn start(&mut self) -> Result<()> {
        if self.watcher.is_some() {
            return Err(DictError::Format(
                "File watcher already started".to_string(),
            ));
        }

        let (tx, rx) = bounded(100);
        let (stop_tx, stop_rx) = bounded(1);

        // notify 감시자 생성
        let mut watcher = RecommendedWatcher::new(
            tx,
            notify::Config::default()
                .with_poll_interval(Duration::from_millis(self.config.debounce_ms)),
        )
        .map_err(|e| DictError::Format(format!("Failed to create watcher: {e}")))?;

        // 사전 디렉토리 감시
        let dicdir = self.dict.dicdir();
        let recursive_mode = if self.config.recursive {
            RecursiveMode::Recursive
        } else {
            RecursiveMode::NonRecursive
        };

        watcher
            .watch(dicdir, recursive_mode)
            .map_err(|e| DictError::Format(format!("Failed to watch directory: {e}")))?;

        self.watcher = Some(watcher);
        self.event_rx = Some(rx);
        self.stop_tx = Some(stop_tx);

        // 워커 스레드 시작
        self.start_worker(stop_rx)?;

        Ok(())
    }

    /// 파일 감시 중지
    pub fn stop(&mut self) -> Result<()> {
        if let Some(stop_tx) = self.stop_tx.take() {
            let _ = stop_tx.send(());
        }

        if let Some(handle) = self.worker_handle.take() {
            let _ = handle.join();
        }

        self.watcher = None;
        self.event_rx = None;

        Ok(())
    }

    /// 감시 중인지 확인
    pub fn is_watching(&self) -> bool {
        self.watcher.is_some()
    }

    /// 워커 스레드 시작
    fn start_worker(&mut self, stop_rx: Receiver<()>) -> Result<()> {
        let event_rx = self
            .event_rx
            .as_ref()
            .ok_or_else(|| DictError::Format("Event receiver not initialized".to_string()))?;

        let dict = Arc::clone(&self.dict);
        let config = self.config.clone();
        let rx = event_rx.clone();

        let handle = thread::spawn(move || {
            Self::worker_loop(dict, config, rx, stop_rx);
        });

        self.worker_handle = Some(handle);

        Ok(())
    }

    /// 워커 루프
    fn worker_loop(
        dict: Arc<HotReloadDictionary>,
        config: WatchConfig,
        event_rx: Receiver<notify::Result<Event>>,
        stop_rx: Receiver<()>,
    ) {
        loop {
            // 종료 신호 확인
            if stop_rx.try_recv().is_ok() {
                break;
            }

            // 이벤트 수신 (타임아웃 설정)
            match event_rx.recv_timeout(Duration::from_millis(100)) {
                Ok(Ok(event)) => {
                    Self::handle_event(&dict, &config, event);
                }
                Ok(Err(e)) => {
                    eprintln!("File watcher error: {e}");
                }
                Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                    // 타임아웃은 정상
                    continue;
                }
                Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
                    // 채널 닫힘
                    break;
                }
            }
        }
    }

    /// 이벤트 처리
    fn handle_event(dict: &Arc<HotReloadDictionary>, config: &WatchConfig, event: Event) {
        match event.kind {
            EventKind::Create(_) | EventKind::Modify(_) => {
                for path in event.paths {
                    if config.should_watch(&path) {
                        Self::reload_dictionary(dict, &path);
                    }
                }
            }
            EventKind::Remove(_) => {
                // 파일 삭제는 무시 (기존 사전 유지)
            }
            _ => {
                // 기타 이벤트는 무시
            }
        }
    }

    /// 사전 리로드
    fn reload_dictionary(dict: &Arc<HotReloadDictionary>, path: &Path) {
        if let Some(filename) = path.file_name() {
            let filename_str = filename.to_string_lossy();

            // 시스템 사전 파일 변경 시
            if filename_str.contains("sys.dic")
                || filename_str.contains("matrix")
                || filename_str.ends_with(".zst")
            {
                match dict.reload_system_dict() {
                    Ok(version) => {
                        println!("Dictionary reloaded successfully (version {version})");
                    }
                    Err(e) => {
                        eprintln!("Failed to reload dictionary: {e}");
                    }
                }
            }
        }
    }
}

impl Drop for FileWatcher {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_watch_config_default() {
        let config = WatchConfig::default();
        assert_eq!(config.debounce_ms, 300);
        assert!(!config.recursive);
        assert!(config.watch_extensions.contains(&"dic".to_string()));
    }

    #[test]
    fn test_watch_config_builder() {
        let config = WatchConfig::default()
            .debounce_ms(500)
            .recursive(true)
            .watch_extension("txt")
            .ignore_pattern(".bak");

        assert_eq!(config.debounce_ms, 500);
        assert!(config.recursive);
        assert!(config.watch_extensions.contains(&"txt".to_string()));
        assert!(config.ignore_patterns.contains(&".bak".to_string()));
    }

    #[test]
    fn test_should_watch() {
        let config = WatchConfig::default();

        assert!(config.should_watch(Path::new("test.dic")));
        assert!(config.should_watch(Path::new("matrix.bin")));
        assert!(config.should_watch(Path::new("user.csv")));
        assert!(!config.should_watch(Path::new("test.txt")));
        assert!(!config.should_watch(Path::new("test.dic~")));
        assert!(!config.should_watch(Path::new(".test.dic.swp")));
    }

    #[test]
    fn test_file_event_types() {
        let created = FileEvent::Created(PathBuf::from("test.dic"));
        let modified = FileEvent::Modified(PathBuf::from("test.dic"));
        let deleted = FileEvent::Deleted(PathBuf::from("test.dic"));
        let renamed = FileEvent::Renamed {
            from: PathBuf::from("old.dic"),
            to: PathBuf::from("new.dic"),
        };

        match created {
            FileEvent::Created(_) => {}
            _ => panic!("should be Created"),
        }

        match modified {
            FileEvent::Modified(_) => {}
            _ => panic!("should be Modified"),
        }

        match deleted {
            FileEvent::Deleted(_) => {}
            _ => panic!("should be Deleted"),
        }

        match renamed {
            FileEvent::Renamed { .. } => {}
            _ => panic!("should be Renamed"),
        }
    }
}
