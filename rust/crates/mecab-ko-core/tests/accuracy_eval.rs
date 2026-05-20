//! 정확도 평가 통합 테스트
//!
//! sample.tsv 데이터셋을 사용한 정확도 측정

#![allow(
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::cast_precision_loss,
    clippy::too_many_lines
)]

use mecab_ko_core::evaluate::{evaluate_dataset_sejong, TestDataset};
use mecab_ko_core::tokenizer::Tokenizer;
use mecab_ko_dict::UserDictionary;

/// 프로젝트 루트 경로 (`CARGO_MANIFEST_DIR`에서 3 단계 상위)
fn project_root() -> std::path::PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    std::path::Path::new(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .unwrap_or_else(|| std::path::Path::new("."))
        .to_path_buf()
}

/// `MECAB_DIC_PATH` env 또는 프로젝트 루트 기준 기본 사전 경로
fn dict_path(project_root: &std::path::Path) -> String {
    std::env::var("MECAB_DIC_PATH").unwrap_or_else(|_| {
        project_root
            .join("data/mecab-ko-dic-2.1.1-20180720")
            .to_string_lossy()
            .to_string()
    })
}

/// 시스템 사전 + 사용자 사전(verb-inflections + klue-domain) 로드한 `Tokenizer` 생성
fn make_tokenizer(project_root: &std::path::Path) -> Tokenizer {
    let dict_path = dict_path(project_root);
    let mut tokenizer = Tokenizer::with_dict(&dict_path).expect("Failed to create tokenizer");
    let user_dict_path = project_root.join("data/user-dict/verb-inflections.csv");
    if user_dict_path.exists() {
        let mut user_dict = UserDictionary::new();
        user_dict
            .load_from_csv(&user_dict_path)
            .expect("Failed to load user dictionary");
        let klue_dict_path = project_root.join("data/user-dict/klue-domain.csv");
        if klue_dict_path.exists() {
            user_dict
                .load_from_csv(&klue_dict_path)
                .expect("Failed to load KLUE domain dictionary");
        }
        tokenizer.set_user_dict(user_dict);
    }
    tokenizer
}

/// 전체 데이터셋 정확도 측정
#[test]
#[ignore = "requires system dictionary data (sys.dic)"]
fn test_full_accuracy_evaluation() {
    let project_root = project_root();
    let mut tokenizer = make_tokenizer(&project_root);

    // 테스트 데이터셋 로드
    let eval_path = std::env::var("MECAB_EVAL_PATH").unwrap_or_else(|_| {
        project_root
            .join("data/eval/sample.tsv")
            .to_string_lossy()
            .to_string()
    });

    let dataset = TestDataset::from_tsv(&eval_path).expect("Failed to load test dataset");

    println!("\n=== 정확도 평가 시작 ===");
    println!("테스트 문장 수: {}", dataset.len());

    // 세종 형식으로 평가
    let result = evaluate_dataset_sejong(&mut tokenizer, &dataset);

    // 결과 출력
    println!("{}", result.format_report());

    // Sprint 122 baseline: 100.0% token / 99.9% sentence (1099/1100)
    // 1 known issue: MBTI cascade (가/JKS analyzed as 가/VV+EC Inflect after NNP).
    // Threshold set to 99.9% to catch any regression beyond the known case.
    assert!(
        result.token_accuracy >= 0.999,
        "Token accuracy {:.1}% is below 99.9% baseline (Sprint 122)",
        result.token_accuracy * 100.0
    );
    assert!(
        result.sentence_accuracy >= 0.998,
        "Sentence accuracy {:.1}% is below 99.8% baseline (Sprint 122)",
        result.sentence_accuracy * 100.0
    );
}

