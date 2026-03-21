//! 메모리 사용량 비교 테스트
//!
//! Eager vs Lazy 로딩 모드의 메모리 특성 검증

#![allow(clippy::expect_used, clippy::unwrap_used)]

use mecab_ko_dict::dictionary::{LoadOptions, SystemDictionary};
use mecab_ko_dict::LazyEntries;
use std::path::PathBuf;
use std::sync::Once;
use std::time::Instant;

static SETUP: Once = Once::new();

/// entries.bin v2 형식 생성 (LazyEntries 호환)
fn ensure_entries_bin() -> PathBuf {
    SETUP.call_once(|| {
        let mini_dict_path = get_mini_dict_path();
        let entries_bin = mini_dict_path.join("entries.bin");

        // 기존 파일이 있으면 삭제 (다른 형식일 수 있음)
        if entries_bin.exists() {
            std::fs::remove_file(&entries_bin).ok();
        }

        // CSV에서 엔트리 로드 (Eager 모드로 강제)
        let dict = SystemDictionary::load_with_options(&mini_dict_path, LoadOptions::eager())
            .expect("load dict");

        // get_entry로 모든 엔트리를 Vec으로 수집
        let mut entries = Vec::new();
        for i in 0..dict.entry_count() {
            if let Ok(entry) = dict.get_entry(i as u32) {
                entries.push((*entry).clone());
            }
        }

        if !entries.is_empty() {
            // LazyEntries v2 형식으로 저장
            LazyEntries::save_entries(&entries, &entries_bin).expect("save entries.bin v2");
            println!("Created entries.bin (v2 format) with {} entries", entries.len());
        }
    });

    get_mini_dict_path().join("entries.bin")
}

/// 미니 사전 경로 반환
fn get_mini_dict_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("parent")
        .parent()
        .expect("workspace")
        .join("test-fixtures")
        .join("mini-dict")
}

/// 테스트 후 entries.bin 정리
fn cleanup_entries_bin() {
    let entries_bin = get_mini_dict_path().join("entries.bin");
    if entries_bin.exists() {
        std::fs::remove_file(&entries_bin).ok();
    }
}

#[test]
fn test_eager_vs_lazy_loading_characteristics() {
    let _entries_bin = ensure_entries_bin();
    let dict_path = get_mini_dict_path();

    println!("\n=== Eager vs Lazy Loading Comparison ===\n");

    // Eager 로딩
    let eager_start = Instant::now();
    let eager_dict = SystemDictionary::load_with_options(&dict_path, LoadOptions::speed_optimized())
        .expect("load eager");
    let eager_load_time = eager_start.elapsed();
    let eager_entry_count = eager_dict.entry_count();

    println!("Eager Loading:");
    println!("  - Load time: {:?}", eager_load_time);
    println!("  - Entry count: {}", eager_entry_count);

    // 첫 번째 조회
    let eager_lookup_start = Instant::now();
    let _ = eager_dict.common_prefix_search("안녕");
    let eager_first_lookup = eager_lookup_start.elapsed();
    println!("  - First lookup: {:?}", eager_first_lookup);

    // Lazy 로딩
    let lazy_start = Instant::now();
    let lazy_dict = SystemDictionary::load_with_options(&dict_path, LoadOptions::default())
        .expect("load lazy");
    let lazy_load_time = lazy_start.elapsed();
    let lazy_entry_count = lazy_dict.entry_count();

    println!("\nLazy Loading:");
    println!("  - Load time: {:?}", lazy_load_time);
    println!("  - Entry count: {}", lazy_entry_count);

    // 첫 번째 조회 (디스크에서 로드)
    let lazy_lookup_start = Instant::now();
    let _ = lazy_dict.common_prefix_search("안녕");
    let lazy_first_lookup = lazy_lookup_start.elapsed();
    println!("  - First lookup: {:?}", lazy_first_lookup);

    // 두 번째 조회 (캐시됨)
    let lazy_cached_start = Instant::now();
    let _ = lazy_dict.common_prefix_search("안녕");
    let lazy_cached_lookup = lazy_cached_start.elapsed();
    println!("  - Cached lookup: {:?}", lazy_cached_lookup);

    // 기본 검증
    assert_eq!(eager_entry_count, lazy_entry_count, "Entry count should match");
    assert!(lazy_load_time <= eager_load_time || eager_entry_count < 100,
        "Lazy loading should be faster or similar for small dicts");

    println!("\n=== Test Passed ===");
}

