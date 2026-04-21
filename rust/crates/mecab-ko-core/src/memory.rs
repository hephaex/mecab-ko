//! # Memory Optimization Module
//!
//! 메모리 사용량 측정 및 최적화 유틸리티
//!
//! ## 주요 기능
//!
//! - **POS Tag Interning**: 품사 태그 문자열 중복 제거
//! - **Memory Stats**: 메모리 사용량 측정
//! - **Feature Deduplication**: Feature 문자열 중복 제거
//!
//! ## Example
//!
//! ```rust
//! use mecab_ko_core::memory::{PosTagInterner, MemoryStats};
//!
//! let interner = PosTagInterner::new();
//! let sym = interner.intern("NNG");
//! assert_eq!(interner.resolve(sym), Some("NNG".to_string()));
//!
//! let stats = MemoryStats::default();
//! println!("Memory: {} bytes", stats.estimate_total());
//! ```

use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

use parking_lot::RwLock;

/// 품사 태그 인터너
///
/// `MeCab` 품사 태그는 약 45개로 제한되어 있어 인터닝에 적합합니다.
/// 스레드 안전하며 여러 토크나이저에서 공유 가능합니다.
#[derive(Debug)]
pub struct PosTagInterner {
    /// 품사 태그 → 인덱스 매핑
    tags: RwLock<HashMap<String, u16>>,
    /// 인덱스 → 품사 태그 매핑 (역방향)
    reverse: RwLock<Vec<String>>,
    /// 통계: intern 호출 횟수
    intern_count: AtomicUsize,
    /// 통계: 캐시 히트 횟수
    hit_count: AtomicUsize,
}

impl PosTagInterner {
    /// 새 인터너 생성
    ///
    /// 일반적인 품사 태그를 사전 등록합니다.
    #[must_use]
    pub fn new() -> Self {
        let interner = Self {
            tags: RwLock::new(HashMap::with_capacity(64)),
            reverse: RwLock::new(Vec::with_capacity(64)),
            intern_count: AtomicUsize::new(0),
            hit_count: AtomicUsize::new(0),
        };

        // 일반적인 품사 태그 사전 등록
        for tag in COMMON_POS_TAGS {
            interner.intern(tag);
        }

        interner
    }

    /// 품사 태그 인터닝
    ///
    /// 이미 존재하면 기존 인덱스 반환, 새로우면 등록 후 인덱스 반환
    #[allow(clippy::significant_drop_tightening)]
    pub fn intern(&self, tag: &str) -> u16 {
        self.intern_count.fetch_add(1, Ordering::Relaxed);

        // 읽기 잠금으로 먼저 확인
        {
            let tags = self.tags.read();
            if let Some(&idx) = tags.get(tag) {
                self.hit_count.fetch_add(1, Ordering::Relaxed);
                return idx;
            }
        }

        // 없으면 쓰기 잠금으로 추가
        let mut tags = self.tags.write();
        let mut reverse = self.reverse.write();

        // Double-check after acquiring write lock
        if let Some(&idx) = tags.get(tag) {
            self.hit_count.fetch_add(1, Ordering::Relaxed);
            return idx;
        }

        let idx = u16::try_from(reverse.len()).unwrap_or(u16::MAX);
        tags.insert(tag.to_string(), idx);
        reverse.push(tag.to_string());
        idx
    }

    /// 인덱스로 품사 태그 조회
    #[must_use]
    pub fn resolve(&self, idx: u16) -> Option<String> {
        let reverse = self.reverse.read();
        reverse.get(idx as usize).cloned()
    }

    /// 인덱스로 품사 태그 참조 (복사 없이)
    pub fn resolve_ref<F, R>(&self, idx: u16, f: F) -> Option<R>
    where
        F: FnOnce(&str) -> R,
    {
        let reverse = self.reverse.read();
        reverse.get(idx as usize).map(|s| f(s.as_str()))
    }

    /// 등록된 품사 태그 수
    #[must_use]
    pub fn len(&self) -> usize {
        self.reverse.read().len()
    }

    /// 비어있는지 확인
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.reverse.read().is_empty()
    }

    /// 통계 정보
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn stats(&self) -> InternerStats {
        let intern_count = self.intern_count.load(Ordering::Relaxed);
        let hit_count = self.hit_count.load(Ordering::Relaxed);
        InternerStats {
            unique_tags: self.len(),
            intern_calls: intern_count,
            cache_hits: hit_count,
            hit_rate: if intern_count > 0 {
                hit_count as f64 / intern_count as f64
            } else {
                0.0
            },
        }
    }

    /// 메모리 사용량 추정 (바이트)
    #[must_use]
    #[allow(clippy::significant_drop_tightening)]
    pub fn memory_usage(&self) -> usize {
        let reverse = self.reverse.read();
        let tags = self.tags.read();

        // Vec capacity
        let vec_overhead = reverse.capacity() * std::mem::size_of::<String>();
        // String contents
        let string_bytes: usize = reverse.iter().map(String::len).sum();
        // HashMap overhead
        let map_overhead = tags.capacity() * (std::mem::size_of::<String>() + 2);

        vec_overhead + string_bytes + map_overhead
    }
}

