//! MeCab-Ko CLI
//!
//! 한국어 형태소 분석기 명령줄 도구

use anyhow::Result;
use clap::{Parser, ValueEnum};
use mecab_ko_core::Tokenizer;
use std::io::{self, BufRead, Write};

/// 한국어 형태소 분석기
#[derive(Parser, Debug)]
#[command(name = "mecab-ko")]
#[command(author = "hephaex <hephaex@gmail.com>")]
#[command(version)]
#[command(about = "한국어 형태소 분석기 - MeCab-Ko Rust 구현")]
struct Args {
    /// 분석할 텍스트 (없으면 stdin에서 읽음)
    input: Option<String>,
    
    /// 사전 경로
    #[arg(short = 'd', long)]
    dicdir: Option<String>,
    
    /// 출력 포맷
    #[arg(short = 'O', long, value_enum, default_value = "default")]
    output_format: OutputFormat,
    
    /// N-best 결과 수
    #[arg(short = 'N', long, default_value = "1")]
    nbest: usize,
    
    /// 전체 분석 결과 표시
    #[arg(short, long)]
    all: bool,
}

/// 출력 포맷
#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputFormat {
    /// 기본 MeCab 포맷
    Default,
    /// 분리만 (wakati)
    Wakati,
    /// 덤프
    Dump,
    /// 품사만
    Pos,
    /// JSON
    Json,
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    // 토크나이저 초기화
    let tokenizer = if let Some(ref dict_path) = args.dicdir {
        Tokenizer::with_dict(dict_path)?
    } else {
        Tokenizer::new()?
    };
    
    // 입력 처리
    if let Some(text) = args.input {
        process_text(&tokenizer, &text, &args)?;
    } else {
        // stdin에서 읽기
        let stdin = io::stdin();
        let stdout = io::stdout();
        let mut stdout = stdout.lock();
        
        for line in stdin.lock().lines() {
            let line = line?;
            if line.is_empty() {
                continue;
            }
            process_text(&tokenizer, &line, &args)?;
            writeln!(stdout)?;
        }
    }
    
    Ok(())
}

fn process_text(tokenizer: &Tokenizer, text: &str, args: &Args) -> Result<()> {
    let tokens = tokenizer.tokenize(text);
    
    match args.output_format {
        OutputFormat::Default => {
            for token in tokens {
                println!("{}\t{}", token.surface, token.pos);
            }
            println!("EOS");
        }
        OutputFormat::Wakati => {
            let words: Vec<_> = tokens.iter().map(|t| t.surface.as_str()).collect();
            println!("{}", words.join(" "));
        }
        OutputFormat::Pos => {
            for token in tokens {
                println!("{}/{}", token.surface, token.pos);
            }
        }
        OutputFormat::Dump => {
            for (i, token) in tokens.iter().enumerate() {
                println!("[{}] surface={} pos={} start={} end={}", 
                    i, token.surface, token.pos, token.start, token.end);
            }
        }
        OutputFormat::Json => {
            let json = serde_json::to_string_pretty(&tokens.iter().map(|t| {
                serde_json::json!({
                    "surface": t.surface,
                    "pos": t.pos,
                    "start": t.start,
                    "end": t.end,
                    "reading": t.reading,
                    "lemma": t.lemma,
                })
            }).collect::<Vec<_>>())?;
            println!("{json}");
        }
    }
    
    Ok(())
}
