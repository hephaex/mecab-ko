//! 사전 빌드 예제
//!
//! mecab-ko-dic CSV 파일을 바이너리 사전으로 변환하는 예제

use mecab_ko_dict_builder::builder::BuildConfig;
use mecab_ko_dict_builder::csv_parser::Encoding;
use mecab_ko_dict_builder::DictionaryBuilder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 빌드 설정
    let config = BuildConfig {
        input_dir: "./mecab-ko-dic".to_string(), // mecab-ko-dic 디렉토리
        output_dir: "./dict".to_string(),        // 출력 디렉토리
        compression_level: 3,                    // zstd 압축 레벨 (0-22)
        encoding: Encoding::Auto,                // 자동 인코딩 감지
        verbose: true,                           // 상세 로그
    };

    println!("=== MeCab-Ko Dictionary Builder Example ===\n");

    // 빌더 생성 및 빌드 실행
    let builder = DictionaryBuilder::new(config);

    println!("Starting dictionary build...\n");
    let result = builder.build()?;

    // 결과 출력
    println!("\n=== Build Completed ===");
    println!("Entries:      {}", result.entry_count);
    println!("Trie size:    {} bytes", result.trie_size);
    println!("Matrix size:  {} entries", result.matrix_size);

    println!("\nOutput files:");
    println!("  - sys.dic.zst     : Trie (compressed)");
    println!("  - matrix.bin.zst  : Connection matrix (compressed)");
    println!("  - char.bin        : Character types (optional)");
    println!("  - unk.bin         : Unknown word rules (optional)");

    Ok(())
}
