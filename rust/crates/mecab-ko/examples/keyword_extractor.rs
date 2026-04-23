//! # Keyword Extractor
//!
//! Extract keywords from Korean text using noun frequency analysis.
//!
//! ## Usage
//!
//! ```bash
//! # Extract keywords from stdin
//! echo "인공지능과 머신러닝은 현대 기술의 핵심입니다" | \
//!   cargo run --example keyword_extractor -p mecab-ko
//!
//! # Extract top 10 keywords
//! cargo run --example keyword_extractor -p mecab-ko -- --top 10 --text "텍스트..."
//!
//! # From file
//! cat document.txt | cargo run --example keyword_extractor -p mecab-ko
//!
//! # With custom POS tags
//! cargo run --example keyword_extractor -p mecab-ko -- --pos NNG,NNP --text "텍스트..."
//! ```
//!
//! ## Algorithm
//!
//! Simple TF (Term Frequency) based scoring:
//! 1. Extract nouns (NNG, NNP by default)
//! 2. Count frequency of each term
//! 3. Return top-N by frequency

use std::collections::HashMap;
use std::fmt::Write as _;
use std::io::{self, Read};
use std::path::PathBuf;

use mecab_ko::Tokenizer;

/// Configuration for keyword extraction
struct Config {
    top_n: usize,
    min_length: usize,
    pos_tags: Vec<String>,
    dict_path: Option<PathBuf>,
    text: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            top_n: 5,
            min_length: 2,
            pos_tags: vec!["NNG".to_string(), "NNP".to_string()],
            dict_path: None,
            text: None,
        }
    }
}

/// Keyword with frequency and score
#[derive(Debug, Clone)]
struct Keyword {
    term: String,
    frequency: usize,
    score: f64,
}

