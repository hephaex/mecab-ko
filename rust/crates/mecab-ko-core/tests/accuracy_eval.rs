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

/// EF 에러 패턴 디버깅
#[test]
fn test_ef_error_analysis() {
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

    // EF 관련 테스트 케이스
    let test_cases = [
        // 기본 종결어미
        ("가다", "가/VV 다/EF"),
        ("먹다", "먹/VV 다/EF"),
        ("했다", "하/VV 았/EP 다/EF"),
        ("갔다", "가/VV 았/EP 다/EF"),
        // 습니다/ㅂ니다 종결
        ("합니다", "하/VV ㅂ니다/EF"),
        ("갑니다", "가/VV ㅂ니다/EF"),
        ("먹습니다", "먹/VV 습니다/EF"),
        ("있습니다", "있/VV 습니다/EF"),
        ("했습니다", "하/VV 았/EP 습니다/EF"),
        // 세요/요 종결
        // sample.tsv 기준: 세요/EF 사용 (시/EP + 어요/EF 분리하지 않음)
        ("하세요", "하/VV 세요/EF"),
        ("가세요", "가/VV 세요/EF"),
        ("드세요", "들/VV 세요/EF"),
        // 어요/아요 종결
        ("먹어요", "먹/VV 어요/EF"),
        ("가요", "가/VV 아요/EF"),
        ("해요", "하/VV 어요/EF"),  // 하+여요 = 해요, 세종 코퍼스 표준
        ("봐요", "보/VV 아요/EF"),
        // ㄹ게요/ㄹ까요 종결
        ("할게요", "하/VV ㄹ게요/EF"),
        ("갈게요", "가/VV ㄹ게요/EF"),
        ("할까요", "하/VV ㄹ까요/EF"),
        ("볼까요", "보/VV ㄹ까요/EF"),
        // 니/냐 의문형
        ("하니", "하/VV 니/EF"),
        ("가니", "가/VV 니/EF"),
        ("먹냐", "먹/VV 냐/EF"),
        // 아야/어야 + 합니다 패턴 (보조동사)
        ("해야 합니다", "하/VV 아야/EC 합니다/EF"),
        ("가야 합니다", "가/VV 아야/EC 합니다/EF"),
    ];

    println!("\n=== EF 에러 분석 ===");
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
        // 실패시 MeCab 원본 출력
        if !is_match {
            let mecab_output: Vec<String> = tokens.iter().map(|t| format!("{}/{}", t.surface, t.pos)).collect();
            println!("   MeCab 원본: {:?}", mecab_output);
        }
    }
    println!("\n통과: {}/{} ({:.1}%)", passed, total, passed as f64 / total as f64 * 100.0);
}

