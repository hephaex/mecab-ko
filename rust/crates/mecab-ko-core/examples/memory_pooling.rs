//! # 메모리 풀링 예제
//!
//! 이 예제는 MeCab-Ko의 메모리 풀링 시스템을 시연합니다.
//!
//! ## 실행 방법
//!
//! ```bash
//! cargo run --example memory_pooling
//! ```

#![allow(clippy::unnecessary_wraps)]

use mecab_ko_core::pool::{PoolManager, SharedStringInterner, TokenPool};
use mecab_ko_core::Tokenizer;

fn main() {
    println!("=== MeCab-Ko Memory Pooling Demo ===\n");

    // 1. Token Pool 데모
    demo_token_pool();
    println!();

    // 2. String Interning 데모
    demo_string_interning();
    println!();

    // 3. Pool Manager 데모
    demo_pool_manager();
    println!();

    // 4. Tokenizer 통합 데모
    if matches!(demo_tokenizer_pooling(), Ok(())) {
        println!();
    }

    println!("=== Demo Complete ===");
}

fn demo_token_pool() {
    println!("1. Token Pool Demo");
    println!("{}", "-".repeat(50));

    let pool = TokenPool::new();

    println!("Initial pool size: {}", pool.size());

    // 토큰 획득 및 사용
    println!("\nAcquiring 5 tokens...");
    let mut tokens = Vec::new();
    for i in 0..5 {
        let mut token = pool.acquire();
        token.surface = format!("word_{i}");
        token.pos = "NNG".to_string();
        tokens.push(token);
    }
    println!("Pool size after acquisition: {}", pool.size());

    // 반환
    println!("\nReleasing tokens back to pool...");
    for token in tokens {
        pool.release(token);
    }
    println!("Pool size after release: {}", pool.size());

    // 재사용
    println!("\nReusing tokens (no new allocation)...");
    let mut token = pool.acquire();
    token.surface = "reused".to_string();
    println!("Acquired token surface: {}", token.surface);
    pool.release(token);

    println!("\nMemory usage: {} bytes", pool.memory_usage());
}

fn demo_string_interning() {
    println!("2. String Interning Demo");
    println!("{}", "-".repeat(50));

    let interner = SharedStringInterner::new();

    println!("Initial strings count: {}", interner.len());

    // 같은 문자열 여러 번 intern
    println!("\nInterning 'NNG' 3 times...");
    let s1 = interner.intern("NNG");
    let s2 = interner.intern("NNG");
    let s3 = interner.intern("NNG");

    println!("Symbol 1: {s1:?}");
    println!("Symbol 2: {s2:?}");
    println!("Symbol 3: {s3:?}");
    println!("All symbols equal: {}", s1 == s2 && s2 == s3);

    // 다른 품사 태그들
    println!("\nInterning various POS tags...");
    let pos_tags = vec!["NNG", "VV", "JKS", "EP", "NNP"];
    for tag in &pos_tags {
        let _ = interner.intern(tag);
    }

    println!("Total unique strings: {}", interner.len());
    println!("Memory usage: ~{} bytes", interner.memory_usage());

    // Resolve
    println!("\nResolving symbols...");
    if let Some(resolved) = interner.resolve(s1) {
        println!("Symbol {s1:?} resolves to: '{resolved}'");
    }
}

fn demo_pool_manager() {
    println!("3. Pool Manager Demo");
    println!("{}", "-".repeat(50));

    let manager = PoolManager::new();

    // 초기 통계
    let stats = manager.stats();
    println!("Initial stats:");
    println!("  {}", stats.format_human_readable());

    // 사용
    println!("\nUsing pools...");
    for i in 0..10 {
        let mut token = manager.token_pool.acquire();
        token.surface = format!("token_{i}");
        manager.token_pool.release(token);
    }

    // String interning
    let pos_tags = vec!["NNG", "VV", "JKS", "EP", "NNP", "VV", "NNG"];
    for tag in pos_tags {
        let _ = manager.string_interner.intern(tag);
    }

    // 업데이트된 통계
    let stats = manager.stats();
    println!("\nAfter usage:");
    println!("  {}", stats.format_human_readable());

    // 정리
    println!("\nClearing all pools...");
    manager.clear_all();

    let stats = manager.stats();
    println!("After clear:");
    println!("  {}", stats.format_human_readable());
}

fn demo_tokenizer_pooling() -> Result<(), Box<dyn std::error::Error>> {
    println!("4. Tokenizer Integration Demo");
    println!("{}", "-".repeat(50));

    // Note: 이 부분은 사전이 설치되어 있어야 동작합니다
    let mut tokenizer = match Tokenizer::new() {
        Ok(t) => t,
        Err(e) => {
            println!("Note: Dictionary not available, skipping tokenizer demo");
            println!("Error: {e}");
            return Ok(());
        }
    };

    // 초기 풀 상태
    let stats = tokenizer.pool_stats();
    println!("Initial pool stats:");
    println!("  {}", stats.format_human_readable());

    // 여러 문장 연속 분석
    let sentences = vec![
        "안녕하세요",
        "한국어 형태소 분석기입니다",
        "메모리 풀링으로 성능을 최적화합니다",
        "반복적인 분석에서 효과적입니다",
        "객체 재사용으로 할당을 줄입니다",
    ];

    println!("\nAnalyzing {} sentences...", sentences.len());
    for (i, sentence) in sentences.iter().enumerate() {
        let tokens = tokenizer.tokenize(sentence);
        println!("  Sentence {}: {} tokens", i + 1, tokens.len());
    }

    // 풀 상태 확인
    let stats = tokenizer.pool_stats();
    println!("\nAfter analysis:");
    println!("  {}", stats.format_human_readable());

    // 추가 분석 (재사용 확인)
    println!("\nAnalyzing 100 more times with same sentences...");
    for _ in 0..100 {
        for sentence in &sentences {
            let _tokens = tokenizer.tokenize(sentence);
        }
    }

    let stats = tokenizer.pool_stats();
    println!("After 100 iterations:");
    println!("  {}", stats.format_human_readable());
    println!("  Note: Pool size stabilizes due to reuse!");

    // 정리
    println!("\nClearing pools...");
    tokenizer.clear_pools();

    let stats = tokenizer.pool_stats();
    println!("After clear:");
    println!("  {}", stats.format_human_readable());

    Ok(())
}
