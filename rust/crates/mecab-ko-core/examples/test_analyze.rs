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

    // 디버깅 케이스
    let debug_cases = [
        // VA 합성어 패턴 (재미있/VA, 맛없/VA)
        ("재미있어요", "재미있/VA 어요/EF"),  // 현재: 재미/NNG 있/VV 어요/EF
        ("맛없어요", "맛없/VA 어요/EF"),  // 현재: 맛/NNG 없/VX 어요/EF
        ("멋있다", "멋있/VA 다/EF"),  // 현재: 멋/NNG 있/VV 다/EF
        ("힘들어요", "힘들/VA 어요/EF"),  // 현재: 힘/NNG 들/VV 어요/EF
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
