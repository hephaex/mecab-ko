//! mecab-ko-dict-builder CLI
//!
//! 한국어 형태소 사전 빌더 명령줄 도구

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::map_unwrap_or
)]

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use indicatif::{ProgressBar, ProgressStyle};
use mecab_ko_dict_builder::builder::BuildConfig;
use mecab_ko_dict_builder::csv_parser::Encoding;
use mecab_ko_dict_builder::DictionaryBuilder;
use std::path::PathBuf;
use std::time::Instant;

/// Dictionary entries binary format version
#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputFormat {
    /// v2 format (MKE2) - `LazyEntries` with u16 feature length limit
    #[value(name = "v2")]
    V2,
    /// v3 format (MKE3) - `LazyEntries` v3 with u32 feature length limit
    #[value(name = "v3")]
    V3,
}

/// 한국어 형태소 사전 빌더
#[derive(Parser, Debug)]
#[command(name = "mecab-ko-dict-builder")]
#[command(author = "hephaex <hephaex@gmail.com>")]
#[command(version)]
#[command(about = "mecab-ko-dic CSV를 바이너리 사전으로 변환")]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,

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

#[derive(Subcommand, Debug)]
enum Commands {
    /// CSV에서 바이너리 사전 빌드 (기본 동작)
    Build {
        /// 입력 디렉토리
        #[arg(short, long, default_value = ".")]
        input: String,

        /// 출력 디렉토리
        #[arg(short, long, default_value = "./dict")]
        output: String,

        /// 압축 레벨
        #[arg(short = 'c', long, default_value = "3")]
        compression: i32,

        /// 인코딩
        #[arg(short = 'e', long, default_value = "auto")]
        encoding: String,

        /// 자세한 출력
        #[arg(short, long)]
        verbose: bool,

        /// 진행 표시 비활성화
        #[arg(long)]
        no_progress: bool,
    },

    /// entries.bin을 v2/v3 포맷으로 변환
    ///
    /// `LazyEntries` 지원을 위해 v1(MKED) 포맷을 v2(MKE2) 또는 v3(MKE3) 포맷으로 변환합니다.
    /// v2/v3 포맷은 메모리 매핑과 지연 로딩을 지원하여 메모리 사용량을 최대 77% 절감합니다.
    /// v3는 feature 문자열 길이 제한(64 KiB)을 u32로 확장합니다.
    Convert {
        /// 사전 디렉토리 (entries.bin 또는 entries.csv 포함)
        #[arg(short, long)]
        dict: PathBuf,

        /// 출력 파일 경로 (기본: <dict>/entries.bin)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// 기존 entries.bin 백업
        #[arg(long, default_value = "true")]
        backup: bool,

        /// 자세한 출력
        #[arg(short, long)]
        verbose: bool,

        /// 출력 포맷 (v2 또는 v3, 기본값: v2)
        #[arg(long, value_enum, default_value = "v2")]
        output_format: OutputFormat,
    },

    /// 사전 정보 표시
    Info {
        /// 사전 디렉토리
        #[arg(short, long)]
        dict: PathBuf,
    },
}

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Some(Commands::Build {
            input,
            output,
            compression,
            encoding,
            verbose,
            no_progress,
        }) => run_build(
            &input,
            &output,
            compression,
            &encoding,
            verbose,
            no_progress,
        ),

        Some(Commands::Convert {
            dict,
            output,
            backup,
            verbose,
            output_format,
        }) => run_convert(&dict, output.as_deref(), backup, verbose, output_format),

        Some(Commands::Info { dict }) => run_info(&dict),

        None => {
            // 기본 동작: build
            run_build(
                &args.input,
                &args.output,
                args.compression,
                &args.encoding,
                args.verbose,
                args.no_progress,
            )
        }
    }
}

