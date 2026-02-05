//! Performance optimization demonstration
//!
//! This example demonstrates the performance improvements from caching and batch processing.

use mecab_ko_elasticsearch::analyzer::NoriAnalyzer;
use mecab_ko_elasticsearch::config::{AnalyzerConfig, DecompoundMode};
use std::time::Instant;

const SAMPLE_QUERIES: &[&str] = &[
    "한국어",
    "형태소 분석",
    "자연어 처리",
    "검색 엔진",
    "머신러닝",
    "딥러닝",
    "빅데이터",
    "클라우드",
];

const SAMPLE_DOCUMENT: &str = "
한국어 형태소 분석기는 자연어 처리의 핵심 도구입니다. \
이를 통해 텍스트를 의미 있는 단위로 분해할 수 있습니다. \
Elasticsearch와 통합하여 강력한 검색 기능을 제공합니다.
";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== MeCab-Ko Elasticsearch Performance Demo ===\n");

    let config = AnalyzerConfig::new()
        .with_decompound_mode(DecompoundMode::Mixed)
        .with_stoptags(vec!["J".to_string(), "E".to_string()]);

    // Demo 1: Cache vs No Cache
    demo_cache_performance(&config)?;

    // Demo 2: Batch Processing
    #[cfg(feature = "batch")]
    demo_batch_processing(&config)?;

    // Demo 3: Cache Statistics
    demo_cache_statistics(&config)?;

    Ok(())
}

fn demo_cache_performance(config: &AnalyzerConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("1. Cache Performance Comparison");
    println!("   Testing repeated query analysis...\n");

    const ITERATIONS: usize = 10000;

    // Without cache
    let analyzer_no_cache = NoriAnalyzer::without_cache(config.clone())?;
    let start = Instant::now();

    for _ in 0..ITERATIONS {
        for query in SAMPLE_QUERIES {
            let _ = analyzer_no_cache.analyze(query)?;
        }
    }

    let no_cache_time = start.elapsed();
    let no_cache_qps = (ITERATIONS * SAMPLE_QUERIES.len()) as f64 / no_cache_time.as_secs_f64();

    println!(
        "   No cache:    {:?} ({:.0} queries/sec)",
        no_cache_time, no_cache_qps
    );

    // With cache
    let analyzer_with_cache = NoriAnalyzer::with_cache_size(config.clone(), 1024)?;
    let start = Instant::now();

    for _ in 0..ITERATIONS {
        for query in SAMPLE_QUERIES {
            let _ = analyzer_with_cache.analyze(query)?;
        }
    }

    let cache_time = start.elapsed();
    let cache_qps = (ITERATIONS * SAMPLE_QUERIES.len()) as f64 / cache_time.as_secs_f64();

    println!(
        "   With cache:  {:?} ({:.0} queries/sec)",
        cache_time, cache_qps
    );

    let speedup = no_cache_time.as_secs_f64() / cache_time.as_secs_f64();
    println!("   Speedup:     {:.1}x faster with cache\n", speedup);

    Ok(())
}

#[cfg(feature = "batch")]
fn demo_batch_processing(config: &AnalyzerConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("2. Batch Processing Performance");
    println!("   Testing parallel vs sequential processing...\n");

    const BATCH_SIZE: usize = 100;
    let documents: Vec<_> = (0..BATCH_SIZE).map(|_| SAMPLE_DOCUMENT).collect();

    let analyzer = NoriAnalyzer::without_cache(config.clone())?;

    // Sequential processing
    let start = Instant::now();
    for doc in &documents {
        let _ = analyzer.analyze(doc)?;
    }
    let seq_time = start.elapsed();
    let seq_throughput = BATCH_SIZE as f64 / seq_time.as_secs_f64();

    println!(
        "   Sequential:  {:?} ({:.0} docs/sec)",
        seq_time, seq_throughput
    );

    // Batch processing
    let start = Instant::now();
    let docs_slice: Vec<&str> = documents.iter().map(|s| s.as_ref()).collect();
    let _ = analyzer.analyze_batch(&docs_slice)?;
    let batch_time = start.elapsed();
    let batch_throughput = BATCH_SIZE as f64 / batch_time.as_secs_f64();

    println!(
        "   Parallel:    {:?} ({:.0} docs/sec)",
        batch_time, batch_throughput
    );

    let speedup = seq_time.as_secs_f64() / batch_time.as_secs_f64();
    println!(
        "   Speedup:     {:.1}x faster with batch processing\n",
        speedup
    );

    Ok(())
}

fn demo_cache_statistics(config: &AnalyzerConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("3. Cache Statistics");
    println!("   Monitoring cache usage...\n");

    let analyzer = NoriAnalyzer::with_cache_size(config.clone(), 16)?;

    if let Some((capacity, size)) = analyzer.cache_stats() {
        println!("   Initial:     {}/{} entries", size, capacity);
    }

    // Add some queries to cache
    for query in SAMPLE_QUERIES {
        let _ = analyzer.analyze(query)?;
    }

    if let Some((capacity, size)) = analyzer.cache_stats() {
        println!("   After 8:     {}/{} entries", size, capacity);
    }

    // Add more queries (will exceed cache size)
    for i in 0..20 {
        let query = format!("추가 쿼리 {}", i);
        let _ = analyzer.analyze(&query)?;
    }

    if let Some((capacity, size)) = analyzer.cache_stats() {
        println!(
            "   After 28:    {}/{} entries (LRU eviction)",
            size, capacity
        );
    }

    // Clear cache
    analyzer.clear_cache();

    if let Some((capacity, size)) = analyzer.cache_stats() {
        println!("   After clear: {}/{} entries\n", size, capacity);
    }

    Ok(())
}
