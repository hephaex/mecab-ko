//! MeCab-Ko CLI
//!
//! 한국어 형태소 분석기 명령줄 도구

use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use mecab_ko_core::Tokenizer;
use mecab_ko_dict::UserDictionary;
use serde::Serialize;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

/// 한국어 형태소 분석기
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
}

/// 출력 포맷
#[derive(Debug, Clone, Copy, ValueEnum, Default)]
enum OutputFormat {
    /// 기본 MeCab 포맷
    #[default]
    Default,
    /// 분리만 (wakati)
    Wakati,
    /// 덤프
    Dump,
    /// 품사만
    Pos,
    /// JSON
    Json,
    /// 간단 출력 (표면형/품사)
    Simple,
    /// CSV 출력
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
    tokenizer: Tokenizer,
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
            tokenizer,
            user_dict,
            args,
        })
    }

    fn process_text(&self, text: &str) -> Result<()> {
        let tokens = self.tokenizer.tokenize(text);

        match self.args.output_format {
            OutputFormat::Default => {
                for token in tokens {
                    println!("{}\t{}", token.surface, token.pos);
                }
                println!("EOS");
            }
            OutputFormat::Wakati => {
                let words: Vec<_> = tokens.iter().map(|t| t.surface.as_str()).collect();
                println!("{}", words.join(&self.args.separator));
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
                        i, token.surface, token.pos, token.start, token.end
                    );
                }
            }
            OutputFormat::Json => {
                let output: Vec<TokenOutput> = tokens
                    .iter()
                    .map(|t| TokenOutput {
                        surface: t.surface.clone(),
                        pos: t.pos.clone(),
                        start: t.start,
                        end: t.end,
                        reading: t.reading.clone(),
                        lemma: t.lemma.clone(),
                    })
                    .collect();
                let json =
                    serde_json::to_string_pretty(&output).context("Failed to serialize to JSON")?;
                println!("{json}");
            }
            OutputFormat::Csv => {
                println!("surface,pos,start,end,reading,lemma");
                for token in tokens {
                    println!(
                        "{},{},{},{},{},{}",
                        escape_csv(&token.surface),
                        token.pos,
                        token.start,
                        token.end,
                        token.reading.as_deref().unwrap_or(""),
                        token.lemma.as_deref().unwrap_or("")
                    );
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
        }
        return Ok(());
    }

    // 기본 동작: 형태소 분석
    let ctx = AnalysisContext::new(args)?;

    if let Some(ref text) = ctx.args.input {
        ctx.process_text(text)?;
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
    println!();
    println!("Repository: https://github.com/hephaex/mecab-ko");
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
}
