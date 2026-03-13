use mecab_ko_core::evaluate::{TestDataset, evaluate_dataset_sejong, GoldSentence, GoldToken};
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

    // sample.tsv 평가
    let sample_path = project_root.join("data/eval/sample.tsv");
    if sample_path.exists() {
        let dataset = TestDataset::from_tsv(&sample_path).expect("Failed to load sample.tsv");
        let result = evaluate_dataset_sejong(&mut tokenizer, &dataset);

        println!("=== Sample.tsv 평가 결과 ===");
        println!("Token Accuracy: {:.1}%", result.token_accuracy * 100.0);
        println!("Sentence Accuracy: {:.1}%", result.sentence_accuracy * 100.0);
        println!("POS Accuracy: {:.1}%", result.pos_accuracy * 100.0);
        println!("F1 Score: {:.3}", result.f1_score);
        println!();

        // 품사별 정확도 (낮은 순서)
        let mut pos_vec: Vec<_> = result.pos_stats.iter().collect();
        pos_vec.sort_by(|a, b| a.1.accuracy.partial_cmp(&b.1.accuracy).unwrap());

        println!("=== 품사별 정확도 (낮은 순) ===");
        for (pos, stats) in pos_vec.iter().take(15) {
            if stats.gold_count >= 3 {
                let errors = stats.gold_count.saturating_sub(stats.correct);
                println!("{:6} {:5.1}% ({:3}/{:3}) errors={}",
                    pos, stats.accuracy * 100.0, stats.correct, stats.gold_count, errors);
            }
        }
        println!();

        // 전체 문장 에러 분석
        println!("=== 문장별 실패 케이스 (상위 10개) ===");
        let mut failures = Vec::new();
        for sentence in &dataset.sentences {
            let tokens = tokenizer.tokenize(&sentence.text);
            let sejong = converter.convert_tokens(&tokens);
            let pred_str = converter.format_sejong(&sejong);
            let gold_str: String = sentence.tokens.iter()
                .map(|t| format!("{}/{}", t.surface, t.pos))
                .collect::<Vec<_>>()
                .join(" ");
            if pred_str != gold_str {
                failures.push((sentence.text.clone(), gold_str, pred_str));
            }
        }
        for (text, gold, pred) in failures.iter().take(10) {
            println!("Text: '{}'", text);
            println!("  Gold: {}", gold);
            println!("  Pred: {}", pred);
            println!();
        }
    }

    // 디버깅 케이스 - sample.tsv에서 실패하는 케이스 분석
    let debug_cases = [
        // EF 에러 - XSA/XSV 관련
        ("미안해요", "미안/NNG 하/XSA 어요/EF"),
        ("만들어지다", "만들/VV 어지/VX 다/EF"),
        ("놀리다 웃기다", "놀리/VV 다/EF 웃기/VV 다/EF"),
        ("깨우다 재우다", "깨우/VV 다/EF 재우/VV 다/EF"),
    ];

    println!("=== 디버깅 케이스 ===");
    for (text, expected) in debug_cases {
        let tokens = tokenizer.tokenize(text);
        println!("--- '{}' ---", text);
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
