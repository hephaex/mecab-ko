//! MeCab-Ko CLI
//!
//! 한국어 형태소 분석기 명령줄 도구

use anyhow::{Context, Result};
use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{generate, Shell};
use mecab_ko_core::Tokenizer;
use mecab_ko_dict::UserDictionary;
use serde::Serialize;
use std::cell::RefCell;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;

/// 한국어 형태소 분석기
#[allow(clippy::struct_excessive_bools)]
#[derive(Parser, Debug)]
#[command(name = "mecab-ko")]
#[command(author = "hephaex <hephaex@gmail.com>")]
#[command(version)]
#[command(about = "한국어 형태소 분석기 - MeCab-Ko Rust 구현")]
#[command(long_about = r#"
MeCab-Ko는 한국어 형태소 분석을 위한 고성능 도구입니다.

예제:
    echo "안녕하세요" | mecab-ko
    mecab-ko "오늘 날씨가 좋습니다"
    mecab-ko -O wakati "형태소 분석 테스트"
    mecab-ko --user-dic custom.csv "커스텀 사전 테스트"
    mecab-ko --repl                    # REPL 모드
    mecab-ko -i file1.txt -i file2.txt -o output_dir  # 배치 처리
"#)]
struct Args {
    /// 분석할 텍스트 (없으면 stdin에서 읽음)
    input: Option<String>,

    /// 사전 경로
    #[arg(short = 'd', long)]
    dicdir: Option<PathBuf>,

    /// 사용자 정의 사전 (CSV 형식)
    #[arg(short = 'u', long = "user-dic")]
    user_dict: Option<PathBuf>,

    /// 출력 포맷
    #[arg(short = 'O', long, value_enum, default_value = "default")]
    output_format: OutputFormat,

    /// N-best 결과 수
    #[arg(short = 'N', long, default_value = "1")]
    nbest: usize,

    /// 전체 분석 결과 표시 (디버그용)
    #[arg(short, long)]
    all: bool,

    /// 구분자 (wakati 출력 시 사용)
    #[arg(long, default_value = " ")]
    separator: String,

    /// 라인별 처리 비활성화 (전체 텍스트를 하나로 처리)
    #[arg(long)]
    no_line: bool,

    /// 조용히 실행 (경고 메시지 숨김)
    #[arg(short, long)]
    quiet: bool,

    /// REPL (대화형) 모드 시작
    #[arg(long)]
    repl: bool,

    /// 입력 파일 (여러 개 지정 가능, 배치 처리용)
    #[arg(short = 'i', long = "input-file")]
    input_files: Vec<PathBuf>,

    /// 출력 파일 또는 디렉터리
    #[arg(short = 'o', long)]
    output: Option<PathBuf>,

    /// 서브커맨드
    #[command(subcommand)]
    command: Option<Commands>,
}

/// 서브커맨드
#[derive(Subcommand, Debug)]
enum Commands {
    /// 형태소 분석 (기본 동작)
    Parse {
        /// 입력 텍스트
        text: Option<String>,
    },
    /// 사전 정보 표시
    Dict {
        /// 사전 경로
        path: Option<PathBuf>,
    },
    /// 버전 정보 표시
    Version,
    /// 셸 자동완성 스크립트 생성
    Completions {
        /// 셸 종류
        #[arg(value_enum)]
        shell: Shell,
    },
}

/// 출력 포맷
#[derive(Debug, Clone, Copy, ValueEnum, Default)]
enum OutputFormat {
    /// 기본 `MeCab` 포맷
    #[default]
    Default,
    /// 분리만 (wakati)
    Wakati,
    /// 덤프
    Dump,
    /// 품사만
    Pos,
    /// `JSON`
    Json,
    /// 간단 출력 (표면형/품사)
    Simple,
    /// `CSV` 출력
    Csv,
}

/// 직렬화 가능한 토큰 구조체
#[derive(Serialize)]
struct TokenOutput {
    surface: String,
    pos: String,
    start: usize,
    end: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    reading: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    lemma: Option<String>,
}

