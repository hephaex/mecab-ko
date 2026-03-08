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
