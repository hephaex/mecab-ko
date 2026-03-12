use mecab_ko_core::sejong::SejongConverter;
use mecab_ko_core::tokenizer::Tokenizer;
use std::path::PathBuf;

fn main() {
    let project_root = PathBuf::from("/Users/mare/Simon/mecab-ko");
    let dict_path = project_root.join("data/dict-output");

    let mut tokenizer = Tokenizer::with_dict(&dict_path).expect("Failed to create tokenizer");
    let converter = SejongConverter::new();

    let test_cases = vec![
        ("오다 왔다 올까요", "오/VV 다/EF 오/VV 았/EP 다/EF 오/VV ㄹ까요/EF"),
        ("볼게요", "보/VV ㄹ게요/EF"),
        ("올래요", "오/VV ㄹ래요/EF"),
    ];

    for (text, gold) in test_cases {
        let tokens = tokenizer.tokenize(text);
        println!("\n\"{}\"", text);
        println!("Gold:     {}", gold);

        let sejong_tokens = converter.convert_tokens(&tokens);
        let sejong_str = converter.format_sejong(&sejong_tokens);
        println!("Pred:     {}", sejong_str);

        // Show raw surface values
        println!("Raw surfaces:");
        for t in &sejong_tokens {
            let bytes: Vec<u8> = t.surface.bytes().collect();
            println!("  '{}' -> {:?}", t.surface, bytes);
        }
    }
}
