//! Command-line interface for MeCab-Ko memory profiling.
#![allow(
    clippy::needless_pass_by_value,
    clippy::unnecessary_wraps,
    clippy::redundant_closure_for_method_calls,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::too_many_lines
)]

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;
use mecab_ko_dict::Matrix;
use mecab_ko_profiler::prelude::*;
use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;
use std::time::Instant;

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
        /// Dictionary path (uses default dict discovery if not specified)
        #[arg(short, long)]
        dict_path: Option<PathBuf>,

        /// Use simulated data instead of real dictionary
        #[arg(long)]
        simulate: bool,

        /// Simulated entry count (only with --simulate)
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

        /// Dictionary path (uses default dict discovery if not specified)
        #[arg(short, long)]
        dict_path: Option<PathBuf>,

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
        /// Benchmark to run (dict, tokenize, trie, all)
        #[arg(short, long)]
        name: String,

        /// Number of iterations
        #[arg(short, long, default_value = "100")]
        iterations: usize,

        /// Dictionary path for real-data benchmarks
        #[arg(short, long)]
        dict_path: Option<PathBuf>,

        /// Baseline JSON file to compare against
        #[arg(long)]
        baseline: Option<PathBuf>,

        /// Save current results as a JSON baseline to this file
        #[arg(long)]
        save_baseline: Option<PathBuf>,
    },

    /// Manage profiling baselines
    Baseline {
        #[command(subcommand)]
        action: BaselineAction,
    },
}

/// Sub-commands for the `baseline` command.
#[derive(Subcommand)]
enum BaselineAction {
    /// Run profiling and save results as a baseline file
    Save {
        /// Output path for the baseline JSON file
        #[arg(short, long, default_value = "baseline.json")]
        output: PathBuf,

        /// Benchmark to run (dict, tokenize, trie, all)
        #[arg(short, long, default_value = "all")]
        name: String,

        /// Number of iterations
        #[arg(short, long, default_value = "100")]
        iterations: usize,

        /// Dictionary path for real-data benchmarks
        #[arg(short, long)]
        dict_path: Option<PathBuf>,
    },

