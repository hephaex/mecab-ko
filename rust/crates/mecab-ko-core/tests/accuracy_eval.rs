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

    // NNB (의존명사) 관련 테스트 케이스
    let test_cases = [
        // 단위 의존명사
        ("백만원", "백만/NR 원/NNB"),
        ("삼십분", "삼십/NR 분/NNB"),
        ("열시", "열/NR 시/NNB"),
        ("세시", "세/NR 시/NNB"),
        // 일반 의존명사 단독
        ("것 수 등", "것/NNB 수/NNB 등/NNB"),
        ("바 데 지", "바/NNB 데/NNB 지/NNB"),
        // 중 (의존명사)
        ("계류 중이다", "계류/NNG 중/NNB 이/VCP 다/EF"),
        ("분석 중이다", "분석/NNG 중/NNB 이/VCP 다/EF"),
        // 지 (시간 경과)
        ("만난 지", "만나/VV ㄴ/ETM 지/NNB"),
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
