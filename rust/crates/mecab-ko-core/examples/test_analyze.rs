use mecab_ko_core::sejong::SejongConverter;
use mecab_ko_core::tokenizer::Tokenizer;
use std::path::PathBuf;

fn main() {
    let project_root = PathBuf::from("/Users/mare/Simon/mecab-ko");
    let dict_path = project_root.join("data/dict-output");

    let mut tokenizer = Tokenizer::with_dict(&dict_path).expect("Failed to create tokenizer");
    let converter = SejongConverter::new();

    // 사용자 사전 로드 테스트
    let user_dict_path = project_root.join("data/user-dict/verb-inflections.csv");
    if user_dict_path.exists() {
        let mut user_dict = mecab_ko_dict::UserDictionary::new();
        if user_dict.load_from_csv(&user_dict_path).is_ok() {
            tokenizer.set_user_dict(user_dict);
            println!("Loaded user dictionary\n");
        }
    }

    // 디버깅 케이스 - sample.tsv에서 실패하는 케이스 분석
    let debug_cases = [
        // JKG 테스트
        ("나의 친구", "나/NP 의/JKG 친구/NNG"),
        ("학교의 운동장", "학교/NNG 의/JKG 운동장/NNG"),
        ("주민들의", "주민/NNG 들/XSN 의/JKG"),
    ];

    for (text, expected) in debug_cases {
        let tokens = tokenizer.tokenize(text);
        println!("=== '{}' ===", text);
        println!("MeCab tokens:");
        for t in &tokens {
            println!("  surface='{}' pos='{}' features='{}'", t.surface, t.pos, t.features);
        }

        let sejong_tokens = converter.convert_tokens(&tokens);
        let result = converter.format_sejong(&sejong_tokens);

        let status = if result == expected { "✓" } else { "✗" };
        println!("{} 결과: {}", status, result);
        if result != expected {
            println!("   예상: {}", expected);
        }
        println!();
    }
}
