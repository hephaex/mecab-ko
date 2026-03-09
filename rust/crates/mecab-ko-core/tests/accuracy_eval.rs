//! 정확도 평가 통합 테스트
//!
//! sample.tsv 데이터셋을 사용한 정확도 측정

#![allow(clippy::expect_used, clippy::unwrap_used)]

use mecab_ko_core::evaluate::{evaluate_dataset_sejong, TestDataset};
use mecab_ko_core::tokenizer::Tokenizer;
use mecab_ko_dict::UserDictionary;

/// 전체 데이터셋 정확도 측정
#[test]
fn test_full_accuracy_evaluation() {
    // 프로젝트 루트 경로 계산
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .unwrap_or_else(|_| ".".to_string());
    let project_root = std::path::Path::new(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .unwrap_or(std::path::Path::new("."));

    // 사전 경로 설정
    let dict_path = std::env::var("MECAB_DIC_PATH")
        .unwrap_or_else(|_| {
            project_root.join("data/mecab-ko-dic-2.1.1-20180720")
                .to_string_lossy()
                .to_string()
        });

    let mut tokenizer = Tokenizer::with_dict(&dict_path)
        .expect("Failed to create tokenizer");

    // 사용자 사전 로드
    let user_dict_path = project_root.join("data/user-dict/verb-inflections.csv");
    if user_dict_path.exists() {
        let mut user_dict = UserDictionary::new();
        user_dict.load_from_csv(&user_dict_path)
            .expect("Failed to load user dictionary");
        tokenizer.set_user_dict(user_dict);
        println!("Loaded user dictionary: {:?}", user_dict_path);
    }

    // 테스트 데이터셋 로드
    let eval_path = std::env::var("MECAB_EVAL_PATH")
        .unwrap_or_else(|_| {
            project_root.join("data/eval/sample.tsv")
                .to_string_lossy()
                .to_string()
        });

    let dataset = TestDataset::from_tsv(&eval_path)
        .expect("Failed to load test dataset");

    println!("\n=== 정확도 평가 시작 ===");
    println!("테스트 문장 수: {}", dataset.len());

    // 세종 형식으로 평가
    let result = evaluate_dataset_sejong(&mut tokenizer, &dataset);

    // 결과 출력
    println!("{}", result.format_report());

    // 현재 목표: 50%+ 정확도
    assert!(
        result.token_accuracy >= 0.50,
        "Token accuracy {:.1}% is below 50% target",
        result.token_accuracy * 100.0
    );
}

/// EC 에러 패턴 디버깅
#[test]
fn test_ec_error_analysis() {
    use mecab_ko_core::sejong::SejongConverter;

    // 프로젝트 루트 경로 계산
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .unwrap_or_else(|_| ".".to_string());
    let project_root = std::path::Path::new(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .unwrap_or(std::path::Path::new("."));

    let dict_path = std::env::var("MECAB_DIC_PATH")
        .unwrap_or_else(|_| {
            project_root.join("data/mecab-ko-dic-2.1.1-20180720")
                .to_string_lossy()
                .to_string()
        });

    let mut tokenizer = Tokenizer::with_dict(&dict_path)
        .expect("Failed to create tokenizer");

    // 사용자 사전 로드
    let user_dict_path = project_root.join("data/user-dict/verb-inflections.csv");
    if user_dict_path.exists() {
        let mut user_dict = UserDictionary::new();
        user_dict.load_from_csv(&user_dict_path)
            .expect("Failed to load user dictionary");
        tokenizer.set_user_dict(user_dict);
    }

    let converter = SejongConverter::new();

    // EC 관련 테스트 케이스 - 다양한 연결어미 패턴
    let test_cases = [
        // 기본 연결어미
        ("가면서", "가/VV 면서/EC"),
        ("먹으면", "먹/VV 으면/EC"),
        ("오니까", "오/VV 니까/EC"),
        ("갔으면서", "가/VV 았/EP 으면서/EC"),
        ("잡고", "잡/VV 고/EC"),
        ("보니까", "보/VV 니까/EC"),
        ("알면서", "알/VV 면서/EC"),
        // 추가 패턴
        ("만나서", "만나/VV 아서/EC"),
        ("좋아서", "좋/VA 아서/EC"),
        ("먹고나서", "먹/VV 고나서/EC"),
        ("하지만", "하/VV 지만/EC"),
        ("가도록", "가/VV 도록/EC"),
        ("오듯이", "오/VV 듯이/EC"),
        ("가자마자", "가/VV 자마자/EC"),
        ("왔다고", "오/VV 았/EP 다고/EC"),
        ("읽고", "읽/VV 고/EC"),
    ];

    println!("\n=== EC 에러 분석 ===");
    let mut passed = 0;
    let total = test_cases.len();
    for (input, expected) in test_cases {
        let tokens = tokenizer.tokenize(input);
        let sejong_tokens = converter.convert_tokens(&tokens);
        let result = converter.format_sejong(&sejong_tokens);
        let is_match = result == expected;
        if is_match {
            passed += 1;
        }
        let match_status = if is_match { "✓" } else { "✗" };
        println!("{} \"{}\": {} (예상: {})", match_status, input, result, expected);
    }
    println!("\n통과: {}/{} ({:.1}%)", passed, total, passed as f64 / total as f64 * 100.0);
}

/// ETM 에러 패턴 디버깅
#[test]
fn test_etm_error_analysis() {
    use mecab_ko_core::sejong::SejongConverter;

    // 프로젝트 루트 경로 계산
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .unwrap_or_else(|_| ".".to_string());
    let project_root = std::path::Path::new(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .unwrap_or(std::path::Path::new("."));

    let dict_path = std::env::var("MECAB_DIC_PATH")
        .unwrap_or_else(|_| {
            project_root.join("data/mecab-ko-dic-2.1.1-20180720")
                .to_string_lossy()
                .to_string()
        });

    let mut tokenizer = Tokenizer::with_dict(&dict_path)
        .expect("Failed to create tokenizer");

    // 사용자 사전 로드
    let user_dict_path = project_root.join("data/user-dict/verb-inflections.csv");
    if user_dict_path.exists() {
        let mut user_dict = UserDictionary::new();
        user_dict.load_from_csv(&user_dict_path)
            .expect("Failed to load user dictionary");
        tokenizer.set_user_dict(user_dict);
    }

    let converter = SejongConverter::new();

    // ETM 관련 테스트 케이스
    let test_cases = [
        // 현재 관형형 (는)
        ("가는", "가/VV 는/ETM"),
        ("오는", "오/VV 는/ETM"),
        ("먹는", "먹/VV 는/ETM"),
        ("하는", "하/VV 는/ETM"),
        // 과거 관형형 (ㄴ/은)
        ("간", "가/VV ㄴ/ETM"),
        ("온", "오/VV ㄴ/ETM"),
        ("먹은", "먹/VV 은/ETM"),
        ("한", "하/VV ㄴ/ETM"),
        ("새로운", "새롭/VA ㄴ/ETM"),
        // 미래 관형형 (ㄹ/을)
        ("갈", "가/VV ㄹ/ETM"),
        ("올", "오/VV ㄹ/ETM"),
        ("먹을", "먹/VV 을/ETM"),
        ("할", "하/VV ㄹ/ETM"),
        // 불규칙 (ㄷ→ㄹ)
        ("걷는", "걷/VV 는/ETM"),
        ("듣는", "듣/VV 는/ETM"),
    ];

    println!("\n=== ETM 에러 분석 ===");
    let mut passed = 0;
    let total = test_cases.len();
    for (input, expected) in test_cases {
        let tokens = tokenizer.tokenize(input);
        let sejong_tokens = converter.convert_tokens(&tokens);
        let result = converter.format_sejong(&sejong_tokens);
        let is_match = result == expected;
        if is_match {
            passed += 1;
        }
        let match_status = if is_match { "✓" } else { "✗" };
        println!("{} \"{}\": {} (예상: {})", match_status, input, result, expected);
    }
    println!("\n통과: {}/{} ({:.1}%)", passed, total, passed as f64 / total as f64 * 100.0);
}

/// 특정 품사별 정확도 검증
#[test]
fn test_pos_accuracy_breakdown() {
    // 프로젝트 루트 경로 계산
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .unwrap_or_else(|_| ".".to_string());
    let project_root = std::path::Path::new(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .unwrap_or(std::path::Path::new("."));

    let dict_path = std::env::var("MECAB_DIC_PATH")
        .unwrap_or_else(|_| {
            project_root.join("data/mecab-ko-dic-2.1.1-20180720")
                .to_string_lossy()
                .to_string()
        });

    let mut tokenizer = Tokenizer::with_dict(&dict_path)
        .expect("Failed to create tokenizer");

    // 사용자 사전 로드
    let user_dict_path = project_root.join("data/user-dict/verb-inflections.csv");
    if user_dict_path.exists() {
        let mut user_dict = UserDictionary::new();
        user_dict.load_from_csv(&user_dict_path)
            .expect("Failed to load user dictionary");
        tokenizer.set_user_dict(user_dict);
    }

    let eval_path = std::env::var("MECAB_EVAL_PATH")
        .unwrap_or_else(|_| {
            project_root.join("data/eval/sample.tsv")
                .to_string_lossy()
                .to_string()
        });

    let dataset = TestDataset::from_tsv(&eval_path)
        .expect("Failed to load test dataset");

    let result = evaluate_dataset_sejong(&mut tokenizer, &dataset);

    // 품사별 정확도 출력
    println!("\n=== 품사별 정확도 ===");
    let mut pos_sorted: Vec<_> = result.pos_stats.iter().collect();
    pos_sorted.sort_by(|a, b| b.1.gold_count.cmp(&a.1.gold_count));

    for (pos, stats) in pos_sorted {
        if stats.gold_count >= 10 {
            println!(
                "{:<6} ({:>3}개): {:>5.1}% ({}/{} correct)",
                pos,
                stats.gold_count,
                stats.accuracy * 100.0,
                stats.correct,
                stats.gold_count
            );
        }
    }
}