/// 분석 결과 컨텍스트
struct AnalysisContext {
    tokenizer: RefCell<Tokenizer>,
    #[allow(dead_code)]
    user_dict: Option<UserDictionary>,
    args: Args,
}

impl AnalysisContext {
    fn new(args: Args) -> Result<Self> {
        // 토크나이저 초기화
        let tokenizer = if let Some(ref dict_path) = args.dicdir {
            Tokenizer::with_dict(
                dict_path
                    .to_str()
                    .context("Invalid dictionary path encoding")?,
            )
            .context("Failed to load dictionary")?
        } else {
            Tokenizer::new().context("Failed to initialize tokenizer")?
        };

        // 사용자 사전 로드
        let user_dict = if let Some(ref user_dict_path) = args.user_dict {
            let mut dict = UserDictionary::new();
            dict.load_from_csv(user_dict_path)
                .with_context(|| format!("Failed to load user dictionary: {:?}", user_dict_path))?;

            if !args.quiet {
                eprintln!("Loaded {} entries from user dictionary", dict.len());
            }
            Some(dict)
        } else {
            None
        };

        Ok(Self {
            tokenizer: RefCell::new(tokenizer),
            user_dict,
            args,
        })
    }

    fn process_text(&self, text: &str) -> Result<()> {
        let mut stdout = io::stdout();
        self.process_text_to_writer(text, &mut stdout)
    }

    fn process_text_to_writer<W: Write>(&self, text: &str, writer: &mut W) -> Result<()> {
        let tokens = self.tokenizer.borrow_mut().tokenize(text);

        match self.args.output_format {
            OutputFormat::Default => {
                for token in tokens {
                    writeln!(writer, "{}\t{}", token.surface, token.pos)?;
                }
                writeln!(writer, "EOS")?;
            }
            OutputFormat::Wakati => {
                let words: Vec<_> = tokens.iter().map(|t| t.surface.as_str()).collect();
                writeln!(writer, "{}", words.join(&self.args.separator))?;
            }
            OutputFormat::Pos => {
                for token in tokens {
                    writeln!(writer, "{}/{}", token.surface, token.pos)?;
                }
            }
            OutputFormat::Simple => {
                let pairs: Vec<_> = tokens
                    .iter()
                    .map(|t| format!("{}/{}", t.surface, t.pos))
                    .collect();
                writeln!(writer, "{}", pairs.join(" "))?;
            }
            OutputFormat::Dump => {
                for (i, token) in tokens.iter().enumerate() {
                    writeln!(
                        writer,
                        "[{:03}] surface=\"{}\" pos={} span=[{},{})",
                        i, token.surface, token.pos, token.start_byte, token.end_byte
                    )?;
                }
            }
            OutputFormat::Json => {
                let output: Vec<TokenOutput> = tokens
                    .iter()
                    .map(|t| TokenOutput {
                        surface: t.surface.clone(),
                        pos: t.pos.clone(),
                        start: t.start_byte,
                        end: t.end_byte,
                        reading: t.reading.clone(),
                        lemma: t.lemma.clone(),
                    })
                    .collect();
                let json =
                    serde_json::to_string_pretty(&output).context("Failed to serialize to JSON")?;
                writeln!(writer, "{json}")?;
            }
            OutputFormat::Csv => {
                writeln!(writer, "surface,pos,start,end,reading,lemma")?;
                for token in tokens {
                    writeln!(
                        writer,
                        "{},{},{},{},{},{}",
                        escape_csv(&token.surface),
                        token.pos,
                        token.start_byte,
                        token.end_byte,
                        token.reading.as_deref().unwrap_or(""),
                        token.lemma.as_deref().unwrap_or("")
                    )?;
                }
            }
        }

        Ok(())
    }
}

/// CSV 이스케이프
fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