/// 특정 품사별 정확도 검증
#[test]
#[ignore = "requires system dictionary data (sys.dic)"]
fn test_pos_accuracy_breakdown() {
    let project_root = project_root();
    let mut tokenizer = make_tokenizer(&project_root);

    let eval_path = std::env::var("MECAB_EVAL_PATH").unwrap_or_else(|_| {
        project_root
            .join("data/eval/sample.tsv")
            .to_string_lossy()
            .to_string()
    });

    let dataset = TestDataset::from_tsv(&eval_path).expect("Failed to load test dataset");

    let result = evaluate_dataset_sejong(&mut tokenizer, &dataset);

    // 품사별 정확도 출력
    println!("\n=== 품사별 정확도 ===");
    let mut pos_sorted: Vec<_> = result.pos_stats.iter().collect();
    pos_sorted.sort_by_key(|entry| std::cmp::Reverse(entry.1.gold_count));

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

/// ㅎ불규칙 형용사 "으면/EC" 테스트
#[test]
#[ignore = "requires system dictionary data (sys.dic)"]
fn test_h_irregular_adjective_ec() {
    use mecab_ko_core::sejong::SejongConverter;

    let mut tokenizer = make_tokenizer(&project_root());

    let converter = SejongConverter::new();

    // ㅎ불규칙 형용사 테스트
    let test_cases = [
        // sample.tsv 기준
        (
            "하얗다 하얘 하얗으면",
            "하얗/VA 다/EF 하얗/VA 아/EF 하얗/VA 으면/EC",
        ),
        (
            "까맣다 까매 까맣으면",
            "까맣/VA 다/EF 까맣/VA 아/EF 까맣/VA 으면/EC",
        ),
        (
            "노랗다 노래 노랗으면",
            "노랗/VA 다/EF 노랗/VA 아/EF 노랗/VA 으면/EC",
        ),
        // 단일 어절
        ("하얗으면", "하얗/VA 으면/EC"),
        ("까맣으면", "까맣/VA 으면/EC"),
    ];

    println!("\n=== ㅎ불규칙 형용사 EC 분석 ===");
    for (input, expected) in test_cases {
        let tokens = tokenizer.tokenize(input);
        let sejong_tokens = converter.convert_tokens(&tokens);
        let result = converter.format_sejong(&sejong_tokens);

        println!("\n문장: {input}");
        println!("  예상: {expected}");
        println!("  결과: {result}");
        println!("  MeCab 원본:");
        for tok in &tokens {
            println!("    {} / {} | {}", tok.surface, tok.pos, tok.features);
        }

        let is_match = result == expected;
        println!("  일치: {}", if is_match { "✓" } else { "✗" });
    }
}

/// ㄷ불규칙 동사 테스트
#[test]
#[ignore = "requires system dictionary data (sys.dic)"]
fn test_d_irregular_verb() {
    use mecab_ko_core::sejong::SejongConverter;

    let mut tokenizer = make_tokenizer(&project_root());

    let converter = SejongConverter::new();

    // ㄷ불규칙 동사 테스트
    let test_cases = [
        // sample.tsv 기준
        (
            "걷다 걸어 걸으면 걷는",
            "걷/VV 다/EF 걷/VV 어/EF 걷/VV 으면/EC 걷/VV 는/ETM",
        ),
        (
            "듣다 들어 들으면 듣는",
            "듣/VV 다/EF 듣/VV 어/EF 듣/VV 으면/EC 듣/VV 는/ETM",
        ),
        (
            "묻다 물어 물으면 묻는",
            "묻/VV 다/EF 묻/VV 어/EF 묻/VV 으면/EC 묻/VV 는/ETM",
        ),
    ];

    println!("\n=== ㄷ불규칙 동사 분석 ===");
    for (input, expected) in test_cases {
        let tokens = tokenizer.tokenize(input);
        let sejong_tokens = converter.convert_tokens(&tokens);
        let result = converter.format_sejong(&sejong_tokens);

        println!("\n문장: {input}");
        println!("  예상: {expected}");
        println!("  결과: {result}");
        println!("  MeCab 원본:");
        for tok in &tokens {
            println!("    {} / {} | {}", tok.surface, tok.pos, tok.features);
        }

        let is_match = result == expected;
        println!("  일치: {}", if is_match { "✓" } else { "✗" });
    }
}

/// CI/CD Accuracy Gate: 99.9%+ token accuracy required
///
/// PR 병합 전 정확도 검증을 위한 테스트.
/// Sprint 122 baseline: 100.0% token / 99.9% sentence (1 known MBTI cascade).
#[test]
#[ignore = "requires system dictionary data (sys.dic)"]
fn test_accuracy_gate() {
    // 99.9% 정확도 게이트 (Sprint 122 raised from 95%)
    const ACCURACY_THRESHOLD: f64 = 0.999;

    let project_root = project_root();
    let mut tokenizer = make_tokenizer(&project_root);

    // 테스트 데이터셋 로드
    let eval_path = std::env::var("MECAB_EVAL_PATH").unwrap_or_else(|_| {
        project_root
            .join("data/eval/sample.tsv")
            .to_string_lossy()
            .to_string()
    });

    let dataset = TestDataset::from_tsv(&eval_path).expect("Failed to load test dataset");

    println!("\n=== CI/CD Accuracy Gate ===");
    println!("테스트 문장 수: {}", dataset.len());

    // 세종 형식으로 평가
    let result = evaluate_dataset_sejong(&mut tokenizer, &dataset);

    // 결과 출력 (CI에서 파싱용)
    let accuracy_percent = result.token_accuracy * 100.0;
    println!("Token Accuracy: {accuracy_percent:.1}");
    println!(
        "Sentence Accuracy: {:.1}%",
        result.sentence_accuracy * 100.0
    );
    println!("F1 Score: {:.3}", result.f1_score);
    assert!(
        result.token_accuracy >= ACCURACY_THRESHOLD,
        "ACCURACY GATE FAILED: Token accuracy {:.1}% is below {:.0}% threshold",
        accuracy_percent,
        ACCURACY_THRESHOLD * 100.0
    );

    println!(
        "ACCURACY GATE PASSED: {:.1}% >= {:.0}%",
        accuracy_percent,
        ACCURACY_THRESHOLD * 100.0
    );
}

/// Sprint 65: 검증 데이터셋 정확도 게이트
///
/// 수작업 검증된 `sprint58_verified.tsv` 데이터셋에 대한 정확도 검증
#[test]
#[ignore = "requires system dictionary data (sys.dic)"]
fn test_accuracy_gate_verified() {
    const VERIFIED_THRESHOLD: f64 = 0.90;

    let project_root = project_root();
    let dict_path = dict_path(&project_root);
    let mut tokenizer = Tokenizer::with_dict(&dict_path).expect("Failed to create tokenizer");

    let eval_path = project_root.join("data/eval/sprint58_verified.tsv");
    if !eval_path.exists() {
        println!("Skipping: sprint58_verified.tsv not found");
        return;
    }

    let dataset = TestDataset::from_tsv(eval_path.to_str().unwrap())
        .expect("Failed to load verified dataset");

    println!("\n=== Verified Dataset Accuracy Gate ===");
    println!("테스트 문장 수: {}", dataset.len());

    let result = evaluate_dataset_sejong(&mut tokenizer, &dataset);

    let accuracy_percent = result.token_accuracy * 100.0;
    println!("Token Accuracy: {accuracy_percent:.1}%");
    println!(
        "Sentence Accuracy: {:.1}%",
        result.sentence_accuracy * 100.0
    );
    println!("F1 Score: {:.3}", result.f1_score);
    assert!(
        result.token_accuracy >= VERIFIED_THRESHOLD,
        "VERIFIED ACCURACY GATE FAILED: Token accuracy {:.1}% is below {:.0}% threshold",
        accuracy_percent,
        VERIFIED_THRESHOLD * 100.0
    );

    println!(
        "VERIFIED ACCURACY GATE PASSED: {:.1}% >= {:.0}%",
        accuracy_percent,
        VERIFIED_THRESHOLD * 100.0
    );
}

/// Sprint 124 Phase 1: KLUE DP dual-metric evaluation
///
/// Evaluates mecab-ko on the KLUE DP validation set (1,995 sentences,
/// CC BY-SA 4.0). Reports both morpheme-level and eojeol-level accuracy.
///
/// Phase 0 baseline (2026-05-11): morpheme 65.8%. Phase 1 adds eojeol metric.
/// Threshold floor set conservatively below baseline so the test catches
/// regressions but does not block on tag-scheme micro-differences that are
/// known to inflate the error rate (planned for Sprint 125 tag equivalence map).
#[test]
#[ignore = "requires KLUE DP eval data + system dictionary"]
fn test_klue_dp_dual_metric() {
    use mecab_ko_core::evaluate::evaluate_dataset_dual;

    // Sprint 124 Phase 1 baseline:
    //   morpheme (greedy-aligned): 65.8%
    //   eojeol (strict per-eojeol): 19.2% (4,299 / 22,404)
    // Eojeol metric is intentionally strict — every morpheme within an
    // eojeol must match. Low absolute number; 80%+ of eojeols fail due to
    // KLUE-vs-mecab tag scheme differences (SP/SC, SS/SY/SSO/SSC,
    // MMD/MMN/MMA/MM, NNG/NNP boundary), planned for Sprint 125 tag
    // equivalence map.
    //
    // Floors set ~5%p below measurement so genuine regressions trigger
    // failure but routine variance does not.
    const MORPHEME_FLOOR: f64 = 0.60;
    const EOJEOL_FLOOR: f64 = 0.15;

    let project_root = project_root();
    let mut tokenizer = make_tokenizer(&project_root);

    let eval_path = project_root.join("data/eval/klue_dp_val.tsv");
    if !eval_path.exists() {
        println!("Skipping: data/eval/klue_dp_val.tsv not found");
        return;
    }

    let dataset = TestDataset::from_tsv(eval_path.to_str().unwrap())
        .expect("Failed to load KLUE DP val");

    println!("\n=== KLUE DP Dual-Metric Evaluation ===");
    println!("Dataset: {} sentences", dataset.len());

    let result = evaluate_dataset_dual(&mut tokenizer, &dataset);
    println!("{}", result.format_report());

    let morpheme_pct = result.morpheme.token_accuracy * 100.0;
    let eojeol_pct = result.eojeol_accuracy * 100.0;

    assert!(
        result.morpheme.token_accuracy >= MORPHEME_FLOOR,
        "KLUE DP morpheme accuracy {:.1}% is below {:.0}% floor (Sprint 124 Phase 1)",
        morpheme_pct,
        MORPHEME_FLOOR * 100.0
    );
    assert!(
        result.eojeol_accuracy >= EOJEOL_FLOOR,
        "KLUE DP eojeol accuracy {:.1}% is below {:.0}% floor (Sprint 124 Phase 1)",
        eojeol_pct,
        EOJEOL_FLOOR * 100.0
    );

    println!(
        "PASSED — morpheme {:.1}% >= {:.0}%, eojeol {:.1}% >= {:.0}%",
        morpheme_pct,
        MORPHEME_FLOOR * 100.0,
        eojeol_pct,
        EOJEOL_FLOOR * 100.0
    );
}

/// Sprint 125 P1: KLUE DP lenient evaluation with tag equivalence map
///
/// Same dataset as `test_klue_dp_dual_metric` but uses
/// `evaluate_dataset_dual_lenient` to absorb tag scheme differences:
///   {SP, SC}, {SS, SY, SSO, SSC}, {MM, MMD, MMN, MMA}
///
/// Both morpheme-level and eojeol-level metrics use the lenient comparison
/// (Sprint 125 — function pointer plumbing through `evaluate_dataset_sejong`).
///
/// Phase 1 reported 65.8% morpheme / 19.2% eojeol under strict comparison.
/// Sprint 125 P1 measures the lift from tag equivalence isolation alone.
#[test]
#[ignore = "requires KLUE DP eval data + system dictionary"]
fn test_klue_dp_dual_metric_lenient() {
    use mecab_ko_core::evaluate::{
        evaluate_dataset_dual, evaluate_dataset_dual_lenient,
        evaluate_dataset_dual_with_pos_match, pos_tags_equivalent_practical,
    };

    // Sprint 125 baseline floor (lift confirmed: strict 19.2% -> lenient 20.8%)
    const LENIENT_EOJEOL_FLOOR: f64 = 0.20;

    let project_root = project_root();
    let mut tokenizer = make_tokenizer(&project_root);

    let eval_path = project_root.join("data/eval/klue_dp_val.tsv");
    if !eval_path.exists() {
        println!("Skipping: data/eval/klue_dp_val.tsv not found");
        return;
    }

    let dataset = TestDataset::from_tsv(eval_path.to_str().unwrap())
        .expect("Failed to load KLUE DP val");

    println!("\n=== KLUE DP Strict vs Lenient ===");
    println!("Dataset: {} sentences", dataset.len());

    let strict = evaluate_dataset_dual(&mut tokenizer, &dataset);
    let lenient = evaluate_dataset_dual_lenient(&mut tokenizer, &dataset);
    let practical = evaluate_dataset_dual_with_pos_match(
        &mut tokenizer, &dataset, pos_tags_equivalent_practical,
    );

    let strict_morph = strict.morpheme.token_accuracy * 100.0;
    let strict_eo = strict.eojeol_accuracy * 100.0;
    let lenient_morph = lenient.morpheme.token_accuracy * 100.0;
    let lenient_eo = lenient.eojeol_accuracy * 100.0;
    let practical_morph = practical.morpheme.token_accuracy * 100.0;
    let practical_eo = practical.eojeol_accuracy * 100.0;

    println!("\n--- Strict ---");
    println!("  Morpheme: {strict_morph:.1}%");
    println!("  Eojeol:   {strict_eo:.1}% ({} / {})",
        strict.eojeol_correct, strict.eojeol_total);

    println!("\n--- Lenient (conservative): SP/SC, SS/SY/SSO/SSC, MM/MMD/MMN/MMA, SL/NNP ---");
    println!("  Morpheme: {lenient_morph:.1}% [Δ +{:.1}pp vs strict]",
        lenient_morph - strict_morph);
    println!("  Eojeol:   {lenient_eo:.1}% ({} / {}) [Δ +{:.1}pp vs strict]",
        lenient.eojeol_correct, lenient.eojeol_total, lenient_eo - strict_eo);

    println!("\n--- Practical: + NNB/NNG (counter words) + VA/VV (있다 convention, S136 P3) ---");
    println!("  Morpheme: {practical_morph:.1}% [Δ +{:.1}pp vs lenient]",
        practical_morph - lenient_morph);
    println!("  Eojeol:   {practical_eo:.1}% ({} / {}) [Δ +{:.1}pp vs lenient]",
        practical.eojeol_correct, practical.eojeol_total, practical_eo - lenient_eo);

    // 회귀 catch
    assert!(lenient.eojeol_accuracy >= strict.eojeol_accuracy,
        "Lenient eojeol {lenient_eo:.1}% must be >= strict {strict_eo:.1}%");
    assert!(practical.eojeol_accuracy >= lenient.eojeol_accuracy,
        "Practical eojeol {practical_eo:.1}% must be >= lenient {lenient_eo:.1}%");
    assert!(practical.morpheme.token_accuracy >= lenient.morpheme.token_accuracy,
        "Practical morpheme {practical_morph:.1}% must be >= lenient {lenient_morph:.1}%");

    assert!(lenient.eojeol_accuracy >= LENIENT_EOJEOL_FLOOR,
        "Lenient eojeol {lenient_eo:.1}% is below {:.0}% floor",
        LENIENT_EOJEOL_FLOOR * 100.0);

    println!("\nPASSED — strict {strict_eo:.1}% / lenient {lenient_eo:.1}% / practical {practical_eo:.1}%");
}

/// Sprint 126 P1: NNG/NNP/NNB confusion case extraction
///
/// Extracts only `POS_ONLY` errors involving NNG, NNP, or NNB tags from
/// the KLUE DP evaluation. Outputs samples grouped by confusion direction
/// (e.g., gold=NNG / pred=NNP) for manual analysis.
///
/// Purpose: gather evidence to decide whether NNG/NNP/NNB should be added
/// to `TAG_EQUIVALENCE_GROUPS`. The decision requires distinguishing:
/// (a) real analysis errors — mecab clearly wrong
/// (b) convention differences — both labels arguable
/// (c) KLUE annotation errors — mecab right, KLUE wrong
#[test]
#[ignore = "requires KLUE DP eval data + system dictionary"]
fn test_klue_dp_nng_nnp_analysis() {
    use mecab_ko_core::sejong::SejongConverter;

    let project_root = project_root();
    let mut tokenizer = make_tokenizer(&project_root);

    let eval_path = project_root.join("data/eval/klue_dp_val.tsv");
    if !eval_path.exists() {
        println!("Skipping: data/eval/klue_dp_val.tsv not found");
        return;
    }

    let dataset = TestDataset::from_tsv(eval_path.to_str().unwrap())
        .expect("Failed to load KLUE DP val");
    let converter = SejongConverter::new();

    // (gold_pos, pred_pos) -> Vec<(sentence, surface, sentence_idx)>
    let mut cases: std::collections::HashMap<(String, String), Vec<(String, String)>> =
        std::collections::HashMap::new();

    let target_tags = ["NNG", "NNP", "NNB"];

    for gold_sentence in &dataset.sentences {
        let pred_raw = tokenizer.tokenize(&gold_sentence.text);
        let sejong_tokens = converter.convert_tokens(&pred_raw);

        let pred_pairs: Vec<(String, String)> = sejong_tokens
            .iter()
            .map(|t| {
                (
                    SejongConverter::normalize_jamo(&t.surface),
                    t.pos.clone(),
                )
            })
            .collect();

        let min_len = gold_sentence.tokens.len().min(pred_pairs.len());
        for (g, (p_surf, p_pos)) in gold_sentence
            .tokens
            .iter()
            .zip(pred_pairs.iter())
            .take(min_len)
        {

            if g.surface != *p_surf {
                continue; // SEGMENTATION case, skip
            }
            if g.pos == *p_pos {
                continue; // Match
            }
            // POS_ONLY case
            if !target_tags.contains(&g.pos.as_str())
                && !target_tags.contains(&p_pos.as_str())
            {
                continue;
            }
            // At least one side is NNG/NNP/NNB
            cases
                .entry((g.pos.clone(), p_pos.clone()))
                .or_default()
                .push((gold_sentence.text.clone(), g.surface.clone()));
        }
    }

    println!("\n=== NNG/NNP/NNB Confusion Analysis ===");
    println!("Dataset: {} sentences\n", dataset.len());

    let mut sorted: Vec<_> = cases.iter().collect();
    sorted.sort_by_key(|x| std::cmp::Reverse(x.1.len()));

    println!("Confusion summary (sorted by count):");
    for ((g, p), v) in &sorted {
        println!("  {} → {}  ({}건)", g, p, v.len());
    }

    println!("\nTotal NNG/NNP/NNB-related POS_ONLY errors: {}",
        sorted.iter().map(|x| x.1.len()).sum::<usize>());

    // Show top N samples for the largest confusion groups
    let sample_count = 10;
    println!("\n--- Top samples per confusion direction ({sample_count} each) ---");
    for ((g, p), v) in sorted.iter().take(6) {
        println!("\n>>> {} → {}  ({}건)", g, p, v.len());
        for (text, surface) in v.iter().take(sample_count) {
            println!("  surface={surface:<12}  in: {text}");
        }
    }
}

/// Sprint 121: Error case classification
///
/// Categorize every mismatch from full dict evaluation into:
/// - `POS_ONLY`: surface matches but POS differs
/// - SEGMENTATION: surface doesn't match (different tokenization)
/// - `TOKEN_COUNT`: gold/pred have different token counts
/// - UNKNOWN: pred token has NNG with features containing UNKNOWN markers
#[test]
#[ignore = "requires system dictionary data (sys.dic)"]
fn test_error_case_classification() {
    use mecab_ko_core::sejong::SejongConverter;

    #[derive(Debug)]
    struct ErrorCase {
        sentence: String,
        category: String,
        gold_tokens: Vec<String>,
        pred_tokens: Vec<String>,
        diff_details: Vec<String>,
    }

    let project_root = project_root();
    let mut tokenizer = make_tokenizer(&project_root);

    let eval_path = std::env::var("MECAB_EVAL_PATH").unwrap_or_else(|_| {
        project_root
            .join("data/eval/sample.tsv")
            .to_string_lossy()
            .to_string()
    });

    let dataset = TestDataset::from_tsv(&eval_path).expect("Failed to load dataset");
    let converter = SejongConverter::new();

    let mut errors: Vec<ErrorCase> = Vec::new();
    let mut cat_counts: std::collections::HashMap<String, usize> =
        std::collections::HashMap::new();
    let mut pos_confusion: std::collections::HashMap<(String, String), usize> =
        std::collections::HashMap::new();

    for gold_sentence in &dataset.sentences {
        let pred_raw = tokenizer.tokenize(&gold_sentence.text);
        let sejong_tokens = converter.convert_tokens(&pred_raw);

        let pred_strs: Vec<String> = sejong_tokens
            .iter()
            .map(|st| {
                format!(
                    "{}/{}",
                    SejongConverter::normalize_jamo(&st.surface),
                    st.pos
                )
            })
            .collect();
        let gold_strs: Vec<String> = gold_sentence
            .tokens
            .iter()
            .map(|gt| format!("{}/{}", gt.surface, gt.pos))
            .collect();

        if gold_strs == pred_strs {
            continue;
        }

        let mut diffs = Vec::new();
        let mut categories = Vec::new();

        if gold_sentence.tokens.len() != sejong_tokens.len() {
            categories.push("TOKEN_COUNT".to_string());
            diffs.push(format!(
                "token count: gold={} pred={}",
                gold_sentence.tokens.len(),
                sejong_tokens.len()
            ));
        }

        let min_len = gold_sentence.tokens.len().min(sejong_tokens.len());
        for (i, (gold_tok, pred_sejong)) in gold_sentence.tokens.iter()
            .zip(sejong_tokens.iter())
            .enumerate()
            .take(min_len)
        {
            let gold_surface = &gold_tok.surface;
            let gold_pos = &gold_tok.pos;
            let pred_surface = SejongConverter::normalize_jamo(&pred_sejong.surface);
            let pred_pos = &pred_sejong.pos;

            if gold_surface == &pred_surface && gold_pos == pred_pos {
                continue;
            }

            if gold_surface == &pred_surface {
                categories.push("POS_ONLY".to_string());
                diffs.push(format!(
                    "[{i}] {gold_surface}: gold={gold_pos} pred={pred_pos}"
                ));
                *pos_confusion
                    .entry((gold_pos.clone(), pred_pos.clone()))
                    .or_insert(0) += 1;
            } else {
                categories.push("SEGMENTATION".to_string());
                diffs.push(format!(
                    "[{i}] gold={gold_surface}/{gold_pos} pred={pred_surface}/{pred_pos}"
                ));
            }
        }

        let primary = if categories.contains(&"SEGMENTATION".to_string()) {
            "SEGMENTATION"
        } else if categories.contains(&"TOKEN_COUNT".to_string()) {
            "TOKEN_COUNT"
        } else if categories.contains(&"POS_ONLY".to_string()) {
            "POS_ONLY"
        } else {
            "OTHER"
        };

        *cat_counts.entry(primary.to_string()).or_insert(0) += 1;

        errors.push(ErrorCase {
            sentence: gold_sentence.text.clone(),
            category: primary.to_string(),
            gold_tokens: gold_strs,
            pred_tokens: pred_strs,
            diff_details: diffs,
        });
    }

    println!("\n{}", "=".repeat(70));
    println!("  ERROR CASE CLASSIFICATION REPORT");
    println!("  Dataset: {} sentences, {} errors", dataset.len(), errors.len());
    println!("{}\n", "=".repeat(70));

    println!("=== Category Summary ===");
    let total_errors = errors.len();
    let mut sorted_cats: Vec<_> = cat_counts.iter().collect();
    sorted_cats.sort_by_key(|x| std::cmp::Reverse(*x.1));
    for (cat, count) in &sorted_cats {
        println!(
            "  {:<15} {:>3} ({:.1}%)",
            cat,
            count,
            **count as f64 / total_errors.max(1) as f64 * 100.0
        );
    }

    println!("\n=== POS Confusion Matrix (gold → pred) ===");
    let mut sorted_confusion: Vec<_> = pos_confusion.iter().collect();
    sorted_confusion.sort_by_key(|x| std::cmp::Reverse(*x.1));
    for ((gold_pos, pred_pos), count) in &sorted_confusion {
        println!("  {gold_pos} → {pred_pos}  ({count}건)");
    }

    println!("\n=== Detailed Error Cases ===");
    for (i, err) in errors.iter().enumerate() {
        println!("\n--- Error #{} [{}] ---", i + 1, err.category);
        println!("  Input: {}", err.sentence);
        println!("  Gold:  {}", err.gold_tokens.join(" "));
        println!("  Pred:  {}", err.pred_tokens.join(" "));
        for diff in &err.diff_details {
            println!("  Diff:  {diff}");
        }
    }

    println!("\n=== Summary ===");
    println!("Total sentences: {}", dataset.len());
    println!("Error sentences: {}", errors.len());
    println!(
        "Sentence accuracy: {:.2}%",
        (1.0 - errors.len() as f64 / dataset.len() as f64) * 100.0
    );

    for (cat, count) in &sorted_cats {
        println!("  {cat}: {count}");
    }
}

/// Sprint 127 P1: Compound noun split policy analysis
///
/// Aligns gold and predicted tokens at eojeol boundaries by **tokenizing each
/// eojeol independently**. This avoids both:
/// - cumulative-surface drift (KLUE morpheme surface uses jamo decomposition,
///   so cumulative char count diverges from original text), and
/// - char-position alignment failure (gold morphemes don't share the original
///   text's coordinate system after decomposition).
///
/// Trade-off: mecab loses cross-eojeol Viterbi context. For analysis purposes
/// this is acceptable since boundary decisions across whitespace are usually
/// independent in Korean morphology.
///
/// Algorithm:
/// 1. `text.split_whitespace()` → eojeol surface list (must match `eojeol_counts.len()`)
/// 2. For each eojeol: gold tokens = `tokens[gold_idx..gold_idx + count_g]`,
///    pred tokens = `tokenizer.tokenize(eojeol)` then convert + `normalize_jamo`
/// 3. Classify each eojeol independently — no cascade across sentences
///
/// Categories:
/// - `EXACT`: surface + POS identical
/// - `POS_DIFF`: same segmentation, POS differs only
/// - `INNER_SPLIT_DIFF`: same morph count but inner surface boundaries differ
/// - `GOLD_SINGLE_PRED_MULTI`: gold has 1 morph, pred splits into N (mecab over-splits)
/// - `GOLD_MULTI_PRED_SINGLE`: gold splits into N, pred has 1 (mecab merges)
/// - `SPLIT_DIFFERENT`: both split (N>=2) but boundary or count differs
/// - `SURFACE_MISMATCH`: gold/pred surface concatenations differ (true orthographic
///   convention diff like 한 → 하+ㄴ vs 한)
///
/// Also computes `SLICE_LENIENT` accuracy: any eojeol where surfaces concatenate
/// to the same string counts as correct, regardless of split. Estimates the
/// upper bound if all compound/split conventions are absorbed.
#[test]
#[ignore = "requires KLUE DP eval data + system dictionary"]
fn test_klue_dp_compound_noun_analysis() {
    use mecab_ko_core::sejong::SejongConverter;
    use std::collections::HashMap;

    let project_root = project_root();
    let mut tokenizer = make_tokenizer(&project_root);

    let eval_path = project_root.join("data/eval/klue_dp_val.tsv");
    if !eval_path.exists() {
        println!("Skipping: data/eval/klue_dp_val.tsv not found");
        return;
    }

    let dataset = TestDataset::from_tsv(eval_path.to_str().unwrap())
        .expect("Failed to load KLUE DP val");
    let converter = SejongConverter::new();

    // Categories
    let mut cat_counts: HashMap<&'static str, usize> = HashMap::new();
    let mut total_eojeols: usize = 0;

    // Sample collectors (cap each)
    let sample_cap = 25;
    let mut samples_g1pn: Vec<(String, String, Vec<String>)> = Vec::new(); // (sentence, eojeol, pred_tokens)
    let mut samples_gnp1: Vec<(String, String, Vec<String>)> = Vec::new();
    let mut samples_split_diff: Vec<(String, String, Vec<String>, Vec<String>)> = Vec::new();
    let mut samples_surface_mismatch: Vec<(String, String, String)> = Vec::new();

    // Slice-lenient: count eojeols where cumulative surface matches (regardless of split)
    let mut slice_lenient_correct: usize = 0;

    // Pattern frequency: (gold_count, pred_count) for surface-aligned eojeols
    let mut split_pattern_counts: HashMap<(usize, usize), usize> = HashMap::new();

    // Top compound nouns gold-1 / pred-multi (the headline pattern)
    let mut top_compounds: HashMap<String, usize> = HashMap::new();

    for gold_sentence in &dataset.sentences {
        let Some(eojeol_counts) = &gold_sentence.eojeol_counts else {
            continue;
        };

        let eojeols: Vec<&str> = gold_sentence.text.split_whitespace().collect();
        if eojeols.len() != eojeol_counts.len() {
            cat_counts.entry("EOJEOL_COUNT_MISMATCH").and_modify(|c| *c += 1).or_insert(1);
            continue;
        }

        let mut gold_idx: usize = 0;

        for (eo_i, &count_g) in eojeol_counts.iter().enumerate() {
            total_eojeols += 1;
            if gold_idx + count_g > gold_sentence.tokens.len() {
                cat_counts.entry("BOUNDS_ERROR").and_modify(|c| *c += 1).or_insert(1);
                gold_idx = gold_sentence.tokens.len();
                continue;
            }
            let gold_slice = &gold_sentence.tokens[gold_idx..gold_idx + count_g];
            gold_idx += count_g;

            let gold_surface: String = gold_slice.iter().map(|t| t.surface.as_str()).collect();

            // Per-eojeol independent tokenization
            let pred_raw = tokenizer.tokenize(eojeols[eo_i]);
            let pred_sejong = converter.convert_tokens(&pred_raw);
            let pred_slice: Vec<(String, String)> = pred_sejong
                .iter()
                .map(|t| (SejongConverter::normalize_jamo(&t.surface), t.pos.clone()))
                .collect();
            let pred_surface: String = pred_slice.iter().map(|(s, _)| s.as_str()).collect();

            if pred_surface != gold_surface {
                cat_counts.entry("SURFACE_MISMATCH").and_modify(|c| *c += 1).or_insert(1);
                if samples_surface_mismatch.len() < sample_cap {
                    samples_surface_mismatch.push((
                        gold_sentence.text.clone(),
                        gold_surface.clone(),
                        pred_surface.clone(),
                    ));
                }
                continue;
            }

            // Surface aligned. Now classify by split count.
            slice_lenient_correct += 1;
            let count_p = pred_slice.len();
            *split_pattern_counts.entry((count_g, count_p)).or_insert(0) += 1;

            let same_split = count_g == count_p;
            let all_match = same_split
                && gold_slice
                    .iter()
                    .zip(pred_slice.iter())
                    .all(|(g, (ps, pp))| g.surface == *ps && g.pos == *pp);

            if all_match {
                cat_counts.entry("EXACT").and_modify(|c| *c += 1).or_insert(1);
                continue;
            }

            if same_split {
                let surface_same = gold_slice
                    .iter()
                    .zip(pred_slice.iter())
                    .all(|(g, (ps, _))| g.surface == *ps);
                if surface_same {
                    cat_counts.entry("POS_DIFF").and_modify(|c| *c += 1).or_insert(1);
                } else {
                    cat_counts.entry("INNER_SPLIT_DIFF").and_modify(|c| *c += 1).or_insert(1);
                }
                continue;
            }

            // Different split count
            if count_g == 1 && count_p > 1 {
                cat_counts.entry("GOLD_SINGLE_PRED_MULTI").and_modify(|c| *c += 1).or_insert(1);
                *top_compounds.entry(gold_surface.clone()).or_insert(0) += 1;
                if samples_g1pn.len() < sample_cap {
                    let pred_tokens: Vec<String> = pred_slice
                        .iter()
                        .map(|(s, p)| format!("{s}/{p}"))
                        .collect();
                    samples_g1pn.push((
                        gold_sentence.text.clone(),
                        format!("{}/{}", gold_slice[0].surface, gold_slice[0].pos),
                        pred_tokens,
                    ));
                }
            } else if count_g > 1 && count_p == 1 {
                cat_counts.entry("GOLD_MULTI_PRED_SINGLE").and_modify(|c| *c += 1).or_insert(1);
                if samples_gnp1.len() < sample_cap {
                    let gold_tokens: Vec<String> =
                        gold_slice.iter().map(|t| format!("{}/{}", t.surface, t.pos)).collect();
                    samples_gnp1.push((
                        gold_sentence.text.clone(),
                        format!("{}/{}", pred_slice[0].0, pred_slice[0].1),
                        gold_tokens,
                    ));
                }
            } else {
                cat_counts.entry("SPLIT_DIFFERENT").and_modify(|c| *c += 1).or_insert(1);
                if samples_split_diff.len() < sample_cap {
                    let gold_tokens: Vec<String> =
                        gold_slice.iter().map(|t| format!("{}/{}", t.surface, t.pos)).collect();
                    let pred_tokens: Vec<String> = pred_slice
                        .iter()
                        .map(|(s, p)| format!("{s}/{p}"))
                        .collect();
                    samples_split_diff.push((
                        gold_sentence.text.clone(),
                        gold_surface.clone(),
                        gold_tokens,
                        pred_tokens,
                    ));
                }
            }
        }
    }

    println!("\n{}", "=".repeat(70));
    println!("  COMPOUND NOUN SPLIT ANALYSIS (Sprint 127 P1)");
    println!("  Dataset: {} sentences, {} eojeols", dataset.len(), total_eojeols);
    println!("{}\n", "=".repeat(70));

    println!("=== Eojeol Category Counts ===");
    let mut sorted_cats: Vec<_> = cat_counts.iter().collect();
    sorted_cats.sort_by_key(|x| std::cmp::Reverse(*x.1));
    for (cat, count) in &sorted_cats {
        let pct = **count as f64 / total_eojeols.max(1) as f64 * 100.0;
        println!("  {cat:<30}  {count:>5} ({pct:.1}%)");
    }

    let exact = *cat_counts.get("EXACT").unwrap_or(&0);
    let exact_pct = exact as f64 / total_eojeols.max(1) as f64 * 100.0;
    let slice_pct = slice_lenient_correct as f64 / total_eojeols.max(1) as f64 * 100.0;
    println!("\n=== Headline ===");
    println!("  Strict eojeol (EXACT only):   {exact_pct:.1}% ({exact} / {total_eojeols})");
    println!(
        "  Slice-lenient ceiling:        {slice_pct:.1}% ({slice_lenient_correct} / {total_eojeols})"
    );
    println!("  (Slice-lenient = any eojeol with matching cumulative surface)");

    println!("\n=== Split Pattern Frequency (gold_count, pred_count) — surface-aligned ===");
    let mut sorted_patterns: Vec<_> = split_pattern_counts.iter().collect();
    sorted_patterns.sort_by_key(|x| std::cmp::Reverse(*x.1));
    for ((g, p), c) in sorted_patterns.iter().take(15) {
        let marker = if g == p { "" } else { "  ←diff" };
        println!("  gold={g}  pred={p}  →  {c}건{marker}");
    }

    println!("\n=== Top compound nouns (gold-1 / pred-N), top 30 ===");
    let mut top: Vec<_> = top_compounds.iter().collect();
    top.sort_by_key(|x| std::cmp::Reverse(*x.1));
    for (surf, cnt) in top.iter().take(30) {
        println!("  {cnt:>3}× {surf}");
    }

    println!(
        "\n=== Samples: GOLD_SINGLE_PRED_MULTI (mecab over-splits compound) — {} ===",
        samples_g1pn.len()
    );
    for (sent, gold_tok, pred_toks) in samples_g1pn.iter().take(15) {
        println!(
            "  gold: {gold_tok:<20}  pred: [{}]  in: {sent}",
            pred_toks.join(", ")
        );
    }

    println!(
        "\n=== Samples: GOLD_MULTI_PRED_SINGLE (mecab merges) — {} ===",
        samples_gnp1.len()
    );
    for (sent, pred_tok, gold_toks) in samples_gnp1.iter().take(15) {
        println!(
            "  gold: [{}]  pred: {pred_tok:<20}  in: {sent}",
            gold_toks.join(", ")
        );
    }

    println!(
        "\n=== Samples: SPLIT_DIFFERENT (both split, different boundary) — {} ===",
        samples_split_diff.len()
    );
    for (sent, surf, gold_toks, pred_toks) in samples_split_diff.iter().take(10) {
        println!("  surface={surf}  in: {sent}");
        println!("    gold: [{}]", gold_toks.join(", "));
        println!("    pred: [{}]", pred_toks.join(", "));
    }

    println!(
        "\n=== Samples: SURFACE_MISMATCH (orthographic transform / boundary drift) — {} ===",
        samples_surface_mismatch.len()
    );
    for (sent, gs, ps) in samples_surface_mismatch.iter().take(10) {
        println!("  gold_surface={gs:<20}  pred_surface={ps:<20}  in: {sent}");
    }
}

/// Sprint 128 P2: Full 5-mode KLUE DP measurement (surface lenient added)
///
/// Compares strict / lenient (conservative) / lenient (practical) /
/// `surface_canonical` / `surface_canonical_lenient` + practical POS modes.
/// Reports morpheme + eojeol for each mode.
#[test]
#[ignore = "requires KLUE DP eval data + system dictionary"]
fn test_klue_dp_surface_lenient_full() {
    use mecab_ko_core::evaluate::{
        evaluate_dataset_dual, evaluate_dataset_dual_lenient,
        evaluate_dataset_dual_per_eojeol_with_match, evaluate_dataset_dual_with_match,
        evaluate_dataset_dual_with_pos_match, pos_eq_strict, pos_tags_equivalent_practical,
        surface_eq_canonical, surface_eq_canonical_lenient, surface_eq_strict,
    };

    let project_root = project_root();
    let mut tokenizer = make_tokenizer(&project_root);

    let eval_path = project_root.join("data/eval/klue_dp_val.tsv");
    if !eval_path.exists() {
        println!("Skipping: data/eval/klue_dp_val.tsv not found");
        return;
    }

    let dataset = TestDataset::from_tsv(eval_path.to_str().unwrap())
        .expect("Failed to load KLUE DP val");

    println!("\n=== KLUE DP 5-mode (Sprint 128 P2) ===");
    println!("Dataset: {} sentences", dataset.len());

    let strict = evaluate_dataset_dual(&mut tokenizer, &dataset);
    let lenient = evaluate_dataset_dual_lenient(&mut tokenizer, &dataset);
    let practical = evaluate_dataset_dual_with_pos_match(
        &mut tokenizer,
        &dataset,
        pos_tags_equivalent_practical,
    );
    let surface_can = evaluate_dataset_dual_with_match(
        &mut tokenizer,
        &dataset,
        pos_eq_strict,
        surface_eq_canonical,
    );
    let combined_seq = evaluate_dataset_dual_with_match(
        &mut tokenizer,
        &dataset,
        pos_tags_equivalent_practical,
        surface_eq_canonical_lenient,
    );

    // Per-eojeol algorithm (Sprint 128 P1)
    let pe_strict = evaluate_dataset_dual_per_eojeol_with_match(
        &mut tokenizer,
        &dataset,
        pos_eq_strict,
        surface_eq_strict,
    );
    let pe_practical = evaluate_dataset_dual_per_eojeol_with_match(
        &mut tokenizer,
        &dataset,
        pos_tags_equivalent_practical,
        surface_eq_strict,
    );
    let pe_surface_can = evaluate_dataset_dual_per_eojeol_with_match(
        &mut tokenizer,
        &dataset,
        pos_eq_strict,
        surface_eq_canonical,
    );
    let pe_combined = evaluate_dataset_dual_per_eojeol_with_match(
        &mut tokenizer,
        &dataset,
        pos_tags_equivalent_practical,
        surface_eq_canonical_lenient,
    );

    let report = |name: &str, r: &mecab_ko_core::evaluate::DualMetricResult, baseline_eo: f64| {
        let m = r.morpheme.token_accuracy * 100.0;
        let e = r.eojeol_accuracy * 100.0;
        let delta = e - baseline_eo;
        println!(
            "  {name:<55}  morph {m:5.1}%   eo {e:5.1}% ({:>5}/{})   Δeo {delta:+.1}pp",
            r.eojeol_correct, r.eojeol_total
        );
    };

    let baseline = strict.eojeol_accuracy * 100.0;
    println!("\n--- Sequence-based eojeol (legacy, cascade) ---");
    report("strict (POS strict / surface strict)", &strict, baseline);
    report("lenient (POS conservative / surface strict)", &lenient, baseline);
    report("practical (POS practical / surface strict)", &practical, baseline);
    report("surface_canonical (POS strict / surface canonical)", &surface_can, baseline);
    report(
        "combined (POS practical / surface canonical+lenient)",
        &combined_seq,
        baseline,
    );

    let pe_baseline = pe_strict.eojeol_accuracy * 100.0;
    println!("\n--- Per-eojeol algorithm (Sprint 128 P1, no cascade) ---");
    report(
        "per-eojeol strict (POS strict / surface strict)",
        &pe_strict,
        pe_baseline,
    );
    report(
        "per-eojeol practical (POS practical / surface strict)",
        &pe_practical,
        pe_baseline,
    );
    report(
        "per-eojeol surface_canonical",
        &pe_surface_can,
        pe_baseline,
    );
    report(
        "per-eojeol combined (POS practical / surface canon+lenient)",
        &pe_combined,
        pe_baseline,
    );

    // Floor: per-eojeol strict must lift far above sequence strict
    assert!(
        pe_strict.eojeol_accuracy >= 0.45,
        "per-eojeol strict eojeol {:.1}% < 45% floor (cascade-free should be ~52%)",
        pe_strict.eojeol_accuracy * 100.0
    );
    assert!(
        pe_combined.eojeol_accuracy >= pe_strict.eojeol_accuracy,
        "per-eojeol combined must be >= per-eojeol strict"
    );
    // Surface canonical alone gives +0pp eojeol lift in per-eojeol mode:
    // morpheme split mismatch still fails after surface matching. This is the same
    // semantic loss tradeoff Sprint 127 P1 rejected (slice-lenient ceiling 87.7%).
    // SURFACE_MISMATCH 12.3% is an eojeol-concat level diff, not morpheme-level.
    assert!(
        pe_surface_can.eojeol_accuracy >= pe_strict.eojeol_accuracy,
        "per-eojeol surface_canonical {:.1}% must be >= strict {:.1}% (monotonic)",
        pe_surface_can.eojeol_accuracy * 100.0,
        pe_strict.eojeol_accuracy * 100.0
    );

    println!(
        "\nPASSED — per-eojeol combined eojeol {:.1}% (sequence practical was 21.7%, per-eojeol strict baseline {:.1}%)",
        pe_combined.eojeol_accuracy * 100.0,
        pe_baseline
    );
}

/// Sprint 128 P2: Surface mismatch normalization analysis
///
/// Re-runs the per-eojeol analysis but for `SURFACE_MISMATCH` cases tries:
/// - NFC compose (jamo → syllable, e.g. "하ㅁ께" → "함께")
/// - NFC + 어미 표기 동치 (였 ↔ 았, 어 ↔ 여)
///
/// Reports how many `SURFACE_MISMATCH` eojeols would be absorbed at each level.
#[test]
#[ignore = "requires KLUE DP eval data + system dictionary"]
fn test_klue_dp_surface_normalization_analysis() {
    use mecab_ko_core::sejong::SejongConverter;
    use mecab_ko_hangul::{compose_str, decompose_str};
    use std::collections::HashMap;

    // Inflectional ending equivalence (analysis-only, mirrors evaluate.rs).
    // Sprint 128: 하았→하였, 하어→하여
    // Sprint 134: 하아→하여, 이습니다→입니다
    fn normalize_endings(s: &str) -> String {
        let mut out = String::with_capacity(s.len());
        let chars: Vec<char> = s.chars().collect();
        for (i, &c) in chars.iter().enumerate() {
            let prev = if i > 0 { chars[i - 1] } else { '\0' };
            if prev == '하' && (c == '았' || c == '아') {
                out.push(if c == '았' { '였' } else { '여' });
            } else if c == '어' && prev == '하' {
                out.push('여');
            } else {
                out.push(c);
            }
        }
        if out.contains("이습니다") {
            out = out.replace("이습니다", "입니다");
        }
        out
    }

    let project_root = project_root();
    let mut tokenizer = make_tokenizer(&project_root);

    let eval_path = project_root.join("data/eval/klue_dp_val.tsv");
    if !eval_path.exists() {
        println!("Skipping: data/eval/klue_dp_val.tsv not found");
        return;
    }

    let dataset = TestDataset::from_tsv(eval_path.to_str().unwrap())
        .expect("Failed to load KLUE DP val");
    let converter = SejongConverter::new();

    // (gold_concat, pred_concat) → count
    let mut nfc_absorbed: usize = 0;
    let mut nfc_plus_endings_absorbed: usize = 0;
    let mut still_mismatch: usize = 0;
    let mut still_mismatch_samples: Vec<(String, String, String, String, String)> = Vec::new();
    let mut total_eojeols: usize = 0;
    let mut total_surface_mismatch: usize = 0;
    let mut diff_pattern_counts: HashMap<(String, String), usize> = HashMap::new();

    for gold_sentence in &dataset.sentences {
        let Some(eojeol_counts) = &gold_sentence.eojeol_counts else {
            continue;
        };
        let eojeols: Vec<&str> = gold_sentence.text.split_whitespace().collect();
        if eojeols.len() != eojeol_counts.len() {
            continue;
        }

        let mut gold_idx: usize = 0;
        for (eo_i, &count_g) in eojeol_counts.iter().enumerate() {
            total_eojeols += 1;
            if gold_idx + count_g > gold_sentence.tokens.len() {
                gold_idx = gold_sentence.tokens.len();
                continue;
            }
            let gold_slice = &gold_sentence.tokens[gold_idx..gold_idx + count_g];
            gold_idx += count_g;

            let gold_surface: String = gold_slice.iter().map(|t| t.surface.as_str()).collect();
            let pred_raw = tokenizer.tokenize(eojeols[eo_i]);
            let pred_sejong = converter.convert_tokens(&pred_raw);
            let pred_surface: String = pred_sejong
                .iter()
                .map(|t| SejongConverter::normalize_jamo(&t.surface))
                .collect();

            if gold_surface == pred_surface {
                continue;
            }
            total_surface_mismatch += 1;

            // Canonical form: fully decompose then re-compose (handles syllable + jamo mix
            // like "하ㄴ" → "ㅎㅏㄴ" → "한")
            let gold_nfc = compose_str(&decompose_str(&gold_surface));
            let pred_nfc = compose_str(&decompose_str(&pred_surface));
            if gold_nfc == pred_nfc {
                nfc_absorbed += 1;
                continue;
            }

            let gold_norm = normalize_endings(&gold_nfc);
            let pred_norm = normalize_endings(&pred_nfc);
            if gold_norm == pred_norm {
                nfc_plus_endings_absorbed += 1;
                continue;
            }

            still_mismatch += 1;
            *diff_pattern_counts
                .entry((gold_nfc.clone(), pred_nfc.clone()))
                .or_insert(0) += 1;
            if still_mismatch_samples.len() < 30 {
                still_mismatch_samples.push((
                    gold_sentence.text.clone(),
                    gold_surface,
                    pred_surface,
                    gold_nfc,
                    pred_nfc,
                ));
            }
        }
    }

    println!("\n{}", "=".repeat(70));
    println!("  SURFACE NORMALIZATION ANALYSIS (Sprint 128 P2)");
    println!(
        "  Total eojeols: {total_eojeols}, raw SURFACE_MISMATCH: {total_surface_mismatch}"
    );
    println!("{}\n", "=".repeat(70));

    let abs_pct = |c: usize| (c as f64 / total_surface_mismatch.max(1) as f64) * 100.0;
    let total_pct = |c: usize| (c as f64 / total_eojeols.max(1) as f64) * 100.0;
    println!("=== Absorption tier ===");
    println!(
        "  NFC compose only:        {nfc_absorbed:>5} ({:.1}% of mismatch / {:.1}% of total)",
        abs_pct(nfc_absorbed),
        total_pct(nfc_absorbed)
    );
    println!(
        "  NFC + 하았→하였/하어→하여: {nfc_plus_endings_absorbed:>5} ({:.1}% of mismatch / {:.1}% of total)",
        abs_pct(nfc_plus_endings_absorbed),
        total_pct(nfc_plus_endings_absorbed)
    );
    println!(
        "  Still mismatch:          {still_mismatch:>5} ({:.1}% of mismatch / {:.1}% of total)",
        abs_pct(still_mismatch),
        total_pct(still_mismatch)
    );
    let total_absorbed = nfc_absorbed + nfc_plus_endings_absorbed;
    println!(
        "\n  Total absorbed:          {total_absorbed:>5} ({:.1}% of mismatch / +{:.1}pp eojeol lift)",
        abs_pct(total_absorbed),
        total_pct(total_absorbed)
    );

    println!("\n=== Top remaining mismatch patterns (after both norms), top 25 ===");
    let mut sorted_patterns: Vec<_> = diff_pattern_counts.iter().collect();
    sorted_patterns.sort_by_key(|x| std::cmp::Reverse(*x.1));
    for ((g, p), c) in sorted_patterns.iter().take(25) {
        println!("  {c:>3}× gold={g:<25}  pred={p}");
    }

    println!("\n=== Sample remaining mismatches (sentence context) ===");
    for (sent, gs, ps, gn, pn) in still_mismatch_samples.iter().take(15) {
        println!(
            "  raw: gold={gs} / pred={ps}\n  nfc: gold={gn} / pred={pn}\n  in: {sent}\n"
        );
    }
}

/// Sprint 129 P3: Real error analysis — extended POS confusion with surface frequency
///
/// Extends Sprint 126 `test_klue_dp_nng_nnp_analysis` to cover MAG↔NNG (95건) and
/// VV↔NNG (43건) confusion in addition to NNG/NNP/NNB. Critically, also reports
/// **per-confusion surface frequency** so we can decide:
///
/// - Same `surface` appearing 10+ times across NNG/NNP confusion → likely missing NNP
///   in dictionary (fix: add to user-dict with `cost=-5000`)
/// - Diverse surfaces, low repetition → context-dependent (fix: CRF retrain only)
/// - High-frequency single-character or 의존 명사-like → KLUE convention diff
///
/// Output groups by `(gold_pos, pred_pos)` and within each group lists top 30
/// surfaces sorted by frequency, plus 5 sample sentences for the top surface.
#[test]
#[ignore = "requires KLUE DP eval data + system dictionary"]
fn test_klue_dp_real_error_analysis() {
    use mecab_ko_core::sejong::SejongConverter;
    use std::collections::HashMap;

    // (gold_pos, pred_pos) -> HashMap<surface, (count, Vec<sentence_sample>)>
    type SurfaceData = HashMap<String, (usize, Vec<String>)>;

    let project_root = project_root();
    let mut tokenizer = make_tokenizer(&project_root);

    let eval_path = project_root.join("data/eval/klue_dp_val.tsv");
    if !eval_path.exists() {
        println!("Skipping: data/eval/klue_dp_val.tsv not found");
        return;
    }

    let dataset = TestDataset::from_tsv(eval_path.to_str().unwrap())
        .expect("Failed to load KLUE DP val");
    let converter = SejongConverter::new();

    // Wider target set than Sprint 126 — includes MAG, VV, VA, MM for real error coverage
    let target_tags = ["NNG", "NNP", "NNB", "MAG", "VV", "VA", "MM"];

    let mut cases: HashMap<(String, String), SurfaceData> = HashMap::new();

    for gold_sentence in &dataset.sentences {
        let pred_raw = tokenizer.tokenize(&gold_sentence.text);
        let sejong_tokens = converter.convert_tokens(&pred_raw);

        let pred_pairs: Vec<(String, String)> = sejong_tokens
            .iter()
            .map(|t| (SejongConverter::normalize_jamo(&t.surface), t.pos.clone()))
            .collect();

        let min_len = gold_sentence.tokens.len().min(pred_pairs.len());
        for (g, (p_surf, p_pos)) in gold_sentence
            .tokens
            .iter()
            .zip(pred_pairs.iter())
            .take(min_len)
        {
            if g.surface != *p_surf {
                continue;
            }
            if g.pos == *p_pos {
                continue;
            }
            if !target_tags.contains(&g.pos.as_str())
                && !target_tags.contains(&p_pos.as_str())
            {
                continue;
            }
            let entry = cases
                .entry((g.pos.clone(), p_pos.clone()))
                .or_default()
                .entry(g.surface.clone())
                .or_insert_with(|| (0, Vec::new()));
            entry.0 += 1;
            if entry.1.len() < 3 {
                entry.1.push(gold_sentence.text.clone());
            }
        }
    }

    println!("\n{}", "=".repeat(70));
    println!("  REAL ERROR ANALYSIS (Sprint 129 P3)");
    println!("  Dataset: {} sentences", dataset.len());
    println!("{}", "=".repeat(70));

    let mut by_count: Vec<((String, String), usize, SurfaceData)> = cases
        .iter()
        .map(|(k, v)| (k.clone(), v.values().map(|(c, _)| c).sum::<usize>(), v.clone()))
        .collect();
    by_count.sort_by_key(|x| std::cmp::Reverse(x.1));

    println!("\n=== Confusion summary (sorted by count) ===");
    for ((g, p), total, _) in &by_count {
        println!("  {g:<6} → {p:<6}  {total:>5}건");
    }

    let grand_total: usize = by_count.iter().map(|x| x.1).sum();
    println!("\nTotal target-related POS_ONLY errors: {grand_total}");

    println!("\n=== Per-confusion top surfaces (with frequency) ===");
    for ((g, p), total, surf_map) in by_count.iter().take(10) {
        println!("\n>>> {g} → {p}  ({total}건, unique surfaces: {})", surf_map.len());
        let mut sorted_surfaces: Vec<_> = surf_map.iter().collect();
        sorted_surfaces.sort_by_key(|x| std::cmp::Reverse(x.1.0));

        // Show top 30 surfaces with counts
        for (i, (surf, (count, samples))) in sorted_surfaces.iter().take(30).enumerate() {
            println!("  [{:>2}] {count:>3}× {surf}", i + 1);
            if i < 3 {
                for s in samples.iter().take(2) {
                    println!("       in: {s}");
                }
            }
        }

        // Frequency tiers — for "add to dict" decision
        let high_freq = sorted_surfaces.iter().filter(|x| x.1.0 >= 5).count();
        let mid_freq = sorted_surfaces.iter().filter(|x| x.1.0 >= 2 && x.1.0 < 5).count();
        let singleton = sorted_surfaces.iter().filter(|x| x.1.0 == 1).count();
        println!("  --- Frequency tiers ---");
        println!("    >= 5 occurrences: {high_freq} surfaces (likely missing dict entries)");
        println!("    2-4 occurrences:  {mid_freq} surfaces (ambiguous / context-dep)");
        println!("    singleton:        {singleton} surfaces (noise / convention)");
    }
}

/// Sprint 129 P3: `GOLD_SINGLE_PRED_MULTI` surface frequency
///
/// Sprint 127 P1 identified 553 cases (2.5%) where KLUE treats a token as a single
/// morpheme but mecab over-splits. This extracts the **frequency-sorted surface list**
/// with their mecab split patterns, to decide:
///
/// - Same surface appears N times with same split → high-confidence missing dict entry
/// - Diverse splits per surface → ambiguous (cost adjust risky)
///
/// Uses per-eojeol independent tokenization (same as Sprint 127). Surface here means
/// the gold's single-morph surface, and we capture both mecab's split count and the
/// surface+pos pattern of the split.
#[test]
#[ignore = "requires KLUE DP eval data + system dictionary"]
fn test_klue_dp_gold_single_pred_multi_analysis() {
    use mecab_ko_core::sejong::SejongConverter;
    use std::collections::HashMap;

    // (gold_surface, gold_pos) -> HashMap<pred_split_pattern, (count, Vec<sentence>)>
    // pred_split_pattern: "한국/NNP + 전자/NNG + 통신/NNG"
    type SplitData = HashMap<String, (usize, Vec<String>)>;

    let project_root = project_root();
    let mut tokenizer = make_tokenizer(&project_root);

    let eval_path = project_root.join("data/eval/klue_dp_val.tsv");
    if !eval_path.exists() {
        println!("Skipping: data/eval/klue_dp_val.tsv not found");
        return;
    }

    let dataset = TestDataset::from_tsv(eval_path.to_str().unwrap())
        .expect("Failed to load KLUE DP val");
    let converter = SejongConverter::new();

    let mut cases: HashMap<(String, String), SplitData> = HashMap::new();
    let mut total_gold_single_pred_multi = 0usize;

    for gold_sentence in &dataset.sentences {
        let Some(eojeol_counts) = &gold_sentence.eojeol_counts else {
            continue;
        };
        let eojeols: Vec<&str> = gold_sentence.text.split_whitespace().collect();
        if eojeols.len() != eojeol_counts.len() {
            continue;
        }

        let mut gold_idx = 0usize;
        for (eo_i, &count_g) in eojeol_counts.iter().enumerate() {
            if gold_idx + count_g > gold_sentence.tokens.len() {
                break;
            }
            let gold_slice = &gold_sentence.tokens[gold_idx..gold_idx + count_g];
            gold_idx += count_g;

            // Only care about gold-single cases
            if gold_slice.len() != 1 {
                continue;
            }
            let gold_morph = &gold_slice[0];

            let pred_raw = tokenizer.tokenize(eojeols[eo_i]);
            let pred_sejong = converter.convert_tokens(&pred_raw);
            let pred_morphs: Vec<(String, String)> = pred_sejong
                .iter()
                .map(|t| (SejongConverter::normalize_jamo(&t.surface), t.pos.clone()))
                .collect();

            // Only gold-single + pred-multi
            if pred_morphs.len() < 2 {
                continue;
            }

            // Surface concat must match (otherwise it's SURFACE_MISMATCH, not over-split)
            let gold_concat: String = gold_slice.iter().map(|t| t.surface.as_str()).collect();
            let pred_concat: String = pred_morphs.iter().map(|(s, _)| s.as_str()).collect();
            if gold_concat != pred_concat {
                continue;
            }

            total_gold_single_pred_multi += 1;
            let pred_pattern: String = pred_morphs
                .iter()
                .map(|(s, p)| format!("{s}/{p}"))
                .collect::<Vec<_>>()
                .join(" + ");

            let entry = cases
                .entry((gold_morph.surface.clone(), gold_morph.pos.clone()))
                .or_default()
                .entry(pred_pattern)
                .or_insert_with(|| (0, Vec::new()));
            entry.0 += 1;
            if entry.1.len() < 2 {
                entry.1.push(gold_sentence.text.clone());
            }
        }
    }

    println!("\n{}", "=".repeat(70));
    println!("  GOLD_SINGLE_PRED_MULTI ANALYSIS (Sprint 129 P3)");
    println!("  Total cases: {total_gold_single_pred_multi}");
    println!("{}", "=".repeat(70));

    // Aggregate per (gold_surface, gold_pos) total
    let mut by_total: Vec<((String, String), usize, SplitData)> = cases
        .iter()
        .map(|(k, v)| (k.clone(), v.values().map(|(c, _)| c).sum::<usize>(), v.clone()))
        .collect();
    by_total.sort_by_key(|x| std::cmp::Reverse(x.1));

    println!("\n=== Top 60 gold-single surfaces (mecab over-splits) ===");
    for (i, ((surf, pos), total, split_map)) in by_total.iter().take(60).enumerate() {
        let unique_splits = split_map.len();
        println!(
            "  [{:>2}] {total:>3}× {surf}/{pos}  ({unique_splits} unique splits)",
            i + 1
        );
        // Show all splits if total >= 3, else just the top one
        let mut sorted_splits: Vec<_> = split_map.iter().collect();
        sorted_splits.sort_by_key(|x| std::cmp::Reverse(x.1.0));
        for (split, (count, samples)) in sorted_splits.iter().take(3) {
            println!("       {count}× pred: {split}");
            if i < 10 {
                for s in samples.iter().take(1) {
                    println!("          in: {s}");
                }
            }
        }
    }

    // Frequency tiers
    let high_freq = by_total.iter().filter(|x| x.1 >= 5).count();
    let mid_freq = by_total.iter().filter(|x| x.1 >= 2 && x.1 < 5).count();
    let singleton = by_total.iter().filter(|x| x.1 == 1).count();
    let high_freq_total: usize = by_total.iter().filter(|x| x.1 >= 5).map(|x| x.1).sum();
    let mid_freq_total: usize = by_total
        .iter()
        .filter(|x| x.1 >= 2 && x.1 < 5)
        .map(|x| x.1)
        .sum();

    println!("\n=== Frequency tiers (gold-single surface) ===");
    println!(
        "  >= 5 occurrences: {high_freq:>4} surfaces, {high_freq_total} cases (dict-add candidates)"
    );
    println!(
        "  2-4 occurrences:  {mid_freq:>4} surfaces, {mid_freq_total} cases (review needed)"
    );
    println!("  singleton:        {singleton:>4} surfaces (long tail)");

    // POS distribution of gold-single morphs
    let mut pos_dist: HashMap<String, usize> = HashMap::new();
    for ((_, pos), total, _) in &by_total {
        *pos_dist.entry(pos.clone()).or_insert(0) += total;
    }
    let mut pos_sorted: Vec<_> = pos_dist.iter().collect();
    pos_sorted.sort_by_key(|x| std::cmp::Reverse(*x.1));
    println!("\n=== Gold POS distribution (what KLUE treats as single) ===");
    for (pos, count) in pos_sorted.iter().take(15) {
        let pct = **count as f64 / total_gold_single_pred_multi.max(1) as f64 * 100.0;
        println!("  {pos:<6} {count:>4} ({pct:.1}%)");
    }
}

/// Sprint 133 P2: Eojeol surface-only metric on KLUE DP
///
/// 검색/인덱싱 use case 측정. POS 무시, split 무시, surface concat 일치만 정답.
/// `surface_eq` 모드 3종 (strict / canonical / `canonical_lenient`) 비교.
///
/// 기대값: Sprint 127 P1의 slice-lenient ceiling 87.7% 근사 (canonical 모드).
/// strict 모드는 jamo decomposition convention 차이로 더 낮을 수 있음.
#[test]
#[ignore = "requires KLUE DP eval data + system dictionary"]
fn test_klue_dp_eojeol_surface_only() {
    use mecab_ko_core::evaluate::{
        evaluate_dataset_eojeol_surface_only_with_match,
        surface_eq_canonical, surface_eq_canonical_lenient, surface_eq_strict,
    };

    // Floor constants (top of function — clippy items-after-statements).
    // Sprint 127 P1 slice-lenient ceiling은 약 87.7% (canonical 근사).
    // Strict는 jamo decomposition convention 차이로 더 낮음.
    const STRICT_FLOOR: f64 = 0.50;
    const CANONICAL_FLOOR: f64 = 0.80;

    let project_root = project_root();
    let mut tokenizer = make_tokenizer(&project_root);

    let eval_path = project_root.join("data/eval/klue_dp_val.tsv");
    if !eval_path.exists() {
        println!("Skipping: data/eval/klue_dp_val.tsv not found");
        return;
    }

    let dataset = TestDataset::from_tsv(eval_path.to_str().unwrap())
        .expect("Failed to load KLUE DP val");

    let strict =
        evaluate_dataset_eojeol_surface_only_with_match(&mut tokenizer, &dataset, surface_eq_strict);
    let canonical = evaluate_dataset_eojeol_surface_only_with_match(
        &mut tokenizer,
        &dataset,
        surface_eq_canonical,
    );
    let canonical_lenient = evaluate_dataset_eojeol_surface_only_with_match(
        &mut tokenizer,
        &dataset,
        surface_eq_canonical_lenient,
    );

    println!("\n{}", "=".repeat(70));
    println!("  EOJEOL SURFACE-ONLY METRIC (Sprint 133 P2)");
    println!("  Use case: 검색/인덱싱. POS와 split 무시, surface concat 일치만 정답.");
    println!("  의미 손실: 형태소 분석 품질은 측정하지 않음.");
    println!("{}", "=".repeat(70));

    println!("\n  strict           {}", strict.format_report());
    println!("  canonical        {}", canonical.format_report());
    println!("  canonical_lenient {}", canonical_lenient.format_report());

    let delta_canon = (canonical.accuracy - strict.accuracy) * 100.0;
    let delta_lenient = (canonical_lenient.accuracy - strict.accuracy) * 100.0;
    println!("\n  canonical Δ vs strict:         {delta_canon:+.1}pp");
    println!("  canonical_lenient Δ vs strict: {delta_lenient:+.1}pp");

    // Sanity: canonical >= strict (canonical만 더 관대)
    assert!(
        canonical.accuracy >= strict.accuracy,
        "canonical ({:.4}) should be >= strict ({:.4})",
        canonical.accuracy,
        strict.accuracy
    );
    assert!(
        canonical_lenient.accuracy >= canonical.accuracy,
        "canonical_lenient ({:.4}) should be >= canonical ({:.4})",
        canonical_lenient.accuracy,
        canonical.accuracy
    );

    // Floor enforcement (constants defined at top of function)
    assert!(
        strict.accuracy >= STRICT_FLOOR,
        "strict {:.1}% < floor {:.0}%",
        strict.accuracy * 100.0,
        STRICT_FLOOR * 100.0
    );
    assert!(
        canonical.accuracy >= CANONICAL_FLOOR,
        "canonical {:.1}% < floor {:.0}%",
        canonical.accuracy * 100.0,
        CANONICAL_FLOOR * 100.0
    );

    println!("\nPASSED — strict {:.1}% / canonical {:.1}% / canonical_lenient {:.1}%",
        strict.accuracy * 100.0,
        canonical.accuracy * 100.0,
        canonical_lenient.accuracy * 100.0);
}

/// Sprint 139 P2 — UD Korean-Kaist silver baseline measurement.
///
/// 데이터셋: `data/eval/ud_kaist_test.tsv` (1,638 sentences, ko_kaist-ud-test.conllu 변환)
/// 라이선스: CC BY-SA 4.0 (UD), 변환 코드 + 변환 결과도 동일 라이선스 상속
///
/// 측정: morph strict / morph practical (lenient) / per-eojeol strict / per-eojeol practical
/// Silver 변환이므로 KLUE DP보다 낮을 가능성 (KAIST→Sejong lossy 매핑).
#[test]
#[ignore = "requires UD Korean-Kaist eval data + system dictionary"]
fn test_ud_kaist_dual_metric() {
    use mecab_ko_core::evaluate::{
        evaluate_dataset_dual, evaluate_dataset_dual_with_pos_match,
        pos_tags_equivalent_practical,
    };

    let project_root = project_root();
    let mut tokenizer = make_tokenizer(&project_root);

    let eval_path = project_root.join("data/eval/ud_kaist_test.tsv");
    if !eval_path.exists() {
        println!("Skipping: data/eval/ud_kaist_test.tsv not found");
        return;
    }

    let dataset = TestDataset::from_tsv(eval_path.to_str().unwrap())
        .expect("Failed to load UD Kaist test TSV");

    let strict = evaluate_dataset_dual(&mut tokenizer, &dataset);
    let practical = evaluate_dataset_dual_with_pos_match(
        &mut tokenizer, &dataset, pos_tags_equivalent_practical,
    );

    let strict_morph = strict.morpheme.token_accuracy * 100.0;
    let strict_eo = strict.eojeol_accuracy * 100.0;
    let practical_morph = practical.morpheme.token_accuracy * 100.0;
    let practical_eo = practical.eojeol_accuracy * 100.0;

    println!("\n=== UD Korean-Kaist (test split, silver) ===");
    println!("Dataset: {} sentences", dataset.len());
    println!("\n--- Strict ---");
    println!("  Morpheme: {strict_morph:.1}%");
    println!("  Eojeol:   {strict_eo:.1}% ({} / {})",
        strict.eojeol_correct, strict.eojeol_total);
    println!("\n--- Practical (NNB/NNG + VA/VV + SP/SC + SS/SY/SSO/SSC + MM 그룹 + SL/NNP) ---");
    println!("  Morpheme: {practical_morph:.1}% [Δ +{:.1}pp vs strict]",
        practical_morph - strict_morph);
    println!("  Eojeol:   {practical_eo:.1}% ({} / {}) [Δ +{:.1}pp vs strict]",
        practical.eojeol_correct, practical.eojeol_total, practical_eo - strict_eo);

    // 회귀 catch: practical >= strict
    assert!(practical.eojeol_accuracy >= strict.eojeol_accuracy);
    assert!(practical.morpheme.token_accuracy >= strict.morpheme.token_accuracy);

    // Silver dataset — KLUE보다 낮은 floor 설정 (lossy 변환)
    assert!(strict.morpheme.token_accuracy >= 0.40,
        "Morph strict {strict_morph:.1}% below 40% floor (UD silver)");

    println!("\nPASSED — strict morph {strict_morph:.1}% / practical morph {practical_morph:.1}%");
}

/// Sprint 137 Track A — Connection cost pair analysis.
///
/// `SPLIT_DIFFERENT` 오류 (both split N>=2, different boundary) 에서 mecab의
/// best path 인접 노드 쌍 `(prev.right_id, curr.left_id)` 빈도를 수집.
/// matrix.def cost 조정 후보를 식별하기 위한 분석.
///
/// 출력: 상위 N개 (`right_id`, `left_id`) 쌍 + left-id.def / right-id.def 자질열.
#[test]
#[ignore = "requires KLUE DP eval data + system dictionary"]
fn test_klue_dp_split_diff_connection_pairs() {
    use mecab_ko_core::sejong::SejongConverter;
    use std::collections::HashMap;
    use std::fs;

    let project_root = project_root();
    let dict_path = dict_path(&project_root);

    // Load left-id.def / right-id.def for feature lookup
    let load_id_def = |path: &std::path::Path| -> HashMap<u16, String> {
        let content = fs::read_to_string(path).unwrap_or_default();
        content.lines()
            .filter_map(|line| {
                let mut parts = line.splitn(2, ' ');
                let id = parts.next()?.parse::<u16>().ok()?;
                let feat = parts.next()?.to_string();
                Some((id, feat))
            })
            .collect()
    };
    let left_id_def = load_id_def(&std::path::Path::new(&dict_path).join("left-id.def"));
    let right_id_def = load_id_def(&std::path::Path::new(&dict_path).join("right-id.def"));
    println!("Loaded left-id.def: {} entries, right-id.def: {} entries",
        left_id_def.len(), right_id_def.len());

    let mut tokenizer = make_tokenizer(&project_root);

    let eval_path = project_root.join("data/eval/klue_dp_val.tsv");
    if !eval_path.exists() {
        println!("Skipping: data/eval/klue_dp_val.tsv not found");
        return;
    }

    let dataset = TestDataset::from_tsv(eval_path.to_str().unwrap())
        .expect("Failed to load KLUE DP val");
    let converter = SejongConverter::new();

    // Pair counts: (right_id_prev, left_id_curr) -> count
    let mut pair_counts: HashMap<(u16, u16), usize> = HashMap::new();
    // Also collect surface bigram for representative examples
    let mut pair_surfaces: HashMap<(u16, u16), Vec<String>> = HashMap::new();
    let surface_sample_cap = 3;

    let mut split_diff_eojeols: usize = 0;
    let mut total_eojeols: usize = 0;

    for gold_sentence in &dataset.sentences {
        let Some(eojeol_counts) = &gold_sentence.eojeol_counts else { continue; };
        let eojeols: Vec<&str> = gold_sentence.text.split_whitespace().collect();
        if eojeols.len() != eojeol_counts.len() { continue; }

        let mut gold_idx: usize = 0;
        for (eo_i, &count_g) in eojeol_counts.iter().enumerate() {
            total_eojeols += 1;
            if gold_idx + count_g > gold_sentence.tokens.len() {
                gold_idx = gold_sentence.tokens.len();
                continue;
            }
            let gold_slice = &gold_sentence.tokens[gold_idx..gold_idx + count_g];
            gold_idx += count_g;
            let gold_surface: String = gold_slice.iter().map(|t| t.surface.as_str()).collect();

            // Per-eojeol independent tokenization (Token level for surface check)
            let pred_raw = tokenizer.tokenize(eojeols[eo_i]);
            let pred_sejong = converter.convert_tokens(&pred_raw);
            let pred_normalized_surface: String = pred_sejong.iter()
                .map(|t| SejongConverter::normalize_jamo(&t.surface))
                .collect();
            if pred_normalized_surface != gold_surface { continue; }

            let count_p = pred_sejong.len();
            // Filter: SPLIT_DIFFERENT only (both split, different count, ignore POS_DIFF/EXACT/G1PM/GMP1)
            if count_g < 2 || count_p < 2 || count_g == count_p { continue; }
            split_diff_eojeols += 1;

            // pred_raw가 만들어진 직후의 lattice를 사용 (Viterbi 결과 포함)
            // 같은 eojeol을 다시 tokenize 하지 않고 직전 결과 활용
            let path = tokenizer.lattice().best_path();
            // best_path already excludes BOS/EOS internally
            let content: &[&_] = &path;
            for window in content.windows(2) {
                let prev_right = window[0].right_id;
                let curr_left = window[1].left_id;
                *pair_counts.entry((prev_right, curr_left)).or_insert(0) += 1;
                let surface_pair = format!("{}|{}", window[0].surface, window[1].surface);
                let entry = pair_surfaces.entry((prev_right, curr_left)).or_default();
                if entry.len() < surface_sample_cap && !entry.contains(&surface_pair) {
                    entry.push(surface_pair);
                }
            }
        }
    }

    println!("\n{}", "=".repeat(70));
    println!("  KLUE DP SPLIT_DIFFERENT Connection Pair Analysis (Sprint 137 A1)");
    println!("  Dataset: {} sentences, total {} eojeols, SPLIT_DIFFERENT {}",
        dataset.len(), total_eojeols, split_diff_eojeols);
    println!("  Total unique pairs: {}, total pair occurrences: {}",
        pair_counts.len(),
        pair_counts.values().sum::<usize>());
    println!("{}\n", "=".repeat(70));

    // Sort pairs by frequency
    let mut sorted: Vec<_> = pair_counts.iter().collect();
    sorted.sort_by_key(|x| std::cmp::Reverse(*x.1));

    let top_n = 30;
    println!("=== Top {top_n} (right_id, left_id) pairs in SPLIT_DIFFERENT eojeols ===");
    println!("{:>5}  {:>5}  {:>6}  {:<40}  {:<40}  samples",
        "RID", "LID", "count", "right.feat (prev node)", "left.feat (curr node)");
    for ((rid, lid), count) in sorted.iter().take(top_n) {
        let r_feat = right_id_def.get(rid).map_or("?", std::string::String::as_str);
        let l_feat = left_id_def.get(lid).map_or("?", std::string::String::as_str);
        let samples = pair_surfaces.get(&(*rid, *lid))
            .map(|v| v.join(", "))
            .unwrap_or_default();
        // Truncate long features for readability
        let r_short = if r_feat.len() > 40 { &r_feat[..40] } else { r_feat };
        let l_short = if l_feat.len() > 40 { &l_feat[..40] } else { l_feat };
        println!("{rid:>5}  {lid:>5}  {count:>6}  {r_short:<40}  {l_short:<40}  {samples}");
    }
}

/// Sprint 143 C — UD Korean-GSD silver baseline measurement.
///
/// 데이터셋: `data/eval/ud_gsd_test.tsv` (971 sentences, ko_gsd-ud-test.conllu 변환)
/// 라이선스: CC BY-SA 4.0 (UD), 변환 결과도 동일 라이선스
///
/// GSD는 KAIST와 다른 XPOS scheme — Sejong 태그를 직접 사용 (identity mapping).
/// 도메인: Google news/web (현대 한국어, KLUE와 유사하나 다른 source).
#[test]
#[ignore = "requires UD Korean-GSD eval data + system dictionary"]
fn test_ud_gsd_dual_metric() {
    use mecab_ko_core::evaluate::{
        evaluate_dataset_dual, evaluate_dataset_dual_with_pos_match,
        pos_tags_equivalent_practical,
    };

    let project_root = project_root();
    let mut tokenizer = make_tokenizer(&project_root);

    let eval_path = project_root.join("data/eval/ud_gsd_test.tsv");
    if !eval_path.exists() {
        println!("Skipping: data/eval/ud_gsd_test.tsv not found");
        return;
    }

    let dataset = TestDataset::from_tsv(eval_path.to_str().unwrap())
        .expect("Failed to load UD GSD test TSV");

    let strict = evaluate_dataset_dual(&mut tokenizer, &dataset);
    let practical = evaluate_dataset_dual_with_pos_match(
        &mut tokenizer, &dataset, pos_tags_equivalent_practical,
    );

    let strict_morph = strict.morpheme.token_accuracy * 100.0;
    let strict_eo = strict.eojeol_accuracy * 100.0;
    let practical_morph = practical.morpheme.token_accuracy * 100.0;
    let practical_eo = practical.eojeol_accuracy * 100.0;

    println!("\n=== UD Korean-GSD (test split, silver, Google news/web) ===");
    println!("Dataset: {} sentences", dataset.len());
    println!("\n--- Strict ---");
    println!("  Morpheme: {strict_morph:.1}%");
    println!("  Eojeol:   {strict_eo:.1}% ({} / {})",
        strict.eojeol_correct, strict.eojeol_total);
    println!("\n--- Practical (NNB/NNG + VA/VV + ...) ---");
    println!("  Morpheme: {practical_morph:.1}% [Δ +{:.1}pp vs strict]",
        practical_morph - strict_morph);
    println!("  Eojeol:   {practical_eo:.1}% ({} / {}) [Δ +{:.1}pp vs strict]",
        practical.eojeol_correct, practical.eojeol_total, practical_eo - strict_eo);

    // 회귀 catch
    assert!(practical.eojeol_accuracy >= strict.eojeol_accuracy);
    assert!(practical.morpheme.token_accuracy >= strict.morpheme.token_accuracy);

    // Silver dataset floor
    assert!(strict.morpheme.token_accuracy >= 0.40,
        "Morph strict {strict_morph:.1}% below 40% floor (UD GSD silver)");

    println!("\nPASSED — strict morph {strict_morph:.1}% / practical morph {practical_morph:.1}%");
}

/// Sprint 140 A — UD Korean-Kaist `SPLIT_DIFFERENT` connection pair 분석.
///
/// Sprint 137에서 KLUE DP에 적용한 분석을 UD Kaist에도 적용.
/// 두 데이터셋의 problematic pair 패턴 비교 → 도메인 독립적 vs 도메인 특화 식별.
#[test]
#[ignore = "requires UD Korean-Kaist eval data + system dictionary"]
fn test_ud_kaist_split_diff_connection_pairs() {
    use mecab_ko_core::sejong::SejongConverter;
    use std::collections::HashMap;
    use std::fs;

    let project_root = project_root();
    let dict_path = dict_path(&project_root);

    let load_id_def = |path: &std::path::Path| -> HashMap<u16, String> {
        let content = fs::read_to_string(path).unwrap_or_default();
        content.lines()
            .filter_map(|line| {
                let mut parts = line.splitn(2, ' ');
                let id = parts.next()?.parse::<u16>().ok()?;
                let feat = parts.next()?.to_string();
                Some((id, feat))
            })
            .collect()
    };
    let left_id_def = load_id_def(&std::path::Path::new(&dict_path).join("left-id.def"));
    let right_id_def = load_id_def(&std::path::Path::new(&dict_path).join("right-id.def"));

    let mut tokenizer = make_tokenizer(&project_root);

    let eval_path = project_root.join("data/eval/ud_kaist_test.tsv");
    if !eval_path.exists() {
        println!("Skipping: data/eval/ud_kaist_test.tsv not found");
        return;
    }

    let dataset = TestDataset::from_tsv(eval_path.to_str().unwrap())
        .expect("Failed to load UD Kaist test TSV");
    let converter = SejongConverter::new();

    let mut pair_counts: HashMap<(u16, u16), usize> = HashMap::new();
    let mut pair_surfaces: HashMap<(u16, u16), Vec<String>> = HashMap::new();
    let surface_sample_cap = 3;
    let mut split_diff_eojeols: usize = 0;
    let mut total_eojeols: usize = 0;

    for gold_sentence in &dataset.sentences {
        let Some(eojeol_counts) = &gold_sentence.eojeol_counts else { continue; };
        let eojeols: Vec<&str> = gold_sentence.text.split_whitespace().collect();
        if eojeols.len() != eojeol_counts.len() { continue; }

        let mut gold_idx: usize = 0;
        for (eo_i, &count_g) in eojeol_counts.iter().enumerate() {
            total_eojeols += 1;
            if gold_idx + count_g > gold_sentence.tokens.len() {
                gold_idx = gold_sentence.tokens.len();
                continue;
            }
            let gold_slice = &gold_sentence.tokens[gold_idx..gold_idx + count_g];
            gold_idx += count_g;
            let gold_surface: String = gold_slice.iter().map(|t| t.surface.as_str()).collect();

            let pred_raw = tokenizer.tokenize(eojeols[eo_i]);
            let pred_sejong = converter.convert_tokens(&pred_raw);
            let pred_normalized_surface: String = pred_sejong.iter()
                .map(|t| SejongConverter::normalize_jamo(&t.surface))
                .collect();
            if pred_normalized_surface != gold_surface { continue; }

            let count_p = pred_sejong.len();
            if count_g < 2 || count_p < 2 || count_g == count_p { continue; }
            split_diff_eojeols += 1;

            let path = tokenizer.lattice().best_path();
            let content: &[&_] = &path;
            for window in content.windows(2) {
                let prev_right = window[0].right_id;
                let curr_left = window[1].left_id;
                *pair_counts.entry((prev_right, curr_left)).or_insert(0) += 1;
                let surface_pair = format!("{}|{}", window[0].surface, window[1].surface);
                let entry = pair_surfaces.entry((prev_right, curr_left)).or_default();
                if entry.len() < surface_sample_cap && !entry.contains(&surface_pair) {
                    entry.push(surface_pair);
                }
            }
        }
    }

    println!("\n{}", "=".repeat(70));
    println!("  UD Kaist SPLIT_DIFFERENT Connection Pair Analysis (Sprint 140 A)");
    println!("  Dataset: {} sentences, total {} eojeols, SPLIT_DIFFERENT {}",
        dataset.len(), total_eojeols, split_diff_eojeols);
    println!("  Total unique pairs: {}, total pair occurrences: {}",
        pair_counts.len(),
        pair_counts.values().sum::<usize>());
    println!("{}\n", "=".repeat(70));

    let mut sorted: Vec<_> = pair_counts.iter().collect();
    sorted.sort_by_key(|x| std::cmp::Reverse(*x.1));

    let top_n = 30;
    println!("=== Top {top_n} (right_id, left_id) pairs in SPLIT_DIFFERENT eojeols ===");
    println!("{:>5}  {:>5}  {:>6}  {:<40}  {:<40}  samples",
        "RID", "LID", "count", "right.feat (prev node)", "left.feat (curr node)");
    for ((rid, lid), count) in sorted.iter().take(top_n) {
        let r_feat = right_id_def.get(rid).map_or("?", std::string::String::as_str);
        let l_feat = left_id_def.get(lid).map_or("?", std::string::String::as_str);
        let samples = pair_surfaces.get(&(*rid, *lid))
            .map(|v| v.join(", "))
            .unwrap_or_default();
        let r_short = if r_feat.len() > 40 { &r_feat[..40] } else { r_feat };
        let l_short = if l_feat.len() > 40 { &l_feat[..40] } else { l_feat };
        println!("{rid:>5}  {lid:>5}  {count:>6}  {r_short:<40}  {l_short:<40}  {samples}");
    }
}


/// Sprint 145 D — mecab 결합 POS feature 빈도 분석.
///
/// KLUE DP val + UD Kaist test + UD GSD test 세 데이터셋 통합 측정.
/// Token의 raw `pos` (mecab feature)에 `+`가 포함된 결합 패턴 빈도 집계.
/// `splitter.rs`에 추가 분리 규칙 후보 식별 (Sprint 141 패턴 확장).
#[test]
#[ignore = "requires KLUE/UD eval data + system dictionary"]
fn test_compound_pos_frequency_analysis() {
    use std::collections::HashMap;

    let project_root = project_root();
    let mut tokenizer = make_tokenizer(&project_root);

    let eval_files = [
        ("KLUE", "data/eval/klue_dp_val.tsv"),
        ("UD-Kaist", "data/eval/ud_kaist_test.tsv"),
        ("UD-GSD", "data/eval/ud_gsd_test.tsv"),
    ];

    let mut pattern_counts: HashMap<String, (usize, Vec<String>)> = HashMap::new();
    let sample_cap = 3;
    let mut total_tokens: usize = 0;
    let mut compound_tokens: usize = 0;

    for (name, rel_path) in &eval_files {
        let path = project_root.join(rel_path);
        if !path.exists() {
            println!("Skipping {name}: {rel_path} not found");
            continue;
        }
        let dataset = TestDataset::from_tsv(path.to_str().unwrap())
            .expect("Failed to load dataset");

        for sentence in &dataset.sentences {
            let tokens = tokenizer.tokenize(&sentence.text);
            for tok in tokens {
                total_tokens += 1;
                if tok.pos.contains('+') {
                    compound_tokens += 1;
                    let entry = pattern_counts.entry(tok.pos.clone()).or_default();
                    entry.0 += 1;
                    if entry.1.len() < sample_cap && !entry.1.contains(&tok.surface) {
                        entry.1.push(tok.surface.clone());
                    }
                }
            }
        }
        println!("Processed: {name}");
    }

    println!("\n{}", "=".repeat(70));
    println!("  Compound POS Frequency Analysis (Sprint 145 D)");
    println!("  Total tokens: {total_tokens}, compound (X+Y...): {compound_tokens} ({:.1}%)",
        compound_tokens as f64 / total_tokens.max(1) as f64 * 100.0);
    println!("  Unique compound patterns: {}", pattern_counts.len());
    println!("{}\n", "=".repeat(70));

    let mut sorted: Vec<_> = pattern_counts.iter().collect();
    sorted.sort_by_key(|x| std::cmp::Reverse(x.1.0));

    let top_n = 40;
    println!("=== Top {top_n} compound POS patterns ===");
    println!("{:>6}  {:<25}  samples", "count", "pattern");
    for (pattern, (count, samples)) in sorted.iter().take(top_n) {
        let samples_str = samples.join(", ");
        println!("{count:>6}  {pattern:<25}  {samples_str}");
    }
}

/// Sprint 150 A — VA+ETM post-splitter mismatch 진단.
///
/// raw mecab VA+ETM 542건 중 splitter 후 어떤 케이스가 gold와 mismatch인지 측정.
/// `ending_rules`가 이미 "은" suffix를 처리하는지 검증 + 불규칙 케이스 식별.
#[test]
#[ignore = "requires KLUE/UD eval data + system dictionary"]
fn test_va_etm_post_splitter_mismatch() {
    use std::collections::HashMap;
    use mecab_ko_core::sejong::SejongConverter;

    let project_root = project_root();
    let mut tokenizer = make_tokenizer(&project_root);

    let converter = SejongConverter::new();
    let eval_files = [
        ("KLUE", "data/eval/klue_dp_val.tsv"),
        ("UD-Kaist", "data/eval/ud_kaist_test.tsv"),
        ("UD-GSD", "data/eval/ud_gsd_test.tsv"),
    ];

    // raw VA+ETM 토큰: splitter가 분리했는가? gold가 분리했는가?
    let mut raw_va_etm = 0usize;
    let mut split_by_splitter = 0usize;  // splitter가 ETM로 분리
    let mut surface_unchanged_pred: HashMap<String, usize> = HashMap::new();

    for (_, rel_path) in &eval_files {
        let path = project_root.join(rel_path);
        if !path.exists() { continue; }
        let dataset = TestDataset::from_tsv(path.to_str().unwrap()).expect("dataset");
        for sentence in &dataset.sentences {
            let raw_tokens = tokenizer.tokenize(&sentence.text);
            for tok in &raw_tokens {
                if tok.pos == "VA+ETM" {
                    raw_va_etm += 1;
                    // splitter 적용 결과 확인
                    let split = converter.convert_tokens(std::slice::from_ref(tok));
                    let has_etm = split.iter().any(|t| t.pos == "ETM");
                    if has_etm {
                        split_by_splitter += 1;
                    } else {
                        // splitter가 분리 못함 — 어떤 surface인지 기록
                        *surface_unchanged_pred.entry(tok.surface.clone()).or_insert(0) += 1;
                    }
                }
            }
        }
    }

    println!("\n{}", "=".repeat(70));
    println!("  VA+ETM post-splitter analysis (Sprint 150 A)");
    println!("{}", "=".repeat(70));
    println!("Raw VA+ETM tokens: {raw_va_etm}");
    println!("Split by splitter (has ETM token): {split_by_splitter}");
    println!("NOT split (still VA+ETM compound): {}", raw_va_etm - split_by_splitter);

    println!("\n=== Surface NOT split by splitter (top 20) ===");
    let mut sorted: Vec<_> = surface_unchanged_pred.iter().collect();
    sorted.sort_by_key(|x| std::cmp::Reverse(x.1));
    for (surface, count) in sorted.iter().take(20) {
        println!("  {surface}  count={count}");
    }
    println!("\n{}", "=".repeat(70));
}

/// Sprint 150 A — VA+ETM multi-syllable raw 빈도 분석 (참고용).
#[test]
#[ignore = "requires KLUE/UD eval data + system dictionary"]
fn test_va_etm_multisyllable_diagnosis() {
    use std::collections::HashMap;

    let project_root = project_root();
    let mut tokenizer = make_tokenizer(&project_root);

    let eval_files = [
        ("KLUE", "data/eval/klue_dp_val.tsv"),
        ("UD-Kaist", "data/eval/ud_kaist_test.tsv"),
        ("UD-GSD", "data/eval/ud_gsd_test.tsv"),
    ];

    let mut total_va_etm = 0usize;
    let mut single_syllable = 0usize;
    let mut multi_syllable = 0usize;
    let mut surface_endings: HashMap<char, (usize, Vec<String>)> = HashMap::new();
    let sample_cap = 6;

    println!("\n{}", "=".repeat(70));
    println!("  VA+ETM Multi-syllable Diagnosis (Sprint 150 A)");
    println!("{}", "=".repeat(70));

    for (ds_name, rel_path) in &eval_files {
        let path = project_root.join(rel_path);
        if !path.exists() { continue; }
        let dataset = TestDataset::from_tsv(path.to_str().unwrap()).expect("dataset");
        for sentence in &dataset.sentences {
            let raw_tokens = tokenizer.tokenize(&sentence.text);
            for tok in &raw_tokens {
                if tok.pos == "VA+ETM" {
                    total_va_etm += 1;
                    let nchar = tok.surface.chars().count();
                    if nchar == 1 {
                        single_syllable += 1;
                    } else {
                        multi_syllable += 1;
                        if let Some(last) = tok.surface.chars().last() {
                            let entry = surface_endings.entry(last).or_default();
                            entry.0 += 1;
                            if entry.1.len() < sample_cap && !entry.1.contains(&tok.surface) {
                                entry.1.push(tok.surface.clone());
                            }
                        }
                    }
                }
            }
        }
        println!("Processed: {ds_name}");
    }

    println!("\nTotal VA+ETM: {total_va_etm}");
    println!("  1-syllable (handled): {single_syllable}");
    println!("  Multi-syllable (NOT handled): {multi_syllable}");
    println!("\n=== Multi-syllable surface last-char endings ===");
    let mut sorted: Vec<_> = surface_endings.iter().collect();
    sorted.sort_by_key(|x| std::cmp::Reverse(x.1.0));
    for (ch, (count, samples)) in sorted.iter().take(15) {
        println!("  '{ch}'  count={count:<4}  samples: {}", samples.join(", "));
    }
    println!("\n{}", "=".repeat(70));
}

/// Sprint 148 D — ETM+ETM "라는" 진단.
///
/// mecab이 ETM+ETM으로 출력하는 "라는"을 gold와 비교하여
/// 실제 mismatch 여부 및 패턴을 파악한다.
#[test]
#[ignore = "requires KLUE DP eval data + system dictionary"]
fn test_etm_etm_raneun_diagnosis() {
    use mecab_ko_core::evaluate::{pos_tags_equivalent_practical, TestDataset};
    use mecab_ko_core::sejong::SejongConverter;

    let project_root = project_root();
    let mut tokenizer = make_tokenizer(&project_root);

    let converter = SejongConverter::new();
    let eval_files = [
        ("KLUE", "data/eval/klue_dp_val.tsv"),
        ("UD-Kaist", "data/eval/ud_kaist_test.tsv"),
        ("UD-GSD", "data/eval/ud_gsd_test.tsv"),
    ];

    let mut total_raneun = 0usize;
    let mut mismatch_count = 0usize;
    let show_limit = 10usize;

    println!("\n{}", "=".repeat(70));
    println!("  ETM+ETM '라는' Diagnosis (Sprint 148 D)");
    println!("{}", "=".repeat(70));

    for (ds_name, rel_path) in &eval_files {
        let path = project_root.join(rel_path);
        if !path.exists() {
            println!("Skipping {ds_name}: not found");
            continue;
        }
        let dataset = TestDataset::from_tsv(path.to_str().unwrap())
            .expect("Failed to load dataset");
        println!("\n--- {ds_name} ---");

        'sentence: for sentence in &dataset.sentences {
            let raw_tokens = tokenizer.tokenize(&sentence.text);
            let pred_morphs: Vec<(String, String)> = converter
                .convert_tokens(&raw_tokens)
                .into_iter()
                .map(|t| (t.surface, t.pos))
                .collect();

            // ETM+ETM 토큰 위치 찾기 (raw)
            let mut raw_etm_positions: Vec<usize> = Vec::new();
            for (i, tok) in raw_tokens.iter().enumerate() {
                if tok.pos == "ETM+ETM" && tok.surface.contains("라는") {
                raw_etm_positions.push(i);
            }
        }
            if raw_etm_positions.is_empty() {
                continue 'sentence;
            }
            total_raneun += raw_etm_positions.len();

            // pred vs gold 비교
            let gold_morphs: Vec<(String, String)> = sentence.tokens.iter()
                .flat_map(|t| {
                    t.pos.split('+').map(move |p| (t.surface.clone(), p.to_string()))
                })
                .collect();

            // pred_morphs에서 "라는/ETM" 위치 찾기
            for (pi, (ps, pp)) in pred_morphs.iter().enumerate() {
                if ps.contains("라는") && pp == "ETM" {
                    let found_in_gold = gold_morphs.iter().any(|(gs, gp)| {
                        gs.contains("라는") && (gp == "ETM" || pos_tags_equivalent_practical(gp, pp))
                    });

                    if !found_in_gold {
                        mismatch_count += 1;
                        if mismatch_count <= show_limit {
                            println!("\n[Mismatch #{mismatch_count}]");
                            println!("  Text: {}", sentence.text);
                            println!("  Pred[{pi}]: {ps}/{pp}");
                            let start = pi.saturating_sub(2);
                            let end = (pi + 3).min(pred_morphs.len());
                            print!("  Pred context: ");
                            for (s, p) in &pred_morphs[start..end] {
                                print!("{s}/{p} ");
                            }
                            println!();
                            print!("  Gold (라는 area): ");
                            for (gs, gp) in &gold_morphs {
                                if gs.contains("라는") || gs.contains("라") {
                                    print!("{gs}/{gp} ");
                                }
                            }
                            println!();
                        }
                    }
                }
            }
        } // end 'sentence
    } // end eval_files

    println!("\n{}", "=".repeat(70));
    println!("  Total ETM+ETM '라는' tokens: {total_raneun}");
    println!("  Gold mismatch (approximate): {mismatch_count}");
    println!("{}", "=".repeat(70));
}
