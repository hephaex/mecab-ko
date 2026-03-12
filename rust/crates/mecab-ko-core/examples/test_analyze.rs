use mecab_ko_core::sejong::SejongConverter;
use mecab_ko_core::tokenizer::Tokenizer;
use std::path::PathBuf;

fn main() {
    let project_root = PathBuf::from("/Users/mare/Simon/mecab-ko");
    let dict_path = project_root.join("data/dict-output");

    let mut tokenizer = Tokenizer::with_dict(&dict_path).expect("Failed to create tokenizer");
    let converter = SejongConverter::new();

    // EC 패턴 테스트
    let test_cases = vec![
        ("빠르면", "빠르/VA 면/EC"),
        ("다르면", "다르/VA 면/EC"),
        ("모르면", "모르/VV 면/EC"),
    ];

    println!("=== EC 패턴 테스트 ===\n");

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