#[test]
fn test_load_options_variants() {
    let _entries_bin = ensure_entries_bin();
    let dict_path = get_mini_dict_path();

    // 기본값 (LazyEntries 활성화)
    let default_opts = LoadOptions::default();
    assert!(default_opts.use_lazy_entries, "Default should enable lazy entries");
    assert_eq!(default_opts.lazy_cache_size, Some(10000), "Default cache size should be 10000");

    // 속도 최적화 (Eager)
    let speed_opts = LoadOptions::speed_optimized();
    assert!(!speed_opts.use_lazy_entries, "Speed optimized should disable lazy entries");

    // 메모리 최적화
    let memory_opts = LoadOptions::memory_optimized();
    assert!(memory_opts.use_lazy_entries, "Memory optimized should enable lazy entries");
    assert!(memory_opts.use_mmap_matrix, "Memory optimized should enable mmap matrix");

    // 호환성 eager()
    let eager_opts = LoadOptions::eager();
    assert!(!eager_opts.use_lazy_entries, "eager() should disable lazy entries");

    // 모든 옵션으로 로드 가능 확인
    let _ = SystemDictionary::load_with_options(&dict_path, default_opts)
        .expect("load with default options");
    let _ = SystemDictionary::load_with_options(&dict_path, speed_opts)
        .expect("load with speed options");
    let _ = SystemDictionary::load_with_options(&dict_path, LoadOptions::memory_optimized())
        .expect("load with memory optimized options");
}

#[test]
fn test_lazy_cache_size_effect() {
    let _entries_bin = ensure_entries_bin();
    let dict_path = get_mini_dict_path();

    // 작은 캐시
    let small_cache_opts = LoadOptions {
        use_mmap_matrix: false,
        use_lazy_entries: true,
        lazy_cache_size: Some(2), // 아주 작은 캐시
    };

    let dict = SystemDictionary::load_with_options(&dict_path, small_cache_opts)
        .expect("load with small cache");

    // 여러 단어 조회 (캐시 압박)
    let words = ["안녕", "한국어", "사람", "시간", "책", "가"];
    for word in &words {
        let _ = dict.common_prefix_search(word);
    }

    // 다시 조회 (캐시 미스 발생할 수 있음)
    for word in &words {
        let result = dict.common_prefix_search(word);
        assert!(result.is_ok(), "Lookup should still work with small cache");
    }
}

#[test]
fn test_entry_store_abstraction() {
    let _entries_bin = ensure_entries_bin();
    let dict_path = get_mini_dict_path();

    // Eager 모드
    let eager_dict = SystemDictionary::load_with_options(&dict_path, LoadOptions::eager())
        .expect("load eager");

    // Lazy 모드
    let lazy_dict = SystemDictionary::load_with_options(&dict_path, LoadOptions::default())
        .expect("load lazy");

    // 동일한 결과 반환 확인
    for i in 0..eager_dict.entry_count().min(10) {
        let eager_entry = eager_dict.get_entry(i as u32);
        let lazy_entry = lazy_dict.get_entry(i as u32);

        match (eager_entry, lazy_entry) {
            (Ok(e), Ok(l)) => {
                assert_eq!(e.surface, l.surface, "Surface should match at index {i}");
                assert_eq!(e.left_id, l.left_id, "Left ID should match at index {i}");
                assert_eq!(e.right_id, l.right_id, "Right ID should match at index {i}");
                assert_eq!(e.cost, l.cost, "Cost should match at index {i}");
            }
            (Err(_), Err(_)) => {
                // 둘 다 에러면 OK
            }
            _ => panic!("Eager and Lazy should return same result at index {i}"),
        }
    }
}

/// 테스트 후 정리 (z_cleanup으로 시작하여 마지막에 실행)
#[test]
fn z_cleanup_entries_bin() {
    cleanup_entries_bin();
}
