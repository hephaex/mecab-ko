//! # Memory Pooling Module
//!
//! 메모리 할당 오버헤드를 줄이기 위한 객체 풀링 시스템
//!
//! ## 주요 기능
//!
//! - **Token Pool**: Token 객체 재사용
//! - **Node Pool**: Lattice Node 재사용
//! - **String Interning**: 중복 문자열 제거
//!
//! ## 설계 원칙
//!
//! 1. **Zero-Copy 우선**: 가능한 한 복사를 피함
//! 2. **Thread-Safe**: 멀티스레드 환경 지원
//! 3. **Memory Bounds**: 메모리 사용량 제한
//! 4. **Fast Path**: 일반적인 경우 최소 오버헤드
//!
//! ## Example
//!
//! ```rust,ignore
//! use mecab_ko_core::pool::{TokenPool, StringInterner};
//!
//! let mut pool = TokenPool::new();
//! let mut interner = StringInterner::new();
//!
//! // Token 빌림
//! let mut token = pool.acquire();
//! token.surface = interner.intern("안녕").to_string();
//! token.pos = interner.intern("NNG").to_string();
//!
//! // 사용 후 자동 반환 (Drop)
//! drop(token);
//!
//! // 재사용
//! let token2 = pool.acquire();
//! ```

use parking_lot::Mutex;
use std::cell::RefCell;
use std::sync::Arc;
use string_interner::{DefaultBackend, DefaultSymbol, StringInterner as Interner};

use crate::lattice::Node;
use crate::tokenizer::Token;

/// String interning을 위한 심볼 타입
pub type Symbol = DefaultSymbol;

/// 스레드 안전한 문자열 인터너
///
/// 중복 문자열을 제거하여 메모리 사용량을 줄입니다.
/// 품사 태그, feature 문자열 등 반복되는 문자열에 효과적입니다.
#[derive(Debug, Clone)]
pub struct SharedStringInterner {
    interner: Arc<Mutex<Interner<DefaultBackend>>>,
}

impl SharedStringInterner {
    /// 새 인터너 생성
    #[must_use]
    pub fn new() -> Self {
        Self {
            interner: Arc::new(Mutex::new(Interner::new())),
        }
    }

    /// 문자열 intern
    ///
    /// 이미 존재하는 문자열이면 기존 참조를 반환하고,
    /// 새로운 문자열이면 저장 후 참조를 반환합니다.
    #[must_use]
    pub fn intern(&self, s: &str) -> Symbol {
        self.interner.lock().get_or_intern(s)
    }

    /// 심볼을 문자열로 해석
    #[must_use]
    pub fn resolve(&self, symbol: Symbol) -> Option<String> {
        self.interner
            .lock()
            .resolve(symbol)
            .map(ToString::to_string)
    }

    /// 저장된 문자열 개수
    #[must_use]
    pub fn len(&self) -> usize {
        self.interner.lock().len()
    }

    /// 비어있는지 확인
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.interner.lock().is_empty()
    }

    /// 메모리 사용량 (바이트)
    #[must_use]
    pub fn memory_usage(&self) -> usize {
        let interner = self.interner.lock();
        // 대략적인 추정: 문자열 개수 * 평균 길이 + 오버헤드
        interner.len() * 20
    }
}

impl Default for SharedStringInterner {
    fn default() -> Self {
        Self::new()
    }
}

/// Token 객체 풀
///
/// Token 객체를 재사용하여 할당 오버헤드를 줄입니다.
pub struct TokenPool {
    pool: RefCell<Vec<Token>>,
    max_size: usize,
}

impl TokenPool {
    /// 기본 크기로 풀 생성
    #[must_use]
    pub fn new() -> Self {
        Self::with_capacity(128)
    }

    /// 지정된 용량으로 풀 생성
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            pool: RefCell::new(Vec::with_capacity(capacity)),
            max_size: capacity * 2, // 최대 크기는 초기 용량의 2배
        }
    }

    /// 풀에서 Token 획득
    ///
    /// 풀에 사용 가능한 Token이 있으면 재사용하고,
    /// 없으면 새로 생성합니다.
    pub fn acquire(&self) -> Token {
        self.pool
            .borrow_mut()
            .pop()
            .unwrap_or_else(|| Token::new(String::new(), String::new(), 0, 0, 0, 0))
    }

    /// Token을 풀에 반환
    ///
    /// # Arguments
    ///
    /// * `token` - 반환할 Token (내용은 초기화됨)
    pub fn release(&self, mut token: Token) {
        let mut pool = self.pool.borrow_mut();

        // 최대 크기 제한
        if pool.len() >= self.max_size {
            return;
        }

        // Token 초기화
        token.surface.clear();
        token.pos.clear();
        token.start_pos = 0;
        token.end_pos = 0;
        token.start_byte = 0;
        token.end_byte = 0;
        token.reading = None;
        token.lemma = None;
        token.cost = 0;
        token.features.clear();
        token.normalized = None;

        pool.push(token);
    }

    /// 풀의 현재 크기
    pub fn size(&self) -> usize {
        self.pool.borrow().len()
    }

    /// 풀 비우기
    pub fn clear(&self) {
        self.pool.borrow_mut().clear();
    }

    /// 메모리 사용량 추정 (바이트)
    pub fn memory_usage(&self) -> usize {
        // 각 Token의 대략적인 크기
        self.pool.borrow().len() * std::mem::size_of::<Token>()
    }
}

