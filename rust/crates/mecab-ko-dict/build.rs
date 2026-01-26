//! build.rs - 테스트 사전 생성
//!
//! 빌드 시 작은 테스트 사전을 번들링합니다.

#![allow(clippy::expect_used)] // Build scripts are allowed to use expect

use std::env;
use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let dict_dir = Path::new(&out_dir).join("test_dict");

    // 디렉토리 생성
    fs::create_dir_all(&dict_dir).expect("create dict dir");

    // 작은 테스트 사전 생성
    create_test_dictionary(&dict_dir);

    println!("cargo:rustc-env=TEST_DICT_PATH={}", dict_dir.display());
}

fn create_test_dictionary(dict_dir: &Path) {
    // 테스트 CSV 데이터
    let csv_content = "가,1,1,100,NNG,*,T,가,*,NNG,NNG,*
가다,2,2,200,VV,*,F,가다,*,VV,VV,*
가방,1,1,150,NNG,*,T,가방,*,NNG,NNG,*
안녕,1,1,120,NNG,*,T,안녕,*,NNG,NNG,*
하다,2,2,180,VV,*,F,하다,*,VV,VV,*
";

    // matrix.def (2x2 행렬)
    let matrix_content = "2 2\n0 0 100\n0 1 200\n1 0 150\n1 1 50\n";

    // char.def
    let char_def_content = "DEFAULT 1 0 0
SPACE   0 1 0
HANGUL  1 1 0
0x0020 SPACE
0x0009 SPACE
";

    // unk.def
    let unk_def_content = "DEFAULT,0,0,0,UNK,*,*,*,*,*,*,*
SPACE,0,0,0,SP,*,*,*,*,*,*,*
HANGUL,1,1,1000,UNK,*,*,*,*,*,*,*
";

    // 파일 작성
    fs::write(dict_dir.join("test.csv"), csv_content).expect("write csv");
    fs::write(dict_dir.join("matrix.def"), matrix_content).expect("write matrix");
    fs::write(dict_dir.join("char.def"), char_def_content).expect("write char.def");
    fs::write(dict_dir.join("unk.def"), unk_def_content).expect("write unk.def");

    // 간단한 Trie 빌드 (수동)
    build_simple_trie(dict_dir);
}

fn build_simple_trie(dict_dir: &Path) {
    // 최소한의 바이너리 Trie 생성
    // 실제로는 yada 라이브러리를 사용해야 하지만,
    // 빌드 스크립트에서는 의존성 문제로 간단히 더미 파일 생성

    let mut trie_data = Vec::new();

    // 매직 헤더 (MKTRIE\0\0)
    trie_data.extend_from_slice(b"MKTRIE\0\0");

    // 버전 (1)
    trie_data.extend_from_slice(&1u32.to_le_bytes());

    // 엔트리 개수 (5)
    trie_data.extend_from_slice(&5u32.to_le_bytes());

    // 더미 데이터 (실제 사전 빌더가 생성할 것)
    trie_data.extend_from_slice(&[0u8; 128]);

    fs::write(dict_dir.join("sys.dic"), &trie_data).expect("write trie");

    // 간단한 매트릭스 바이너리
    let mut matrix_data = Vec::new();

    // left_size (2)
    matrix_data.extend_from_slice(&2u16.to_le_bytes());

    // right_size (2)
    matrix_data.extend_from_slice(&2u16.to_le_bytes());

    // 비용 데이터 (2x2 = 4 entries)
    matrix_data.extend_from_slice(&100i16.to_le_bytes()); // 0,0
    matrix_data.extend_from_slice(&200i16.to_le_bytes()); // 0,1
    matrix_data.extend_from_slice(&150i16.to_le_bytes()); // 1,0
    matrix_data.extend_from_slice(&50i16.to_le_bytes()); // 1,1

    fs::write(dict_dir.join("matrix.bin"), &matrix_data).expect("write matrix.bin");
}
