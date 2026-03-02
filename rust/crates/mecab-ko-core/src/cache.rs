//! 토큰화 캐싱
//!
//! 반복되는 입력에 대해 토큰화 결과를 캐싱하여 성능을 향상시킵니다.
//!
//! # 특징
//!
//! - **LRU 캐시**: Least Recently Used 방식으로 오래된 항목 자동 제거
//! - **스레드 안전**: `RwLock` 기반 동시 접근 지원
//! - **통계 추적**: 히트/미스 비율 모니터링
//!
//! # 예제
//!
//! ```rust,no_run
//! use mecab_ko_core::cache::{TokenCache, CacheConfig};
//!
//! let config = CacheConfig::default();
//! let cache = TokenCache::new(config);
//!
//! // 캐시 키 생성 (문자열 해시)
//! let key = cache.make_key("안녕하세요");
//!
//! // 캐시 조회 또는 계산
//! let tokens = cache.get_or_insert(key, || {
//!     vec![] // 실제로는 토큰화 수행
//! });
//!
//! // 통계 확인
//! let stats = cache.stats();
//! println!("Hit rate: {:.1}%", stats.hit_rate() * 100.0);
//! ```

use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;

/// 캐시된 토큰 정보
#[derive(Debug, Clone)]
pub struct CachedToken {
    /// 표면형
    pub surface: String,
    /// 품사 태그
    pub pos: String,
    /// 시작 바이트 위치
    pub start_byte: usize,
    /// 끝 바이트 위치
    pub end_byte: usize,
}

/// 캐시 키 타입
pub type CacheKey = u64;

/// 캐시 설정
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// 최대 캐시 항목 수
    pub max_entries: usize,
    /// 최대 키 길이 (바이트)
    pub max_key_length: usize,
    /// 통계 추적 활성화
    pub track_stats: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 10_000,
            max_key_length: 1024,
            track_stats: true,
        }
    }
}

impl CacheConfig {
    /// 새 설정 생성
    #[must_use]
    pub const fn new() -> Self {
        Self {
            max_entries: 10_000,
            max_key_length: 1024,
            track_stats: true,
        }
    }

    /// 최대 항목 수 설정
    #[must_use]
    pub const fn with_max_entries(mut self, max: usize) -> Self {
        self.max_entries = max;
        self
    }

    /// 최대 키 길이 설정
    #[must_use]
    pub const fn with_max_key_length(mut self, max: usize) -> Self {
        self.max_key_length = max;
        self
    }

    /// 통계 추적 설정
    #[must_use]
    pub const fn with_track_stats(mut self, track: bool) -> Self {
        self.track_stats = track;
        self
    }
}

/// 캐시 통계
#[derive(Debug, Default)]
pub struct CacheStats {
    /// 캐시 히트 횟수
    hits: AtomicU64,
    /// 캐시 미스 횟수
    misses: AtomicU64,
    /// 제거된 항목 수
    evictions: AtomicU64,
}

impl CacheStats {
    /// 히트 횟수
    #[must_use]
    pub fn hits(&self) -> u64 {
        self.hits.load(Ordering::Relaxed)
    }

    /// 미스 횟수
    #[must_use]
    pub fn misses(&self) -> u64 {
        self.misses.load(Ordering::Relaxed)
    }

    /// 총 요청 횟수
    #[must_use]
    pub fn total_requests(&self) -> u64 {
        self.hits() + self.misses()
    }

    /// 히트율 (0.0 ~ 1.0)
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn hit_rate(&self) -> f64 {
        let total = self.total_requests();
        if total == 0 {
            0.0
        } else {
            self.hits() as f64 / total as f64
        }
    }

    /// 제거된 항목 수
    #[must_use]
    pub fn evictions(&self) -> u64 {
        self.evictions.load(Ordering::Relaxed)
    }

    fn record_hit(&self) {
        self.hits.fetch_add(1, Ordering::Relaxed);
    }

    fn record_miss(&self) {
        self.misses.fetch_add(1, Ordering::Relaxed);
    }

    fn record_eviction(&self) {
        self.evictions.fetch_add(1, Ordering::Relaxed);
    }

    /// 통계 리셋
    pub fn reset(&self) {
        self.hits.store(0, Ordering::Relaxed);
        self.misses.store(0, Ordering::Relaxed);
        self.evictions.store(0, Ordering::Relaxed);
    }
}

/// LRU 캐시 항목
struct CacheEntry {
    /// 캐시된 토큰들
    tokens: Vec<CachedToken>,
    /// 마지막 접근 시간 (순서 카운터)
    last_access: u64,
}

/// 토큰화 캐시
pub struct TokenCache {
    config: CacheConfig,
    entries: RwLock<HashMap<CacheKey, CacheEntry>>,
    stats: CacheStats,
    access_counter: AtomicU64,
}

impl TokenCache {
    /// 새 캐시 생성
    #[must_use]
    pub fn new(config: CacheConfig) -> Self {
        Self {
            config,
            entries: RwLock::new(HashMap::new()),
            stats: CacheStats::default(),
            access_counter: AtomicU64::new(0),
        }
    }

