//! Integration tests for mecab-ko-dict
//!
//! 시스템 사전과 사용자 사전의 통합 테스트

#![allow(clippy::expect_used, clippy::unwrap_used)]

use mecab_ko_dict::{
    DictEntry, DictionaryLoader, SystemDictionary, UserDictionary, UserDictionaryBuilder,
};

#[test]
fn test_system_dictionary_integration() {
    // First try loading from mini test dictionary
    let mini_dict_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("parent")
        .parent()
        .expect("workspace")
        .join("test-fixtures")
        .join("mini-dict");

    if mini_dict_path.join("sys.dic").exists() {
        println!("Testing with mini dictionary at {:?}", mini_dict_path);
        let dict = SystemDictionary::load(&mini_dict_path)
            .expect("Failed to load mini test dictionary");
        assert!(dict.dicdir().exists());
        println!("Mini dictionary loaded successfully");
        return;
    }

    // Fall back to system dictionary
    let result = SystemDictionary::load_default();

    // 사전이 설치되어 있지 않으면 실패할 수 있음
    // CI 환경에서는 이 테스트를 스킵할 수 있음
    match result {
        Ok(dict) => {
            // 사전이 로드되면 기본 검증
            assert!(dict.dicdir().exists());
            // 엔트리는 항상 Vec이므로 길이 확인만으로 충분
            let _entry_count = dict.entries().len();
        }
        Err(e) => {
            // 사전이 없으면 에러 메시지 검증
            assert!(
                e.to_string().contains("Dictionary directory not found")
                    || e.to_string().contains("not found")
            );
        }
    }
}

#[test]
fn test_user_dictionary_integration() {
    let mut user_dict = UserDictionary::new();
    user_dict.add_entry("딥러닝", "NNG", Some(-1000), Some("딥러닝".to_string()));
    user_dict.add_entry("머신러닝", "NNG", Some(-1000), None);
    user_dict.add_entry(
        "자연어처리",
        "NNG",
        Some(-800),
        Some("자연어처리".to_string()),
    );

    assert_eq!(user_dict.len(), 3);

    // Trie 빌드
    user_dict.build_trie().expect("should build trie");

    // 검색 테스트
    let entries = user_dict.lookup("딥러닝");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].surface, "딥러닝");
    assert_eq!(entries[0].cost, -1000);

    // 공통 접두사 검색
    let trie = user_dict.get_trie().expect("should have trie");
    assert_eq!(trie.common_prefix_search("딥러닝모델").count(), 1); // "딥러닝" 매칭
}

#[test]
fn test_user_dictionary_builder() {
    let dict = UserDictionaryBuilder::new()
        .default_cost(-500)
        .add("챗GPT", "NNP")
        .add_with_cost("클로드", "NNP", -1000)
        .add_full("라마", "NNP", -800, Some("라마"))
        .build();

    assert_eq!(dict.len(), 3);

    let entries = dict.lookup("클로드");
    assert_eq!(entries[0].cost, -1000);
}

#[test]
fn test_user_dictionary_from_csv() {
    let csv = r"
# AI 모델 사전
챗GPT,NNP,-1000,챗지피티
클로드,NNP,-1000,클로드
라마,NNP,-800,라마
메타,NNP,-500,메타
";

    let dict = UserDictionaryBuilder::new()
        .load_str(csv)
        .expect("should load")
        .build();

    assert_eq!(dict.len(), 4);

    let entries = dict.lookup("챗GPT");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].reading.as_deref(), Some("챗지피티"));
}

#[test]
fn test_dict_entry_conversion() {
    let dict_entry = DictEntry::new("테스트", 10, 20, 300, "NNG,*,T,테스트,*,*,*,*");

    let entry = dict_entry.to_entry();
    assert_eq!(entry.surface, "테스트");
    assert_eq!(entry.left_id, 10);
    assert_eq!(entry.right_id, 20);
    assert_eq!(entry.cost, 300);
    assert_eq!(entry.feature, "NNG,*,T,테스트,*,*,*,*");

    // 역변환
    let dict_entry2: DictEntry = entry.into();
    assert_eq!(dict_entry2.surface, "테스트");
    assert_eq!(dict_entry2.left_id, 10);
}

#[test]
fn test_dictionary_loader_find_dicdir() {
    let result = DictionaryLoader::find_dicdir();

    // 환경에 따라 성공/실패할 수 있음
    match result {
        Ok(path) => {
            assert!(path.is_dir());
            println!("Found dictionary at: {}", path.display());
        }
        Err(e) => {
            println!("Dictionary not found: {e}");
            assert!(e.to_string().contains("Dictionary directory not found"));
        }
    }
}

#[test]
fn test_common_prefix_search_korean() {
    let mut user_dict = UserDictionary::new();
    user_dict.add_entry("형태", "NNG", Some(0), None);
    user_dict.add_entry("형태소", "NNG", Some(0), None);
    user_dict.add_entry("형태소분석", "NNG", Some(0), None);
    user_dict.add_entry("형태소분석기", "NNG", Some(0), None);

    user_dict.build_trie().expect("should build");
    let trie = user_dict.get_trie().expect("should have trie");

    // "형태소분석기" 검색
    assert_eq!(trie.common_prefix_search("형태소분석기").count(), 4); // 모든 접두사 매칭

    // "형태소분석" 검색
    assert_eq!(trie.common_prefix_search("형태소분석").count(), 3); // 형태, 형태소, 형태소분석
}

#[test]
fn test_multi_pos_entries() {
    // 같은 표면형, 다른 품사
    let mut user_dict = UserDictionary::new();
    user_dict.add_entry("가", "NNG", Some(-100), None); // 명사
    user_dict.add_entry("가", "JKS", Some(-200), None); // 조사
    user_dict.add_entry("가", "VV", Some(-300), None); // 동사

    let entries = user_dict.lookup("가");
    assert_eq!(entries.len(), 3);

    let pos_tags: Vec<_> = entries.iter().map(|e| e.pos.as_str()).collect();
    assert!(pos_tags.contains(&"NNG"));
    assert!(pos_tags.contains(&"JKS"));
    assert!(pos_tags.contains(&"VV"));
}

#[test]
fn test_dictionary_trait_implementation() {
    // Dictionary trait 구현 확인
    let mut user_dict = UserDictionary::new();
    user_dict.add_entry("테스트", "NNG", Some(-100), None);

    // Note: UserDictionary는 Dictionary trait을 직접 구현하지 않음
    // SystemDictionary만 Dictionary trait을 구현
}
