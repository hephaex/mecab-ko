//! # CLI Analyzer
//!
//! Interactive REPL for Korean morphological analysis using mecab-ko.
//!
//! ## Usage
//!
//! ```bash
//! # Basic REPL mode
//! cargo run --example cli_analyzer -p mecab-ko
//!
//! # Analyze single text
//! cargo run --example cli_analyzer -p mecab-ko -- --text "아버지가방에들어가신다"
//!
//! # JSON output format
//! cargo run --example cli_analyzer -p mecab-ko -- --format json --text "안녕하세요"
//!
//! # Custom dictionary path
//! cargo run --example cli_analyzer -p mecab-ko -- --dict-path /path/to/dict
//! ```
//!
//! ## Output Formats
//!
//! - `table`: Human-readable table (default)
//! - `raw`: MeCab-compatible format (surface\tfeatures)
//! - `json`: JSON array of tokens
//!
//! ## REPL Commands
//!
//! - `:format <table|raw|json>` - Change output format
//! - `:quit` or `:q` - Exit REPL
//! - `:help` - Show help

use std::collections::HashMap;
use std::io::{self, Write};
use std::path::PathBuf;

use mecab_ko::{Tokenizer, Token};

/// Output format for analysis results
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutputFormat {
    Table,
    Raw,
    Json,
}

impl OutputFormat {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "table" => Some(Self::Table),
            "raw" => Some(Self::Raw),
            "json" => Some(Self::Json),
            _ => None,
        }
    }
}

/// CLI configuration
struct Config {
    format: OutputFormat,
    dict_path: Option<PathBuf>,
    text: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            format: OutputFormat::Table,
            dict_path: None,
            text: None,
        }
    }
}

/// Parse command line arguments
fn parse_args() -> Config {
    let mut config = Config::default();
    let args: Vec<String> = std::env::args().skip(1).collect();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--format" | "-f" => {
                if i + 1 < args.len() {
                    if let Some(fmt) = OutputFormat::from_str(&args[i + 1]) {
                        config.format = fmt;
                    } else {
                        eprintln!("Invalid format: {}. Using table format.", args[i + 1]);
                    }
                    i += 2;
                } else {
                    eprintln!("Missing format value. Using table format.");
                    i += 1;
                }
            }
            "--dict-path" | "-d" => {
                if i + 1 < args.len() {
                    config.dict_path = Some(PathBuf::from(&args[i + 1]));
                    i += 2;
                } else {
                    eprintln!("Missing dictionary path.");
                    i += 1;
                }
            }
            "--text" | "-t" => {
                if i + 1 < args.len() {
                    config.text = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Missing text value.");
                    i += 1;
                }
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            _ => {
                eprintln!("Unknown argument: {}", args[i]);
                i += 1;
            }
        }
    }

    config
}

/// Print help message
fn print_help() {
    println!("CLI Analyzer - Korean Morphological Analysis Tool");
    println!();
    println!("USAGE:");
    println!("    cli_analyzer [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    -f, --format <FORMAT>    Output format: table, raw, json (default: table)");
    println!("    -d, --dict-path <PATH>   Custom dictionary path");
    println!("    -t, --text <TEXT>        Analyze single text (non-interactive)");
    println!("    -h, --help               Print help information");
    println!();
    println!("REPL COMMANDS:");
    println!("    :format <table|raw|json> Change output format");
    println!("    :quit, :q                Exit REPL");
    println!("    :help                    Show help");
}

/// Format tokens as table
fn format_table(tokens: &[Token]) -> String {
    let mut output = String::new();

    // Header
    output.push_str("┌────────┬──────┬─────────┬─────────┬──────────┬──────────┐\n");
    output.push_str("│ Surface│ POS  │ Reading │  Lemma  │ Start    │ End      │\n");
    output.push_str("├────────┼──────┼─────────┼─────────┼──────────┼──────────┤\n");

    // Rows
    for token in tokens {
        let reading = token.reading.as_deref().unwrap_or("-");
        let lemma = token.lemma.as_deref().unwrap_or("-");

        output.push_str(&format!(
            "│ {:6} │ {:4} │ {:7} │ {:7} │ {:8} │ {:8} │\n",
            truncate(&token.surface, 6),
            truncate(&token.pos, 4),
            truncate(reading, 7),
            truncate(lemma, 7),
            token.start_pos,
            token.end_pos
        ));
    }

    // Footer
    output.push_str("└────────┴──────┴─────────┴─────────┴──────────┴──────────┘\n");
    output.push_str(&format!("Total tokens: {}\n", tokens.len()));

    output
}