impl Default for PosTagInterner {
    fn default() -> Self {
        Self::new()
    }
}

/// 일반적인 품사 태그 (세종 품사 체계 + `MeCab` 확장)
const COMMON_POS_TAGS: &[&str] = &[
    // 체언
    "NNG", "NNP", "NNB", "NR", "NP", // 용언
    "VV", "VA", "VX", "VCP", "VCN", // 수식언
    "MM", "MAG", "MAJ", // 독립언
    "IC",  // 관계언
    "JKS", "JKC", "JKG", "JKO", "JKB", "JKV", "JKQ", "JX", "JC", // 의존형태
    "EP", "EF", "EC", "ETN", "ETM", "XPN", "XSN", "XSV", "XSA", "XR", // 기호
    "SF", "SE", "SS", "SP", "SO", "SL", "SH", "SN", "SW", // 분석 불능
    "NA", // Unknown
    "UNK", "UNKNOWN", // 기타 확장
    "*", "NNBC",
];

/// 인터너 통계
#[derive(Debug, Clone, Copy)]
pub struct InternerStats {
    /// 고유 태그 수
    pub unique_tags: usize,
    /// intern 호출 횟수
    pub intern_calls: usize,
    /// 캐시 히트 횟수
    pub cache_hits: usize,
    /// 캐시 히트율
    pub hit_rate: f64,
}

impl InternerStats {
    /// 통계를 문자열로 포맷
    #[must_use]
    pub fn format(&self) -> String {
        format!(
            "POS Interner: {} unique tags, {} calls, {:.1}% hit rate",
            self.unique_tags,
            self.intern_calls,
            self.hit_rate * 100.0
        )
    }
}

/// 메모리 사용량 통계
#[derive(Debug, Clone, Default)]
pub struct MemoryStats {
    /// 사전 메모리 (바이트)
    pub dictionary_bytes: usize,
    /// Lattice 메모리 (바이트)
    pub lattice_bytes: usize,
    /// 풀 메모리 (바이트)
    pub pool_bytes: usize,
    /// 캐시 메모리 (바이트)
    pub cache_bytes: usize,
    /// 인터너 메모리 (바이트)
    pub interner_bytes: usize,
    /// 토큰 메모리 (바이트)
    pub token_bytes: usize,
}

impl MemoryStats {
    /// 총 메모리 추정
    #[must_use]
    pub const fn estimate_total(&self) -> usize {
        self.dictionary_bytes
            + self.lattice_bytes
            + self.pool_bytes
            + self.cache_bytes
            + self.interner_bytes
            + self.token_bytes
    }

    /// 사람이 읽기 좋은 형식으로 포맷
    #[must_use]
    pub fn format_human_readable(&self) -> String {
        format!(
            "Memory Usage:\n\
             - Dictionary: {} KB\n\
             - Lattice: {} KB\n\
             - Pool: {} KB\n\
             - Cache: {} KB\n\
             - Interner: {} KB\n\
             - Tokens: {} KB\n\
             - Total: {} KB",
            self.dictionary_bytes / 1024,
            self.lattice_bytes / 1024,
            self.pool_bytes / 1024,
            self.cache_bytes / 1024,
            self.interner_bytes / 1024,
            self.token_bytes / 1024,
            self.estimate_total() / 1024
        )
    }
}

/// Feature 문자열 중복 제거 캐시
///
/// Feature 문자열은 품사 태그보다 다양하지만,
/// 동일 품사의 엔트리들은 비슷한 feature를 공유합니다.
#[derive(Debug)]
pub struct FeatureCache {
    /// Feature → 인덱스
    features: RwLock<HashMap<String, u32>>,
    /// 인덱스 → Feature
    reverse: RwLock<Vec<String>>,
    /// 최대 캐시 크기
    max_size: usize,
}

impl FeatureCache {
    /// 새 캐시 생성
    #[must_use]
    pub fn new(max_size: usize) -> Self {
        Self {
            features: RwLock::new(HashMap::with_capacity(max_size.min(10000))),
            reverse: RwLock::new(Vec::with_capacity(max_size.min(10000))),
            max_size,
        }
    }

    /// Feature 인터닝
    ///
    /// 캐시가 가득 차면 새 feature는 인터닝하지 않고 None 반환
    #[allow(clippy::significant_drop_tightening)]
    pub fn intern(&self, feature: &str) -> Option<u32> {
        // 읽기 잠금으로 먼저 확인
        {
            let features = self.features.read();
            if let Some(&idx) = features.get(feature) {
                return Some(idx);
            }
        }

        // 캐시 크기 확인
        let len = self.reverse.read().len();
        if len >= self.max_size {
            return None;
        }

        // 쓰기 잠금으로 추가
        let mut features = self.features.write();
        let mut reverse = self.reverse.write();

        if let Some(&idx) = features.get(feature) {
            return Some(idx);
        }

        if reverse.len() >= self.max_size {
            return None;
        }

        let idx = u32::try_from(reverse.len()).ok()?;
        features.insert(feature.to_string(), idx);
        reverse.push(feature.to_string());
        Some(idx)
    }