/// ETN 에러 패턴 디버깅
#[test]
fn test_etn_error_analysis() {
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

    // ETN 관련 테스트 케이스 (명사형 전성어미)
    let test_cases = [
        // 기본 명사형어미
        ("가기", "가/VV 기/ETN"),
        ("먹기", "먹/VV 기/ETN"),
        ("하기", "하/VV 기/ETN"),
        ("보기", "보/VV 기/ETN"),
        // 음/ㅁ 명사형어미
        ("감", "가/VV ㅁ/ETN"),
        ("봄", "보/VV ㅁ/ETN"),
        ("함", "하/VV ㅁ/ETN"),
        ("먹음", "먹/VV 음/ETN"),
        // 복합 문맥
        ("가기 전에", "가/VV 기/ETN 전/NNG 에/JKB"),
        ("하기 위해", "하/VV 기/ETN 위하/VV 어/EC"),
        // sample.tsv ETN 패턴
        ("놀이", "놀/VV 이/ETN"),
        ("먹이", "먹/VV 이/ETN"),
        ("잠", "자/VV ㅁ/ETN"),
        ("꿈", "꾸/VV ㅁ/ETN"),
        ("웃음", "웃/VV 음/ETN"),
        ("울음", "울/VV 음/ETN"),
    ];

    println!("\n=== ETN 에러 분석 ===");
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

/// XSV 에러 패턴 디버깅
#[test]
fn test_xsv_error_analysis() {
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

    // XSV 관련 테스트 케이스
    let test_cases = [
        // 하다 동사
        ("축하해요", "축하/NNG 하/XSV 어요/EF"),
        ("사랑해요", "사랑/NNG 하/XSV 어요/EF"),
        ("발표했다", "발표/NNG 하/XSV 았/EP 다/EF"),
        ("투자하고", "투자/NNG 하/XSV 고/EC"),
        // 되다 동사
        ("사용되다", "사용/NNG 되/XSV 다/EF"),
        ("발견되다", "발견/NNG 되/XSV 다/EF"),
        ("통과되었다", "통과/NNG 되/XSV 었/EP 다/EF"),
        ("개선됐다", "개선/NNG 되/XSV 었/EP 다/EF"),
    ];

    println!("\n=== XSV 에러 분석 ===");
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

/// VCP 에러 패턴 디버깅
#[test]
fn test_vcp_error_analysis() {
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

    // VCP 관련 테스트 케이스
    let test_cases = [
        ("학생입니다", "학생/NNG 이/VCP 습니다/EF"),
        ("누구세요", "누구/NP 이/VCP 세요/EF"),
        ("얼마예요", "얼마/NP 이/VCP 에요/EF"),
        ("뭐야", "뭐/NP 이/VCP 야/EF"),
        ("계류 중이다", "계류/NNG 중/NNB 이/VCP 다/EF"),
        ("분석 중이다", "분석/NNG 중/NNB 이/VCP 다/EF"),
        ("대세야", "대세/NNG 이/VCP 야/EF"),
        ("꿀잼이야", "꿀잼/NNG 이/VCP 야/EF"),
    ];

    println!("\n=== VCP 에러 분석 ===");
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

/// NNG 에러 패턴 디버깅 - 틀린 사례 분석
#[test]
fn test_nng_error_analysis() {
    use mecab_ko_core::evaluate::TestDataset;
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

    let eval_path = std::env::var("MECAB_EVAL_PATH")
        .unwrap_or_else(|_| {
            project_root.join("data/eval/sample.tsv")
                .to_string_lossy()
                .to_string()
        });

    let dataset = TestDataset::from_tsv(&eval_path)
        .expect("Failed to load test dataset");

    // NNG 에러 수집
    println!("\n=== NNG 에러 분석 (샘플 10개) ===");
    let mut error_count = 0;
    let max_errors = 10;

    for sentence in &dataset.sentences {
        if error_count >= max_errors {
            break;
        }

        let tokens = tokenizer.tokenize(&sentence.text);
        let sejong_tokens = converter.convert_tokens(&tokens);

        // Gold에서 NNG 위치 찾기
        for (i, gold) in sentence.tokens.iter().enumerate() {
            if gold.pos == "NNG" {
                // 해당 위치의 예측 토큰 찾기
                let pred = if i < sejong_tokens.len() {
                    format!("{}/{}", sejong_tokens[i].surface, sejong_tokens[i].pos)
                } else {
                    "MISSING".to_string()
                };

                // 불일치시 출력
                if !pred.starts_with(&format!("{}/NNG", gold.surface)) {
                    error_count += 1;
                    println!("문장: {}", sentence.text);
                    println!("  정답: {}/NNG → 예측: {}", gold.surface, pred);
                    println!();
                    if error_count >= max_errors {
                        break;
                    }
                }
            }
        }
    }
}

/// XPN 에러 패턴 디버깅
#[test]
fn test_xpn_error_analysis() {
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

    // XPN (체언 접두사) 관련 테스트 케이스
    let test_cases = [
        // 신 (新)
        ("신제품", "신/XPN 제품/NNG"),
        ("신기술", "신/XPN 기술/NNG"),
        // 구 (舊)
        ("구버전", "구/XPN 버전/NNG"),
        ("구시대", "구/XPN 시대/NNG"),
        // 전 (前)
        ("전회장", "전/XPN 회장/NNG"),
        ("전대통령", "전/XPN 대통령/NNG"),
        // 현 (現)
        ("현정부", "현/XPN 정부/NNG"),
        ("현대통령", "현/XPN 대통령/NNG"),
        // 불 (不)
        ("불합격", "불/XPN 합격/NNG"),
        // sample.tsv의 XPN 패턴
        ("순우리말", "순/XPN 우리말/NNG"),
        ("맨손", "맨/XPN 손/NNG"),
    ];

    println!("\n=== XPN 에러 분석 ===");
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

/// NNB 에러 패턴 디버깅
#[test]
fn test_nnb_error_analysis() {
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

    // NNB (의존명사) 관련 테스트 케이스 (sample.tsv 전체)
    let test_cases = [
        // 단위 의존명사
        ("백만원", "백만/NR 원/NNB"),
        ("삼십분", "삼십/NR 분/NNB"),
        ("열시", "열/NR 시/NNB"),
        ("세시", "세/NR 시/NNB"),
        // 일반 의존명사 단독
        ("것 수 등", "것/NNB 수/NNB 등/NNB"),
        ("바 데 지", "바/NNB 데/NNB 지/NNB"),
        ("만큼 뿐 채", "만큼/NNB 뿐/NNB 채/NNB"),
        ("듯 양 체", "듯/NNB 양/NNB 체/NNB"),
        ("대로 따라", "대로/NNB 따라/NNB"),
        // 중 (의존명사)
        ("계류 중이다", "계류/NNG 중/NNB 이/VCP 다/EF"),
        ("분석 중이다", "분석/NNG 중/NNB 이/VCP 다/EF"),
        // 지 (시간 경과)
        ("만난 지", "만나/VV ㄴ/ETM 지/NNB"),
        // 년 (의존명사)
        ("십 년", "십/NR 년/NNB"),
        // VV/EF 관련 테스트 (sample.tsv 기준: 세요/EF 사용)
        ("오세요", "오/VV 세요/EF"),
        ("말씀하세요", "말씀/NNG 하/XSV 세요/EF"),
    ];

    println!("\n=== NNB 에러 분석 ===");
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

/// EC 에러 패턴 sample.tsv에서 분석
#[test]
fn test_ec_sample_errors() {
    use mecab_ko_core::evaluate::TestDataset;
    use mecab_ko_core::sejong::SejongConverter;

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

    let user_dict_path = project_root.join("data/user-dict/verb-inflections.csv");
    if user_dict_path.exists() {
        let mut user_dict = UserDictionary::new();
        user_dict.load_from_csv(&user_dict_path)
            .expect("Failed to load user dictionary");
        tokenizer.set_user_dict(user_dict);
    }

    let converter = SejongConverter::new();

    let eval_path = std::env::var("MECAB_EVAL_PATH")
        .unwrap_or_else(|_| {
            project_root.join("data/eval/sample.tsv")
                .to_string_lossy()
                .to_string()
        });

    let dataset = TestDataset::from_tsv(&eval_path)
        .expect("Failed to load test dataset");

    println!("\n=== EC 에러 분석 (sample.tsv) ===");
    let mut error_count = 0;
    let max_errors = 30;

    for sentence in &dataset.sentences {
        if error_count >= max_errors {
            break;
        }

        let tokens = tokenizer.tokenize(&sentence.text);
        let sejong_tokens = converter.convert_tokens(&tokens);

        for (i, gold) in sentence.tokens.iter().enumerate() {
            if gold.pos == "EC" {
                let pred = if i < sejong_tokens.len() {
                    format!("{}/{}", sejong_tokens[i].surface, sejong_tokens[i].pos)
                } else {
                    "MISSING".to_string()
                };

                if !pred.ends_with("/EC") || !pred.starts_with(&gold.surface) {
                    error_count += 1;
                    println!("문장: {}", sentence.text);
                    println!("  정답: {}/EC → 예측: {}", gold.surface, pred);
                    println!();
                    if error_count >= max_errors {
                        break;
                    }
                }
            }
        }
    }
    println!("EC 오류 총 {}개 발견", error_count);
}

/// JKS 에러 패턴 sample.tsv에서 분석
#[test]
fn test_jks_sample_errors() {
    use mecab_ko_core::evaluate::TestDataset;
    use mecab_ko_core::sejong::SejongConverter;

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

    let user_dict_path = project_root.join("data/user-dict/verb-inflections.csv");
    if user_dict_path.exists() {
        let mut user_dict = UserDictionary::new();
        user_dict.load_from_csv(&user_dict_path)
            .expect("Failed to load user dictionary");
        tokenizer.set_user_dict(user_dict);
    }

    let converter = SejongConverter::new();

    let eval_path = std::env::var("MECAB_EVAL_PATH")
        .unwrap_or_else(|_| {
            project_root.join("data/eval/sample.tsv")
                .to_string_lossy()
                .to_string()
        });

    let dataset = TestDataset::from_tsv(&eval_path)
        .expect("Failed to load test dataset");

    println!("\n=== JKS 에러 분석 (sample.tsv) ===");
    let mut error_count = 0;
    let max_errors = 30;

    for sentence in &dataset.sentences {
        if error_count >= max_errors {
            break;
        }

        let tokens = tokenizer.tokenize(&sentence.text);
        let sejong_tokens = converter.convert_tokens(&tokens);

        for (i, gold) in sentence.tokens.iter().enumerate() {
            if gold.pos == "JKS" {
                let pred = if i < sejong_tokens.len() {
                    format!("{}/{}", sejong_tokens[i].surface, sejong_tokens[i].pos)
                } else {
                    "MISSING".to_string()
                };

                if !pred.ends_with("/JKS") {
                    error_count += 1;
                    println!("문장: {}", sentence.text);
                    println!("  정답: {}/JKS → 예측: {}", gold.surface, pred);
                    println!();
                    if error_count >= max_errors {
                        break;
                    }
                }
            }
        }
    }
    println!("JKS 오류 총 {}개 발견", error_count);
}

/// MAG 에러 패턴 sample.tsv에서 분석
#[test]
fn test_mag_sample_errors() {
    use mecab_ko_core::evaluate::TestDataset;
    use mecab_ko_core::sejong::SejongConverter;

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

    let user_dict_path = project_root.join("data/user-dict/verb-inflections.csv");
    if user_dict_path.exists() {
        let mut user_dict = UserDictionary::new();
        user_dict.load_from_csv(&user_dict_path)
            .expect("Failed to load user dictionary");
        tokenizer.set_user_dict(user_dict);
    }

    let converter = SejongConverter::new();

    let eval_path = std::env::var("MECAB_EVAL_PATH")
        .unwrap_or_else(|_| {
            project_root.join("data/eval/sample.tsv")
                .to_string_lossy()
                .to_string()
        });

    let dataset = TestDataset::from_tsv(&eval_path)
        .expect("Failed to load test dataset");

    println!("\n=== MAG 에러 분석 (sample.tsv) ===");
    let mut error_count = 0;
    let max_errors = 20;

    for sentence in &dataset.sentences {
        if error_count >= max_errors {
            break;
        }

        let tokens = tokenizer.tokenize(&sentence.text);
        let sejong_tokens = converter.convert_tokens(&tokens);

        for (i, gold) in sentence.tokens.iter().enumerate() {
            if gold.pos == "MAG" {
                let pred = if i < sejong_tokens.len() {
                    format!("{}/{}", sejong_tokens[i].surface, sejong_tokens[i].pos)
                } else {
                    "MISSING".to_string()
                };

                if !pred.ends_with("/MAG") {
                    error_count += 1;
                    println!("문장: {}", sentence.text);
                    println!("  정답: {}/MAG → 예측: {}", gold.surface, pred);
                    println!();
                    if error_count >= max_errors {
                        break;
                    }
                }
            }
        }
    }
    println!("MAG 오류 총 {}개 발견", error_count);
}

/// EF 에러 패턴 sample.tsv에서 분석
#[test]
fn test_ef_sample_errors() {
    use mecab_ko_core::evaluate::TestDataset;
    use mecab_ko_core::sejong::SejongConverter;

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

    let user_dict_path = project_root.join("data/user-dict/verb-inflections.csv");
    if user_dict_path.exists() {
        let mut user_dict = UserDictionary::new();
        user_dict.load_from_csv(&user_dict_path)
            .expect("Failed to load user dictionary");
        tokenizer.set_user_dict(user_dict);
    }

    let converter = SejongConverter::new();

    let eval_path = std::env::var("MECAB_EVAL_PATH")
        .unwrap_or_else(|_| {
            project_root.join("data/eval/sample.tsv")
                .to_string_lossy()
                .to_string()
        });

    let dataset = TestDataset::from_tsv(&eval_path)
        .expect("Failed to load test dataset");

    println!("\n=== EF 에러 분석 (sample.tsv) ===");
    let mut error_count = 0;
    let max_errors = 30;

    for sentence in &dataset.sentences {
        if error_count >= max_errors {
            break;
        }

        let tokens = tokenizer.tokenize(&sentence.text);
        let sejong_tokens = converter.convert_tokens(&tokens);

        for (i, gold) in sentence.tokens.iter().enumerate() {
            if gold.pos == "EF" {
                let pred = if i < sejong_tokens.len() {
                    format!("{}/{}", sejong_tokens[i].surface, sejong_tokens[i].pos)
                } else {
                    "MISSING".to_string()
                };

                if !pred.ends_with("/EF") {
                    error_count += 1;
                    println!("문장: {}", sentence.text);
                    println!("  정답: {}/EF → 예측: {}", gold.surface, pred);
                    println!();
                    if error_count >= max_errors {
                        break;
                    }
                }
            }
        }
    }
    println!("EF 오류 총 {}개 발견", error_count);
}

/// VX 에러 패턴 sample.tsv에서 분석
#[test]
fn test_vx_sample_errors() {
    use mecab_ko_core::evaluate::TestDataset;
    use mecab_ko_core::sejong::SejongConverter;

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

    let user_dict_path = project_root.join("data/user-dict/verb-inflections.csv");
    if user_dict_path.exists() {
        let mut user_dict = UserDictionary::new();
        user_dict.load_from_csv(&user_dict_path)
            .expect("Failed to load user dictionary");
        tokenizer.set_user_dict(user_dict);
    }

    let converter = SejongConverter::new();

    let eval_path = std::env::var("MECAB_EVAL_PATH")
        .unwrap_or_else(|_| {
            project_root.join("data/eval/sample.tsv")
                .to_string_lossy()
                .to_string()
        });

    let dataset = TestDataset::from_tsv(&eval_path)
        .expect("Failed to load test dataset");

    println!("\n=== VX 에러 분석 (sample.tsv) ===");
    let mut error_count = 0;
    let max_errors = 20;

    for sentence in &dataset.sentences {
        if error_count >= max_errors {
            break;
        }

        let tokens = tokenizer.tokenize(&sentence.text);
        let sejong_tokens = converter.convert_tokens(&tokens);

        for (i, gold) in sentence.tokens.iter().enumerate() {
            if gold.pos == "VX" {
                let pred = if i < sejong_tokens.len() {
                    format!("{}/{}", sejong_tokens[i].surface, sejong_tokens[i].pos)
                } else {
                    "MISSING".to_string()
                };

                if !pred.ends_with("/VX") {
                    error_count += 1;
                    println!("문장: {}", sentence.text);
                    println!("  정답: {}/VX → 예측: {}", gold.surface, pred);
                    println!();
                    if error_count >= max_errors {
                        break;
                    }
                }
            }
        }
    }
    println!("VX 오류 총 {}개 발견", error_count);
}

/// XSV 에러 패턴 sample.tsv에서 분석
#[test]
fn test_xsv_sample_errors() {
    use mecab_ko_core::evaluate::TestDataset;
    use mecab_ko_core::sejong::SejongConverter;

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

    let user_dict_path = project_root.join("data/user-dict/verb-inflections.csv");
    if user_dict_path.exists() {
        let mut user_dict = UserDictionary::new();
        user_dict.load_from_csv(&user_dict_path)
            .expect("Failed to load user dictionary");
        tokenizer.set_user_dict(user_dict);
    }

    let converter = SejongConverter::new();

    let eval_path = std::env::var("MECAB_EVAL_PATH")
        .unwrap_or_else(|_| {
            project_root.join("data/eval/sample.tsv")
                .to_string_lossy()
                .to_string()
        });

    let dataset = TestDataset::from_tsv(&eval_path)
        .expect("Failed to load test dataset");

    println!("\n=== XSV 에러 분석 (sample.tsv) ===");
    let mut error_count = 0;
    let max_errors = 30;

    for sentence in &dataset.sentences {
        if error_count >= max_errors {
            break;
        }

        let tokens = tokenizer.tokenize(&sentence.text);
        let sejong_tokens = converter.convert_tokens(&tokens);

        for (i, gold) in sentence.tokens.iter().enumerate() {
            if gold.pos == "XSV" {
                let pred = if i < sejong_tokens.len() {
                    format!("{}/{}", sejong_tokens[i].surface, sejong_tokens[i].pos)
                } else {
                    "MISSING".to_string()
                };

                if !pred.ends_with("/XSV") {
                    error_count += 1;
                    println!("문장: {}", sentence.text);
                    println!("  정답: {}/XSV → 예측: {}", gold.surface, pred);
                    println!();
                    if error_count >= max_errors {
                        break;
                    }
                }
            }
        }
    }
    println!("XSV 오류 총 {}개 발견", error_count);
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

/// VX 패턴 디버깅 - 세부 분석
#[test]
fn test_vx_pattern_debug() {
    use mecab_ko_core::sejong::SejongConverter;

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

    let user_dict_path = project_root.join("data/user-dict/verb-inflections.csv");
    if user_dict_path.exists() {
        let mut user_dict = UserDictionary::new();
        user_dict.load_from_csv(&user_dict_path)
            .expect("Failed to load user dictionary");
        tokenizer.set_user_dict(user_dict);
    }

    let converter = SejongConverter::new();

    // VX 관련 테스트 케이스
    let test_cases = [
        // -고 있다 패턴
        ("보이고 있다", "보이/VV 고/EC 있/VX 다/EF"),
        ("다하고 있다", "다하/VV 고/EC 있/VX 다/EF"),
        ("살고 있어", "살/VV 고/EC 있/VX 어/EF"),
        ("진행하고 있다", "진행/NNG 하/XSV 고/EC 있/VX 다/EF"),
        // -해 주다 패턴
        ("해주세요", "하/VV 어/EC 주/VX 세요/EF"),
        ("추천해 준", "추천/NNG 하/XSV 어/EC 주/VX ㄴ/ETM"),
        ("해주셨다", "하/VV 어/EC 주/VX 시/EP 었/EP 다/EF"),
    ];

    println!("\n=== VX 패턴 디버깅 ===");
    for (input, expected) in test_cases {
        let tokens = tokenizer.tokenize(input);
        let mecab_output: Vec<String> = tokens.iter().map(|t| format!("{}/{}", t.surface, t.pos)).collect();

        let sejong_tokens = converter.convert_tokens(&tokens);
        let result = converter.format_sejong(&sejong_tokens);

        let is_match = result == expected;
        let match_status = if is_match { "✓" } else { "✗" };
        println!("{} \"{}\"", match_status, input);
        println!("   MeCab:  {:?}", mecab_output);
        println!("   Sejong: {}", result);
        println!("   예상:   {}", expected);
        println!();
    }
}

/// EP 에러 패턴 디버깅
#[test]
fn test_ep_error_analysis() {
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

    // EP 관련 테스트 케이스 (sample.tsv 기반)
    let test_cases = [
        // 과거 시제
        ("갔다", "가/VV 았/EP 다/EF"),
        ("먹었습니다", "먹/VV 었/EP 습니다/EF"),
        ("만났어요", "만나/VV 았/EP 어요/EF"),
        ("봤다", "보/VV 았/EP 다/EF"),
        ("했다", "하/VV 았/EP 다/EF"),
        ("왔다", "오/VV 았/EP 다/EF"),
        // 추측/의지
        ("가겠다", "가/VV 겠/EP 다/EF"),
        ("먹겠다", "먹/VV 겠/EP 다/EF"),
        ("하겠습니다", "하/VV 겠/EP 습니다/EF"),
        // 존칭
        ("오셨습니다", "오/VV 시/EP 었/EP 습니다/EF"),
        ("드시겠어요", "드/VV 시/EP 겠/EP 어요/EF"),
        ("계십니다", "계/VV 시/EP ㅂ니다/EF"),
        // 보조용언 + EP
        ("먹어 버렸다", "먹/VV 어/EC 버리/VX 었/EP 다/EF"),
        ("해 보았다", "하/VV 어/EC 보/VX 았/EP 다/EF"),
        ("해주셨다", "하/VV 어/EC 주/VX 시/EP 었/EP 다/EF"),
    ];

    println!("\n=== EP 에러 분석 ===");
    let mut passed = 0;
    let total = test_cases.len();
    for (input, expected) in test_cases {
        let tokens = tokenizer.tokenize(input);
        let mecab_output: Vec<String> = tokens.iter().map(|t| format!("{}/{}", t.surface, t.pos)).collect();

        let sejong_tokens = converter.convert_tokens(&tokens);
        let result = converter.format_sejong(&sejong_tokens);

        let is_match = result == expected;
        if is_match {
            passed += 1;
        }
        let match_status = if is_match { "✓" } else { "✗" };
        println!("{} \"{}\"", match_status, input);
        println!("   MeCab:  {:?}", mecab_output);
        println!("   Sejong: {}", result);
        println!("   예상:   {}", expected);
        println!();
    }
    println!("통과: {}/{} ({:.1}%)", passed, total, passed as f64 / total as f64 * 100.0);
}

/// EF 오류 케이스 상세 분석
#[test]
fn test_ef_error_cases_detailed() {
    use mecab_ko_core::sejong::SejongConverter;

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

    let user_dict_path = project_root.join("data/user-dict/verb-inflections.csv");
    if user_dict_path.exists() {
        let mut user_dict = UserDictionary::new();
        user_dict.load_from_csv(&user_dict_path)
            .expect("Failed to load user dictionary");
        tokenizer.set_user_dict(user_dict);
    }

    let converter = SejongConverter::new();

    // EF 오류 케이스 (sample.tsv 기반)
    let test_cases = [
        ("목말라요", "목마르/VA 아요/EF"),
        // sample.tsv 기준: 심심하/VA 어요/EF
        ("심심해요", "심심하/VA 어요/EF"),
        ("재미있어요", "재미있/VA 어요/EF"),
        ("맛없어요", "맛없/VA 어요/EF"),
        ("만나요", "만나/VV 아요/EF"),
        ("피곤해요", "피곤/NNG 하/XSV 어요/EF"),
        ("미안해요", "미안/NNG 하/XSV 어요/EF"),
        ("힘들어요", "힘들/VA 어요/EF"),
    ];

    println!("\n=== EF 오류 케이스 상세 분석 ===");
    let mut passed = 0;
    let total = test_cases.len();
    for (input, expected) in test_cases {
        let tokens = tokenizer.tokenize(input);
        let mecab_output: Vec<String> = tokens.iter().map(|t| format!("{}/{}", t.surface, t.pos)).collect();

        let sejong_tokens = converter.convert_tokens(&tokens);
        let result = converter.format_sejong(&sejong_tokens);

        let is_match = result == expected;
        if is_match {
            passed += 1;
        }
        let match_status = if is_match { "✓" } else { "✗" };
        println!("{} \"{}\"", match_status, input);
        println!("   MeCab:  {:?}", mecab_output);
        println!("   Sejong: {}", result);
        println!("   예상:   {}", expected);
        println!();
    }
    println!("통과: {}/{} ({:.1}%)", passed, total, passed as f64 / total as f64 * 100.0);
}

/// VCP 에러 패턴 sample.tsv에서 분석
#[test]
fn test_vcp_sample_errors() {
    use mecab_ko_core::evaluate::TestDataset;
    use mecab_ko_core::sejong::SejongConverter;

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

    let user_dict_path = project_root.join("data/user-dict/verb-inflections.csv");
    if user_dict_path.exists() {
        let mut user_dict = UserDictionary::new();
        user_dict.load_from_csv(&user_dict_path)
            .expect("Failed to load user dictionary");
        tokenizer.set_user_dict(user_dict);
    }

    let converter = SejongConverter::new();

    let eval_path = std::env::var("MECAB_EVAL_PATH")
        .unwrap_or_else(|_| {
            project_root.join("data/eval/sample.tsv")
                .to_string_lossy()
                .to_string()
        });

    let dataset = TestDataset::from_tsv(&eval_path)
        .expect("Failed to load test dataset");

    println!("\n=== VCP 에러 분석 (sample.tsv) ===");
    let mut error_count = 0;
    let max_errors = 30;

    for sentence in &dataset.sentences {
        if error_count >= max_errors {
            break;
        }

        let tokens = tokenizer.tokenize(&sentence.text);
        let sejong_tokens = converter.convert_tokens(&tokens);

        for (i, gold) in sentence.tokens.iter().enumerate() {
            if gold.pos == "VCP" {
                let pred = if i < sejong_tokens.len() {
                    format!("{}/{}", sejong_tokens[i].surface, sejong_tokens[i].pos)
                } else {
                    "MISSING".to_string()
                };

                if !pred.ends_with("/VCP") {
                    error_count += 1;
                    println!("문장: {}", sentence.text);
                    println!("  정답: {}/VCP → 예측: {}", gold.surface, pred);

                    // 전체 예측 결과 출력
                    let pred_all: Vec<String> = sejong_tokens.iter()
                        .map(|t| format!("{}/{}", t.surface, t.pos))
                        .collect();
                    println!("  예측 전체: {}", pred_all.join(" "));

                    // 정답 전체 출력
                    let gold_all: Vec<String> = sentence.tokens.iter()
                        .map(|t| format!("{}/{}", t.surface, t.pos))
                        .collect();
                    println!("  정답 전체: {}", gold_all.join(" "));
                    println!();

                    if error_count >= max_errors {
                        break;
                    }
                }
            }
        }
    }
    println!("VCP 오류 총 {}개 발견", error_count);
}

/// VV 에러 패턴 sample.tsv에서 분석
#[test]
fn test_vv_sample_errors() {
    use mecab_ko_core::evaluate::TestDataset;
    use mecab_ko_core::sejong::SejongConverter;

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

    let user_dict_path = project_root.join("data/user-dict/verb-inflections.csv");
    if user_dict_path.exists() {
        let mut user_dict = UserDictionary::new();
        user_dict.load_from_csv(&user_dict_path)
            .expect("Failed to load user dictionary");
        tokenizer.set_user_dict(user_dict);
    }

    let converter = SejongConverter::new();

    let eval_path = std::env::var("MECAB_EVAL_PATH")
        .unwrap_or_else(|_| {
            project_root.join("data/eval/sample.tsv")
                .to_string_lossy()
                .to_string()
        });

    let dataset = TestDataset::from_tsv(&eval_path)
        .expect("Failed to load test dataset");

    println!("\n=== VV 에러 분석 (sample.tsv) ===");
    let mut error_count = 0;
    let max_errors = 30;

    for sentence in &dataset.sentences {
        if error_count >= max_errors {
            break;
        }

        let tokens = tokenizer.tokenize(&sentence.text);
        let sejong_tokens = converter.convert_tokens(&tokens);

        for (i, gold) in sentence.tokens.iter().enumerate() {
            if gold.pos == "VV" {
                let pred = if i < sejong_tokens.len() {
                    format!("{}/{}", sejong_tokens[i].surface, sejong_tokens[i].pos)
                } else {
                    "MISSING".to_string()
                };

                if !pred.ends_with("/VV") || !pred.starts_with(&gold.surface) {
                    error_count += 1;
                    println!("문장: {}", sentence.text);
                    println!("  정답: {}/VV → 예측: {}", gold.surface, pred);
                    println!();
                    if error_count >= max_errors {
                        break;
                    }
                }
            }
        }
    }
    println!("VV 오류 총 {}개 발견", error_count);
}

/// Sprint 41: XSV 오류 문장 상세 분석
#[test]
fn test_xsv_debug_sentences() {
    use mecab_ko_core::sejong::SejongConverter;

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

    let user_dict_path = project_root.join("data/user-dict/verb-inflections.csv");
    if user_dict_path.exists() {
        let mut user_dict = UserDictionary::new();
        user_dict.load_from_csv(&user_dict_path)
            .expect("Failed to load user dictionary");
        tokenizer.set_user_dict(user_dict);
        println!("Loaded user dictionary: {:?}", user_dict_path);
    }

    let converter = SejongConverter::new();

    // XSV 오류 문장들
    let test_cases = [
        (
            "수출이 증가하면서 무역수지가 개선됐다",
            "수출/NNG 이/JKS 증가/NNG 하/XSV 면서/EC 무역수지/NNG 가/JKS 개선/NNG 되/XSV 었/EP 다/EF"
        ),
        (
            "국민 여론조사 결과가 공개됐다",
            "국민/NNG 여론조사/NNG 결과/NNG 가/JKS 공개/NNG 되/XSV 었/EP 다/EF"
        ),
        (
            "시민단체가 성명을 발표했다",
            "시민단체/NNG 가/JKS 성명/NNG 을/JKO 발표/NNG 하/XSV 았/EP 다/EF"
        ),
        // 205차: VV 유지 케이스
        (
            "크리에이터 되고 싶어",
            "크리에이터/NNG 되/VV 고/EC 싶/VX 어/EF"
        ),
        // 209차: 뭐예요 분리
        (
            "MBTI가 뭐예요",
            "MBTI/SL 가/JKS 뭐/NP 이/VCP 에요/EF"
        ),
        // 210차: MAJ → VV + EC 분리
        (
            "하지만 가지만",
            "하/VV 지만/EC 가/VV 지만/EC"
        ),
        // 211차: NNG + 만/JX → VV + EC 분리
        (
            "보지만 하지만",
            "보/VV 지만/EC 하/VV 지만/EC"
        ),
        // 212차: 들리다 단일 동사 처리
        (
            "보이다 들리다",
            "보/VV 이/VX 다/EF 들리/VV 다/EF"
        ),
        // 213차: 진짜 NNG 처리
        (
            "진짜요",
            "진짜/NNG 요/JX"
        ),
        // 214차: ㄹ까 EC 분리
        (
            "갈까 한다",
            "가/VV ㄹ까/EC 하/VV ㄴ다/EF"
        ),
    ];

    println!("\n=== XSV 오류 문장 상세 분석 ===");
    for (input, expected) in test_cases {
        let tokens = tokenizer.tokenize(input);
        let sejong_tokens = converter.convert_tokens(&tokens);
        let result = converter.format_sejong(&sejong_tokens);

        println!("\n문장: {}", input);
        println!("  예상: {}", expected);
        println!("  결과: {}", result);
        println!("  MeCab 원본:");
        for tok in &tokens {
            println!("    {} / {} | features: {}", tok.surface, tok.pos, tok.features);
        }
        println!("  Sejong 변환 후:");
        for tok in &sejong_tokens {
            println!("    {} / {}", tok.surface, tok.pos);
        }

        let is_match = result == expected;
        println!("  일치: {}", if is_match { "✓" } else { "✗" });
    }
}

/// ㅎ불규칙 형용사 "으면/EC" 테스트
#[test]
fn test_h_irregular_adjective_ec() {
    use mecab_ko_core::sejong::SejongConverter;

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

    let user_dict_path = project_root.join("data/user-dict/verb-inflections.csv");
    if user_dict_path.exists() {
        let mut user_dict = UserDictionary::new();
        user_dict.load_from_csv(&user_dict_path)
            .expect("Failed to load user dictionary");
        tokenizer.set_user_dict(user_dict);
    }

    let converter = SejongConverter::new();

    // ㅎ불규칙 형용사 테스트
    let test_cases = [
        // sample.tsv 기준
        ("하얗다 하얘 하얗으면", "하얗/VA 다/EF 하얗/VA 아/EF 하얗/VA 으면/EC"),
        ("까맣다 까매 까맣으면", "까맣/VA 다/EF 까맣/VA 아/EF 까맣/VA 으면/EC"),
        ("노랗다 노래 노랗으면", "노랗/VA 다/EF 노랗/VA 아/EF 노랗/VA 으면/EC"),
        // 단일 어절
        ("하얗으면", "하얗/VA 으면/EC"),
        ("까맣으면", "까맣/VA 으면/EC"),
    ];

    println!("\n=== ㅎ불규칙 형용사 EC 분석 ===");
    for (input, expected) in test_cases {
        let tokens = tokenizer.tokenize(input);
        let sejong_tokens = converter.convert_tokens(&tokens);
        let result = converter.format_sejong(&sejong_tokens);

        println!("\n문장: {}", input);
        println!("  예상: {}", expected);
        println!("  결과: {}", result);
        println!("  MeCab 원본:");
        for tok in &tokens {
            println!("    {} / {} | {}", tok.surface, tok.pos, tok.features);
        }

        let is_match = result == expected;
        println!("  일치: {}", if is_match { "✓" } else { "✗" });
    }
}

/// 특정 문장 디버그 테스트
#[test]
fn test_specific_sentence_debug() {
    use mecab_ko_core::sejong::SejongConverter;

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

    let user_dict_path = project_root.join("data/user-dict/verb-inflections.csv");
    if user_dict_path.exists() {
        let mut user_dict = UserDictionary::new();
        user_dict.load_from_csv(&user_dict_path)
            .expect("Failed to load user dictionary");
        tokenizer.set_user_dict(user_dict);
    }

    let converter = SejongConverter::new();

    let test_cases = [
        ("갈까 한다", "가/VV ㄹ까/EC 하/VV ㄴ다/EF"),
        ("함 봄", "하/VV ㅁ/ETN 보/VV ㅁ/ETN"),
        // sample.tsv 기준: "쓰/VV 임/ETN" (피동 VX가 ETN에 병합)
        ("말함 쓰임", "말하/VV ㅁ/ETN 쓰/VV 임/ETN"),
        // NNG 오류 디버깅
        ("선생님 할머님", "선생님/NNG 할머님/NNG"),
        // 목말라요 디버깅
        ("목말라요", "목마르/VA 아요/EF"),
        // 존칭 시/EP 패턴 (sample.tsv)
        ("오셨습니다", "오/VV 시/EP 었/EP 습니다/EF"),
        ("드시겠어요", "드/VV 시/EP 겠/EP 어요/EF"),
        ("계십니다", "계/VV 시/EP ㅂ니다/EF"),
        // NNG 토큰화 오류 디버깅 (sample.tsv: 있/VV 어요/EF)
        ("내일 시간 있어요", "내일/NNG 시간/NNG 있/VV 어요/EF"),
        // "주말" 오류 분석
        ("주말에 영화 보러 갈래", "주말/NNG 에/JKB 영화/NNG 보/VV 러/EC 가/VV ㄹ래/EF"),
        // "갈등" 오류 분석
        ("갈등이 심화됐다", "갈등/NNG 이/JKS 심화/NNG 되/XSV 었/EP 다/EF"),
        // "전망" 오류 분석
        ("기온이 영하로 떨어질 전망입니다", "기온/NNG 이/JKS 영하/NNG 로/JKB 떨어지/VV ㄹ/ETM 전망/NNG 이/VCP 습니다/EF"),
        // "진행하고" 오류 분석
        ("협력하여 진행하고 있다", "협력/NNG 하/XSV 어/EC 진행/NNG 하/XSV 고/EC 있/VX 다/EF"),
        // "호출하여" 오류 분석 - sample.tsv 기준
        ("API를 호출하여 결과를 받았다", "API/SL 를/JKO 호출/NNG 하/XSV 어/EC 결과/NNG 를/JKO 받/VV 았/EP 다/EF"),
        // JKB 테스트 케이스 추가
        ("어디서 만날까요", "어디/NP 에서/JKB 만나/VV ㄹ까요/EF"),
        ("예상보다 높았다", "예상/NNG 보다/JKB 높/VA 았/EP 다/EF"),
        // "인해" 테스트
        ("인해 통제되고", "인하/VV 어/EC 통제/NNG 되/XSV 고/EC"),
        // "되었는데" 테스트
        ("되었는데 그동안", "되/VV 었/EP 는데/EC 그동안/NNG"),
        // "코드 리뷰" 테스트
        ("코드 리뷰를 진행했다", "코드/NNG 리뷰/NNG 를/JKO 진행/NNG 하/XSV 았/EP 다/EF"),
        // ㅂ불규칙 동사 분석
        ("줍다 주워 주우면", "줍/VV 다/EF 줍/VV 어/EF 줍/VV 으면/EC"),
        // ㅂ불규칙 형용사 분석
        ("덥다 더워 더우면", "덥/VA 다/EF 덥/VA 어/EF 덥/VA 으면/EC"),
        ("어렵다 어려워", "어렵/VA 다/EF 어렵/VA 어/EF"),
        // 단어 나열에서 각 어절의 마지막은 EF
        ("먹다 먹어", "먹/VV 다/EF 먹/VV 어/EF"),
        // "해" 분해: "하/VV 어/EC"
        ("해 보았다", "하/VV 어/EC 보/VX 았/EP 다/EF"),
        // ㅂ불규칙 형용사 "무겁다" 분석
        ("무겁다 무거워 무거우면", "무겁/VA 다/EF 무겁/VA 어/EF 무겁/VA 으면/EC"),
        // ㄹ탈락 "이르다" 분석
        ("이르다 일러 이르면", "이르/VV 다/EF 이르/VV 어/EF 이르/VV 면/EC"),
        // ㅎ불규칙 "노랗다" 분석
        ("노랗다 노래 노랗으면", "노랗/VA 다/EF 노랗/VA 아/EF 노랗/VA 으면/EC"),
        // "있으며" 패턴 (VX + 으며/EC)
        ("진행하고 있으며", "진행/NNG 하/XSV 고/EC 있/VX 으며/EC"),
    ];

    println!("\n=== 특정 문장 디버그 분석 ===");
    for (input, expected) in test_cases {
        let tokens = tokenizer.tokenize(input);
        let sejong_tokens = converter.convert_tokens(&tokens);
        let result = converter.format_sejong(&sejong_tokens);

        println!("\n문장: {}", input);
        println!("  예상: {}", expected);
        println!("  결과: {}", result);
        println!("  MeCab 원본:");
        for tok in &tokens {
            println!("    {} / {} [{}-{}] | {}", tok.surface, tok.pos, tok.start_pos, tok.end_pos, tok.features);
        }
        println!("  Sejong 변환:");
        for tok in &sejong_tokens {
            println!("    {} / {} [{}-{}]", tok.surface, tok.pos, tok.start_pos, tok.end_pos);
        }
    }
}

/// EP 샘플 오류 분석
#[test]
fn test_ep_sample_errors() {
    use mecab_ko_core::sejong::SejongConverter;
    use mecab_ko_core::evaluate::TestDataset;

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

    let user_dict_path = project_root.join("data/user-dict/verb-inflections.csv");
    if user_dict_path.exists() {
        let mut user_dict = UserDictionary::new();
        user_dict.load_from_csv(&user_dict_path)
            .expect("Failed to load user dictionary");
        tokenizer.set_user_dict(user_dict);
    }

    let converter = SejongConverter::new();

    let eval_path = project_root.join("data/eval/sample.tsv");
    let dataset = TestDataset::from_tsv(&eval_path)
        .expect("Failed to load test dataset");

    println!("\n=== EP 포함 문장 오류 분석 ===");
    let mut ep_errors: Vec<(String, String, String)> = Vec::new();

    for sentence in &dataset.sentences {
        // expected 재구성
        let expected: String = sentence.tokens.iter()
            .map(|t| format!("{}/{}", t.surface, t.pos))
            .collect::<Vec<_>>()
            .join(" ");

        // EP가 있는 문장만
        if !expected.contains("/EP") {
            continue;
        }

        let tokens = tokenizer.tokenize(&sentence.text);
        let sejong_tokens = converter.convert_tokens(&tokens);
        let result = converter.format_sejong(&sejong_tokens);

        if result != expected {
            ep_errors.push((sentence.text.clone(), expected, result));
        }
    }

    println!("EP 포함 오류 문장 수: {}\n", ep_errors.len());

    for (input, expected, result) in &ep_errors {
        println!("입력: {}", input);
        println!("예상: {}", expected);
        println!("결과: {}", result);

        // 토큰별 비교
        let exp_tokens: Vec<&str> = expected.split_whitespace().collect();
        let res_tokens: Vec<&str> = result.split_whitespace().collect();

        println!("차이:");
        let max_len = std::cmp::max(exp_tokens.len(), res_tokens.len());
        for i in 0..max_len {
            let exp = exp_tokens.get(i).unwrap_or(&"MISSING");
            let res = res_tokens.get(i).unwrap_or(&"MISSING");
            if exp != res {
                println!("  [{}] 예상: {} → 결과: {}", i, exp, res);
            }
        }
        println!("---");
    }
}

/// ㄷ불규칙 동사 테스트
#[test]
fn test_d_irregular_verb() {
    use mecab_ko_core::sejong::SejongConverter;

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

    let user_dict_path = project_root.join("data/user-dict/verb-inflections.csv");
    if user_dict_path.exists() {
        let mut user_dict = UserDictionary::new();
        user_dict.load_from_csv(&user_dict_path)
            .expect("Failed to load user dictionary");
        tokenizer.set_user_dict(user_dict);
    }

    let converter = SejongConverter::new();

    // ㄷ불규칙 동사 테스트
    let test_cases = [
        // sample.tsv 기준
        ("걷다 걸어 걸으면 걷는", "걷/VV 다/EF 걷/VV 어/EF 걷/VV 으면/EC 걷/VV 는/ETM"),
        ("듣다 들어 들으면 듣는", "듣/VV 다/EF 듣/VV 어/EF 듣/VV 으면/EC 듣/VV 는/ETM"),
        ("묻다 물어 물으면 묻는", "묻/VV 다/EF 묻/VV 어/EF 묻/VV 으면/EC 묻/VV 는/ETM"),
    ];

    println!("\n=== ㄷ불규칙 동사 분석 ===");
    for (input, expected) in test_cases {
        let tokens = tokenizer.tokenize(input);
        let sejong_tokens = converter.convert_tokens(&tokens);
        let result = converter.format_sejong(&sejong_tokens);

        println!("\n문장: {}", input);
        println!("  예상: {}", expected);
        println!("  결과: {}", result);
        println!("  MeCab 원본:");
        for tok in &tokens {
            println!("    {} / {} | {}", tok.surface, tok.pos, tok.features);
        }

        let is_match = result == expected;
        println!("  일치: {}", if is_match { "✓" } else { "✗" });
    }
}

/// 전체 데이터셋에서 틀린 문장 목록 출력
#[test]
fn test_list_all_mismatches() {
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

    let user_dict_path = project_root.join("data/user-dict/verb-inflections.csv");
    if user_dict_path.exists() {
        let mut user_dict = UserDictionary::new();
        user_dict.load_from_csv(&user_dict_path)
            .expect("Failed to load user dictionary");
        tokenizer.set_user_dict(user_dict);
    }

    let converter = SejongConverter::new();

    let eval_path = std::env::var("MECAB_EVAL_PATH")
        .unwrap_or_else(|_| {
            project_root.join("data/eval/sample.tsv")
                .to_string_lossy()
                .to_string()
        });

    let dataset = TestDataset::from_tsv(&eval_path)
        .expect("Failed to load test dataset");

    println!("\n=== 틀린 문장 목록 ===\n");
    let mut mismatch_count = 0;

    for sentence in &dataset.sentences {
        let tokens = tokenizer.tokenize(&sentence.text);
        let sejong_tokens = converter.convert_tokens(&tokens);
        let result = converter.format_sejong(&sejong_tokens);

        // Gold 형식으로 변환
        let expected: String = sentence.tokens.iter()
            .map(|t| format!("{}/{}", t.surface, t.pos))
            .collect::<Vec<_>>()
            .join(" ");

        if result.trim() != expected.trim() {
            mismatch_count += 1;
            println!("{}. 문장: {}", mismatch_count, sentence.text);
            println!("   예상: {}", expected);
            println!("   결과: {}", result);

            // MeCab 원본 출력
            println!("   MeCab:");
            for tok in &tokens {
                println!("     {} / {} | {}", tok.surface, tok.pos, tok.features);
            }
            println!();
        }
    }

    println!("\n총 틀린 문장: {} / {} ({:.1}%)",
        mismatch_count, dataset.sentences.len(),
        (dataset.sentences.len() - mismatch_count) as f64 / dataset.sentences.len() as f64 * 100.0);
}

/// CI/CD Accuracy Gate: 95%+ 정확도 요구
///
/// PR 병합 전 정확도 검증을 위한 테스트
#[test]
fn test_accuracy_gate() {
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

    println!("\n=== CI/CD Accuracy Gate ===");
    println!("테스트 문장 수: {}", dataset.len());

    // 세종 형식으로 평가
    let result = evaluate_dataset_sejong(&mut tokenizer, &dataset);

    // 결과 출력 (CI에서 파싱용)
    let accuracy_percent = result.token_accuracy * 100.0;
    println!("Token Accuracy: {:.1}", accuracy_percent);
    println!("Sentence Accuracy: {:.1}%", result.sentence_accuracy * 100.0);
    println!("F1 Score: {:.3}", result.f1_score);

    // 95% 정확도 게이트
    const ACCURACY_THRESHOLD: f64 = 0.95;
    assert!(
        result.token_accuracy >= ACCURACY_THRESHOLD,
        "ACCURACY GATE FAILED: Token accuracy {:.1}% is below {:.0}% threshold",
        accuracy_percent,
        ACCURACY_THRESHOLD * 100.0
    );

    println!("ACCURACY GATE PASSED: {:.1}% >= {:.0}%", accuracy_percent, ACCURACY_THRESHOLD * 100.0);
}

/// Sprint 65: 검증 데이터셋 정확도 게이트
///
/// 수작업 검증된 sprint58_verified.tsv 데이터셋에 대한 정확도 검증
#[test]
fn test_accuracy_gate_verified() {
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
    println!("Token Accuracy: {:.1}%", accuracy_percent);
    println!("Sentence Accuracy: {:.1}%", result.sentence_accuracy * 100.0);
    println!("F1 Score: {:.3}", result.f1_score);

    const VERIFIED_THRESHOLD: f64 = 0.90;
    assert!(
        result.token_accuracy >= VERIFIED_THRESHOLD,
        "VERIFIED ACCURACY GATE FAILED: Token accuracy {:.1}% is below {:.0}% threshold",
        accuracy_percent,
        VERIFIED_THRESHOLD * 100.0
    );

    println!("VERIFIED ACCURACY GATE PASSED: {:.1}% >= {:.0}%", accuracy_percent, VERIFIED_THRESHOLD * 100.0);
}
