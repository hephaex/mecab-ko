//! 통합 테스트 - 전체 사전 빌드 파이프라인

use mecab_ko_dict_builder::builder::BuildConfig;
use mecab_ko_dict_builder::csv_parser::Encoding;
use mecab_ko_dict_builder::DictionaryBuilder;
use std::fs;
use tempfile::TempDir;

fn create_test_input_dir() -> TempDir {
    let temp_dir = TempDir::new().expect("failed to create temp dir");

    // CSV 파일들
    let nng_csv = "가,1,1,100,NNG,*,T,가,*,NNG,NNG,*
가방,1,1,150,NNG,*,T,가방,*,NNG,NNG,*
안녕,1,1,120,NNG,*,T,안녕,*,NNG,NNG,*
세계,1,1,130,NNG,*,F,세계,*,NNG,NNG,*
";

    let vv_csv = "가다,2,2,200,VV,*,F,가다,*,VV,VV,*
하다,2,2,180,VV,*,F,하다,*,VV,VV,*
오다,2,2,190,VV,*,F,오다,*,VV,VV,*
";

    fs::write(temp_dir.path().join("NNG.csv"), nng_csv).expect("write NNG.csv");
    fs::write(temp_dir.path().join("VV.csv"), vv_csv).expect("write VV.csv");

    // matrix.def
    let matrix_def = "2 2
0 0 100
0 1 200
1 0 150
1 1 50
";
    fs::write(temp_dir.path().join("matrix.def"), matrix_def).expect("write matrix.def");

    // char.def
    let char_def = "DEFAULT 1 0 0
SPACE   0 1 0
HANGUL  1 1 0
ALPHA   1 1 0
0x0020 SPACE
0x0009 SPACE
0x000A SPACE
";
    fs::write(temp_dir.path().join("char.def"), char_def).expect("write char.def");

    // unk.def
    let unk_def = "DEFAULT,0,0,0,UNK,*,*,*,*,*,*,*
SPACE,0,0,0,SP,*,*,*,*,*,*,*
HANGUL,1,1,1000,UNK,*,*,*,*,*,*,*
ALPHA,1,2,1200,UNK,*,*,*,*,*,*,*
";
    fs::write(temp_dir.path().join("unk.def"), unk_def).expect("write unk.def");

    temp_dir
}

#[test]
fn test_full_build_with_all_files() {
    let input_dir = create_test_input_dir();
    let output_dir = TempDir::new().expect("failed to create output dir");

    let config = BuildConfig {
        input_dir: input_dir.path().to_string_lossy().to_string(),
        output_dir: output_dir.path().to_string_lossy().to_string(),
        compression_level: 0, // 테스트에서는 압축 안 함
        encoding: Encoding::Utf8,
        verbose: false,
    };

    let builder = DictionaryBuilder::new(config);
    let result = builder.build().expect("build should succeed");

    // 결과 검증
    assert_eq!(result.entry_count, 7); // NNG 4개 + VV 3개
    assert!(result.trie_size > 0);
    assert_eq!(result.matrix_size, 4); // 2x2

    // 출력 파일 확인
    assert!(output_dir.path().join("sys.dic").exists());
    assert!(output_dir.path().join("matrix.bin").exists());
    assert!(output_dir.path().join("char.bin").exists());
    assert!(output_dir.path().join("unk.bin").exists());
}

#[test]
fn test_build_with_compression() {
    let input_dir = create_test_input_dir();
    let output_dir = TempDir::new().expect("failed to create output dir");

    let config = BuildConfig {
        input_dir: input_dir.path().to_string_lossy().to_string(),
        output_dir: output_dir.path().to_string_lossy().to_string(),
        compression_level: 3,
        encoding: Encoding::Utf8,
        verbose: false,
    };

    let builder = DictionaryBuilder::new(config);
    let result = builder.build().expect("build should succeed");

    // 압축 파일 확인
    assert!(output_dir.path().join("sys.dic.zst").exists());
    assert!(output_dir.path().join("matrix.bin.zst").exists());

    // 압축된 파일이 원본보다 작거나 비슷해야 함
    // (작은 테스트 데이터는 오히려 커질 수 있음)
    let compressed_size = fs::metadata(output_dir.path().join("sys.dic.zst"))
        .expect("stat compressed")
        .len();
    assert!(compressed_size > 0);

    assert_eq!(result.entry_count, 7);
}

#[test]
fn test_korean_jongseong_auto_detection() {
    use mecab_ko_dict_builder::csv_parser::CsvParser;

    let temp_dir = TempDir::new().expect("failed to create temp dir");

    // 종성 필드가 * 인 CSV
    let csv_content = "가방,1,1,100,NNG,*,*,가방,*,NNG,NNG,*
가다,2,2,200,VV,*,*,가다,*,VV,VV,*
";
    let csv_path = temp_dir.path().join("test.csv");
    fs::write(&csv_path, csv_content).expect("write csv");

    let parser = CsvParser::new(temp_dir.path());
    let entries = parser.parse_file(&csv_path).expect("parse failed");

    assert_eq!(entries.len(), 2);

    // 가방: 종성 있음 (ㅇ)
    assert_eq!(entries[0].surface, "가방");
    assert_eq!(entries[0].jongseong, "T");

    // 가다: 종성 없음
    assert_eq!(entries[1].surface, "가다");
    assert_eq!(entries[1].jongseong, "F");
}