impl Default for TokenPool {
    fn default() -> Self {
        Self::new()
    }
}

/// Node 벡터 풀
///
/// Lattice 구축 시 Node 벡터를 재사용하여 할당을 줄입니다.
pub struct NodeVecPool {
    pool: RefCell<Vec<Vec<Node>>>,
    max_size: usize,
}

impl NodeVecPool {
    /// 기본 크기로 풀 생성
    #[must_use]
    pub fn new() -> Self {
        Self::with_capacity(32)
    }

    /// 지정된 용량으로 풀 생성
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            pool: RefCell::new(Vec::with_capacity(capacity)),
            max_size: capacity * 2,
        }
    }

    /// 풀에서 Node 벡터 획득
    pub fn acquire(&self) -> Vec<Node> {
        self.pool.borrow_mut().pop().unwrap_or_default()
    }

    /// Node 벡터를 풀에 반환
    pub fn release(&self, mut vec: Vec<Node>) {
        let mut pool = self.pool.borrow_mut();

        if pool.len() >= self.max_size {
            return;
        }

        // 벡터 초기화 (용량은 유지)
        vec.clear();

        pool.push(vec);
    }

    /// 풀의 현재 크기
    pub fn size(&self) -> usize {
        self.pool.borrow().len()
    }

    /// 풀 비우기
    pub fn clear(&self) {
        self.pool.borrow_mut().clear();
    }

    /// 메모리 사용량 추정 (바이트)
    pub fn memory_usage(&self) -> usize {
        let pool = self.pool.borrow();
        pool.iter()
            .map(|v| v.capacity() * std::mem::size_of::<Node>())
            .sum()
    }
}

impl Default for NodeVecPool {
    fn default() -> Self {
        Self::new()
    }
}

/// ID 벡터 풀
///
/// Lattice의 starts_at, ends_at 벡터 재사용
pub struct IdVecPool {
    pool: RefCell<Vec<Vec<u32>>>,
    max_size: usize,
}

impl IdVecPool {
    /// 기본 크기로 풀 생성
    #[must_use]
    pub fn new() -> Self {
        Self::with_capacity(64)
    }

    /// 지정된 용량으로 풀 생성
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            pool: RefCell::new(Vec::with_capacity(capacity)),
            max_size: capacity * 2,
        }
    }

    /// 풀에서 ID 벡터 획득
    pub fn acquire(&self) -> Vec<u32> {
        self.pool.borrow_mut().pop().unwrap_or_default()
    }

    /// ID 벡터를 풀에 반환
    pub fn release(&self, mut vec: Vec<u32>) {
        let mut pool = self.pool.borrow_mut();

        if pool.len() >= self.max_size {
            return;
        }

        vec.clear();
        pool.push(vec);
    }

    /// 풀의 현재 크기
    pub fn size(&self) -> usize {
        self.pool.borrow().len()
    }

    /// 풀 비우기
    pub fn clear(&self) {
        self.pool.borrow_mut().clear();
    }
}

impl Default for IdVecPool {
    fn default() -> Self {
        Self::new()
    }
}

/// 통합 메모리 풀 관리자
///
/// 모든 풀을 하나의 인터페이스로 관리합니다.
#[derive(Default)]
pub struct PoolManager {
    /// Token 객체 풀
    pub token_pool: TokenPool,
    /// Node 벡터 풀
    pub node_vec_pool: NodeVecPool,
    /// ID 벡터 풀
    pub id_vec_pool: IdVecPool,
    /// 문자열 인터너
    pub string_interner: SharedStringInterner,
}

impl PoolManager {
    /// 새 풀 관리자 생성
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 통계 정보
    pub fn stats(&self) -> PoolStats {
        PoolStats {
            token_pool_size: self.token_pool.size(),
            node_vec_pool_size: self.node_vec_pool.size(),
            id_vec_pool_size: self.id_vec_pool.size(),
            interned_strings: self.string_interner.len(),
            total_memory: self.total_memory_usage(),
        }
    }

    /// 모든 풀 비우기
    pub fn clear_all(&self) {
        self.token_pool.clear();
        self.node_vec_pool.clear();
        self.id_vec_pool.clear();
    }