    /// 기본 설정으로 캐시 생성
    #[must_use]
    pub fn with_defaults() -> Self {
        Self::new(CacheConfig::default())
    }

    /// 문자열에서 캐시 키 생성
    #[must_use]
    pub fn make_key(&self, text: &str) -> CacheKey {
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        hasher.finish()
    }

    /// 캐시에서 조회
    #[must_use]
    pub fn get(&self, key: CacheKey) -> Option<Vec<CachedToken>> {
        let mut entries = self.entries.write().ok()?;

        if let Some(entry) = entries.get_mut(&key) {
            entry.last_access = self.access_counter.fetch_add(1, Ordering::Relaxed);
            if self.config.track_stats {
                self.stats.record_hit();
            }
            Some(entry.tokens.clone())
        } else {
            if self.config.track_stats {
                self.stats.record_miss();
            }
            None
        }
    }

    /// 캐시에 저장
    pub fn insert(&self, key: CacheKey, tokens: Vec<CachedToken>) {
        let Ok(mut entries) = self.entries.write() else {
            return;
        };

        // 캐시 용량 초과 시 LRU 제거
        while entries.len() >= self.config.max_entries {
            self.evict_lru(&mut entries);
        }

        let access = self.access_counter.fetch_add(1, Ordering::Relaxed);
        entries.insert(key, CacheEntry {
            tokens,
            last_access: access,
        });
    }

    /// 캐시 조회 또는 계산 후 삽입
    pub fn get_or_insert<F>(&self, key: CacheKey, compute: F) -> Vec<CachedToken>
    where
        F: FnOnce() -> Vec<CachedToken>,
    {
        // 먼저 읽기 시도
        if let Some(tokens) = self.get(key) {
            return tokens;
        }

        // 없으면 계산 후 삽입
        let tokens = compute();
        self.insert(key, tokens.clone());
        tokens
    }

    /// 캐시에서 텍스트로 조회 또는 계산 후 삽입
    pub fn get_or_insert_with_text<F>(&self, text: &str, compute: F) -> Vec<CachedToken>
    where
        F: FnOnce() -> Vec<CachedToken>,
    {
        // 너무 긴 텍스트는 캐시하지 않음
        if text.len() > self.config.max_key_length {
            return compute();
        }

        let key = self.make_key(text);
        self.get_or_insert(key, compute)
    }

    /// LRU 항목 제거
    fn evict_lru(&self, entries: &mut HashMap<CacheKey, CacheEntry>) {
        if entries.is_empty() {
            return;
        }

        // 가장 오래된 항목 찾기
        let oldest_key = entries
            .iter()
            .min_by_key(|(_, entry)| entry.last_access)
            .map(|(key, _)| *key);

        if let Some(key) = oldest_key {
            entries.remove(&key);
            if self.config.track_stats {
                self.stats.record_eviction();
            }
        }
    }

    /// 캐시 전체 삭제
    pub fn clear(&self) {
        if let Ok(mut entries) = self.entries.write() {
            entries.clear();
        }
    }

    /// 현재 캐시 항목 수
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.read().map(|e| e.len()).unwrap_or(0)
    }

    /// 캐시가 비어있는지 확인
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// 캐시 통계 참조
    #[must_use]
    pub const fn stats(&self) -> &CacheStats {
        &self.stats
    }

    /// 설정 참조
    #[must_use]
    pub const fn config(&self) -> &CacheConfig {
        &self.config
    }
}

impl Default for TokenCache {
    fn default() -> Self {
        Self::with_defaults()
    }
}

/// 캐싱 가능한 토크나이저 래퍼
pub struct CachingTokenizer<T> {
    inner: T,
    cache: TokenCache,
}

impl<T> CachingTokenizer<T> {
    /// 새 캐싱 토크나이저 생성
    pub fn new(inner: T, config: CacheConfig) -> Self {
        Self {
            inner,
            cache: TokenCache::new(config),
        }
    }

    /// 기본 캐시 설정으로 생성
    pub fn with_defaults(inner: T) -> Self {
        Self::new(inner, CacheConfig::default())
    }

    /// 내부 토크나이저 참조
    #[must_use]
    pub const fn inner(&self) -> &T {
        &self.inner
    }

    /// 내부 토크나이저 가변 참조
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    /// 캐시 참조
    #[must_use]
    pub const fn cache(&self) -> &TokenCache {
        &self.cache
    }

    /// 캐시 통계
    #[must_use]
    pub const fn stats(&self) -> &CacheStats {
        self.cache.stats()
    }