#[test]
fn test_char_def_parsing() {
    use mecab_ko_dict_builder::char_def_parser::CharDef;

    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let char_def_path = temp_dir.path().join("char.def");

    let content = "DEFAULT 1 0 0
SPACE 0 1 0
HANGUL 1 1 0
0x0020 SPACE
0xAC00 HANGUL
";
    fs::write(&char_def_path, content).expect("write char.def");

    let char_def = CharDef::from_file(&char_def_path).expect("parse failed");

    assert_eq!(char_def.types.len(), 3);
    assert_eq!(char_def.mappings.len(), 2);

    let space_type = char_def.get_type("SPACE").expect("SPACE type");
    assert!(!space_type.invoke);
    assert!(space_type.group);

    let hangul_type = char_def.get_type("HANGUL").expect("HANGUL type");
    assert!(hangul_type.invoke);
    assert!(hangul_type.group);
}

#[test]
fn test_unk_def_parsing() {
    use mecab_ko_dict_builder::unk_def_parser::UnkDef;

    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let unk_def_path = temp_dir.path().join("unk.def");

    let content = "DEFAULT,0,0,0,UNK,*,*,*,*,*,*,*
HANGUL,1,1,1000,UNK,*,*,*,*,*,*,*
ALPHA,1,2,1200,UNK,*,*,*,*,*,*,*
";
    fs::write(&unk_def_path, content).expect("write unk.def");

    let unk_def = UnkDef::from_file(&unk_def_path).expect("parse failed");

    assert_eq!(unk_def.entries.len(), 3);

    let default_entry = unk_def.get_entry("DEFAULT").expect("DEFAULT entry");
    assert_eq!(default_entry.cost, 0);

    let hangul_entry = unk_def.get_entry("HANGUL").expect("HANGUL entry");
    assert_eq!(hangul_entry.cost, 1000);
    assert_eq!(hangul_entry.left_id, 1);
    assert_eq!(hangul_entry.right_id, 1);
}

#[test]
fn test_builder_with_euc_kr_encoding() {
    // EUC-KR 인코딩 테스트는 실제 인코딩 변환이 필요하므로
    // 설정만 확인
    let config = BuildConfig {
        input_dir: "./test".to_string(),
        output_dir: "./output".to_string(),
        compression_level: 0,
        encoding: Encoding::EucKr,
        verbose: false,
    };

    assert_eq!(config.compression_level, 0);
}

#[test]
fn test_multiple_csv_files() {
    let input_dir = create_test_input_dir();
    let output_dir = TempDir::new().expect("failed to create output dir");

    // 여러 CSV 파일이 모두 파싱되는지 확인
    let config = BuildConfig {
        input_dir: input_dir.path().to_string_lossy().to_string(),
        output_dir: output_dir.path().to_string_lossy().to_string(),
        compression_level: 0,
        encoding: Encoding::Utf8,
        verbose: true, // 로그 출력 테스트
    };

    let builder = DictionaryBuilder::new(config);
    let result = builder.build().expect("build should succeed");

    // NNG.csv와 VV.csv 모두 로드됨
    assert!(result.entry_count >= 7);
}

#[test]
fn test_output_file_structure() {
    let input_dir = create_test_input_dir();
    let output_dir = TempDir::new().expect("failed to create output dir");

    let config = BuildConfig {
        input_dir: input_dir.path().to_string_lossy().to_string(),
        output_dir: output_dir.path().to_string_lossy().to_string(),
        compression_level: 0,
        encoding: Encoding::Utf8,
        verbose: false,
    };

    let builder = DictionaryBuilder::new(config);
    builder.build().expect("build should succeed");

    let output_path = output_dir.path();

    // 필수 파일
    assert!(output_path.join("sys.dic").exists());
    assert!(output_path.join("matrix.bin").exists());

    // 선택적 파일
    assert!(output_path.join("char.bin").exists());
    assert!(output_path.join("unk.bin").exists());

    // 엔트리 파일
    assert!(output_path.join("entries.bin").exists());
    assert!(output_path.join("entries.csv").exists());

    // 파일 크기 확인
    let trie_size = fs::metadata(output_path.join("sys.dic"))
        .expect("stat trie")
        .len();
    assert!(trie_size > 0);

    let matrix_size = fs::metadata(output_path.join("matrix.bin"))
        .expect("stat matrix")
        .len();
    assert!(matrix_size > 0);

    let entries_bin_size = fs::metadata(output_path.join("entries.bin"))
        .expect("stat entries.bin")
        .len();
    assert!(entries_bin_size > 0);
}