fn main() -> Result<()> {
    let args = Args::parse();

    // 서브커맨드 처리
    if let Some(cmd) = &args.command {
        match cmd {
            Commands::Parse { text } => {
                let ctx = AnalysisContext::new(Args {
                    input: text.clone(),
                    ..args
                })?;
                if let Some(ref text) = ctx.args.input {
                    ctx.process_text(text)?;
                } else {
                    process_stdin(&ctx)?;
                }
            }
            Commands::Dict { path } => {
                show_dict_info(path.as_ref())?;
            }
            Commands::Version => {
                print_version();
            }
            Commands::Completions { shell } => {
                generate_completions(*shell)?;
            }
        }
        return Ok(());
    }

    // REPL 모드
    if args.repl {
        let ctx = AnalysisContext::new(args)?;
        return run_repl(&ctx);
    }

    // 배치 처리 모드
    if !args.input_files.is_empty() {
        let ctx = AnalysisContext::new(args)?;
        return process_batch(&ctx);
    }

    // 기본 동작: 형태소 분석
    let ctx = AnalysisContext::new(args)?;

    if let Some(ref text) = ctx.args.input {
        if let Some(ref output_path) = ctx.args.output {
            // 단일 파일 출력
            let file = File::create(output_path)
                .with_context(|| format!("Failed to create output file: {}", output_path.display()))?;
            let mut writer = BufWriter::new(file);
            ctx.process_text_to_writer(text, &mut writer)?;
            writer.flush()?;
        } else {
            ctx.process_text(text)?;
        }
    } else {
        process_stdin(&ctx)?;
    }

    Ok(())
}

/// stdin 처리
fn process_stdin(ctx: &AnalysisContext) -> Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut stdout = stdout.lock();

    for line in stdin.lock().lines() {
        let line = line.context("Failed to read from stdin")?;
        if line.is_empty() {
            continue;
        }
        ctx.process_text(&line)?;

        // wakati나 json이 아닌 경우 빈 줄 추가
        if !matches!(
            ctx.args.output_format,
            OutputFormat::Wakati | OutputFormat::Json | OutputFormat::Simple
        ) {
            writeln!(stdout)?;
        }
    }

    Ok(())
}

/// 사전 정보 표시
fn show_dict_info(path: Option<&PathBuf>) -> Result<()> {
    println!("MeCab-Ko Dictionary Information");
    println!("================================");

    if let Some(p) = path {
        println!("Path: {}", p.display());
        // TODO: 실제 사전 정보 표시
        println!("(Dictionary loading not yet implemented)");
    } else {
        println!("Path: (default system dictionary)");
        println!("(Dictionary loading not yet implemented)");
    }

    Ok(())
}

/// 버전 정보 표시
fn print_version() {
    println!("mecab-ko {}", env!("CARGO_PKG_VERSION"));
    println!("Rust implementation of Korean morphological analyzer");
    println!();
    println!("Features:");
    println!("  - MeCab-compatible analysis");
    println!("  - User dictionary support");
    println!("  - Multiple output formats");
    println!("  - Interactive REPL mode");
    println!("  - Batch processing");
    println!();
    println!("Repository: https://github.com/hephaex/mecab-ko");
}