    /// 총 메모리 사용량 (바이트)
    pub fn total_memory_usage(&self) -> usize {
        self.token_pool.memory_usage()
            + self.node_vec_pool.memory_usage()
            + self.id_vec_pool.size() * std::mem::size_of::<Vec<u32>>()
            + self.string_interner.memory_usage()
    }
}

/// 풀 통계 정보
#[derive(Debug, Clone, Copy)]
pub struct PoolStats {
    /// Token 풀 크기
    pub token_pool_size: usize,
    /// Node 벡터 풀 크기
    pub node_vec_pool_size: usize,
    /// ID 벡터 풀 크기
    pub id_vec_pool_size: usize,
    /// Interned 문자열 개수
    pub interned_strings: usize,
    /// 총 메모리 사용량 (바이트)
    pub total_memory: usize,
}

impl PoolStats {
    /// 통계를 사람이 읽기 좋은 문자열로 변환
    #[must_use]
    pub fn format_human_readable(&self) -> String {
        format!(
            "Token Pool: {}, Node Vec Pool: {}, ID Vec Pool: {}, Interned Strings: {}, Memory: {} KB",
            self.token_pool_size,
            self.node_vec_pool_size,
            self.id_vec_pool_size,
            self.interned_strings,
            self.total_memory / 1024
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_interner() {
        let interner = SharedStringInterner::new();

        let s1 = interner.intern("NNG");
        let s2 = interner.intern("NNG");
        let s3 = interner.intern("VV");

        // 같은 문자열은 같은 심볼
        assert_eq!(s1, s2);
        assert_ne!(s1, s3);

        // 해석
        assert_eq!(interner.resolve(s1), Some("NNG".to_string()));
        assert_eq!(interner.resolve(s3), Some("VV".to_string()));

        // 개수 확인
        assert_eq!(interner.len(), 2);
    }

    #[test]
    fn test_token_pool() {
        let pool = TokenPool::new();

        // 첫 획득 (새로 생성)
        let token1 = pool.acquire();
        assert_eq!(pool.size(), 0);

        // 반환
        pool.release(token1);
        assert_eq!(pool.size(), 1);

        // 재사용
        let mut token2 = pool.acquire();
        assert_eq!(pool.size(), 0);

        token2.surface = "테스트".to_string();
        pool.release(token2);

        // 초기화 확인
        let token3 = pool.acquire();
        assert!(token3.surface.is_empty());
    }

    #[test]
    fn test_node_vec_pool() {
        let pool = NodeVecPool::new();

        let mut vec1 = pool.acquire();
        assert_eq!(pool.size(), 0);

        // 사용
        vec1.push(Node::bos());
        vec1.push(Node::eos(1, 10, 30));

        // 반환
        pool.release(vec1);
        assert_eq!(pool.size(), 1);

        // 재사용 (초기화 확인)
        let vec2 = pool.acquire();
        assert_eq!(vec2.len(), 0);
    }

    #[test]
    fn test_pool_manager() {
        let manager = PoolManager::new();

        // Token 사용
        let token = manager.token_pool.acquire();
        manager.token_pool.release(token);

        // Node Vec 사용
        let vec = manager.node_vec_pool.acquire();
        manager.node_vec_pool.release(vec);

        // String interning
        let _s1 = manager.string_interner.intern("NNG");
        let _s2 = manager.string_interner.intern("VV");

        // 통계 확인
        let stats = manager.stats();
        assert_eq!(stats.token_pool_size, 1);
        assert_eq!(stats.node_vec_pool_size, 1);
        assert_eq!(stats.interned_strings, 2);
    }

    #[test]
    fn test_pool_max_size() {
        let pool = TokenPool::with_capacity(2);

        // 최대 크기 초과
        for _ in 0..10 {
            let token = pool.acquire();
            pool.release(token);
        }

        // 최대 크기 제한 확인
        assert!(pool.size() <= 4); // max_size = capacity * 2
    }

    #[test]
    fn test_pool_clear() {
        let pool = TokenPool::new();

        // Acquire multiple tokens
        let mut tokens = Vec::new();
        for _ in 0..5 {
            tokens.push(pool.acquire());
        }

        // Release all tokens
        for token in tokens {
            pool.release(token);
        }

        assert_eq!(pool.size(), 5);

        pool.clear();
        assert_eq!(pool.size(), 0);
    }

    #[test]
    fn test_pool_manager_clear_all() {
        let manager = PoolManager::new();

        let token = manager.token_pool.acquire();
        manager.token_pool.release(token);

        let vec = manager.node_vec_pool.acquire();
        manager.node_vec_pool.release(vec);

        assert!(manager.token_pool.size() > 0);
        assert!(manager.node_vec_pool.size() > 0);

        manager.clear_all();

        assert_eq!(manager.token_pool.size(), 0);
        assert_eq!(manager.node_vec_pool.size(), 0);
    }
}
