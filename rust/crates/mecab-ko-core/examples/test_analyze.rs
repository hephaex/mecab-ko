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

    // 토큰 경계 테스트
    let test_cases = vec![
        ("올해", "올해/NNG"),
        ("내년", "내년/NNG"),
        ("산책을 했어요", "산책/NNG 을/JKO 하/VV 았/EP 어요/EF"),
    ];

    println!("=== 토큰 경계 테스트 ===\n");

    for (text, gold) in test_cases {
        let tokens = tokenizer.tokenize(text);
        println!("MeCab 원본 \"{}\":", text);
        for t in &tokens {
            println!("  {} / {}", t.surface, t.pos);
        }

        let sejong_tokens = converter.convert_tokens(&tokens);
        let sejong_str = converter.format_sejong(&sejong_tokens);

        let matches = gold == sejong_str;
        let status = if matches { "✓" } else { "✗" };
        println!("\n{} \"{}\"", status, text);
        println!("  Gold: {}", gold);
        println!("  Pred: {}", sejong_str);
        println!();
    }
}
