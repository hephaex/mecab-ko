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
        // 인터넷 패턴 테스트 (166차)
        ("컴퓨터 인터넷", "컴퓨터/NNG 인터넷/NNG"),
        // EP 64% - XSV 뒤 EP
        ("증가하면서", "증가/NNG 하/XSV 면서/EC"),
        ("진행되고", "진행/NNG 되/XSV 고/EC"),
        // VCP 63% - 지정사
        ("예정입니다", "예정/NNG 이/VCP 습니다/EF"),
        ("성공적으로", "성공적/NNG 으로/JKB"),
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