impl Keyword {
    fn new(term: String, frequency: usize, total_terms: usize) -> Self {
        // Simple TF score: frequency / total_terms
        let score = if total_terms > 0 {
            frequency as f64 / total_terms as f64
        } else {
            0.0
        };

        Self {
            term,
            frequency,
            score,
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
            "--top" | "-n" => {
                if i + 1 < args.len() {
                    if let Ok(n) = args[i + 1].parse::<usize>() {
                        config.top_n = n;
                    } else {
                        eprintln!("Invalid top-N value: {}", args[i + 1]);
                    }
                    i += 2;
                } else {
                    eprintln!("Missing top-N value.");
                    i += 1;
                }
            }
            "--min-length" | "-m" => {
                if i + 1 < args.len() {
                    if let Ok(len) = args[i + 1].parse::<usize>() {
                        config.min_length = len;
                    } else {
                        eprintln!("Invalid min-length value: {}", args[i + 1]);
                    }
                    i += 2;
                } else {
                    eprintln!("Missing min-length value.");
                    i += 1;
                }
            }
            "--pos" | "-p" => {
                if i + 1 < args.len() {
                    config.pos_tags = args[i + 1]
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .collect();
                    i += 2;
                } else {
                    eprintln!("Missing POS tags value.");
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
    println!("Keyword Extractor - Extract keywords from Korean text");
    println!();
    println!("USAGE:");
    println!("    keyword_extractor [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    -n, --top <N>            Number of keywords to extract (default: 5)");
    println!("    -m, --min-length <LEN>   Minimum keyword length (default: 2)");
    println!(
        "    -p, --pos <TAGS>         POS tags to include (comma-separated, default: NNG,NNP)"
    );
    println!("    -d, --dict-path <PATH>   Custom dictionary path");
    println!("    -t, --text <TEXT>        Text to analyze (reads from stdin if not provided)");
    println!("    -h, --help               Print help information");
    println!();
    println!("EXAMPLES:");
    println!("    # From stdin");
    println!("    echo \"텍스트\" | keyword_extractor");
    println!();
    println!("    # From command line");
    println!("    keyword_extractor --text \"인공지능은 미래 기술입니다\"");
    println!();
    println!("    # Custom settings");
    println!("    keyword_extractor --top 10 --min-length 3 --pos NNG,NNP,VV");
}

/// Extract keywords from text
fn extract_keywords(tokenizer: &mut Tokenizer, text: &str, config: &Config) -> Vec<Keyword> {
    // Tokenize and filter by POS tags
    let tokens = tokenizer.tokenize(text);

    let mut term_counts: HashMap<String, usize> = HashMap::new();
    let mut total_terms = 0;

    for token in tokens {
        // Check if token matches any configured POS tag
        if config.pos_tags.iter().any(|pos| token.pos.starts_with(pos)) {
            let surface = token.surface.trim();

            // Filter by minimum length
            if surface.chars().count() >= config.min_length {
                *term_counts.entry(surface.to_string()).or_insert(0) += 1;
                total_terms += 1;
            }
        }
    }

    // Convert to keywords with scores
    let mut keywords: Vec<Keyword> = term_counts
        .into_iter()
        .map(|(term, freq)| Keyword::new(term, freq, total_terms))
        .collect();

    // Sort by frequency (descending)
    keywords.sort_by(|a, b| {
        b.frequency
            .cmp(&a.frequency)
            .then_with(|| a.term.cmp(&b.term)) // Stable sort by term name
    });

    // Return top-N
    keywords.into_iter().take(config.top_n).collect()
}

/// Format keywords as table
fn format_keywords(keywords: &[Keyword]) -> String {
    if keywords.is_empty() {
        return "No keywords found.\n".to_string();
    }

    let mut output = String::new();

    // Header
    output.push_str("┌─────┬──────────────────┬───────────┬─────────┐\n");
    output.push_str("│ Rank│ Keyword          │ Frequency │ Score   │\n");
    output.push_str("├─────┼──────────────────┼───────────┼─────────┤\n");

    // Rows
    for (i, keyword) in keywords.iter().enumerate() {
        let _ = writeln!(
            output,
            "│ {:3} │ {:16} │ {:9} │ {:.5} │",
            i + 1,
            truncate(&keyword.term, 16),
            keyword.frequency,
            keyword.score
        );
    }

    // Footer
    output.push_str("└─────┴──────────────────┴───────────┴─────────┘\n");

    output
}

/// Truncate string to max length with ellipsis
fn truncate(s: &str, max_len: usize) -> String {
    if s.chars().count() > max_len {
        let truncated: String = s.chars().take(max_len - 1).collect();
        format!("{truncated}…")
    } else {
        s.to_string()
    }
}

/// Read text from stdin
fn read_stdin() -> io::Result<String> {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;
    Ok(buffer)
}

fn main() {
    let config = parse_args();

    // Initialize tokenizer
    let mut tokenizer = config.dict_path.as_ref().map_or_else(
        || match Tokenizer::new() {
            Ok(t) => t,
            Err(e) => {
                eprintln!("Failed to initialize tokenizer: {e}");
                eprintln!("Make sure the dictionary is available at the default location.");
                std::process::exit(1);
            }
        },
        |dict_path| match Tokenizer::with_dict(dict_path) {
            Ok(t) => t,
            Err(e) => {
                eprintln!(
                    "Failed to load dictionary from {}: {e}",
                    dict_path.display()
                );
                std::process::exit(1);
            }
        },
    );

    // Get input text
    let text = config.text.as_ref().map_or_else(
        || match read_stdin() {
            Ok(text) => text,
            Err(e) => {
                eprintln!("Failed to read from stdin: {e}");
                std::process::exit(1);
            }
        },
        std::clone::Clone::clone,
    );

    if text.trim().is_empty() {
        eprintln!("No input text provided.");
        std::process::exit(1);
    }

    // Extract keywords
    let keywords = extract_keywords(&mut tokenizer, &text, &config);

    // Display results
    println!("Keyword Extraction Results");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Text length: {} characters", text.chars().count());
    println!("POS tags: {}", config.pos_tags.join(", "));
    println!("Min length: {} characters", config.min_length);
    println!("Top-N: {}", config.top_n);
    println!();
    println!("{}", format_keywords(&keywords));
}
