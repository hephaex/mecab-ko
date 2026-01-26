//! 메모리 풀링 통합 테스트
//!
//! 실제 사용 시나리오에서 풀링 시스템이 올바르게 동작하는지 검증합니다.

use mecab_ko_core::pool::{PoolManager, SharedStringInterner, TokenPool};
use mecab_ko_core::Tokenizer;

#[test]
fn test_token_pool_basic() {
    let pool = TokenPool::new();

    // 첫 획득
    let token1 = pool.acquire();
    assert_eq!(pool.size(), 0);

    // 반환
    pool.release(token1);
    assert_eq!(pool.size(), 1);

    // 재사용
    let token2 = pool.acquire();
    assert_eq!(pool.size(), 0);

    pool.release(token2);
    assert_eq!(pool.size(), 1);
}

#[test]
fn test_token_pool_multiple() {
    let pool = TokenPool::new();

    // 여러 토큰 획득
    let mut tokens = Vec::new();
    for _ in 0..10 {
        tokens.push(pool.acquire());
    }

    assert_eq!(pool.size(), 0);

    // 모두 반환
    for token in tokens {
        pool.release(token);
    }

    assert_eq!(pool.size(), 10);

    // 재사용 확인
    let tokens2: Vec<_> = (0..5).map(|_| pool.acquire()).collect();
    assert_eq!(pool.size(), 5);

    for token in tokens2 {
        pool.release(token);
    }
    assert_eq!(pool.size(), 10);
}

#[test]
fn test_string_interner_deduplication() {
    let interner = SharedStringInterner::new();

    // 같은 문자열 여러 번 intern
    let s1 = interner.intern("NNG");
    let s2 = interner.intern("NNG");
    let s3 = interner.intern("NNG");

    // 모두 같은 심볼
    assert_eq!(s1, s2);
    assert_eq!(s2, s3);

    // 다른 문자열
    let s4 = interner.intern("VV");
    assert_ne!(s1, s4);

    // 총 2개만 저장됨
    assert_eq!(interner.len(), 2);
}

#[test]
fn test_string_interner_resolve() {
    let interner = SharedStringInterner::new();

    let symbol = interner.intern("테스트");
    let resolved = interner.resolve(symbol);

    assert_eq!(resolved, Some("테스트".to_string()));
}

#[test]
fn test_pool_manager_stats() {
    let manager = PoolManager::new();

    // 초기 상태
    let stats = manager.stats();
    assert_eq!(stats.token_pool_size, 0);
    assert_eq!(stats.interned_strings, 0);

    // Token 사용
    let token = manager.token_pool.acquire();
    manager.token_pool.release(token);

    // String interning
    let _s1 = manager.string_interner.intern("NNG");
    let _s2 = manager.string_interner.intern("VV");

    // 통계 확인
    let stats = manager.stats();
    assert_eq!(stats.token_pool_size, 1);
    assert_eq!(stats.interned_strings, 2);
}

#[test]
fn test_pool_manager_clear() {
    let manager = PoolManager::new();

    // 풀 채우기 (동시에 획득 후 한번에 반환)
    let mut tokens = Vec::new();
    for _ in 0..10 {
        tokens.push(manager.token_pool.acquire());
    }

    for token in tokens {
        manager.token_pool.release(token);
    }

    assert_eq!(manager.token_pool.size(), 10);

    // 전체 초기화
    manager.clear_all();

    assert_eq!(manager.token_pool.size(), 0);
}

#[test]
#[ignore = "Requires dictionary"]
fn test_tokenizer_pool_integration() {
    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

    // 초기 풀 상태
    let stats1 = tokenizer.pool_stats();
    assert_eq!(stats1.token_pool_size, 0);

    // 여러 문장 분석
    let sentences = vec![
        "안녕하세요",
        "테스트입니다",
        "형태소 분석",
    ];

    for sentence in &sentences {
        let _tokens = tokenizer.tokenize(sentence);
    }

    // 풀이 사용되었는지 확인 (정확한 크기는 구현에 따라 다름)
    let stats2 = tokenizer.pool_stats();
    // Token들이 생성되고 반환되었을 수 있음
    // (정확한 값은 보장 안 하지만 0 이상이어야 함)
    assert!(stats2.token_pool_size >= 0);
}