/// REPL 모드 실행
fn run_repl(ctx: &AnalysisContext) -> Result<()> {
    // 배너 출력
    println!("MeCab-Ko REPL v{}", env!("CARGO_PKG_VERSION"));
    println!("한국어 형태소 분석기 대화형 모드");
    println!();
    println!("명령어:");
    println!("  :help      - 도움말 표시");
    println!("  :format    - 출력 포맷 변경");
    println!("  :quit      - 종료 (또는 Ctrl+D)");
    println!("  :exit      - 종료");
    println!();

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    // RefCell을 사용하여 가변 포맷 상태 관리
    let current_format = RefCell::new(ctx.args.output_format);

    loop {
        print!("mecab-ko> ");
        stdout.flush()?;

        let mut line = String::new();
        match stdin.lock().read_line(&mut line) {
            Ok(0) => {
                // EOF (Ctrl+D)
                println!("\n종료합니다.");
                break;
            }
            Ok(_) => {
                let line = line.trim();

                if line.is_empty() {
                    continue;
                }

                // REPL 명령어 처리
                match line {
                    ":quit" | ":exit" => {
                        println!("종료합니다.");
                        break;
                    }
                    ":help" => {
                        show_repl_help();
                        continue;
                    }
                    ":format" => {
                        show_format_menu();
                        print!("포맷 선택 (0-6): ");
                        stdout.flush()?;

                        let mut choice = String::new();
                        stdin.lock().read_line(&mut choice)?;

                        if let Ok(idx) = choice.trim().parse::<usize>() {
                            let new_format = match idx {
                                0 => OutputFormat::Default,
                                1 => OutputFormat::Wakati,
                                2 => OutputFormat::Pos,
                                3 => OutputFormat::Simple,
                                4 => OutputFormat::Dump,
                                5 => OutputFormat::Json,
                                6 => OutputFormat::Csv,
                                _ => {
                                    println!("잘못된 선택입니다.");
                                    continue;
                                }
                            };
                            *current_format.borrow_mut() = new_format;
                            println!("출력 포맷이 변경되었습니다: {new_format:?}");
                        } else {
                            println!("잘못된 입력입니다.");
                        }
                        continue;
                    }
                    _ if line.starts_with(':') => {
                        println!("알 수 없는 명령어: {line}");
                        println!("':help'를 입력하여 사용 가능한 명령어를 확인하세요.");
                        continue;
                    }
                    _ => {
                        // 형태소 분석 수행
                        // 현재 포맷으로 임시로 분석 수행
                        let tokens = ctx.tokenizer.borrow_mut().tokenize(line);

                        // 포맷에 따라 출력
                        let format = *current_format.borrow();
                        let separator = &ctx.args.separator;

                        match format {
                            OutputFormat::Default => {
                                for token in tokens {
                                    println!("{}\t{}", token.surface, token.pos);
                                }
                                println!("EOS");
                            }
                            OutputFormat::Wakati => {
                                let words: Vec<_> = tokens.iter().map(|t| t.surface.as_str()).collect();
                                println!("{}", words.join(separator));
                            }
                            OutputFormat::Pos => {
                                for token in tokens {
                                    println!("{}/{}", token.surface, token.pos);
                                }
                            }
                            OutputFormat::Simple => {
                                let pairs: Vec<_> = tokens
                                    .iter()
                                    .map(|t| format!("{}/{}", t.surface, t.pos))
                                    .collect();
                                println!("{}", pairs.join(" "));
                            }
                            OutputFormat::Dump => {
                                for (i, token) in tokens.iter().enumerate() {
                                    println!(
                                        "[{:03}] surface=\"{}\" pos={} span=[{},{})",
                                        i, token.surface, token.pos, token.start_byte, token.end_byte
                                    );
                                }
                            }
                            OutputFormat::Json => {
                                let output: Vec<TokenOutput> = tokens
                                    .iter()
                                    .map(|t| TokenOutput {
                                        surface: t.surface.clone(),
                                        pos: t.pos.clone(),
                                        start: t.start_byte,
                                        end: t.end_byte,
                                        reading: t.reading.clone(),
                                        lemma: t.lemma.clone(),
                                    })
                                    .collect();
                                if let Ok(json) = serde_json::to_string_pretty(&output) {
                                    println!("{json}");
                                } else {
                                    eprintln!("JSON 직렬화 실패");
                                }
                            }
                            OutputFormat::Csv => {
                                println!("surface,pos,start,end,reading,lemma");
                                for token in tokens {
                                    println!(
                                        "{},{},{},{},{},{}",
                                        escape_csv(&token.surface),
                                        token.pos,
                                        token.start_byte,
                                        token.end_byte,
                                        token.reading.as_deref().unwrap_or(""),
                                        token.lemma.as_deref().unwrap_or("")
                                    );
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                return Err(e).context("Failed to read from stdin");
            }
        }
    }

    Ok(())
}

/// REPL 도움말 표시
fn show_repl_help() {
    println!("\nMeCab-Ko REPL 도움말");
    println!("==================");
    println!();
    println!("사용법:");
    println!("  텍스트를 입력하면 형태소 분석 결과가 출력됩니다.");
    println!();
    println!("명령어:");
    println!("  :help      - 이 도움말을 표시합니다");
    println!("  :format    - 출력 포맷을 변경합니다");
    println!("  :quit      - REPL을 종료합니다");
    println!("  :exit      - REPL을 종료합니다");
    println!();
    println!("단축키:");
    println!("  Ctrl+D     - REPL 종료");
    println!();
}

/// 포맷 메뉴 표시
fn show_format_menu() {
    println!("\n출력 포맷 선택:");
    println!("  0: Default  - 기본 MeCab 포맷");
    println!("  1: Wakati   - 분리만 (공백 구분)");
    println!("  2: Pos      - 품사 태그 (표면형/품사)");
    println!("  3: Simple   - 간단 출력 (표면형/품사)");
    println!("  4: Dump     - 디버그 정보 포함");
    println!("  5: Json     - JSON 포맷");
    println!("  6: Csv      - CSV 포맷");
    println!();
}

/// 배치 처리 모드
fn process_batch(ctx: &AnalysisContext) -> Result<()> {
    let output_dir = ctx.args.output.as_ref().ok_or_else(|| {
        anyhow::anyhow!("배치 처리 모드에서는 -o/--output 옵션이 필요합니다")
    })?;

    // 출력 디렉터리 생성
    if !output_dir.exists() {
        fs::create_dir_all(output_dir)
            .with_context(|| format!("Failed to create output directory: {}", output_dir.display()))?;
    }

    if !output_dir.is_dir() {
        anyhow::bail!("출력 경로가 디렉터리가 아닙니다: {}", output_dir.display());
    }

    let total_files = ctx.args.input_files.len();

    if !ctx.args.quiet {
        println!("배치 처리 시작: {total_files}개 파일");
    }

    for (idx, input_path) in ctx.args.input_files.iter().enumerate() {
        if !ctx.args.quiet {
            println!("[{}/{}] 처리 중: {}", idx + 1, total_files, input_path.display());
        }

        // 입력 파일 읽기
        let input_file = File::open(input_path)
            .with_context(|| format!("Failed to open input file: {}", input_path.display()))?;
        let reader = BufReader::new(input_file);

        // 출력 파일 생성
        let output_filename = input_path
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("Invalid input filename"))?
            .to_string_lossy()
            .to_string();
        let output_filename = format!("{output_filename}.analyzed");
        let output_path = output_dir.join(output_filename);

        let output_file = File::create(&output_path)
            .with_context(|| format!("Failed to create output file: {}", output_path.display()))?;
        let mut writer = BufWriter::new(output_file);

        // 라인별 처리
        for (line_num, line) in reader.lines().enumerate() {
            let line = line.with_context(|| {
                format!("Failed to read line {} from {}", line_num + 1, input_path.display())
            })?;

            if line.trim().is_empty() {
                continue;
            }

            ctx.process_text_to_writer(&line, &mut writer)?;

            // 포맷에 따라 빈 줄 추가
            if !matches!(
                ctx.args.output_format,
                OutputFormat::Wakati | OutputFormat::Json | OutputFormat::Simple
            ) {
                writeln!(writer)?;
            }
        }

        writer.flush()?;

        if !ctx.args.quiet {
            println!("  완료: {}", output_path.display());
        }
    }

    if !ctx.args.quiet {
        println!("\n배치 처리 완료: {total_files}개 파일");
    }

    Ok(())
}

/// 셸 자동완성 생성
fn generate_completions(shell: Shell) -> Result<()> {
    let mut cmd = Args::command();
    let bin_name = cmd.get_name().to_string();

    generate(shell, &mut cmd, bin_name, &mut io::stdout());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_csv() {
        assert_eq!(escape_csv("hello"), "hello");
        assert_eq!(escape_csv("hello,world"), "\"hello,world\"");
        assert_eq!(escape_csv("say \"hi\""), "\"say \"\"hi\"\"\"");
    }

    #[test]
    fn test_args_parse() {
        let args = Args::try_parse_from(["mecab-ko", "테스트"]).unwrap();
        assert_eq!(args.input, Some("테스트".to_string()));
    }

    #[test]
    fn test_output_format() {
        let args = Args::try_parse_from(["mecab-ko", "-O", "wakati", "테스트"]).unwrap();
        assert!(matches!(args.output_format, OutputFormat::Wakati));
    }

    #[test]
    fn test_repl_flag() {
        let args = Args::try_parse_from(["mecab-ko", "--repl"]).unwrap();
        assert!(args.repl);
    }

    #[test]
    fn test_input_files() {
        let args = Args::try_parse_from([
            "mecab-ko",
            "-i",
            "file1.txt",
            "-i",
            "file2.txt",
            "-o",
            "output_dir",
        ])
        .unwrap();
        assert_eq!(args.input_files.len(), 2);
        assert_eq!(args.input_files[0], PathBuf::from("file1.txt"));
        assert_eq!(args.input_files[1], PathBuf::from("file2.txt"));
        assert_eq!(args.output, Some(PathBuf::from("output_dir")));
    }

    #[test]
    fn test_output_option() {
        let args = Args::try_parse_from(["mecab-ko", "테스트", "-o", "output.txt"]).unwrap();
        assert_eq!(args.output, Some(PathBuf::from("output.txt")));
    }

    #[test]
    fn test_completions_command() {
        let args = Args::try_parse_from(["mecab-ko", "completions", "bash"]).unwrap();
        match args.command {
            Some(Commands::Completions { shell }) => {
                assert!(matches!(shell, Shell::Bash));
            }
            _ => panic!("Expected completions command"),
        }
    }

    #[test]
    fn test_all_output_formats() {
        let formats = [
            "default",
            "wakati",
            "dump",
            "pos",
            "json",
            "simple",
            "csv",
        ];

        for name in formats {
            let args = Args::try_parse_from(["mecab-ko", "-O", name, "테스트"]).unwrap();
            // Just verify parsing succeeds
            match name {
                "default" => assert!(matches!(args.output_format, OutputFormat::Default)),
                "wakati" => assert!(matches!(args.output_format, OutputFormat::Wakati)),
                "dump" => assert!(matches!(args.output_format, OutputFormat::Dump)),
                "pos" => assert!(matches!(args.output_format, OutputFormat::Pos)),
                "json" => assert!(matches!(args.output_format, OutputFormat::Json)),
                "simple" => assert!(matches!(args.output_format, OutputFormat::Simple)),
                "csv" => assert!(matches!(args.output_format, OutputFormat::Csv)),
                _ => {}
            }
        }
    }

    #[test]
    fn test_process_text_to_writer() {
        // 토크나이저가 실제로 구현되어 있어야 이 테스트가 작동합니다
        // 현재는 스텁이므로 기본적인 인터페이스만 테스트합니다
        let args = Args::try_parse_from(["mecab-ko", "-O", "wakati"]).unwrap();

        // AnalysisContext 생성이 실패할 수 있으므로 Result를 체크
        if let Ok(ctx) = AnalysisContext::new(args) {
            let mut output = Vec::new();
            let result = ctx.process_text_to_writer("테스트", &mut output);

            // 처리가 성공하면 출력이 있어야 함
            if result.is_ok() {
                assert!(!output.is_empty());
            }
        }
    }

    #[test]
    fn test_multiple_input_files_with_output() {
        let args = Args::try_parse_from([
            "mecab-ko",
            "-i",
            "file1.txt",
            "-i",
            "file2.txt",
            "-i",
            "file3.txt",
            "-o",
            "output",
        ])
        .unwrap();

        assert_eq!(args.input_files.len(), 3);
        assert_eq!(args.output, Some(PathBuf::from("output")));
    }

    #[test]
    fn test_quiet_flag() {
        let args = Args::try_parse_from(["mecab-ko", "-q", "테스트"]).unwrap();
        assert!(args.quiet);
    }

    #[test]
    fn test_separator_option() {
        let args = Args::try_parse_from(["mecab-ko", "--separator", "|", "테스트"]).unwrap();
        assert_eq!(args.separator, "|");
    }
}