/// Truncate string to max length with ellipsis
fn truncate(s: &str, max_len: usize) -> String {
    if s.chars().count() > max_len {
        let truncated: String = s.chars().take(max_len - 1).collect();
        format!("{}…", truncated)
    } else {
        s.to_string()
    }
}

/// Format tokens as raw MeCab format
fn format_raw(tokens: &[Token]) -> String {
    let mut output = String::new();

    for token in tokens {
        output.push_str(&format!("{}\t{}\n", token.surface, token.features));
    }
    output.push_str("EOS\n");

    output
}

/// Format tokens as JSON
fn format_json(tokens: &[Token]) -> String {
    let json_tokens: Vec<HashMap<&str, serde_json::Value>> = tokens
        .iter()
        .map(|token| {
            let mut map = HashMap::new();
            map.insert("surface", serde_json::json!(token.surface));
            map.insert("pos", serde_json::json!(token.pos));
            map.insert("start_pos", serde_json::json!(token.start_pos));
            map.insert("end_pos", serde_json::json!(token.end_pos));
            map.insert("cost", serde_json::json!(token.cost));

            if let Some(ref reading) = token.reading {
                map.insert("reading", serde_json::json!(reading));
            }
            if let Some(ref lemma) = token.lemma {
                map.insert("lemma", serde_json::json!(lemma));
            }

            map
        })
        .collect();

    serde_json::to_string_pretty(&json_tokens).unwrap_or_else(|_| "[]".to_string())
}

/// Format tokens based on output format
fn format_output(tokens: &[Token], format: OutputFormat) -> String {
    match format {
        OutputFormat::Table => format_table(tokens),
        OutputFormat::Raw => format_raw(tokens),
        OutputFormat::Json => format_json(tokens),
    }
}

/// Process REPL command
fn process_command(line: &str, format: &mut OutputFormat) -> Option<()> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.is_empty() {
        return Some(());
    }

    match parts[0] {
        ":quit" | ":q" => {
            println!("Goodbye!");
            return None;
        }
        ":help" => {
            println!("REPL Commands:");
            println!("  :format <table|raw|json> - Change output format");
            println!("  :quit, :q                - Exit REPL");
            println!("  :help                    - Show this help");
        }
        ":format" => {
            if parts.len() < 2 {
                println!("Usage: :format <table|raw|json>");
                println!("Current format: {:?}", format);
            } else if let Some(new_format) = OutputFormat::from_str(parts[1]) {
                *format = new_format;
                println!("Output format changed to: {:?}", format);
            } else {
                println!("Invalid format: {}. Available: table, raw, json", parts[1]);
            }
        }
        _ => {
            println!("Unknown command: {}. Type :help for available commands.", parts[0]);
        }
    }

    Some(())
}

/// Run REPL mode
fn run_repl(mut tokenizer: Tokenizer, mut format: OutputFormat) {
    println!("MeCab-Ko CLI Analyzer - Interactive Mode");
    println!("Type :help for commands, :quit to exit");
    println!();

    loop {
        print!("mecab> ");
        io::stdout().flush().unwrap();

        let mut line = String::new();
        if io::stdin().read_line(&mut line).is_err() {
            break;
        }

        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Handle commands
        if line.starts_with(':') {
            if process_command(line, &mut format).is_none() {
                break;
            }
            continue;
        }

        // Analyze text
        let tokens = tokenizer.tokenize(line);
        let output = format_output(&tokens, format);
        println!("{}", output);
    }
}

/// Analyze single text (non-interactive)
fn analyze_text(mut tokenizer: Tokenizer, text: &str, format: OutputFormat) {
    let tokens = tokenizer.tokenize(text);
    let output = format_output(&tokens, format);
    print!("{}", output);
}

fn main() {
    let config = parse_args();

    // Initialize tokenizer
    let tokenizer = if let Some(ref dict_path) = config.dict_path {
        match Tokenizer::with_dict(dict_path) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Failed to load dictionary from {:?}: {}", dict_path, e);
                std::process::exit(1);
            }
        }
    } else {
        match Tokenizer::new() {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Failed to initialize tokenizer: {}", e);
                eprintln!("Make sure the dictionary is available at the default location.");
                std::process::exit(1);
            }
        }
    };

    // Run in appropriate mode
    if let Some(ref text) = config.text {
        analyze_text(tokenizer, text, config.format);
    } else {
        run_repl(tokenizer, config.format);
    }
}
