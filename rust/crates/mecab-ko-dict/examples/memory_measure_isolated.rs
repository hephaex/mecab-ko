//! 격리된 메모리 측정 (단일 모드만 테스트)
//!
//! Usage:
//!   MODE=eager cargo run --example `memory_measure_isolated` --release
//!   MODE=lazy cargo run --example `memory_measure_isolated` --release
//!   MODE=memory cargo run --example `memory_measure_isolated` --release

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::cast_precision_loss)]

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
        format!("{bytes} B")
    }
}

fn main() {
    let dict_path = env::var("DICT_PATH").unwrap_or_else(|_| {
        "/Users/mare/OrbStack/docker/images/ghcr.io/hephaex/mecab-ko:latest/usr/share/mecab-ko-dic"
            .to_string()
    });

    let mode = env::var("MODE").unwrap_or_else(|_| "eager".to_string());

    let initial_mem = get_memory_usage().unwrap_or(0);
    println!("=== {} 모드 메모리 측정 ===", mode.to_uppercase());
    println!("사전: {dict_path}");
    println!("초기 메모리: {}", format_bytes(initial_mem));
    println!();

    let options = match mode.as_str() {
        "lazy" => LoadOptions::default(),
        "memory" => LoadOptions::memory_optimized(),
        _ => LoadOptions::speed_optimized(),
    };

    let start = Instant::now();
    let dict = SystemDictionary::load_with_options(&dict_path, options);
    let load_time = start.elapsed();

    match dict {
        Ok(dict) => {
            let after_load_mem = get_memory_usage().unwrap_or(0);
            let mem_increase = after_load_mem.saturating_sub(initial_mem);

            println!("로드 시간: {load_time:?}");
            println!("엔트리 수: {}", dict.entry_count());
            println!(
                "메모리: {} (증가분: {})",
                format_bytes(after_load_mem),
                format_bytes(mem_increase)
            );
            println!();

            // 조회 테스트
            let queries = [
                "안녕하세요",
                "한국어",
                "형태소",
                "분석기",
                "테스트",
                "대한민국",
                "서울시",
                "프로그래밍",
                "인공지능",
                "데이터",
            ];

            // Cold 조회
            let cold_start = Instant::now();
            for q in &queries[..5] {
                let _ = dict.common_prefix_search(q);
            }
            let cold_time = cold_start.elapsed();

            // Warm 조회 (동일 쿼리)
            let warm_start = Instant::now();
            for q in &queries[..5] {
                let _ = dict.common_prefix_search(q);
            }
            let warm_time = warm_start.elapsed();

            let after_lookup_mem = get_memory_usage().unwrap_or(0);

            println!("조회 성능:");
            println!("  Cold (5회): {cold_time:?}");
            println!("  Warm (5회): {warm_time:?}");
            println!("  조회 후 메모리: {}", format_bytes(after_lookup_mem));

            // 대량 조회
            let bulk_start = Instant::now();
            for _ in 0..1000 {
                for q in &queries {
                    let _ = dict.common_prefix_search(q);
                }
            }
            let bulk_time = bulk_start.elapsed();
            println!("  대량 조회 (10,000회): {bulk_time:?}");

            drop(dict);
            std::thread::sleep(std::time::Duration::from_millis(100));
            let final_mem = get_memory_usage().unwrap_or(0);
            println!("\n해제 후 메모리: {}", format_bytes(final_mem));
        }
        Err(e) => {
            println!("로드 실패: {e}");
        }
    }
}