    /// Compare current profiling results against a saved baseline
    Compare {
        /// Path to the saved baseline JSON file
        #[arg(short, long, default_value = "baseline.json")]
        baseline: PathBuf,

        /// Benchmark to run for comparison (dict, tokenize, trie, all)
        #[arg(short, long, default_value = "all")]
        name: String,

        /// Number of iterations
        #[arg(short, long, default_value = "100")]
        iterations: usize,

        /// Dictionary path for real-data benchmarks
        #[arg(short, long)]
        dict_path: Option<PathBuf>,

        /// Regression threshold in percent (default: 10)
        #[arg(long, default_value = "10")]
        threshold: f64,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.verbose {
        eprintln!("{}", "MeCab-Ko Memory Profiler".bold().green());
        eprintln!("{}", "========================".green());
        eprintln!();
    }

    match cli.command {
        Commands::Dict {
            dict_path,
            simulate,
            entries,
        } => {
            let stats = if simulate {
                profile_dict_simulated(entries, cli.verbose)?
            } else {
                profile_dict_real(dict_path, cli.verbose)?
            };
            let report = ProfilingReport::new(stats);
            write_report(&report, &cli.format, cli.output)?;
        }
        Commands::Tokenize {
            text,
            file,
            dict_path,
            scaling,
        } => {
            let stats = profile_tokenize(text, file, dict_path, scaling, cli.verbose)?;
            let report = ProfilingReport::new(stats);
            write_report(&report, &cli.format, cli.output)?;
        }
        Commands::Trie { entries, trie_type } => {
            let stats = profile_trie(entries, &trie_type, cli.verbose)?;
            let report = ProfilingReport::new(stats);
            write_report(&report, &cli.format, cli.output)?;
        }
        Commands::Report { input } => {
            return generate_report_from_file(input, &cli.format, cli.output);
        }
        Commands::Benchmark {
            name,
            iterations,
            dict_path,
            baseline,
            save_baseline,
        } => {
            let (stats, had_regression) =
                profile_benchmark(&name, iterations, dict_path, baseline, cli.verbose)?;

            if let Some(path) = save_baseline {
                save_baseline_file(&stats, &path, cli.verbose)?;
            }

            let report = ProfilingReport::new(stats);
            write_report(&report, &cli.format, cli.output)?;

            if had_regression {
                std::process::exit(1);
            }
        }
        Commands::Baseline { action } => {
            run_baseline_action(action, &cli.format, cli.output, cli.verbose)?;
        }
    }

    if cli.verbose {
        eprintln!();
        eprintln!("{}", "Profiling completed successfully!".bold().green());
    }

    Ok(())
}

/// Profile real dictionary loading
fn profile_dict_real(dict_path: Option<PathBuf>, verbose: bool) -> Result<DetailedStats> {
    if verbose {
        eprintln!("{}", "Profiling real dictionary operations...".cyan());
    }

    let mut profiler = DictProfiler::new();

    // Load the dictionary
    let start = Instant::now();

    let dict = if let Some(path) = &dict_path {
        if verbose {
            eprintln!("  Loading dictionary from: {}", path.display());
        }
        profiler.profile_load("system_dictionary", || {
            mecab_ko_dict::SystemDictionary::load(path)
        })
    } else {
        if verbose {
            eprintln!("  Loading default dictionary...");
        }
        profiler.profile_load("system_dictionary", || {
            mecab_ko_dict::SystemDictionary::load_default()
        })
    };

    let load_time = start.elapsed();

    match dict {
        Ok(dict) => {
            if verbose {
                eprintln!("  Dictionary loaded in {load_time:?}");
                eprintln!("  Entries: {}", dict.entry_count());
                eprintln!(
                    "  Matrix: {}x{} ({})",
                    dict.matrix().left_size(),
                    dict.matrix().right_size(),
                    humansize::format_size(
                        dict.matrix().left_size() * dict.matrix().right_size() * 2,
                        humansize::BINARY
                    ),
                );
                eprintln!("  Dictionary dir: {}", dict.dicdir().display());
            }

            // Profile a trie lookup
            profiler.profile_lookup("한국어", || {
                let _result = dict.common_prefix_search("한국어");
            });
        }
        Err(e) => {
            if verbose {
                eprintln!("  {} Dictionary load failed: {e}", "WARNING:".yellow());
                eprintln!("  Falling back to simulated data...");
            }
            return profile_dict_simulated(None, verbose);
        }
    }

    Ok(profiler.finish())
}

/// Profile with simulated dictionary data
fn profile_dict_simulated(entries: Option<usize>, verbose: bool) -> Result<DetailedStats> {
    if verbose {
        eprintln!(
            "{}",
            "Profiling dictionary operations (simulated)...".cyan()
        );
    }

    let mut profiler = DictProfiler::new();
    let entry_count = entries.unwrap_or(10000);

    if verbose {
        eprintln!("  Simulating lexicon with {entry_count} entries...");
    }

    profiler.profile_lexicon(entry_count, || {
        let _data: Vec<Vec<u8>> = (0..entry_count)
            .map(|i| format!("word_{i}").into_bytes())
            .collect();
    });

    if verbose {
        eprintln!("  Simulating connection costs...");
    }

    profiler.profile_connection_costs(1000, || {
        let _matrix: Vec<i16> = vec![0; 1000 * 1000];
    });

    if verbose {
        eprintln!("  Simulating features...");
    }

    profiler.profile_features(entry_count, || {
        let _features: Vec<String> = (0..entry_count)
            .map(|i| format!("NNG,*,*,*,*,*,word_{i}"))
            .collect();
    });

    Ok(profiler.finish())
}

fn profile_tokenize(
    text: Option<String>,
    file: Option<PathBuf>,
    dict_path: Option<PathBuf>,
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

    // Try to create a real tokenizer
    let mut tokenizer = create_tokenizer(dict_path.as_deref(), verbose);

    if scaling {
        if verbose {
            eprintln!("  Analyzing scaling behavior...");
        }

        let sizes = vec![10, 50, 100, 500, 1000];
        for size in &sizes {
            let test_text: String = input_text.chars().cycle().take(*size).collect();

            if verbose {
                eprintln!(
                    "    Testing with {} characters ({} bytes)...",
                    size,
                    test_text.len()
                );
            }

            let text_for_closure = test_text.clone();
            profiler.profile_tokenize(&test_text, || {
                if let Some(ref mut tok) = tokenizer {
                    let _tokens = tok.tokenize(&text_for_closure);
                } else {
                    // Simulated tokenization
                    let _tokens: Vec<String> = text_for_closure
                        .split_whitespace()
                        .map(|s| s.to_string())
                        .collect();
                }
            });
        }

        if verbose {
            let byte_sizes: Vec<usize> = sizes
                .iter()
                .map(|s| {
                    input_text
                        .chars()
                        .cycle()
                        .take(*s)
                        .collect::<String>()
                        .len()
                })
                .collect();
            let scaling_analysis = profiler.analyze_scaling(&byte_sizes);
            let complexity = scaling_analysis.estimate_complexity();
            eprintln!("    Estimated complexity: O(n^{complexity:.2})");
        }
    } else {
        if verbose {
            eprintln!("  Tokenizing {} characters...", input_text.len());
        }

        let text_for_closure = input_text.clone();
        profiler.profile_tokenize(&input_text, || {
            if let Some(ref mut tok) = tokenizer {
                let tokens = tok.tokenize(&text_for_closure);
                if verbose {
                    eprintln!("    Produced {} tokens", tokens.len());
                }
            } else {
                let _tokens: Vec<String> = text_for_closure
                    .split_whitespace()
                    .map(|s| s.to_string())
                    .collect();
            }
        });
    }

    Ok(profiler.finish())
}

/// Try to create a real tokenizer, falling back to None
fn create_tokenizer(
    dict_path: Option<&std::path::Path>,
    verbose: bool,
) -> Option<mecab_ko_core::Tokenizer> {
    let result = if let Some(path) = dict_path {
        mecab_ko_core::Tokenizer::with_dict(path)
    } else {
        mecab_ko_core::Tokenizer::new()
    };

    match result {
        Ok(tok) => {
            if verbose {
                eprintln!("  Using real tokenizer with dictionary");
            }
            Some(tok)
        }
        Err(e) => {
            if verbose {
                eprintln!(
                    "  {} Could not initialize tokenizer: {e}",
                    "WARNING:".yellow()
                );
                eprintln!("  Using simulated tokenization");
            }
            None
        }
    }
}

fn profile_trie(entries: usize, trie_type: &str, verbose: bool) -> Result<DetailedStats> {
    if verbose {
        eprintln!("{}", "Profiling Trie operations...".cyan());
        eprintln!("  Type: {trie_type}");
        eprintln!("  Entries: {entries}");
    }

    let stats = match trie_type {
        "fst" => {
            let mut profiler = mecab_ko_profiler::trie_profiler::FstProfiler::new();

            if verbose {
                eprintln!("  Building FST...");
            }

            profiler.profile_build(entries, || {
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
                let _data: Vec<String> = (0..entries).map(|i| format!("key_{i:08}")).collect();
            });

            profiler.finish()
        }
        _ => anyhow::bail!("Unknown trie type: {trie_type}"),
    };

    Ok(stats)
}

/// Run a benchmark and return the stats along with a flag indicating whether
/// a regression was detected during baseline comparison.
///
/// Returns `(stats, had_regression)` where `had_regression` is `true` if any
/// metric exceeded the 10% regression threshold.
fn profile_benchmark(
    name: &str,
    iterations: usize,
    dict_path: Option<PathBuf>,
    baseline: Option<PathBuf>,
    verbose: bool,
) -> Result<(DetailedStats, bool)> {
    if verbose {
        eprintln!("{}", "Running benchmark...".cyan());
        eprintln!("  Name: {name}");
        eprintln!("  Iterations: {iterations}");
    }

    let mut collector = StatsCollector::new();
    let start = Instant::now();

    match name {
        "tokenize" | "all" => {
            let mut tokenizer = create_tokenizer(dict_path.as_deref(), verbose);
            let test_text = "한국어 형태소 분석기를 사용하여 자연어 처리를 수행합니다.";

            for i in 0..iterations {
                if verbose && i % 10 == 0 {
                    eprintln!("  Iteration {i}/{iterations}");
                }

                let snapshot_before = snapshot();

                if let Some(ref mut tok) = tokenizer {
                    let _tokens = tok.tokenize(test_text);
                } else {
                    let _data: Vec<u8> = vec![0; 1024];
                }

                let snapshot_after = snapshot();
                let diff = snapshot_after.diff(&snapshot_before);
                collector.add_component(format!("iteration_{i}"), diff);
            }
        }
        "dict" => {
            for i in 0..iterations.min(5) {
                if verbose {
                    eprintln!("  Dict load iteration {i}");
                }

                let snapshot_before = snapshot();

                if let Some(path) = &dict_path {
                    let _dict = mecab_ko_dict::SystemDictionary::load(path);
                } else {
                    let _dict = mecab_ko_dict::SystemDictionary::load_default();
                }

                let snapshot_after = snapshot();
                let diff = snapshot_after.diff(&snapshot_before);
                collector.add_component(format!("dict_load_{i}"), diff);
            }
        }
        _ => {
            // Generic benchmark with simulated work
            for i in 0..iterations {
                if verbose && i % 10 == 0 {
                    eprintln!("  Iteration {i}/{iterations}");
                }

                let snapshot_before = snapshot();
                let _data: Vec<u8> = vec![0; 1024];
                let snapshot_after = snapshot();
                let diff = snapshot_after.diff(&snapshot_before);
                collector.add_component(format!("iteration_{i}"), diff);
            }
        }
    }

    let elapsed = start.elapsed();
    if verbose {
        eprintln!("  Completed {iterations} iterations in {elapsed:?}");
        eprintln!("  Average: {:?} per iteration", elapsed / iterations as u32);
    }

    let stats = collector.finish();

    // Baseline comparison
    let had_regression = if let Some(baseline_path) = baseline {
        compare_baseline(&baseline_path, &stats, 10.0, verbose)?
    } else {
        false
    };

    Ok((stats, had_regression))
}

/// A single metric comparison result.
#[derive(Debug)]
struct MetricResult {
    label: String,
    baseline_value: u64,
    current_value: u64,
    /// Positive means regression (current > baseline), negative means improvement.
    change_pct: f64,
}

impl MetricResult {
    fn new(label: impl Into<String>, baseline_value: u64, current_value: u64) -> Self {
        let change_pct = if baseline_value > 0 {
            (current_value as f64 / baseline_value as f64 - 1.0) * 100.0
        } else {
            0.0
        };
        Self {
            label: label.into(),
            baseline_value,
            current_value,
            change_pct,
        }
    }

