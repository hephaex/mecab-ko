use mecab_ko_core::sejong::SejongConverter;
use mecab_ko_core::tokenizer::Tokenizer;
use std::path::PathBuf;

fn main() {
    let project_root = PathBuf::from("/Users/mare/Simon/mecab-ko");
    let dict_path = project_root.join("data/dict-output");

    let mut tokenizer = Tokenizer::with_dict(&dict_path).expect("Failed to create tokenizer");
    let converter = SejongConverter::new();

    // VV/XSV 패턴 테스트
    let test_cases = vec![
        ("하니까 보니까", "하/VV 니까/EC 보/VV 니까/EC"),
        ("말씀하세요", "말씀/NNG 하/VV 세요/EF"),
        ("먹으면 가면", "먹/VV 으면/EC 가/VV 면/EC"),
    ];

    println!("=== VV/XSV 패턴 테스트 ===\n");

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
