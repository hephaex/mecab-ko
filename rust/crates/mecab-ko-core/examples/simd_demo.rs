//! SIMD 최적화 데모
//!
//! 이 예제는 SIMD 최적화가 활성화되었을 때의 성능을 보여줍니다.
//!
//! 실행:
//! ```bash
//! cargo run --example simd_demo --features simd --release
//! ```

use mecab_ko_core::lattice::{Lattice, NodeBuilder};
use mecab_ko_core::viterbi::ViterbiSearcher;
use mecab_ko_dict::{DenseMatrix, Matrix};
use std::time::Instant;

#[cfg(feature = "simd")]
use mecab_ko_dict::matrix::simd::SimdMatrix;

fn create_test_matrix() -> DenseMatrix {
    let size = 200;
    let mut matrix = DenseMatrix::new(size, size, 0);

    // 현실적인 연접 비용 패턴 설정
    for left in 0..size {
        for right in 0..size {
            let cost = ((left * 17 + right * 13) % 10000) as i16 - 5000;
            matrix.set(right as u16, left as u16, cost);
        }
    }

    matrix
}

fn create_test_lattice(text: &str) -> Lattice {
    let mut lattice = Lattice::new(text);
    let char_len = lattice.char_len();

    // 각 위치에서 여러 노드 추가
    for start_pos in 0..char_len {
        for len in 1..=2.min(char_len - start_pos) {
            for variant in 0..3 {
                let end_pos = start_pos + len;

                // 문자 경계 찾기
                let surface = text.chars().take(end_pos).collect::<String>();

                lattice.add_node(
                    NodeBuilder::new(&surface, start_pos, end_pos)
                        .left_id((variant * 10 + start_pos) as u16 % 200)
                        .right_id((variant * 20 + end_pos) as u16 % 200)
                        .word_cost((variant * 100 + len * 50) as i32),
                );
            }
        }
    }

    lattice
}

fn bench_scalar_lookup(matrix: &DenseMatrix, iterations: usize) -> std::time::Duration {
    let start = Instant::now();

    for _ in 0..iterations {
        let mut sum = 0i32;
        for i in 0..100 {
            let right = (i * 7) % 200;
            let left = (i * 13) % 200;
            sum = sum.wrapping_add(matrix.get(right as u16, left as u16));
        }
        std::hint::black_box(sum);
    }

    start.elapsed()
}

#[cfg(feature = "simd")]
fn bench_simd_lookup(matrix: &DenseMatrix, iterations: usize) -> std::time::Duration {
    let start = Instant::now();

    for _ in 0..iterations {
        let mut sum = 0i32;
        // 8개씩 배치 조회
        for chunk in 0..12 {
            let base = chunk * 8;
            let right_ids = [
                ((base + 0) * 7 % 200) as u16,
                ((base + 1) * 7 % 200) as u16,
                ((base + 2) * 7 % 200) as u16,
                ((base + 3) * 7 % 200) as u16,
                ((base + 4) * 7 % 200) as u16,
                ((base + 5) * 7 % 200) as u16,
                ((base + 6) * 7 % 200) as u16,
                ((base + 7) * 7 % 200) as u16,
            ];
            let left_ids = [
                ((base + 0) * 13 % 200) as u16,
                ((base + 1) * 13 % 200) as u16,
                ((base + 2) * 13 % 200) as u16,
                ((base + 3) * 13 % 200) as u16,
                ((base + 4) * 13 % 200) as u16,
                ((base + 5) * 13 % 200) as u16,
                ((base + 6) * 13 % 200) as u16,
                ((base + 7) * 13 % 200) as u16,
            ];

            let costs = matrix.batch_get_8(&right_ids, &left_ids);
            for cost in costs.iter() {
                sum = sum.wrapping_add(*cost);
            }
        }
        std::hint::black_box(sum);
    }

    start.elapsed()
}

fn bench_viterbi(text: &str, iterations: usize) -> std::time::Duration {
    let matrix = create_test_matrix();

    let start = Instant::now();

    for _ in 0..iterations {
        let mut lattice = create_test_lattice(text);
        let searcher = ViterbiSearcher::new();
        let path = searcher.search(&mut lattice, &matrix);
        std::hint::black_box(path);
    }

    start.elapsed()
}

fn main() {
    println!("=== MeCab-Ko SIMD 최적화 데모 ===\n");

    let matrix = create_test_matrix();
    let iterations = 10000;

    // 1. 연접 비용 행렬 조회 벤치마크
    println!("1. 연접 비용 행렬 조회 ({}회 반복)", iterations);
    println!("   행렬 크기: 200x200");

    let scalar_time = bench_scalar_lookup(&matrix, iterations);
    println!("   스칼라 조회: {:.2?}", scalar_time);

    #[cfg(feature = "simd")]
    {
        let simd_time = bench_simd_lookup(&matrix, iterations);
        println!("   SIMD 배치 조회: {:.2?}", simd_time);

        let speedup = scalar_time.as_secs_f64() / simd_time.as_secs_f64();
        println!("   성능 향상: {:.2}배\n", speedup);
    }

    #[cfg(not(feature = "simd"))]
    {
        println!("   SIMD: 비활성화됨\n");
    }

    // 2. Viterbi 알고리즘 벤치마크
    println!("2. Viterbi 알고리즘 (100회 반복)");

    let test_texts = vec![
        ("짧은 텍스트", "안녕하세요"),
        ("중간 텍스트", "형태소분석기테스트입니다"),
        ("긴 텍스트", "자연어처리는매우흥미로운분야입니다"),
    ];

    for (name, text) in test_texts {
        println!("   {}: \"{}\"", name, text);
        let time = bench_viterbi(text, 100);
        println!("   시간: {:.2?}\n", time);
    }

    // 3. SIMD 기능 상태
    println!("3. SIMD 기능 상태");
    #[cfg(feature = "simd")]
    {
        println!("   ✓ SIMD 최적화 활성화됨");
        println!("   - portable_simd 사용");
        println!("   - 배치 크기: 8 (i32x8, u16x8)");
        println!("   - 지원 플랫폼: x86_64, aarch64");
    }

    #[cfg(not(feature = "simd"))]
    {
        println!("   ✗ SIMD 최적화 비활성화됨");
        println!("   활성화 방법: cargo run --example simd_demo --features simd --release");
    }
}
