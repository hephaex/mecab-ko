//! Command-line interface for MeCab-Ko memory profiling.

#![allow(clippy::uninlined_format_args)]

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;
use mecab_ko_profiler::prelude::*;
use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "mecab-profile")]
#[command(about = "Memory profiling tool for MeCab-Ko components", long_about = None)]
#[command(version)]
struct Cli {
    /// Subcommand to execute
    #[command(subcommand)]
    command: Commands,

    /// Output format (json or text)
    #[arg(short, long, default_value = "text")]
    format: String,

    /// Output file (stdout if not specified)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Profile dictionary operations
    Dict {
        /// Dictionary path
        #[arg(short, long)]
        dict_path: Option<PathBuf>,

        /// Simulated entry count for testing
        #[arg(short, long)]
        entries: Option<usize>,
    },

    /// Profile tokenization operations
    Tokenize {
        /// Text to tokenize
        #[arg(short, long)]
        text: Option<String>,

        /// Text file to read
        #[arg(short, long)]
        file: Option<PathBuf>,

        /// Analyze scaling with multiple sizes
        #[arg(short, long)]
        scaling: bool,
    },

    /// Profile Trie data structure
    Trie {
        /// Number of entries to insert
        #[arg(short, long, default_value = "10000")]
        entries: usize,

        /// Trie type (fst or double-array)
        #[arg(short = 't', long, default_value = "fst")]
        trie_type: String,
    },

    /// Generate report from profiling data
    Report {
        /// Input JSON file with profiling data
        #[arg(short, long)]
        input: PathBuf,
    },