    fn is_regression(&self, threshold_pct: f64) -> bool {
        self.change_pct > threshold_pct
    }

    fn is_improvement(&self, threshold_pct: f64) -> bool {
        self.change_pct < -threshold_pct
    }
}

/// Compare current `DetailedStats` against a saved baseline file.
///
/// Compares overall statistics and each component present in both snapshots.
/// Prints a formatted regression report to stderr.
///
/// Returns `true` if any metric regressed by more than `threshold_pct` percent.
fn compare_baseline(
    baseline_path: &std::path::Path,
    current: &DetailedStats,
    threshold_pct: f64,
    verbose: bool,
) -> Result<bool> {
    let file = File::open(baseline_path)
        .with_context(|| format!("Failed to open baseline: {}", baseline_path.display()))?;

    let baseline: DetailedStats =
        serde_json::from_reader(file).context("Failed to parse baseline JSON")?;

    let mut metrics: Vec<MetricResult> = Vec::new();

    // --- Overall metrics ---
    metrics.push(MetricResult::new(
        "overall.total_allocated",
        baseline.overall.total_allocated,
        current.overall.total_allocated,
    ));
    metrics.push(MetricResult::new(
        "overall.current_usage",
        baseline.overall.current_usage,
        current.overall.current_usage,
    ));
    metrics.push(MetricResult::new(
        "overall.peak_usage",
        baseline.overall.peak_usage,
        current.overall.peak_usage,
    ));

    // --- Per-component metrics ---
    // Only compare components that exist in both snapshots.
    let mut component_names: Vec<&str> = baseline
        .components
        .keys()
        .filter(|k| current.components.contains_key(k.as_str()))
        .map(String::as_str)
        .collect();
    component_names.sort_unstable();

    for name in &component_names {
        let bl = &baseline.components[*name];
        let cur = &current.components[*name];

        metrics.push(MetricResult::new(
            format!("{name}.total_allocated"),
            bl.total_allocated,
            cur.total_allocated,
        ));
        metrics.push(MetricResult::new(
            format!("{name}.current_usage"),
            bl.current_usage,
            cur.current_usage,
        ));
        metrics.push(MetricResult::new(
            format!("{name}.peak_usage"),
            bl.peak_usage,
            cur.peak_usage,
        ));
    }

    // --- Print report ---
    let had_regression = metrics.iter().any(|m| m.is_regression(threshold_pct));

    if verbose || had_regression {
        eprintln!();
        eprintln!("{}", "  Baseline Regression Report:".bold());
        eprintln!(
            "  {}",
            "─────────────────────────────────────────────────────".dimmed()
        );
        eprintln!(
            "  {:<40} {:>12} {:>12} {:>10}",
            "Metric", "Baseline", "Current", "Change"
        );
        eprintln!(
            "  {}",
            "─────────────────────────────────────────────────────".dimmed()
        );

        for m in &metrics {
            let baseline_str = humansize::format_size(m.baseline_value, humansize::BINARY);
            let current_str = humansize::format_size(m.current_value, humansize::BINARY);
            let change_str = format!("{:+.1}%", m.change_pct);

            if m.is_regression(threshold_pct) {
                eprintln!(
                    "  {:<40} {:>12} {:>12} {}",
                    m.label,
                    baseline_str,
                    current_str,
                    change_str.red().bold()
                );
            } else if m.is_improvement(threshold_pct) {
                eprintln!(
                    "  {:<40} {:>12} {:>12} {}",
                    m.label,
                    baseline_str,
                    current_str,
                    change_str.green()
                );
            } else {
                eprintln!(
                    "  {:<40} {:>12} {:>12} {change_str}",
                    m.label, baseline_str, current_str,
                );
            }
        }

        eprintln!(
            "  {}",
            "─────────────────────────────────────────────────────".dimmed()
        );

        let regression_count = metrics
            .iter()
            .filter(|m| m.is_regression(threshold_pct))
            .count();
        let improvement_count = metrics
            .iter()
            .filter(|m| m.is_improvement(threshold_pct))
            .count();

        if had_regression {
            eprintln!(
                "  {} {regression_count} regression(s) detected (threshold: {threshold_pct:.0}%)",
                "REGRESSION:".red().bold()
            );
        } else {
            eprintln!(
                "  {} No regressions detected (threshold: {threshold_pct:.0}%)",
                "OK:".green().bold()
            );
        }

        if improvement_count > 0 {
            eprintln!(
                "  {} {improvement_count} improvement(s) detected",
                "IMPROVED:".green()
            );
        }

        eprintln!();
    }

    Ok(had_regression)
}

/// Save `DetailedStats` to a JSON baseline file.
fn save_baseline_file(stats: &DetailedStats, path: &std::path::Path, verbose: bool) -> Result<()> {
    let json = serde_json::to_string_pretty(stats).context("Failed to serialize baseline")?;
    std::fs::write(path, json)
        .with_context(|| format!("Failed to write baseline to {}", path.display()))?;
    if verbose {
        eprintln!(
            "  {} Baseline saved to {}",
            "SAVED:".green().bold(),
            path.display()
        );
    }
    Ok(())
}

/// Execute a `Baseline` subcommand action.
fn run_baseline_action(
    action: BaselineAction,
    format: &str,
    output: Option<PathBuf>,
    verbose: bool,
) -> Result<()> {
    match action {
        BaselineAction::Save {
            output: baseline_path,
            name,
            iterations,
            dict_path,
        } => {
            if verbose {
                eprintln!("{}", "Running profiling for baseline...".cyan());
            }

            let (stats, _) = profile_benchmark(&name, iterations, dict_path, None, verbose)?;

            save_baseline_file(&stats, &baseline_path, verbose)?;

            // Also write the full report so the user can inspect it.
            let report = ProfilingReport::new(stats);
            write_report(&report, format, output)?;
        }

        BaselineAction::Compare {
            baseline: baseline_path,
            name,
            iterations,
            dict_path,
            threshold,
        } => {
            if verbose {
                eprintln!("{}", "Running profiling for comparison...".cyan());
            }

            let (stats, _) = profile_benchmark(&name, iterations, dict_path, None, verbose)?;

            let had_regression = compare_baseline(&baseline_path, &stats, threshold, true)?;

            let report = ProfilingReport::new(stats);
            write_report(&report, format, output)?;

            if had_regression {
                std::process::exit(1);
            }
        }
    }

    Ok(())
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
    let report_format =
        ReportFormat::parse(format).ok_or_else(|| anyhow::anyhow!("Unknown format: {format}"))?;

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