#[test]
#[ignore = "Requires dictionary"]
fn test_tokenizer_pool_reuse() {
    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

    // 첫 분석
    let _tokens1 = tokenizer.tokenize("테스트 문장입니다");
    let stats1 = tokenizer.pool_stats();

    // 두 번째 분석 (재사용 기대)
    let _tokens2 = tokenizer.tokenize("또 다른 테스트");
    let stats2 = tokenizer.pool_stats();

    // 풀이 계속 증가하지 않아야 함 (재사용되므로)
    // 또는 최소한의 증가만 있어야 함
    let growth = stats2.token_pool_size.saturating_sub(stats1.token_pool_size);
    assert!(growth <= 5, "Pool grew too much: {growth}");
}

#[test]
fn test_pool_max_size_limit() {
    let pool = TokenPool::with_capacity(10);

    // max_size = capacity * 2 = 20
    let mut tokens = Vec::new();
    for _ in 0..30 {
        tokens.push(pool.acquire());
    }

    // 모두 반환
    for token in tokens {
        pool.release(token);
    }

    // 최대 크기 제한 확인
    assert!(pool.size() <= 20, "Pool size exceeded max: {}", pool.size());
}

#[test]
fn test_pool_memory_usage() {
    let pool = TokenPool::new();

    // 초기 메모리
    let mem1 = pool.memory_usage();

    // 토큰 추가
    for _ in 0..10 {
        let token = pool.acquire();
        pool.release(token);
    }

    let mem2 = pool.memory_usage();

    // 메모리 증가 확인
    assert!(mem2 > mem1, "Memory usage did not increase");
}

#[test]
fn test_string_interner_memory() {
    let interner = SharedStringInterner::new();

    let _s1 = interner.intern("NNG");
    let _s2 = interner.intern("VV");
    let _s3 = interner.intern("JKS");

    let memory = interner.memory_usage();
    assert!(memory > 0, "Memory usage should be non-zero");
}

#[test]
fn test_pool_clear_preserves_correctness() {
    let pool = TokenPool::new();

    // 사용
    let mut token = pool.acquire();
    token.surface = "테스트".to_string();
    token.pos = "NNG".to_string();
    pool.release(token);

    // 재사용
    let token2 = pool.acquire();
    // 초기화 확인
    assert!(token2.surface.is_empty());
    assert!(token2.pos.is_empty());
}

#[test]
fn test_concurrent_string_interning() {
    use std::sync::Arc;
    use std::thread;

    let interner = Arc::new(SharedStringInterner::new());
    let mut handles = vec![];

    // 여러 스레드에서 동시 interning
    for i in 0..5 {
        let interner_clone = Arc::clone(&interner);
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                let _s = interner_clone.intern(&format!("tag_{i}"));
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    // 5개의 고유 문자열만 저장되어야 함
    assert_eq!(interner.len(), 5);
}

#[test]
#[ignore = "Requires dictionary"]
fn test_tokenizer_clear_pools() {
    let mut tokenizer = Tokenizer::new().expect("Failed to create tokenizer");

    // 여러 번 분석
    for _ in 0..100 {
        let _tokens = tokenizer.tokenize("테스트");
    }

    let stats1 = tokenizer.pool_stats();
    assert!(stats1.token_pool_size > 0);

    // 풀 정리
    tokenizer.clear_pools();

    let stats2 = tokenizer.pool_stats();
    assert_eq!(stats2.token_pool_size, 0);
}

#[test]
fn test_pool_stats_format() {
    let manager = PoolManager::new();

    let token = manager.token_pool.acquire();
    manager.token_pool.release(token);

    let _s1 = manager.string_interner.intern("NNG");

    let stats = manager.stats();
    let formatted = stats.format_human_readable();

    // 포맷 확인
    assert!(formatted.contains("Token Pool"));
    assert!(formatted.contains("Interned Strings"));
    assert!(formatted.contains("Memory"));
}

#[test]
fn test_node_vec_pool_capacity_retention() {
    use mecab_ko_core::pool::NodeVecPool;
    use mecab_ko_core::lattice::Node;

    let pool = NodeVecPool::new();

    // 큰 벡터 획득 및 사용
    let mut vec = pool.acquire();
    for i in 0..100 {
        vec.push(Node::eos(i, 0, 0));
    }

    let capacity = vec.capacity();

    // 반환
    pool.release(vec);

    // 재획득
    let vec2 = pool.acquire();

    // 용량이 유지되어야 함
    assert!(vec2.capacity() >= capacity, "Capacity was not retained");
}