    /// 캐시 삭제
    pub fn clear_cache(&self) {
        self.cache.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_config_default() {
        let config = CacheConfig::default();
        assert_eq!(config.max_entries, 10_000);
        assert_eq!(config.max_key_length, 1024);
        assert!(config.track_stats);
    }

    #[test]
    fn test_cache_config_builder() {
        let config = CacheConfig::new()
            .with_max_entries(1000)
            .with_max_key_length(512)
            .with_track_stats(false);

        assert_eq!(config.max_entries, 1000);
        assert_eq!(config.max_key_length, 512);
        assert!(!config.track_stats);
    }

    #[test]
    fn test_cache_basic_operations() {
        let cache = TokenCache::with_defaults();

        let key = cache.make_key("테스트");

        // 처음에는 없음
        assert!(cache.get(key).is_none());
        assert_eq!(cache.stats().misses(), 1);

        // 삽입
        let tokens = vec![CachedToken {
            surface: "테스트".to_string(),
            pos: "NNG".to_string(),
            start_byte: 0,
            end_byte: 9,
        }];
        cache.insert(key, tokens.clone());

        // 조회
        let cached = cache.get(key).unwrap();
        assert_eq!(cached.len(), 1);
        assert_eq!(cached[0].surface, "테스트");
        assert_eq!(cache.stats().hits(), 1);
    }

    #[test]
    fn test_cache_get_or_insert() {
        let cache = TokenCache::with_defaults();

        let key = cache.make_key("안녕");
        let mut call_count = 0;

        // 첫 번째 호출 - compute 실행
        let tokens1 = cache.get_or_insert(key, || {
            call_count += 1;
            vec![CachedToken {
                surface: "안녕".to_string(),
                pos: "IC".to_string(),
                start_byte: 0,
                end_byte: 6,
            }]
        });
        assert_eq!(call_count, 1);
        assert_eq!(tokens1.len(), 1);

        // 두 번째 호출 - 캐시에서 반환
        let tokens2 = cache.get_or_insert(key, || {
            call_count += 1;
            vec![]
        });
        assert_eq!(call_count, 1); // compute가 호출되지 않음
        assert_eq!(tokens2.len(), 1);
    }

    #[test]
    fn test_cache_lru_eviction() {
        let config = CacheConfig::new().with_max_entries(3);
        let cache = TokenCache::new(config);

        // 3개 삽입
        for i in 0..3 {
            let key = cache.make_key(&format!("text{i}"));
            cache.insert(key, vec![]);
        }
        assert_eq!(cache.len(), 3);

        // 첫 번째 항목 접근 (LRU 갱신)
        let key0 = cache.make_key("text0");
        let _ = cache.get(key0);

        // 4번째 삽입 시 text1이 제거됨 (가장 오래됨)
        let key3 = cache.make_key("text3");
        cache.insert(key3, vec![]);
        assert_eq!(cache.len(), 3);
        assert_eq!(cache.stats().evictions(), 1);

        // text0은 여전히 존재 (최근 접근)
        assert!(cache.get(key0).is_some());

        // text1은 제거됨
        let key1 = cache.make_key("text1");
        assert!(cache.get(key1).is_none());
    }

    #[test]
    fn test_cache_stats() {
        let cache = TokenCache::with_defaults();

        let key = cache.make_key("test");

        // 미스
        let _ = cache.get(key);
        assert_eq!(cache.stats().misses(), 1);
        assert_eq!(cache.stats().hits(), 0);
        assert!((cache.stats().hit_rate() - 0.0).abs() < f64::EPSILON);

        // 삽입 후 히트
        cache.insert(key, vec![]);
        let _ = cache.get(key);
        assert_eq!(cache.stats().hits(), 1);
        assert!((cache.stats().hit_rate() - 0.5).abs() < f64::EPSILON);

        // 리셋
        cache.stats().reset();
        assert_eq!(cache.stats().total_requests(), 0);
    }

    #[test]
    fn test_cache_clear() {
        let cache = TokenCache::with_defaults();

        for i in 0..10 {
            let key = cache.make_key(&format!("text{i}"));
            cache.insert(key, vec![]);
        }
        assert_eq!(cache.len(), 10);

        cache.clear();
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_skip_long_text() {
        let config = CacheConfig::new().with_max_key_length(10);
        let cache = TokenCache::new(config);

        let mut call_count = 0;

        // 짧은 텍스트 - 캐시됨
        let short = "짧은";
        cache.get_or_insert_with_text(short, || {
            call_count += 1;
            vec![]
        });
        cache.get_or_insert_with_text(short, || {
            call_count += 1;
            vec![]
        });
        assert_eq!(call_count, 1);

        // 긴 텍스트 - 캐시되지 않음
        let long = "이것은 아주 긴 텍스트입니다";
        cache.get_or_insert_with_text(long, || {
            call_count += 1;
            vec![]
        });
        cache.get_or_insert_with_text(long, || {
            call_count += 1;
            vec![]
        });
        assert_eq!(call_count, 3); // 매번 compute 호출
    }

    #[test]
    fn test_caching_tokenizer() {
        struct DummyTokenizer;

        let caching = CachingTokenizer::with_defaults(DummyTokenizer);

        assert!(caching.cache().is_empty());
        assert_eq!(caching.stats().total_requests(), 0);

        // 캐시에 항목 추가
        let key = caching.cache().make_key("test");
        caching.cache().insert(key, vec![]);

        assert_eq!(caching.cache().len(), 1);

        // 캐시 삭제
        caching.clear_cache();
        assert!(caching.cache().is_empty());
    }
}
