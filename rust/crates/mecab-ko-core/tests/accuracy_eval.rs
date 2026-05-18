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

/// 전체 데이터셋 정확도 측정
#[test]
#[ignore = "requires system dictionary data (sys.dic)"]
fn test_full_accuracy_evaluation() {
    // 프로젝트 루트 경로 계산
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let project_root = std::path::Path::new(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .unwrap_or_else(|| std::path::Path::new("."));

    // 사전 경로 설정
    let dict_path = std::env::var("MECAB_DIC_PATH").unwrap_or_else(|_| {
        project_root
            .join("data/mecab-ko-dic-2.1.1-20180720")
            .to_string_lossy()
            .to_string()
    });

    let mut tokenizer = Tokenizer::with_dict(&dict_path).expect("Failed to create tokenizer");

    // 사용자 사전 로드
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
        println!("Loaded user dictionary: {user_dict_path:?}");
    }

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

/// EC 에러 패턴 디버깅
#[test]
#[ignore = "requires system dictionary data (sys.dic)"]
fn test_ec_error_analysis() {
    use mecab_ko_core::sejong::SejongConverter;

    // 프로젝트 루트 경로 계산
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let project_root = std::path::Path::new(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .unwrap_or_else(|| std::path::Path::new("."));

    let dict_path = std::env::var("MECAB_DIC_PATH").unwrap_or_else(|_| {
        project_root
            .join("data/mecab-ko-dic-2.1.1-20180720")
            .to_string_lossy()
            .to_string()
    });

    let mut tokenizer = Tokenizer::with_dict(&dict_path).expect("Failed to create tokenizer");

    // 사용자 사전 로드
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
        println!("{match_status} \"{input}\": {result} (예상: {expected})");
    }
    println!(
        "\n통과: {}/{} ({:.1}%)",
        passed,
        total,
        f64::from(passed) / total as f64 * 100.0
    );
}

/// ETM 에러 패턴 디버깅
#[test]
#[ignore = "requires system dictionary data (sys.dic)"]
fn test_etm_error_analysis() {
    use mecab_ko_core::sejong::SejongConverter;

    // 프로젝트 루트 경로 계산
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let project_root = std::path::Path::new(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .unwrap_or_else(|| std::path::Path::new("."));

    let dict_path = std::env::var("MECAB_DIC_PATH").unwrap_or_else(|_| {
        project_root
            .join("data/mecab-ko-dic-2.1.1-20180720")
            .to_string_lossy()
            .to_string()
    });

    let mut tokenizer = Tokenizer::with_dict(&dict_path).expect("Failed to create tokenizer");

    // 사용자 사전 로드
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
        println!("{match_status} \"{input}\": {result} (예상: {expected})");
    }
    println!(
        "\n통과: {}/{} ({:.1}%)",
        passed,
        total,
        f64::from(passed) / total as f64 * 100.0
    );
}

/// EF 에러 패턴 디버깅
#[test]
#[ignore = "requires system dictionary data (sys.dic)"]
fn test_ef_error_analysis() {
    use mecab_ko_core::sejong::SejongConverter;

    // 프로젝트 루트 경로 계산
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let project_root = std::path::Path::new(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .unwrap_or_else(|| std::path::Path::new("."));

    let dict_path = std::env::var("MECAB_DIC_PATH").unwrap_or_else(|_| {
        project_root
            .join("data/mecab-ko-dic-2.1.1-20180720")
            .to_string_lossy()
            .to_string()
    });

    let mut tokenizer = Tokenizer::with_dict(&dict_path).expect("Failed to create tokenizer");

    // 사용자 사전 로드
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
        ("해요", "하/VV 어요/EF"), // 하+여요 = 해요, 세종 코퍼스 표준
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
        println!("{match_status} \"{input}\": {result} (예상: {expected})");
        // 실패시 MeCab 원본 출력
        if !is_match {
            let mecab_output: Vec<String> = tokens
                .iter()
                .map(|t| format!("{}/{}", t.surface, t.pos))
                .collect();
            println!("   MeCab 원본: {mecab_output:?}");
        }
    }
    println!(
        "\n통과: {}/{} ({:.1}%)",
        passed,
        total,
        f64::from(passed) / total as f64 * 100.0
    );
}

/// ETN 에러 패턴 디버깅
#[test]
#[ignore = "requires system dictionary data (sys.dic)"]
fn test_etn_error_analysis() {
    use mecab_ko_core::sejong::SejongConverter;

    // 프로젝트 루트 경로 계산
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let project_root = std::path::Path::new(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .unwrap_or_else(|| std::path::Path::new("."));

    let dict_path = std::env::var("MECAB_DIC_PATH").unwrap_or_else(|_| {
        project_root
            .join("data/mecab-ko-dic-2.1.1-20180720")
            .to_string_lossy()
            .to_string()
    });

    let mut tokenizer = Tokenizer::with_dict(&dict_path).expect("Failed to create tokenizer");

    // 사용자 사전 로드
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
        println!("{match_status} \"{input}\": {result} (예상: {expected})");
    }
    println!(
        "\n통과: {}/{} ({:.1}%)",
        passed,
        total,
        f64::from(passed) / total as f64 * 100.0
    );
}

/// XSV 에러 패턴 디버깅
#[test]
#[ignore = "requires system dictionary data (sys.dic)"]
fn test_xsv_error_analysis() {
    use mecab_ko_core::sejong::SejongConverter;

    // 프로젝트 루트 경로 계산
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let project_root = std::path::Path::new(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .unwrap_or_else(|| std::path::Path::new("."));

    let dict_path = std::env::var("MECAB_DIC_PATH").unwrap_or_else(|_| {
        project_root
            .join("data/mecab-ko-dic-2.1.1-20180720")
            .to_string_lossy()
            .to_string()
    });

    let mut tokenizer = Tokenizer::with_dict(&dict_path).expect("Failed to create tokenizer");

    // 사용자 사전 로드
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
        println!("{match_status} \"{input}\": {result} (예상: {expected})");
    }
    println!(
        "\n통과: {}/{} ({:.1}%)",
        passed,
        total,
        f64::from(passed) / total as f64 * 100.0
    );
}

/// VCP 에러 패턴 디버깅
#[test]
#[ignore = "requires system dictionary data (sys.dic)"]
fn test_vcp_error_analysis() {
    use mecab_ko_core::sejong::SejongConverter;

    // 프로젝트 루트 경로 계산
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let project_root = std::path::Path::new(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .unwrap_or_else(|| std::path::Path::new("."));

    let dict_path = std::env::var("MECAB_DIC_PATH").unwrap_or_else(|_| {
        project_root
            .join("data/mecab-ko-dic-2.1.1-20180720")
            .to_string_lossy()
            .to_string()
    });

    let mut tokenizer = Tokenizer::with_dict(&dict_path).expect("Failed to create tokenizer");

    // 사용자 사전 로드
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
        println!("{match_status} \"{input}\": {result} (예상: {expected})");
    }
    println!(
        "\n통과: {}/{} ({:.1}%)",
        passed,
        total,
        f64::from(passed) / total as f64 * 100.0
    );
}

/// NNG 에러 패턴 디버깅 - 틀린 사례 분석
#[test]
#[ignore = "requires system dictionary data (sys.dic)"]
fn test_nng_error_analysis() {
    use mecab_ko_core::evaluate::TestDataset;
    use mecab_ko_core::sejong::SejongConverter;

    // 프로젝트 루트 경로 계산
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let project_root = std::path::Path::new(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .unwrap_or_else(|| std::path::Path::new("."));

    let dict_path = std::env::var("MECAB_DIC_PATH").unwrap_or_else(|_| {
        project_root
            .join("data/mecab-ko-dic-2.1.1-20180720")
            .to_string_lossy()
            .to_string()
    });

    let mut tokenizer = Tokenizer::with_dict(&dict_path).expect("Failed to create tokenizer");

    // 사용자 사전 로드
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

    let converter = SejongConverter::new();

    let eval_path = std::env::var("MECAB_EVAL_PATH").unwrap_or_else(|_| {
        project_root
            .join("data/eval/sample.tsv")
            .to_string_lossy()
            .to_string()
    });

    let dataset = TestDataset::from_tsv(&eval_path).expect("Failed to load test dataset");

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
#[ignore = "requires system dictionary data (sys.dic)"]
fn test_xpn_error_analysis() {
    use mecab_ko_core::sejong::SejongConverter;

    // 프로젝트 루트 경로 계산
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let project_root = std::path::Path::new(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .unwrap_or_else(|| std::path::Path::new("."));

    let dict_path = std::env::var("MECAB_DIC_PATH").unwrap_or_else(|_| {
        project_root
            .join("data/mecab-ko-dic-2.1.1-20180720")
            .to_string_lossy()
            .to_string()
    });

    let mut tokenizer = Tokenizer::with_dict(&dict_path).expect("Failed to create tokenizer");

    // 사용자 사전 로드
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
        println!("{match_status} \"{input}\": {result} (예상: {expected})");
    }
    println!(
        "\n통과: {}/{} ({:.1}%)",
        passed,
        total,
        f64::from(passed) / total as f64 * 100.0
    );
}

/// NNB 에러 패턴 디버깅
#[test]
#[ignore = "requires system dictionary data (sys.dic)"]
fn test_nnb_error_analysis() {
    use mecab_ko_core::sejong::SejongConverter;

    // 프로젝트 루트 경로 계산
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let project_root = std::path::Path::new(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .unwrap_or_else(|| std::path::Path::new("."));

    let dict_path = std::env::var("MECAB_DIC_PATH").unwrap_or_else(|_| {
        project_root
            .join("data/mecab-ko-dic-2.1.1-20180720")
            .to_string_lossy()
            .to_string()
    });

    let mut tokenizer = Tokenizer::with_dict(&dict_path).expect("Failed to create tokenizer");

    // 사용자 사전 로드
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
        println!("{match_status} \"{input}\": {result} (예상: {expected})");
    }
    println!(
        "\n통과: {}/{} ({:.1}%)",
        passed,
        total,
        f64::from(passed) / total as f64 * 100.0
    );
}

/// EC 에러 패턴 sample.tsv에서 분석
#[test]
#[ignore = "requires system dictionary data (sys.dic)"]
fn test_ec_sample_errors() {
    use mecab_ko_core::evaluate::TestDataset;
    use mecab_ko_core::sejong::SejongConverter;

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let project_root = std::path::Path::new(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .unwrap_or_else(|| std::path::Path::new("."));

    let dict_path = std::env::var("MECAB_DIC_PATH").unwrap_or_else(|_| {
        project_root
            .join("data/mecab-ko-dic-2.1.1-20180720")
            .to_string_lossy()
            .to_string()
    });

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

    let converter = SejongConverter::new();

    let eval_path = std::env::var("MECAB_EVAL_PATH").unwrap_or_else(|_| {
        project_root
            .join("data/eval/sample.tsv")
            .to_string_lossy()
            .to_string()
    });

    let dataset = TestDataset::from_tsv(&eval_path).expect("Failed to load test dataset");

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
    println!("EC 오류 총 {error_count}개 발견");
}

/// JKS 에러 패턴 sample.tsv에서 분석
#[test]
#[ignore = "requires system dictionary data (sys.dic)"]
fn test_jks_sample_errors() {
    use mecab_ko_core::evaluate::TestDataset;
    use mecab_ko_core::sejong::SejongConverter;

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let project_root = std::path::Path::new(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .unwrap_or_else(|| std::path::Path::new("."));

    let dict_path = std::env::var("MECAB_DIC_PATH").unwrap_or_else(|_| {
        project_root
            .join("data/mecab-ko-dic-2.1.1-20180720")
            .to_string_lossy()
            .to_string()
    });

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

    let converter = SejongConverter::new();

    let eval_path = std::env::var("MECAB_EVAL_PATH").unwrap_or_else(|_| {
        project_root
            .join("data/eval/sample.tsv")
            .to_string_lossy()
            .to_string()
    });

    let dataset = TestDataset::from_tsv(&eval_path).expect("Failed to load test dataset");

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
    println!("JKS 오류 총 {error_count}개 발견");
}

/// MAG 에러 패턴 sample.tsv에서 분석
#[test]
#[ignore = "requires system dictionary data (sys.dic)"]
fn test_mag_sample_errors() {
    use mecab_ko_core::evaluate::TestDataset;
    use mecab_ko_core::sejong::SejongConverter;

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let project_root = std::path::Path::new(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .unwrap_or_else(|| std::path::Path::new("."));

    let dict_path = std::env::var("MECAB_DIC_PATH").unwrap_or_else(|_| {
        project_root
            .join("data/mecab-ko-dic-2.1.1-20180720")
            .to_string_lossy()
            .to_string()
    });

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

    let converter = SejongConverter::new();

    let eval_path = std::env::var("MECAB_EVAL_PATH").unwrap_or_else(|_| {
        project_root
            .join("data/eval/sample.tsv")
            .to_string_lossy()
            .to_string()
    });

    let dataset = TestDataset::from_tsv(&eval_path).expect("Failed to load test dataset");

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
    println!("MAG 오류 총 {error_count}개 발견");
}

/// EF 에러 패턴 sample.tsv에서 분석
#[test]
#[ignore = "requires system dictionary data (sys.dic)"]
fn test_ef_sample_errors() {
    use mecab_ko_core::evaluate::TestDataset;
    use mecab_ko_core::sejong::SejongConverter;

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let project_root = std::path::Path::new(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .unwrap_or_else(|| std::path::Path::new("."));

    let dict_path = std::env::var("MECAB_DIC_PATH").unwrap_or_else(|_| {
        project_root
            .join("data/mecab-ko-dic-2.1.1-20180720")
            .to_string_lossy()
            .to_string()
    });

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

    let converter = SejongConverter::new();

    let eval_path = std::env::var("MECAB_EVAL_PATH").unwrap_or_else(|_| {
        project_root
            .join("data/eval/sample.tsv")
            .to_string_lossy()
            .to_string()
    });

    let dataset = TestDataset::from_tsv(&eval_path).expect("Failed to load test dataset");

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
    println!("EF 오류 총 {error_count}개 발견");
}

/// VX 에러 패턴 sample.tsv에서 분석
#[test]
#[ignore = "requires system dictionary data (sys.dic)"]
fn test_vx_sample_errors() {
    use mecab_ko_core::evaluate::TestDataset;
    use mecab_ko_core::sejong::SejongConverter;

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let project_root = std::path::Path::new(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .unwrap_or_else(|| std::path::Path::new("."));

    let dict_path = std::env::var("MECAB_DIC_PATH").unwrap_or_else(|_| {
        project_root
            .join("data/mecab-ko-dic-2.1.1-20180720")
            .to_string_lossy()
            .to_string()
    });

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

    let converter = SejongConverter::new();

    let eval_path = std::env::var("MECAB_EVAL_PATH").unwrap_or_else(|_| {
        project_root
            .join("data/eval/sample.tsv")
            .to_string_lossy()
            .to_string()
    });

    let dataset = TestDataset::from_tsv(&eval_path).expect("Failed to load test dataset");

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
    println!("VX 오류 총 {error_count}개 발견");
}

/// XSV 에러 패턴 sample.tsv에서 분석
#[test]
#[ignore = "requires system dictionary data (sys.dic)"]
fn test_xsv_sample_errors() {
    use mecab_ko_core::evaluate::TestDataset;
    use mecab_ko_core::sejong::SejongConverter;

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let project_root = std::path::Path::new(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .unwrap_or_else(|| std::path::Path::new("."));

    let dict_path = std::env::var("MECAB_DIC_PATH").unwrap_or_else(|_| {
        project_root
            .join("data/mecab-ko-dic-2.1.1-20180720")
            .to_string_lossy()
            .to_string()
    });

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

    let converter = SejongConverter::new();

    let eval_path = std::env::var("MECAB_EVAL_PATH").unwrap_or_else(|_| {
        project_root
            .join("data/eval/sample.tsv")
            .to_string_lossy()
            .to_string()
    });

    let dataset = TestDataset::from_tsv(&eval_path).expect("Failed to load test dataset");

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
    println!("XSV 오류 총 {error_count}개 발견");
}

/// 특정 품사별 정확도 검증
#[test]
#[ignore = "requires system dictionary data (sys.dic)"]
fn test_pos_accuracy_breakdown() {
    // 프로젝트 루트 경로 계산
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let project_root = std::path::Path::new(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .unwrap_or_else(|| std::path::Path::new("."));

    let dict_path = std::env::var("MECAB_DIC_PATH").unwrap_or_else(|_| {
        project_root
            .join("data/mecab-ko-dic-2.1.1-20180720")
            .to_string_lossy()
            .to_string()
    });

    let mut tokenizer = Tokenizer::with_dict(&dict_path).expect("Failed to create tokenizer");

    // 사용자 사전 로드
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

/// VX 패턴 디버깅 - 세부 분석
#[test]
#[ignore = "requires system dictionary data (sys.dic)"]
fn test_vx_pattern_debug() {
    use mecab_ko_core::sejong::SejongConverter;

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let project_root = std::path::Path::new(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .unwrap_or_else(|| std::path::Path::new("."));

    let dict_path = std::env::var("MECAB_DIC_PATH").unwrap_or_else(|_| {
        project_root
            .join("data/mecab-ko-dic-2.1.1-20180720")
            .to_string_lossy()
            .to_string()
    });

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
        let mecab_output: Vec<String> = tokens
            .iter()
            .map(|t| format!("{}/{}", t.surface, t.pos))
            .collect();

        let sejong_tokens = converter.convert_tokens(&tokens);
        let result = converter.format_sejong(&sejong_tokens);

        let is_match = result == expected;
        let match_status = if is_match { "✓" } else { "✗" };
        println!("{match_status} \"{input}\"");
        println!("   MeCab:  {mecab_output:?}");
        println!("   Sejong: {result}");
        println!("   예상:   {expected}");
        println!();
    }
}

/// EP 에러 패턴 디버깅
#[test]
#[ignore = "requires system dictionary data (sys.dic)"]
fn test_ep_error_analysis() {
    use mecab_ko_core::sejong::SejongConverter;

    // 프로젝트 루트 경로 계산
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let project_root = std::path::Path::new(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .unwrap_or_else(|| std::path::Path::new("."));

    let dict_path = std::env::var("MECAB_DIC_PATH").unwrap_or_else(|_| {
        project_root
            .join("data/mecab-ko-dic-2.1.1-20180720")
            .to_string_lossy()
            .to_string()
    });

    let mut tokenizer = Tokenizer::with_dict(&dict_path).expect("Failed to create tokenizer");

    // 사용자 사전 로드
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
        let mecab_output: Vec<String> = tokens
            .iter()
            .map(|t| format!("{}/{}", t.surface, t.pos))
            .collect();

        let sejong_tokens = converter.convert_tokens(&tokens);
        let result = converter.format_sejong(&sejong_tokens);

        let is_match = result == expected;
        if is_match {
            passed += 1;
        }
        let match_status = if is_match { "✓" } else { "✗" };
        println!("{match_status} \"{input}\"");
        println!("   MeCab:  {mecab_output:?}");
        println!("   Sejong: {result}");
        println!("   예상:   {expected}");
        println!();
    }
    println!(
        "통과: {}/{} ({:.1}%)",
        passed,
        total,
        f64::from(passed) / total as f64 * 100.0
    );
}

/// EF 오류 케이스 상세 분석
#[test]
#[ignore = "requires system dictionary data (sys.dic)"]
fn test_ef_error_cases_detailed() {
    use mecab_ko_core::sejong::SejongConverter;

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let project_root = std::path::Path::new(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .unwrap_or_else(|| std::path::Path::new("."));

    let dict_path = std::env::var("MECAB_DIC_PATH").unwrap_or_else(|_| {
        project_root
            .join("data/mecab-ko-dic-2.1.1-20180720")
            .to_string_lossy()
            .to_string()
    });

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
        let mecab_output: Vec<String> = tokens
            .iter()
            .map(|t| format!("{}/{}", t.surface, t.pos))
            .collect();

        let sejong_tokens = converter.convert_tokens(&tokens);
        let result = converter.format_sejong(&sejong_tokens);

        let is_match = result == expected;
        if is_match {
            passed += 1;
        }
        let match_status = if is_match { "✓" } else { "✗" };
        println!("{match_status} \"{input}\"");
        println!("   MeCab:  {mecab_output:?}");
        println!("   Sejong: {result}");
        println!("   예상:   {expected}");
        println!();
    }
    println!(
        "통과: {}/{} ({:.1}%)",
        passed,
        total,
        f64::from(passed) / total as f64 * 100.0
    );
}

/// VCP 에러 패턴 sample.tsv에서 분석
#[test]
#[ignore = "requires system dictionary data (sys.dic)"]
fn test_vcp_sample_errors() {
    use mecab_ko_core::evaluate::TestDataset;
    use mecab_ko_core::sejong::SejongConverter;

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let project_root = std::path::Path::new(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .unwrap_or_else(|| std::path::Path::new("."));

    let dict_path = std::env::var("MECAB_DIC_PATH").unwrap_or_else(|_| {
        project_root
            .join("data/mecab-ko-dic-2.1.1-20180720")
            .to_string_lossy()
            .to_string()
    });

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

    let converter = SejongConverter::new();

    let eval_path = std::env::var("MECAB_EVAL_PATH").unwrap_or_else(|_| {
        project_root
            .join("data/eval/sample.tsv")
            .to_string_lossy()
            .to_string()
    });

    let dataset = TestDataset::from_tsv(&eval_path).expect("Failed to load test dataset");

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
                    let pred_all: Vec<String> = sejong_tokens
                        .iter()
                        .map(|t| format!("{}/{}", t.surface, t.pos))
                        .collect();
                    println!("  예측 전체: {}", pred_all.join(" "));

                    // 정답 전체 출력
                    let gold_all: Vec<String> = sentence
                        .tokens
                        .iter()
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
    println!("VCP 오류 총 {error_count}개 발견");
}

/// VV 에러 패턴 sample.tsv에서 분석
#[test]
#[ignore = "requires system dictionary data (sys.dic)"]
fn test_vv_sample_errors() {
    use mecab_ko_core::evaluate::TestDataset;
    use mecab_ko_core::sejong::SejongConverter;

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let project_root = std::path::Path::new(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .unwrap_or_else(|| std::path::Path::new("."));

    let dict_path = std::env::var("MECAB_DIC_PATH").unwrap_or_else(|_| {
        project_root
            .join("data/mecab-ko-dic-2.1.1-20180720")
            .to_string_lossy()
            .to_string()
    });

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

    let converter = SejongConverter::new();

    let eval_path = std::env::var("MECAB_EVAL_PATH").unwrap_or_else(|_| {
        project_root
            .join("data/eval/sample.tsv")
            .to_string_lossy()
            .to_string()
    });

    let dataset = TestDataset::from_tsv(&eval_path).expect("Failed to load test dataset");

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
    println!("VV 오류 총 {error_count}개 발견");
}

/// Sprint 41: XSV 오류 문장 상세 분석
#[test]
#[ignore = "requires system dictionary data (sys.dic)"]
fn test_xsv_debug_sentences() {
    use mecab_ko_core::sejong::SejongConverter;

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let project_root = std::path::Path::new(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .unwrap_or_else(|| std::path::Path::new("."));

    let dict_path = std::env::var("MECAB_DIC_PATH").unwrap_or_else(|_| {
        project_root
            .join("data/mecab-ko-dic-2.1.1-20180720")
            .to_string_lossy()
            .to_string()
    });

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
        println!("Loaded user dictionary: {user_dict_path:?}");
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

        println!("\n문장: {input}");
        println!("  예상: {expected}");
        println!("  결과: {result}");
        println!("  MeCab 원본:");
        for tok in &tokens {
            println!(
                "    {} / {} | features: {}",
                tok.surface, tok.pos, tok.features
            );
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
#[ignore = "requires system dictionary data (sys.dic)"]
fn test_h_irregular_adjective_ec() {
    use mecab_ko_core::sejong::SejongConverter;

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let project_root = std::path::Path::new(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .unwrap_or_else(|| std::path::Path::new("."));

    let dict_path = std::env::var("MECAB_DIC_PATH").unwrap_or_else(|_| {
        project_root
            .join("data/mecab-ko-dic-2.1.1-20180720")
            .to_string_lossy()
            .to_string()
    });

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

/// 특정 문장 디버그 테스트
#[test]
#[ignore = "requires system dictionary data (sys.dic)"]
fn test_specific_sentence_debug() {
    use mecab_ko_core::sejong::SejongConverter;

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let project_root = std::path::Path::new(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .unwrap_or_else(|| std::path::Path::new("."));

    let dict_path = std::env::var("MECAB_DIC_PATH").unwrap_or_else(|_| {
        project_root
            .join("data/mecab-ko-dic-2.1.1-20180720")
            .to_string_lossy()
            .to_string()
    });

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
        (
            "주말에 영화 보러 갈래",
            "주말/NNG 에/JKB 영화/NNG 보/VV 러/EC 가/VV ㄹ래/EF",
        ),
        // "갈등" 오류 분석
        (
            "갈등이 심화됐다",
            "갈등/NNG 이/JKS 심화/NNG 되/XSV 었/EP 다/EF",
        ),
        // "전망" 오류 분석
        (
            "기온이 영하로 떨어질 전망입니다",
            "기온/NNG 이/JKS 영하/NNG 로/JKB 떨어지/VV ㄹ/ETM 전망/NNG 이/VCP 습니다/EF",
        ),
        // "진행하고" 오류 분석
        (
            "협력하여 진행하고 있다",
            "협력/NNG 하/XSV 어/EC 진행/NNG 하/XSV 고/EC 있/VX 다/EF",
        ),
        // "호출하여" 오류 분석 - sample.tsv 기준
        (
            "API를 호출하여 결과를 받았다",
            "API/SL 를/JKO 호출/NNG 하/XSV 어/EC 결과/NNG 를/JKO 받/VV 았/EP 다/EF",
        ),
        // JKB 테스트 케이스 추가
        ("어디서 만날까요", "어디/NP 에서/JKB 만나/VV ㄹ까요/EF"),
        ("예상보다 높았다", "예상/NNG 보다/JKB 높/VA 았/EP 다/EF"),
        // "인해" 테스트
        ("인해 통제되고", "인하/VV 어/EC 통제/NNG 되/XSV 고/EC"),
        // "되었는데" 테스트
        ("되었는데 그동안", "되/VV 었/EP 는데/EC 그동안/NNG"),
        // "코드 리뷰" 테스트
        (
            "코드 리뷰를 진행했다",
            "코드/NNG 리뷰/NNG 를/JKO 진행/NNG 하/XSV 았/EP 다/EF",
        ),
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
        (
            "무겁다 무거워 무거우면",
            "무겁/VA 다/EF 무겁/VA 어/EF 무겁/VA 으면/EC",
        ),
        // ㄹ탈락 "이르다" 분석
        (
            "이르다 일러 이르면",
            "이르/VV 다/EF 이르/VV 어/EF 이르/VV 면/EC",
        ),
        // ㅎ불규칙 "노랗다" 분석
        (
            "노랗다 노래 노랗으면",
            "노랗/VA 다/EF 노랗/VA 아/EF 노랗/VA 으면/EC",
        ),
        // "있으며" 패턴 (VX + 으며/EC)
        ("진행하고 있으며", "진행/NNG 하/XSV 고/EC 있/VX 으며/EC"),
    ];

    println!("\n=== 특정 문장 디버그 분석 ===");
    for (input, expected) in test_cases {
        let tokens = tokenizer.tokenize(input);
        let sejong_tokens = converter.convert_tokens(&tokens);
        let result = converter.format_sejong(&sejong_tokens);

        println!("\n문장: {input}");
        println!("  예상: {expected}");
        println!("  결과: {result}");
        println!("  MeCab 원본:");
        for tok in &tokens {
            println!(
                "    {} / {} [{}-{}] | {}",
                tok.surface, tok.pos, tok.start_pos, tok.end_pos, tok.features
            );
        }
        println!("  Sejong 변환:");
        for tok in &sejong_tokens {
            println!(
                "    {} / {} [{}-{}]",
                tok.surface, tok.pos, tok.start_pos, tok.end_pos
            );
        }
    }
}

/// EP 샘플 오류 분석
#[test]
#[ignore = "requires system dictionary data (sys.dic)"]
fn test_ep_sample_errors() {
    use mecab_ko_core::evaluate::TestDataset;
    use mecab_ko_core::sejong::SejongConverter;

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let project_root = std::path::Path::new(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .unwrap_or_else(|| std::path::Path::new("."));

    let dict_path = std::env::var("MECAB_DIC_PATH").unwrap_or_else(|_| {
        project_root
            .join("data/mecab-ko-dic-2.1.1-20180720")
            .to_string_lossy()
            .to_string()
    });

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

    let converter = SejongConverter::new();

    let eval_path = project_root.join("data/eval/sample.tsv");
    let dataset = TestDataset::from_tsv(&eval_path).expect("Failed to load test dataset");

    println!("\n=== EP 포함 문장 오류 분석 ===");
    let mut ep_errors: Vec<(String, String, String)> = Vec::new();

    for sentence in &dataset.sentences {
        // expected 재구성
        let expected: String = sentence
            .tokens
            .iter()
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
        println!("입력: {input}");
        println!("예상: {expected}");
        println!("결과: {result}");

        // 토큰별 비교
        let exp_tokens: Vec<&str> = expected.split_whitespace().collect();
        let res_tokens: Vec<&str> = result.split_whitespace().collect();

        println!("차이:");
        let max_len = std::cmp::max(exp_tokens.len(), res_tokens.len());
        for i in 0..max_len {
            let exp = exp_tokens.get(i).unwrap_or(&"MISSING");
            let res = res_tokens.get(i).unwrap_or(&"MISSING");
            if exp != res {
                println!("  [{i}] 예상: {exp} → 결과: {res}");
            }
        }
        println!("---");
    }
}

/// ㄷ불규칙 동사 테스트
#[test]
#[ignore = "requires system dictionary data (sys.dic)"]
fn test_d_irregular_verb() {
    use mecab_ko_core::sejong::SejongConverter;

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let project_root = std::path::Path::new(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .unwrap_or_else(|| std::path::Path::new("."));

    let dict_path = std::env::var("MECAB_DIC_PATH").unwrap_or_else(|_| {
        project_root
            .join("data/mecab-ko-dic-2.1.1-20180720")
            .to_string_lossy()
            .to_string()
    });

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

/// 전체 데이터셋에서 틀린 문장 목록 출력
#[test]
#[ignore = "requires system dictionary data (sys.dic)"]
fn test_list_all_mismatches() {
    use mecab_ko_core::sejong::SejongConverter;

    // 프로젝트 루트 경로 계산
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let project_root = std::path::Path::new(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .unwrap_or_else(|| std::path::Path::new("."));

    let dict_path = std::env::var("MECAB_DIC_PATH").unwrap_or_else(|_| {
        project_root
            .join("data/mecab-ko-dic-2.1.1-20180720")
            .to_string_lossy()
            .to_string()
    });

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

    let converter = SejongConverter::new();

    let eval_path = std::env::var("MECAB_EVAL_PATH").unwrap_or_else(|_| {
        project_root
            .join("data/eval/sample.tsv")
            .to_string_lossy()
            .to_string()
    });

    let dataset = TestDataset::from_tsv(&eval_path).expect("Failed to load test dataset");

    println!("\n=== 틀린 문장 목록 ===\n");
    let mut mismatch_count = 0;

    for sentence in &dataset.sentences {
        let tokens = tokenizer.tokenize(&sentence.text);
        let sejong_tokens = converter.convert_tokens(&tokens);
        let result = converter.format_sejong(&sejong_tokens);

        // Gold 형식으로 변환
        let expected: String = sentence
            .tokens
            .iter()
            .map(|t| format!("{}/{}", t.surface, t.pos))
            .collect::<Vec<_>>()
            .join(" ");

        if result.trim() != expected.trim() {
            mismatch_count += 1;
            println!("{}. 문장: {}", mismatch_count, sentence.text);
            println!("   예상: {expected}");
            println!("   결과: {result}");

            // MeCab 원본 출력
            println!("   MeCab:");
            for tok in &tokens {
                println!("     {} / {} | {}", tok.surface, tok.pos, tok.features);
            }
            println!();
        }
    }

    println!(
        "\n총 틀린 문장: {} / {} ({:.1}%)",
        mismatch_count,
        dataset.sentences.len(),
        (dataset.sentences.len() - mismatch_count) as f64 / dataset.sentences.len() as f64 * 100.0
    );
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

    // 프로젝트 루트 경로 계산
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let project_root = std::path::Path::new(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .unwrap_or_else(|| std::path::Path::new("."));

    // 사전 경로 설정
    let dict_path = std::env::var("MECAB_DIC_PATH").unwrap_or_else(|_| {
        project_root
            .join("data/mecab-ko-dic-2.1.1-20180720")
            .to_string_lossy()
            .to_string()
    });

    let mut tokenizer = Tokenizer::with_dict(&dict_path).expect("Failed to create tokenizer");

    // 사용자 사전 로드
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

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let project_root = std::path::Path::new(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .unwrap_or_else(|| std::path::Path::new("."));

    let dict_path = std::env::var("MECAB_DIC_PATH").unwrap_or_else(|_| {
        project_root
            .join("data/mecab-ko-dic-2.1.1-20180720")
            .to_string_lossy()
            .to_string()
    });

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

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let project_root = std::path::Path::new(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .unwrap_or_else(|| std::path::Path::new("."));

    let dict_path = std::env::var("MECAB_DIC_PATH").unwrap_or_else(|_| {
        project_root
            .join("data/mecab-ko-dic-2.1.1-20180720")
            .to_string_lossy()
            .to_string()
    });

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

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let project_root = std::path::Path::new(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .unwrap_or_else(|| std::path::Path::new("."));

    let dict_path = std::env::var("MECAB_DIC_PATH").unwrap_or_else(|_| {
        project_root
            .join("data/mecab-ko-dic-2.1.1-20180720")
            .to_string_lossy()
            .to_string()
    });

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

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let project_root = std::path::Path::new(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .unwrap_or_else(|| std::path::Path::new("."));

    let dict_path = std::env::var("MECAB_DIC_PATH").unwrap_or_else(|_| {
        project_root
            .join("data/mecab-ko-dic-2.1.1-20180720")
            .to_string_lossy()
            .to_string()
    });

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

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let project_root = std::path::Path::new(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .unwrap_or_else(|| std::path::Path::new("."));

    let dict_path = std::env::var("MECAB_DIC_PATH").unwrap_or_else(|_| {
        project_root
            .join("data/mecab-ko-dic-2.1.1-20180720")
            .to_string_lossy()
            .to_string()
    });

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

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let project_root = std::path::Path::new(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .unwrap_or_else(|| std::path::Path::new("."));

    let dict_path = std::env::var("MECAB_DIC_PATH").unwrap_or_else(|_| {
        project_root
            .join("data/mecab-ko-dic-2.1.1-20180720")
            .to_string_lossy()
            .to_string()
    });

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

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let project_root = std::path::Path::new(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .unwrap_or_else(|| std::path::Path::new("."));

    let dict_path = std::env::var("MECAB_DIC_PATH").unwrap_or_else(|_| {
        project_root
            .join("data/mecab-ko-dic-2.1.1-20180720")
            .to_string_lossy()
            .to_string()
    });

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

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let project_root = std::path::Path::new(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .unwrap_or_else(|| std::path::Path::new("."));

    let dict_path = std::env::var("MECAB_DIC_PATH").unwrap_or_else(|_| {
        project_root
            .join("data/mecab-ko-dic-2.1.1-20180720")
            .to_string_lossy()
            .to_string()
    });

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

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let project_root = std::path::Path::new(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .unwrap_or_else(|| std::path::Path::new("."));

    let dict_path = std::env::var("MECAB_DIC_PATH").unwrap_or_else(|_| {
        project_root
            .join("data/mecab-ko-dic-2.1.1-20180720")
            .to_string_lossy()
            .to_string()
    });

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

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let project_root = std::path::Path::new(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .unwrap_or_else(|| std::path::Path::new("."));

    let dict_path = std::env::var("MECAB_DIC_PATH").unwrap_or_else(|_| {
        project_root
            .join("data/mecab-ko-dic-2.1.1-20180720")
            .to_string_lossy()
            .to_string()
    });

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

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let project_root = std::path::Path::new(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .unwrap_or_else(|| std::path::Path::new("."));

    let dict_path = std::env::var("MECAB_DIC_PATH").unwrap_or_else(|_| {
        project_root
            .join("data/mecab-ko-dic-2.1.1-20180720")
            .to_string_lossy()
            .to_string()
    });

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
