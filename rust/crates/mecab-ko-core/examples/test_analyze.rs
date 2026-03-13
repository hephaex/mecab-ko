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
        // EP 64% - 낮은 정확도 패턴
        ("선생님께서 오셨습니다", "선생님/NNG 께서/JKS 오/VV 시/EP 었/EP 습니다/EF"),
        ("드시겠어요", "드/VV 시/EP 겠/EP 어요/EF"),
        ("계십니다", "계/VV 시/EP ㅂ니다/EF"),
        ("피곤해서 일찍 자야겠다", "피곤/NNG 하/XSA 아서/EC 일찍/MAG 자/VV 아야겠/EP 다/EF"),
        // VCP 63% - 낮은 정확도 패턴
        ("얼마예요", "얼마/NP 이/VCP 에요/EF"),
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
