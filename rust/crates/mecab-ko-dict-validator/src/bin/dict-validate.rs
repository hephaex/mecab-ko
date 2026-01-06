//! Dictionary validation CLI tool.

#![allow(clippy::write_with_newline)]
#![allow(clippy::format_push_string)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::expect_used)] // CLI tool can use expect for user-facing errors

use anyhow::{Context, Result};
use clap::{ArgAction, Parser};
use indicatif::{ProgressBar, ProgressStyle};
use mecab_ko_dict_validator::{
    config::{generate_default_config, load_config},
    DictValidator, ValidationConfig,
};
use std::fs;
use std::path::PathBuf;
use tracing::{error, info, warn};

#[derive(Parser)]
#[command(
    name = "dict-validate",
    about = "MeCab-Ko dictionary validation tool",
    version,
    author
)]
struct Args {
    /// Dictionary file(s) to validate
    #[arg(value_name = "FILE", required_unless_present = "generate_config")]
    files: Vec<PathBuf>,

    /// Configuration file (TOML format)
    #[arg(short, long, value_name = "CONFIG")]
    config: Option<PathBuf>,

    /// Generate default configuration file
    #[arg(long, value_name = "OUTPUT")]
    generate_config: Option<PathBuf>,

    /// Output format
    #[arg(short, long, value_enum, default_value = "text")]
    format: OutputFormat,

    /// Output file (default: stdout)
    #[arg(short, long, value_name = "FILE")]
    output: Option<PathBuf>,

    /// Fail on warnings
    #[arg(long, action = ArgAction::SetTrue)]
    strict: bool,

    /// Verbose output
    #[arg(short, long, action = ArgAction::Count)]
    verbose: u8,

    /// Quiet mode (errors only)
    #[arg(short, long, action = ArgAction::SetTrue)]
    quiet: bool,

    /// Show progress bar
    #[arg(long, action = ArgAction::SetTrue)]
    progress: bool,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
enum OutputFormat {
    /// Human-readable text format
    Text,
    /// JSON format
    Json,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    let log_level = if args.quiet {
        tracing::Level::ERROR
    } else {
        match args.verbose {
            0 => tracing::Level::WARN,
            1 => tracing::Level::INFO,
            2 => tracing::Level::DEBUG,
            _ => tracing::Level::TRACE,
        }
    };

    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_target(false)
        .init();

    // Handle config generation
    if let Some(config_path) = args.generate_config {
        return generate_config_file(&config_path);
    }

    // Load configuration
    let config = if let Some(config_path) = &args.config {
        info!("Loading configuration from {}", config_path.display());
        load_config(config_path).with_context(|| {
            format!(
                "Failed to load configuration from {}",
                config_path.display()
            )
        })?
    } else {
        info!("Using default configuration");
        ValidationConfig::default()
    };

    // Validate files
    let validator = DictValidator::new(config);
    let mut all_valid = true;
    let mut all_reports = Vec::new();

    for file in &args.files {
        if !file.exists() {
            error!("File not found: {}", file.display());
            all_valid = false;
            continue;
        }

        info!("Validating {}", file.display());

        let pb = if args.progress {
            let pb = ProgressBar::new_spinner();
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.green} [{elapsed_precise}] {msg}")
                    .expect("Failed to set progress style"),
            );
            pb.set_message(format!("Validating {}...", file.display()));
            Some(pb)
        } else {
            None
        };

        let report = validator
            .validate_file(file)
            .with_context(|| format!("Failed to validate {}", file.display()))?;

        if let Some(pb) = pb {
            pb.finish_with_message(format!("Completed {}", file.display()));
        }

        if !report.is_valid() {
            all_valid = false;
            error!(
                "{}: {} errors, {} warnings",
                file.display(),
                report.error_entries,
                report.warning_entries
            );
        } else if report.has_warnings() {
            warn!("{}: {} warnings", file.display(), report.warning_entries);
            if args.strict {
                all_valid = false;
            }
        } else {
            info!("{}: OK", file.display());
        }

        all_reports.push(report);
    }

    // Generate output
    let output_content = match args.format {
        OutputFormat::Text => {
            if all_reports.len() == 1 {
                all_reports[0].to_text()
            } else {
                generate_combined_text_report(&all_reports)
            }
        }
        OutputFormat::Json => {
            if all_reports.len() == 1 {
                all_reports[0]
                    .to_json()
                    .context("Failed to serialize report to JSON")?
            } else {
                serde_json::to_string_pretty(&all_reports)
                    .context("Failed to serialize reports to JSON")?
            }
        }
    };

    // Write output
    if let Some(output_path) = args.output {
        fs::write(&output_path, output_content)
            .with_context(|| format!("Failed to write output to {}", output_path.display()))?;
        info!("Report written to {}", output_path.display());
    } else {
        println!("{output_content}");
    }

    // Exit with appropriate code
    if !all_valid {
        std::process::exit(1);
    }

    Ok(())
}

fn generate_config_file(path: &PathBuf) -> Result<()> {
    if path.exists() {
        anyhow::bail!("Configuration file already exists: {}", path.display());
    }

    let config_content = generate_default_config();
    fs::write(path, config_content)
        .with_context(|| format!("Failed to write configuration to {}", path.display()))?;

    println!("Generated default configuration at {}", path.display());
    println!("You can now customize the validation rules by editing this file.");

    Ok(())
}

fn generate_combined_text_report(reports: &[mecab_ko_dict_validator::ValidationReport]) -> String {
    let mut output = String::new();

    output.push_str("═══════════════════════════════════════════════════════════\n");
    output.push_str("  MeCab-Ko Dictionary Validation - Combined Report\n");
    output.push_str("═══════════════════════════════════════════════════════════\n\n");

    let total_files = reports.len();
    let total_entries: usize = reports.iter().map(|r| r.total_entries).sum();
    let total_errors: usize = reports.iter().map(|r| r.error_entries).sum();
    let total_warnings: usize = reports.iter().map(|r| r.warning_entries).sum();
    let valid_files = reports.iter().filter(|r| r.is_valid()).count();

    output.push_str("Overall Summary:\n");
    output.push_str("───────────────────────────────────────────────────────────\n");
    output.push_str(&format!("  Total files:     {total_files}\n"));
    output.push_str(&format!("  Valid files:     {valid_files}\n"));
    output.push_str(&format!("  Total entries:   {total_entries}\n"));
    output.push_str(&format!("  Total errors:    {total_errors}\n"));
    output.push_str(&format!("  Total warnings:  {total_warnings}\n\n"));

    output.push_str("File Details:\n");
    output.push_str("───────────────────────────────────────────────────────────\n");

    for report in reports {
        let status = if report.is_valid() {
            if report.has_warnings() {
                "PASSED (with warnings)"
            } else {
                "PASSED"
            }
        } else {
            "FAILED"
        };

        output.push_str(&format!(
            "  {} - {status}\n    Entries: {}, Errors: {}, Warnings: {}\n\n",
            report.file_path.display(),
            report.total_entries,
            report.error_entries,
            report.warning_entries
        ));
    }

    output.push_str("═══════════════════════════════════════════════════════════\n");
    output.push_str("\nUse --output <file> with individual files for detailed reports.\n");

    output
}
