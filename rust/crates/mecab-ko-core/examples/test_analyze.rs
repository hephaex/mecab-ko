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
        ("또한", "또한/MAG"),  // 현재: 또/MAG 하/VV ㄴ/ETM
        ("한국의 수도", "한국/NNP 의/JKG 수도/NNG"),  // 현재: 하/VV ㄴ/ETM 국의/NNG
        ("아버지", "아버지/NNG"),  // 현재: 아버/NNP 지/VX
        ("간다", "가/VV ㄴ다/EF"),  // 현재: 가/VV ㄴ/ETM 다/NNG
        ("먹자", "먹/VV 자/EF"),  // 현재: 먹/VV 자/NNG
        ("가지고", "가지/VV 고/EC"),  // 현재: 가/VV 어/EC 지/VX 고/MM
        ("하지만", "하/VV 지만/EC"),  // 현재: 하지만/MAJ
        ("가지만", "가/VV 지만/EC"),  // 현재: 가지/NNG 만/JX
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