    /// 인덱스로 Feature 조회
    #[must_use]
    pub fn resolve(&self, idx: u32) -> Option<String> {
        self.reverse.read().get(idx as usize).cloned()
    }

    /// 캐시 크기
    #[must_use]
    pub fn len(&self) -> usize {
        self.reverse.read().len()
    }

    /// 비어있는지 확인
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.reverse.read().is_empty()
    }

    /// 메모리 사용량 (바이트)
    #[must_use]
    #[allow(clippy::significant_drop_tightening)]
    pub fn memory_usage(&self) -> usize {
        let reverse = self.reverse.read();
        let features = self.features.read();

        let vec_bytes: usize = reverse.iter().map(String::len).sum();
        let map_overhead = features.capacity() * (std::mem::size_of::<String>() + 4);

        vec_bytes + map_overhead
    }
}

impl Default for FeatureCache {
    fn default() -> Self {
        Self::new(50000)
    }
}

/// 토큰 메모리 사용량 추정
///
/// 토큰 벡터의 메모리 사용량을 추정합니다.
#[must_use]
pub fn estimate_tokens_memory(tokens: &[crate::tokenizer::Token]) -> usize {
    let base_size = std::mem::size_of_val(tokens);
    let string_bytes: usize = tokens
        .iter()
        .map(|t| {
            t.surface.len()
                + t.pos.len()
                + t.features.len()
                + t.reading.as_ref().map_or(0, String::len)
                + t.lemma.as_ref().map_or(0, String::len)
                + t.normalized.as_ref().map_or(0, String::len)
        })
        .sum();

    base_size + string_bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pos_tag_interner() {
        let interner = PosTagInterner::new();

        // 기본 태그는 이미 등록됨
        let idx1 = interner.intern("NNG");
        let idx2 = interner.intern("NNG");
        assert_eq!(idx1, idx2);

        // 새 태그 등록
        let idx3 = interner.intern("CUSTOM_TAG");
        assert_ne!(idx1, idx3);

        // 해석
        assert_eq!(interner.resolve(idx1), Some("NNG".to_string()));
        assert_eq!(interner.resolve(idx3), Some("CUSTOM_TAG".to_string()));
    }

    #[test]
    fn test_pos_interner_stats() {
        let interner = PosTagInterner::new();

        // 여러 번 호출
        for _ in 0..100 {
            interner.intern("NNG");
            interner.intern("VV");
        }

        let stats = interner.stats();
        assert!(stats.unique_tags > 0);
        assert!(stats.intern_calls > 200); // 초기화 + 200
                                           // 초기화 시 ~45개 태그가 미스로 카운트되므로 히트율은 ~0.8
        assert!(stats.hit_rate > 0.75, "hit_rate: {}", stats.hit_rate);
    }

    #[test]
    fn test_feature_cache() {
        let cache = FeatureCache::new(100);

        let idx1 = cache.intern("NNG,*,T,테스트,*,*,*,*");
        assert!(idx1.is_some());

        let idx2 = cache.intern("NNG,*,T,테스트,*,*,*,*");
        assert_eq!(idx1, idx2);

        assert_eq!(
            cache.resolve(idx1.unwrap()),
            Some("NNG,*,T,테스트,*,*,*,*".to_string())
        );
    }

    #[test]
    fn test_feature_cache_max_size() {
        let cache = FeatureCache::new(2);

        assert!(cache.intern("feature1").is_some());
        assert!(cache.intern("feature2").is_some());
        // 캐시가 가득 차면 새 항목은 추가되지 않음
        assert!(cache.intern("feature3").is_none());
    }

    #[test]
    fn test_memory_stats_format() {
        let stats = MemoryStats {
            dictionary_bytes: 100 * 1024,
            lattice_bytes: 10 * 1024,
            pool_bytes: 5 * 1024,
            cache_bytes: 20 * 1024,
            interner_bytes: 1024,
            token_bytes: 2 * 1024,
        };

        let formatted = stats.format_human_readable();
        assert!(formatted.contains("Dictionary: 100 KB"));
        assert!(formatted.contains("Total: 138 KB"));
    }

    #[test]
    fn test_common_pos_tags_preloaded() {
        let interner = PosTagInterner::new();

        // 일반적인 태그는 이미 로드됨
        assert!(interner.len() > 30);

        // 모든 기본 태그가 등록되어 있어야 함
        for tag in COMMON_POS_TAGS {
            let idx = interner.intern(tag);
            assert!(idx < 100);
        }
    }
}
