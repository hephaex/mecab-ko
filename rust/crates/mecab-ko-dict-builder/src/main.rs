//! mecab-ko-dict-builder CLI
//!
//! 한국어 형태소 사전 빌더 명령줄 도구

use anyhow::Result;
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use mecab_ko_dict_builder::builder::BuildConfig;
use mecab_ko_dict_builder::csv_parser::Encoding;
use mecab_ko_dict_builder::DictionaryBuilder;
use std::time::Instant;

/// 한국어 형태소 사전 빌더
#[derive(Parser, Debug)]
#[command(name = "mecab-ko-dict-builder")]
#[command(author = "hephaex <hephaex@gmail.com>")]
#[command(version)]
#[command(about = "mecab-ko-dic CSV를 바이너리 사전으로 변환")]
struct Args {
    /// 입력 디렉토리 (mecab-ko-dic CSV 파일들)
    #[arg(short, long, default_value = ".")]
    input: String,

    /// 출력 디렉토리
    #[arg(short, long, default_value = "./dict")]
    output: String,

    /// 압축 레벨 (0-22, 0=압축 안 함, 기본값: 3)
    #[arg(short = 'c', long, default_value = "3")]
    compression: i32,

    /// 인코딩 (utf8, euc-kr, auto)
    #[arg(short = 'e', long, default_value = "auto")]
    encoding: String,

    /// 자세한 출력
    #[arg(short, long)]
    verbose: bool,

    /// 진행 표시 비활성화
    #[arg(long)]
    no_progress: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // 로깅 설정
    if args.verbose {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .init();
    }

    // 인코딩 파싱
    let encoding = match args.encoding.to_lowercase().as_str() {
        "utf8" | "utf-8" => Encoding::Utf8,
        "euc-kr" | "euckr" | "cp949" => Encoding::EucKr,
        "auto" => Encoding::Auto,
        _ => {
            eprintln!("Invalid encoding: {}. Using auto detection.", args.encoding);
            Encoding::Auto
        }
    };

    // 빌드 설정
    let config = BuildConfig {
        input_dir: args.input.clone(),
        output_dir: args.output.clone(),
        compression_level: args.compression,
        encoding,
        verbose: args.verbose,
    };

    println!("=== MeCab-Ko Dictionary Builder ===");
    println!("Input:       {}", args.input);
    println!("Output:      {}", args.output);
    println!("Compression: level {}", args.compression);
    println!("Encoding:    {encoding:?}");
    println!();

    // 진행 표시
    let progress = if args.no_progress {
        None
    } else {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap_or_else(|_| ProgressStyle::default_spinner()),
        );
        pb.enable_steady_tick(std::time::Duration::from_millis(100));
        Some(pb)
    };

    // 빌드 시작
    let start = Instant::now();

    if let Some(ref pb) = progress {
        pb.set_message("Parsing CSV files...");
    }

    let builder = DictionaryBuilder::new(config);
    let result = builder.build()?;

    if let Some(ref pb) = progress {
        pb.finish_with_message("Build completed!");
    }

    let elapsed = start.elapsed();

    // 결과 출력
    result.print_summary();
    println!("\nTime elapsed: {:.2}s", elapsed.as_secs_f64());
    println!(
        "Output directory: {}",
        std::path::Path::new(&args.output)
            .canonicalize()
            .unwrap_or_else(|_| std::path::PathBuf::from(&args.output))
            .display()
    );

    println!("\nDictionary build successful!");

    Ok(())
}
