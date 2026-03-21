//! 실제 사전으로 메모리 사용량 측정
//!
//! Usage: DICT_PATH=/path/to/dict cargo run --example memory_measure --release

use mecab_ko_dict::dictionary::{LoadOptions, SystemDictionary};
use std::env;
use std::time::Instant;

#[cfg(target_os = "macos")]
fn get_memory_usage() -> Option<usize> {
    use std::process::Command;
    let pid = std::process::id();
    let output = Command::new("ps")
        .args(["-o", "rss=", "-p", &pid.to_string()])
        .output()
        .ok()?;
    let rss = String::from_utf8_lossy(&output.stdout);
    rss.trim().parse::<usize>().ok().map(|kb| kb * 1024)
}

#[cfg(not(target_os = "macos"))]
fn get_memory_usage() -> Option<usize> {
    None
}

fn format_bytes(bytes: usize) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / 1024.0 / 1024.0)
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

fn main() {
    let dict_path = env::var("DICT_PATH").unwrap_or_else(|_| {
        "/Users/mare/OrbStack/docker/images/ghcr.io/hephaex/mecab-ko:latest/usr/share/mecab-ko-dic"
            .to_string()
    });

    println!("=== mecab-ko 메모리 사용량 측정 ===\n");
    println!("사전 경로: {}\n", dict_path);

    // 초기 메모리
    let initial_mem = get_memory_usage().unwrap_or(0);
    println!("초기 메모리: {}\n", format_bytes(initial_mem));

    // 1. Eager 모드 측정
    println!("--- Eager 모드 (LoadOptions::speed_optimized()) ---");
    let eager_start = Instant::now();
    let eager_dict = SystemDictionary::load_with_options(&dict_path, LoadOptions::speed_optimized());
    let eager_load_time = eager_start.elapsed();

    match eager_dict {
        Ok(dict) => {
            let eager_mem = get_memory_usage().unwrap_or(0);
            let eager_mem_delta = eager_mem.saturating_sub(initial_mem);
            println!("  로드 시간: {:?}", eager_load_time);
            println!("  엔트리 수: {}", dict.entry_count());
            println!("  메모리 사용: {} (증가분: {})", format_bytes(eager_mem), format_bytes(eager_mem_delta));

            // 조회 테스트
            let lookup_start = Instant::now();
            let _ = dict.common_prefix_search("안녕하세요");
            let _ = dict.common_prefix_search("한국어");
            let _ = dict.common_prefix_search("형태소");
            let _ = dict.common_prefix_search("분석기");
            let _ = dict.common_prefix_search("테스트");
            println!("  5회 조회: {:?}", lookup_start.elapsed());

            drop(dict);
        }
        Err(e) => {
            println!("  로드 실패: {}", e);
        }
    }

    // GC 유도
    std::thread::sleep(std::time::Duration::from_millis(100));
    let after_drop_mem = get_memory_usage().unwrap_or(0);
    println!("  해제 후: {}\n", format_bytes(after_drop_mem));

    // 2. Lazy 모드 측정
    println!("--- Lazy 모드 (LoadOptions::default()) ---");
    let lazy_start = Instant::now();
    let lazy_dict = SystemDictionary::load_with_options(&dict_path, LoadOptions::default());
    let lazy_load_time = lazy_start.elapsed();

    match lazy_dict {
        Ok(dict) => {
            let lazy_mem = get_memory_usage().unwrap_or(0);
            let lazy_mem_delta = lazy_mem.saturating_sub(after_drop_mem);
            println!("  로드 시간: {:?}", lazy_load_time);
            println!("  엔트리 수: {}", dict.entry_count());
            println!("  메모리 사용: {} (증가분: {})", format_bytes(lazy_mem), format_bytes(lazy_mem_delta));

            // 첫 번째 조회 (cold cache)
            let cold_start = Instant::now();
            let _ = dict.common_prefix_search("안녕하세요");
            println!("  첫 조회 (cold): {:?}", cold_start.elapsed());

            // 추가 조회 (warming cache)
            let warm_start = Instant::now();
            let _ = dict.common_prefix_search("한국어");
            let _ = dict.common_prefix_search("형태소");
            let _ = dict.common_prefix_search("분석기");
            let _ = dict.common_prefix_search("테스트");
            println!("  4회 조회 (warm): {:?}", warm_start.elapsed());

            // 캐시 워밍 후 메모리
            let after_warm_mem = get_memory_usage().unwrap_or(0);
            println!("  캐시 후 메모리: {}", format_bytes(after_warm_mem));

            drop(dict);
        }
        Err(e) => {
            println!("  로드 실패: {}", e);
        }
    }

    // 최종 메모리
    std::thread::sleep(std::time::Duration::from_millis(100));
    let final_mem = get_memory_usage().unwrap_or(0);
    println!("  해제 후: {}\n", format_bytes(final_mem));

    // 3. Memory Optimized 모드
    println!("--- Memory Optimized 모드 (mmap + lazy) ---");
    let mem_opt_start = Instant::now();
    let mem_opt_dict = SystemDictionary::load_with_options(&dict_path, LoadOptions::memory_optimized());
    let mem_opt_load_time = mem_opt_start.elapsed();

    match mem_opt_dict {
        Ok(dict) => {
            let mem_opt_mem = get_memory_usage().unwrap_or(0);
            let mem_opt_delta = mem_opt_mem.saturating_sub(final_mem);
            println!("  로드 시간: {:?}", mem_opt_load_time);
            println!("  엔트리 수: {}", dict.entry_count());
            println!("  메모리 사용: {} (증가분: {})", format_bytes(mem_opt_mem), format_bytes(mem_opt_delta));

            // 조회 테스트
            let lookup_start = Instant::now();
            let _ = dict.common_prefix_search("안녕하세요");
            let _ = dict.common_prefix_search("한국어");
            let _ = dict.common_prefix_search("형태소");
            let _ = dict.common_prefix_search("분석기");
            let _ = dict.common_prefix_search("테스트");
            println!("  5회 조회: {:?}", lookup_start.elapsed());

            drop(dict);
        }
        Err(e) => {
            println!("  로드 실패: {}", e);
        }
    }

    println!("\n=== 측정 완료 ===");
}
