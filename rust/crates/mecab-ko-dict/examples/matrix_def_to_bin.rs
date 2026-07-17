//! matrix.def (텍스트) → matrix.bin.zst (압축 바이너리) 단독 변환 도구.
//!
//! Sprint 138 P1: dict-builder의 CSV 파싱 버그를 우회하여 matrix만 갱신.
//!
//! 사용법:
//!   cargo run --release --example `matrix_def_to_bin` -- <input.def> <output.bin.zst>

#![allow(clippy::expect_used)]

use mecab_ko_dict::matrix::DenseMatrix;
use mecab_ko_dict::Matrix;
use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <input.def> <output.bin.zst>", args[0]);
        return ExitCode::from(1);
    }
    let input = &args[1];
    let output = &args[2];

    println!("Loading matrix from {input}");
    let matrix = match DenseMatrix::from_def_file(input) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Failed to load matrix: {e}");
            return ExitCode::from(2);
        }
    };
    println!(
        "Loaded: lsize={} rsize={}",
        matrix.left_size(),
        matrix.right_size()
    );

    println!("Writing compressed binary to {output}");
    if let Err(e) = matrix.to_compressed_file(output, 3) {
        eprintln!("Failed to write: {e}");
        return ExitCode::from(3);
    }
    println!("Done.");
    ExitCode::SUCCESS
}
