use mecab_ko_core::sejong::SejongConverter;
use mecab_ko_core::tokenizer::Tokenizer;
use std::path::PathBuf;

fn main() {
    let project_root = PathBuf::from("/Users/mare/Simon/mecab-ko");
    let dict_path = project_root.join("data/dict-output");

    let mut tokenizer = Tokenizer::with_dict(&dict_path).expect("Failed to create tokenizer");
    let converter = SejongConverter::new();

    // MAG 패턴 테스트
    let test_cases = vec![
        ("아주 좋다", "아주/MAG 좋/VA 다/EF"),
        ("매우 크다", "매우/MAG 크/VA 다/EF"),
        ("정말 예쁘다", "정말/MAG 예쁘/VA 다/EF"),
        ("안 가요", "안/MAG 가/VV 아요/EF"),
        ("못 가요", "못/MAG 가/VV 아요/EF"),
    ];

    let mut correct = 0;
    let mut total = 0;

    for (text, gold) in test_cases {
        let tokens = tokenizer.tokenize(text);
        let sejong_tokens = converter.convert_tokens(&tokens);
        let sejong_str = converter.format_sejong(&sejong_tokens);

        let matches = gold == sejong_str;
        if matches {
            correct += 1;
        }
        total += 1;

        let status = if matches { "✓" } else { "✗" };
        println!("\n{} \"{}\"", status, text);
        if !matches {
            println!("  Gold: {}", gold);
            println!("  Pred: {}", sejong_str);
        }
    }

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("EC Pattern Tests: {}/{} ({:.1}%)", correct, total, (correct as f64 / total as f64) * 100.0);
}
