//! Batch-tokenize lines from a text file and print Sejong-formatted results.

use mecab_ko_core::sejong::SejongConverter;
use mecab_ko_core::tokenizer::Tokenizer;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let project_root = PathBuf::from("/Users/mare/Simon/mecab-ko");
    let dict_path = project_root.join("data/dict-output");

    let mut tokenizer = Tokenizer::with_dict(&dict_path)?;
    let converter = SejongConverter::new();

    // 사용자 사전 로드
    let user_dict_path = project_root.join("data/user-dict/verb-inflections.csv");
    if user_dict_path.exists() {
        let mut user_dict = mecab_ko_dict::UserDictionary::new();
        if user_dict.load_from_csv(&user_dict_path).is_ok() {
            tokenizer.set_user_dict(user_dict);
        }
    }

    // 입력 파일 처리
    let input_path = "/tmp/curated_sentences.txt";
    let file = File::open(input_path)?;
    let reader = BufReader::new(file);

    let mut current_section = String::new();

    for line in reader.lines() {
        let line = line?;
        let line = line.trim();

        // 빈 줄 건너뛰기
        if line.is_empty() {
            continue;
        }

        // 섹션 헤더 출력
        if line.starts_with('#') {
            if !current_section.is_empty() {
                println!();
            }
            current_section = line.to_string();
            println!("{line}");
            continue;
        }

        // 토큰화
        let tokens = tokenizer.tokenize(line);
        let sejong_tokens = converter.convert_tokens(&tokens);
        let result = converter.format_sejong(&sejong_tokens);

        // TSV 형식 출력
        println!("{line}\t{result}");
    }

    Ok(())
}