fn run_build(
    input: &str,
    output: &str,
    compression: i32,
    encoding_str: &str,
    verbose: bool,
    no_progress: bool,
) -> Result<()> {
    // 로깅 설정
    if verbose {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .init();
    }

    // 인코딩 파싱
    let encoding = match encoding_str.to_lowercase().as_str() {
        "utf8" | "utf-8" => Encoding::Utf8,
        "euc-kr" | "euckr" | "cp949" => Encoding::EucKr,
        "auto" => Encoding::Auto,
        _ => {
            eprintln!("Invalid encoding: {encoding_str}. Using auto detection.");
            Encoding::Auto
        }
    };

    // 빌드 설정
    let config = BuildConfig {
        input_dir: input.to_string(),
        output_dir: output.to_string(),
        compression_level: compression,
        encoding,
        verbose,
    };

    println!("=== MeCab-Ko Dictionary Builder ===");
    println!("Input:       {input}");
    println!("Output:      {output}");
    println!("Compression: level {compression}");
    println!("Encoding:    {encoding:?}");
    println!();

    // 진행 표시
    let progress = if no_progress {
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
        std::path::Path::new(output)
            .canonicalize()
            .unwrap_or_else(|_| std::path::PathBuf::from(output))
            .display()
    );

    println!("\nDictionary build successful!");

    Ok(())
}

fn save_entries_format(
    entries: &[mecab_ko_dict::DictEntry],
    output_path: &std::path::Path,
    output_format: OutputFormat,
) -> Result<()> {
    use mecab_ko_dict::lazy_entries_v3::save_entries_v3;
    use mecab_ko_dict::LazyEntries;

    let (fmt_label, magic_label) = match output_format {
        OutputFormat::V3 => ("v3", "MKE3"),
        OutputFormat::V2 => ("v2", "MKE2"),
    };
    println!("4. Saving as {fmt_label} format ({magic_label})...");
    let save_start = Instant::now();
    match output_format {
        OutputFormat::V3 => save_entries_v3(entries, output_path)?,
        OutputFormat::V2 => LazyEntries::save_entries(entries, output_path)?,
    }
    println!("   Saved in {:?}", save_start.elapsed());

    Ok(())
}

fn verify_entries_format(output_path: &std::path::Path, output_format: OutputFormat) -> Result<()> {
    use mecab_ko_dict::lazy_entries_v3::LazyEntriesV3;
    use mecab_ko_dict::LazyEntries;

    let fmt_label = match output_format {
        OutputFormat::V3 => "v3",
        OutputFormat::V2 => "v2",
    };
    println!("\n5. Verifying {fmt_label} format...");
    let entry_count = match output_format {
        OutputFormat::V3 => {
            let lazy = LazyEntriesV3::from_file(output_path)?;
            let count = lazy.len();
            println!("   LazyEntriesV3 loaded: {count} entries");
            if let Ok(first) = lazy.get(0) {
                println!("   First entry: {}", first.surface);
            }
            count
        }
        OutputFormat::V2 => {
            let lazy = LazyEntries::from_file(output_path)?;
            let count = lazy.len();
            println!("   LazyEntries loaded: {count} entries");
            if let Ok(first) = lazy.get(0) {
                println!("   First entry: {}", first.surface);
            }
            count
        }
    };

    if entry_count == 0 {
        anyhow::bail!("Verification failed: entries.bin contains 0 entries");
    }

    Ok(())
}

