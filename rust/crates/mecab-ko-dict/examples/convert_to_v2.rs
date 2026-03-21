//! entries.bin v1 포맷을 v2 포맷으로 변환
//!
//! Usage: DICT_PATH=/path/to/dict cargo run --example convert_to_v2 --release

use mecab_ko_dict::dictionary::{LoadOptions, SystemDictionary};
use mecab_ko_dict::LazyEntries;
use std::env;
use std::path::PathBuf;

fn main() {
    let dict_path = env::var("DICT_PATH").unwrap_or_else(|_| {
        "/Users/mare/OrbStack/docker/images/ghcr.io/hephaex/mecab-ko:latest/usr/share/mecab-ko-dic"
            .to_string()
    });
    let dict_path = PathBuf::from(&dict_path);

    println!("=== entries.bin v1 → v2 변환 ===\n");
    println!("사전 경로: {}\n", dict_path.display());

    // Eager 모드로 로드 (v1 포맷 지원)
    println!("1. Eager 모드로 사전 로드...");
    let dict = SystemDictionary::load_with_options(&dict_path, LoadOptions::eager())
        .expect("사전 로드 실패");

    let entry_count = dict.entry_count();
    println!("   엔트리 수: {}", entry_count);

    // 모든 엔트리 수집
    println!("2. 엔트리 수집...");
    let mut entries = Vec::with_capacity(entry_count);
    for i in 0..entry_count {
        if let Ok(entry) = dict.get_entry(i as u32) {
            entries.push((*entry).clone());
        }
    }
    println!("   수집된 엔트리: {}", entries.len());

    // v2 포맷으로 저장 (테스트용 별도 파일)
    let v2_path = dict_path.join("entries_v2.bin");
    println!("3. v2 포맷으로 저장: {}", v2_path.display());

    LazyEntries::save_entries(&entries, &v2_path).expect("v2 저장 실패");

    // 파일 크기 확인
    let v1_path = dict_path.join("entries.bin");
    let v1_size = std::fs::metadata(&v1_path).map(|m| m.len()).unwrap_or(0);
    let v2_size = std::fs::metadata(&v2_path).map(|m| m.len()).unwrap_or(0);

    println!("\n=== 결과 ===");
    println!("v1 (entries.bin): {:.1} MB", v1_size as f64 / 1024.0 / 1024.0);
    println!("v2 (entries_v2.bin): {:.1} MB", v2_size as f64 / 1024.0 / 1024.0);
    println!("크기 차이: {:.1}%", (v2_size as f64 / v1_size as f64 - 1.0) * 100.0);

    // v2 검증
    println!("\n4. v2 포맷 검증...");
    let lazy = LazyEntries::from_file(&v2_path).expect("v2 로드 실패");
    println!("   LazyEntries 엔트리 수: {}", lazy.len());

    // 첫 번째 엔트리 확인
    if let Ok(entry) = lazy.get(0) {
        println!("   첫 번째 엔트리: {}", entry.surface);
    }

    println!("\n=== 변환 완료! ===");
    println!("\nLazy 모드 테스트:");
    println!("  cp {} {}", v2_path.display(), v1_path.display());
    println!("  MODE=lazy cargo run --example memory_measure_isolated --release");
}