    /// Run benchmark and generate profile
    Benchmark {
        /// Benchmark to run
        #[arg(short, long)]
        name: String,

        /// Number of iterations
        #[arg(short, long, default_value = "100")]
        iterations: usize,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.verbose {
        eprintln!("{}", "MeCab-Ko Memory Profiler".bold().green());
        eprintln!("{}", "========================".green());
        eprintln!();
    }

    let stats = match cli.command {
        Commands::Dict { dict_path, entries } => profile_dict(dict_path, entries, cli.verbose)?,
        Commands::Tokenize {
            text,
            file,
            scaling,
        } => profile_tokenize(text, file, scaling, cli.verbose)?,
        Commands::Trie { entries, trie_type } => profile_trie(entries, &trie_type, cli.verbose)?,
        Commands::Report { input } => {
            return generate_report_from_file(input, &cli.format, cli.output);
        }
        Commands::Benchmark { name, iterations } => {
            profile_benchmark(&name, iterations, cli.verbose)?
        }
    };

    // Generate and output report
    let report = ProfilingReport::new(stats);
    write_report(&report, &cli.format, cli.output)?;

    if cli.verbose {
        eprintln!();
        eprintln!("{}", "Profiling completed successfully!".bold().green());
    }

    Ok(())
}

fn profile_dict(
    _dict_path: Option<PathBuf>,
    entries: Option<usize>,
    verbose: bool,
) -> Result<DetailedStats> {
    if verbose {
        eprintln!("{}", "Profiling dictionary operations...".cyan());
    }

    let mut profiler = DictProfiler::new();

    // Simulate dictionary loading
    let entry_count = entries.unwrap_or(10000);

    if verbose {
        eprintln!("  Loading lexicon with {} entries...", entry_count);
    }

    profiler.profile_lexicon(entry_count, || {
        // Simulate lexicon data
        let _data: Vec<Vec<u8>> = (0..entry_count)
            .map(|i| format!("word_{i}").into_bytes())
            .collect();
    });

    if verbose {
        eprintln!("  Loading connection costs...");
    }

    profiler.profile_connection_costs(1000, || {
        // Simulate connection cost matrix
        let _matrix: Vec<i16> = vec![0; 1000 * 1000];
    });

    if verbose {
        eprintln!("  Loading features...");
    }

    profiler.profile_features(entry_count, || {
        // Simulate feature data
        let _features: Vec<String> = (0..entry_count)
            .map(|i| format!("NNG,*,*,*,*,*,word_{i}"))
            .collect();
    });

    Ok(profiler.finish())
}

fn profile_tokenize(
    text: Option<String>,
    file: Option<PathBuf>,
    scaling: bool,
    verbose: bool,
) -> Result<DetailedStats> {
    if verbose {
        eprintln!("{}", "Profiling tokenization operations...".cyan());
    }

    let mut profiler = TokenizerProfiler::new();

    let input_text = if let Some(path) = file {
        std::fs::read_to_string(path).context("Failed to read input file")?
    } else {
        text.unwrap_or_else(|| "한국어 형태소 분석 테스트".to_string())
    };

    if scaling {
        if verbose {
            eprintln!("  Analyzing scaling behavior...");
        }

        // Test with different text sizes
        let sizes = vec![10, 50, 100, 500, 1000];
        for size in &sizes {
            let test_text: String = input_text.chars().cycle().take(*size).collect();

            if verbose {
                eprintln!("    Testing with {} characters...", size);
            }

            profiler.profile_tokenize(&test_text, || {
                // Simulate tokenization
                let _tokens: Vec<String> = test_text
                    .split_whitespace()
                    .map(|s| s.to_string())
                    .collect();
            });
        }

        if verbose {
            let scaling_analysis = profiler.analyze_scaling(&sizes);
            let complexity = scaling_analysis.estimate_complexity();
            eprintln!("    Estimated complexity: O(n^{:.2})", complexity);
        }
    } else {
        if verbose {
            eprintln!("  Tokenizing {} characters...", input_text.len());
        }

        profiler.profile_tokenize(&input_text, || {
            // Simulate tokenization
            let _tokens: Vec<String> = input_text
                .split_whitespace()
                .map(|s| s.to_string())
                .collect();
        });
    }

    Ok(profiler.finish())
}

fn profile_trie(entries: usize, trie_type: &str, verbose: bool) -> Result<DetailedStats> {
    if verbose {
        eprintln!("{}", "Profiling Trie operations...".cyan());
        eprintln!("  Type: {}", trie_type);
        eprintln!("  Entries: {}", entries);
    }

    let stats = match trie_type {
        "fst" => {
            let mut profiler = mecab_ko_profiler::trie_profiler::FstProfiler::new();

            if verbose {
                eprintln!("  Building FST...");
            }

            profiler.profile_build(entries, || {
                // Simulate FST construction
                let _data: Vec<(String, u64)> = (0..entries)
                    .map(|i| (format!("key_{i:08}"), i as u64))
                    .collect();
            });

            profiler.finish()
        }
        "double-array" | "da" => {
            let mut profiler = mecab_ko_profiler::trie_profiler::DoubleArrayProfiler::new();

            if verbose {
                eprintln!("  Building Double-Array Trie...");
            }

            profiler.profile_build(entries, || {
                // Simulate double-array construction
                let _data: Vec<String> = (0..entries).map(|i| format!("key_{i:08}")).collect();
            });

            profiler.finish()
        }
        _ => anyhow::bail!("Unknown trie type: {}", trie_type),
    };

    Ok(stats)
}

fn profile_benchmark(name: &str, iterations: usize, verbose: bool) -> Result<DetailedStats> {
    if verbose {
        eprintln!("{}", "Running benchmark...".cyan());
        eprintln!("  Name: {}", name);
        eprintln!("  Iterations: {}", iterations);
    }

    let mut collector = StatsCollector::new();

    for i in 0..iterations {
        if verbose && i % 10 == 0 {
            eprintln!("  Iteration {}/{}", i, iterations);
        }

        let snapshot_before = snapshot();

        // Simulate some work
        let _data: Vec<u8> = vec![0; 1024];

        let snapshot_after = snapshot();
        let diff = snapshot_after.diff(&snapshot_before);

        collector.add_component(format!("iteration_{i}"), diff);
    }

    Ok(collector.finish())
}

fn generate_report_from_file(input: PathBuf, format: &str, output: Option<PathBuf>) -> Result<()> {
    let file = File::open(&input)
        .with_context(|| format!("Failed to open input file: {}", input.display()))?;

    let stats: DetailedStats =
        serde_json::from_reader(file).context("Failed to parse profiling data from JSON")?;

    let report = ProfilingReport::new(stats);
    write_report(&report, format, output)
}

fn write_report(report: &ProfilingReport, format: &str, output: Option<PathBuf>) -> Result<()> {
    let report_format = ReportFormat::parse(format)
        .ok_or_else(|| anyhow::anyhow!("Unknown format: {format}"))?;

    let content = match report_format {
        ReportFormat::Json => report.to_json()?,
        ReportFormat::Text => report.to_text(),
    };

    if let Some(path) = output {
        let mut file = File::create(&path)
            .with_context(|| format!("Failed to create file: {}", path.display()))?;
        file.write_all(content.as_bytes())
            .context("Failed to write to file")?;
    } else {
        io::stdout()
            .write_all(content.as_bytes())
            .context("Failed to write to stdout")?;
    }

    Ok(())
}
