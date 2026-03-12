use mecab_ko_core::sejong::SejongConverter;
use mecab_ko_core::tokenizer::Tokenizer;
use std::path::PathBuf;

fn main() {
    let project_root = PathBuf::from("/Users/mare/Simon/mecab-ko");
    let dict_path = project_root.join("data/dict-output");

    let mut tokenizer = Tokenizer::with_dict(&dict_path).expect("Failed to create tokenizer");
    let converter = SejongConverter::new();

    // 인용 패턴 테스트
    let test_cases = vec![
        ("가자고 한다", "가/VV 자고/EC 하/VV ㄴ다/EF"),
        ("예쁘다고 한다", "예쁘/VA 다고/EC 하/VV ㄴ다/EF"),
        ("학생이라고 한다", "학생/NNG 이/VCP 라고/EC 하/VV ㄴ다/EF"),
    ];

    println!("=== 인용 패턴 테스트 ===\n");

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