fn run_convert(
    dict: &PathBuf,
    output: Option<&std::path::Path>,
    backup: bool,
    verbose: bool,
    output_format: OutputFormat,
) -> Result<()> {
    use mecab_ko_dict::dictionary::{LoadOptions, SystemDictionary};
    use mecab_ko_dict::lazy_entries_v3::{detect_entries_format, EntriesFormat};

    let target_format_label = match output_format {
        OutputFormat::V2 => "v2",
        OutputFormat::V3 => "v3",
    };

    println!(
        "=== entries.bin {} Format Converter ===\n",
        target_format_label.to_uppercase()
    );
    println!("Dictionary: {}", dict.display());

    let entries_bin = dict.join("entries.bin");
    let output_path = output.map_or_else(|| entries_bin.clone(), std::path::Path::to_path_buf);

    // 기존 포맷 확인
    if entries_bin.exists() {
        let current_fmt = detect_entries_format(&entries_bin);
        let current_label = match &current_fmt {
            Ok(EntriesFormat::V1) => "v1 (MKED) - Eager only",
            Ok(EntriesFormat::V2) => "v2 (MKE2) - LazyEntries supported",
            Ok(EntriesFormat::V3) => "v3 (MKE3) - LazyEntries v3 supported",
            _ => "Unknown",
        };
        println!("Current format: {current_label}");

        // Skip if already in the requested format.
        if matches!(
            (&current_fmt, output_format),
            (Ok(EntriesFormat::V2), OutputFormat::V2) | (Ok(EntriesFormat::V3), OutputFormat::V3)
        ) {
            println!(
                "\n이미 {} 포맷입니다. 변환이 필요하지 않습니다.",
                target_format_label.to_uppercase()
            );
            return Ok(());
        }
    }

    // Eager 모드로 사전 로드
    println!("\n1. Loading dictionary (Eager mode)...");
    let start = Instant::now();
    let dict_loaded = SystemDictionary::load_with_options(dict, LoadOptions::eager())?;
    println!("   Loaded in {:?}", start.elapsed());

    let entry_count = dict_loaded.entry_count();
    println!("   Entry count: {entry_count}");

    // 엔트리 수집
    println!("2. Collecting entries...");
    let mut entries = Vec::with_capacity(entry_count);
    for i in 0..entry_count {
        if let Ok(entry) = dict_loaded.get_entry(i as u32) {
            entries.push((*entry).clone());
        }
    }
    println!("   Collected: {} entries", entries.len());

    // 백업
    if backup && entries_bin.exists() && output_path == entries_bin {
        let bak_suffix = match detect_entries_format(&entries_bin) {
            Ok(EntriesFormat::V1) => "v1.bak",
            Ok(EntriesFormat::V2) => "v2.bak",
            Ok(EntriesFormat::V3) => "v3.bak",
            _ => "orig.bak",
        };
        let backup_path = dict.join(format!("entries.bin.{bak_suffix}"));
        println!("3. Backing up to {}", backup_path.display());
        std::fs::copy(&entries_bin, &backup_path)?;
    }

    // 지정된 포맷으로 저장
    save_entries_format(&entries, &output_path, output_format)?;

    // 결과 확인
    let out_size = std::fs::metadata(&output_path)?.len();
    println!("\n=== Conversion Complete ===");
    println!("Output: {}", output_path.display());
    println!("Size: {:.1} MB", out_size as f64 / 1024.0 / 1024.0);

    // 검증
    if verbose {
        verify_entries_format(&output_path, output_format)?;
    }

    println!("\n메모리 절감 효과:");
    println!("  - Lazy 모드 사용 시 메모리 최대 77% 절감");
    println!("  - 로드 시간 최대 95% 단축");
    if matches!(output_format, OutputFormat::V3) {
        println!("  - v3: feature 문자열 길이 제한 u16 → u32 (최대 4 GiB)");
    }
    println!("\n사용법:");
    println!("  let dict = SystemDictionary::load_with_options(path, LoadOptions::default())?;");

    Ok(())
}

fn run_info(dict: &PathBuf) -> Result<()> {
    use mecab_ko_dict::dictionary::{LoadOptions, SystemDictionary};
    use mecab_ko_dict::lazy_entries_v3::{detect_entries_format, EntriesFormat};

    println!("=== Dictionary Info ===\n");
    println!("Path: {}", dict.display());

    // 파일 확인
    let files = [
        "sys.dic",
        "sys.dic.zst",
        "matrix.bin",
        "matrix.bin.zst",
        "entries.bin",
        "entries.csv",
        "unk.bin",
    ];

    println!("\nFiles:");
    for file in &files {
        let path = dict.join(file);
        if path.exists() {
            let size = std::fs::metadata(&path)?.len();
            println!("  {} - {:.1} MB", file, size as f64 / 1024.0 / 1024.0);
        }
    }

    // entries.bin 포맷 확인
    let entries_bin = dict.join("entries.bin");
    if entries_bin.exists() {
        let fmt = detect_entries_format(&entries_bin);
        let fmt_label = match &fmt {
            Ok(EntriesFormat::V1) => "V1 (MKED) - Eager loading only",
            Ok(EntriesFormat::V2) => "V2 (MKE2) - LazyEntries supported",
            Ok(EntriesFormat::V3) => "V3 (MKE3) - LazyEntries v3 supported",
            Ok(_) | Err(_) => "Unknown format",
        };
        println!("\nentries.bin format: {fmt_label}");

        if matches!(fmt, Ok(EntriesFormat::V1)) {
            println!("  → Run 'convert' command for memory optimization");
        }
    }

    // 사전 로드 테스트
    println!("\nLoading dictionary...");
    let start = Instant::now();
    let dict_loaded = SystemDictionary::load_with_options(dict, LoadOptions::eager())?;
    let load_time = start.elapsed();

    println!("  Load time: {load_time:?}");
    println!("  Entry count: {}", dict_loaded.entry_count());

    Ok(())
}
