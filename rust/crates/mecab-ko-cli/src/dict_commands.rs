//! # Dictionary Management CLI Commands
//!
//! 사전 관리 CLI 커맨드
//!
//! This module provides command-line interfaces for managing MeCab-Ko dictionaries,
//! including real-time dictionary updates, user dictionary management, and versioning.
//!
//! # Features
//!
//! - **Hot Reload**: Update dictionaries without restarting the tokenizer
//! - **User Dictionary Management**: Add, remove, and list custom entries
//! - **Import/Export**: Backup and restore user dictionaries
//! - **Versioning**: Track dictionary changes with automatic versioning
//! - **Rollback**: Restore previous dictionary states
//!
//! # Available Commands
//!
//! ## Dictionary Management
//!
//! - `mecab-ko dict reload` - Reload system dictionary
//! - `mecab-ko dict add <surface> <pos> [cost]` - Add entry to user dictionary
//! - `mecab-ko dict remove <surface>` - Remove entry from user dictionary
//! - `mecab-ko dict clear` - Clear all user dictionary entries
//!
//! ## Inspection
//!
//! - `mecab-ko dict list` - List user dictionary entries
//! - `mecab-ko dict info` - Show dictionary information
//! - `mecab-ko dict version` - Display version information
//!
//! ## Import/Export
//!
//! - `mecab-ko dict export <file>` - Export user dictionary to CSV
//! - `mecab-ko dict import <file>` - Import user dictionary from CSV
//!
//! ## Version Control
//!
//! - `mecab-ko dict version --history` - Show version history
//! - `mecab-ko dict rollback <version>` - Rollback to specific version
//!
//! # Examples
//!
//! ## Adding Custom Words
//!
//! ```bash
//! # Add a proper noun
//! mecab-ko dict add "카카오톡" NNP -1000
//!
//! # Add with reading
//! mecab-ko dict add "iPhone" NNP -1000 --reading "아이폰"
//! ```
//!
//! ## Managing Entries
//!
//! ```bash
//! # List all entries
//! mecab-ko dict list
//!
//! # Search for specific entries
//! mecab-ko dict list --pattern "카카오"
//!
//! # Remove an entry
//! mecab-ko dict remove "카카오톡"
//! ```
//!
//! ## Backup and Restore
//!
//! ```bash
//! # Export to file
//! mecab-ko dict export my-dictionary.csv
//!
//! # Import from file
//! mecab-ko dict import my-dictionary.csv
//! ```
//!
//! ## Version Management
//!
//! ```bash
//! # Check current version
//! mecab-ko dict version
//!
//! # View history
//! mecab-ko dict version --history
//!
//! # Rollback to previous version
//! mecab-ko dict rollback 5
//! ```
//!
//! # CSV Format
//!
//! User dictionary CSV files should follow this format:
//!
//! ```csv
//! surface,pos,cost,reading
//! 카카오톡,NNP,-1000,
//! iPhone,NNP,-1000,아이폰
//! ```
//!
//! Fields:
//! - `surface`: The surface form (required)
//! - `pos`: Part-of-speech tag (required)
//! - `cost`: Word cost, lower = higher priority (optional, default: -1000)
//! - `reading`: Pronunciation or reading (optional)
//!
//! # Part-of-Speech Tags
//!
//! Common Korean POS tags:
//! - `NNG`: General noun
//! - `NNP`: Proper noun
//! - `VV`: Verb
//! - `VA`: Adjective
//! - `MAG`: General adverb
//! - `SL`: Foreign language
//!
//! # Performance Considerations
//!
//! - Dictionary updates are atomic and thread-safe
//! - Version history is maintained automatically
//! - Rollback operations are fast and efficient
//! - User dictionaries are separate from system dictionaries
//!
//! # See Also
//!
//! - [`HotReloadDictionary`]: Core hot-reload functionality
//! - [`UserDictionary`]: User dictionary management

use anyhow::{Context, Result};
use clap::{Args as ClapArgs, Subcommand};
use mecab_ko_dict::{HotReloadDictionary, UserDictionary};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;

/// Dictionary management commands
///
/// Subcommands for managing system and user dictionaries, including
/// adding/removing entries, import/export, and version control.
#[derive(Subcommand, Debug)]
pub enum DictCommand {
    /// 시스템 사전 리로드
    Reload {
        /// 사전 경로
        #[arg(short, long)]
        dicdir: Option<PathBuf>,
    },

    /// 사용자 사전에 엔트리 추가
    Add {
        /// 표면형
        surface: String,
        /// 품사 (예: NNG, NNP, VV)
        pos: String,
        /// 비용 (낮을수록 우선, 기본값: -1000)
        #[arg(short, long, default_value = "-1000")]
        cost: i16,
        /// 읽기 (발음)
        #[arg(short, long)]
        reading: Option<String>,
        /// 사전 경로
        #[arg(short, long)]
        dicdir: Option<PathBuf>,
    },

    /// 사용자 사전에서 엔트리 제거
    Remove {
        /// 표면형
        surface: String,
        /// 사전 경로
        #[arg(short, long)]
        dicdir: Option<PathBuf>,
    },

    /// 사용자 사전 목록 표시
    List {
        /// 사전 경로
        #[arg(short, long)]
        dicdir: Option<PathBuf>,
        /// 검색 패턴
        #[arg(short, long)]
        pattern: Option<String>,
    },

    /// 사용자 사전 내보내기
    Export {
        /// 출력 파일 경로
        output: PathBuf,
        /// 사전 경로
        #[arg(short, long)]
        dicdir: Option<PathBuf>,
    },

    /// 사용자 사전 가져오기
    Import {
        /// 입력 파일 경로
        input: PathBuf,
        /// 사전 경로
        #[arg(short, long)]
        dicdir: Option<PathBuf>,
    },

    /// 사전 버전 정보 표시
    Version {
        /// 사전 경로
        #[arg(short, long)]
        dicdir: Option<PathBuf>,
        /// 히스토리 표시
        #[arg(long)]
        history: bool,
    },

    /// 특정 버전으로 롤백
    Rollback {
        /// 롤백할 버전 번호
        version: u64,
        /// 사전 경로
        #[arg(short, long)]
        dicdir: Option<PathBuf>,
    },

    /// 사용자 사전 초기화
    Clear {
        /// 사전 경로
        #[arg(short, long)]
        dicdir: Option<PathBuf>,
        /// 확인 없이 실행
        #[arg(short, long)]
        yes: bool,
    },

    /// 사전 정보 표시
    Info {
        /// 사전 경로
        #[arg(short, long)]
        dicdir: Option<PathBuf>,
    },
}

/// Executes a dictionary management command
///
/// Dispatches the specified dictionary command to the appropriate handler function.
///
/// # Arguments
///
/// * `cmd` - The dictionary command to execute
///
/// # Returns
///
/// Returns `Ok(())` on success, or an error if the operation fails.
///
/// # Errors
///
/// Returns an error if:
/// - Dictionary loading fails
/// - File I/O operations fail
/// - Invalid arguments are provided
/// - Version rollback fails
///
/// # Examples
///
/// ```no_run
/// use mecab_ko_cli::dict_commands::{DictCommand, execute_dict_command};
///
/// let cmd = DictCommand::Info { dicdir: None };
/// execute_dict_command(&cmd)?;
/// # Ok::<(), anyhow::Error>(())
/// ```
pub fn execute_dict_command(cmd: &DictCommand) -> Result<()> {
    match cmd {
        DictCommand::Reload { dicdir } => reload_dict(dicdir.as_ref()),
        DictCommand::Add {
            surface,
            pos,
            cost,
            reading,
            dicdir,
        } => add_entry(dicdir.as_ref(), surface, pos, *cost, reading.as_deref()),
        DictCommand::Remove { surface, dicdir } => remove_entry(dicdir.as_ref(), surface),
        DictCommand::List { dicdir, pattern } => list_entries(dicdir.as_ref(), pattern.as_deref()),
        DictCommand::Export { output, dicdir } => export_dict(dicdir.as_ref(), output),
        DictCommand::Import { input, dicdir } => import_dict(dicdir.as_ref(), input),
        DictCommand::Version { dicdir, history } => show_version(dicdir.as_ref(), *history),
        DictCommand::Rollback { version, dicdir } => rollback_version(dicdir.as_ref(), *version),
        DictCommand::Clear { dicdir, yes } => clear_dict(dicdir.as_ref(), *yes),
        DictCommand::Info { dicdir } => show_info(dicdir.as_ref()),
    }
}

/// Reloads the system dictionary
///
/// Forces a reload of the system dictionary files, picking up any changes
/// made to the dictionary files on disk.
///
/// # Arguments
///
/// * `dicdir` - Optional path to dictionary directory (uses default if None)
///
/// # Returns
///
/// Returns the new version number on success.
///
/// # Errors
///
/// Returns an error if:
/// - Dictionary directory is not found
/// - Dictionary files are corrupted
/// - Insufficient permissions to read dictionary files
///
/// # Examples
///
/// ```bash
/// # Reload default dictionary
/// mecab-ko dict reload
///
/// # Reload specific dictionary
/// mecab-ko dict reload --dicdir /path/to/dict
/// ```
fn reload_dict(dicdir: Option<&PathBuf>) -> Result<()> {
    println!("사전 리로드 중...");

    let dict = create_hot_reload_dict(dicdir)?;
    let version = dict
        .reload_system_dict()
        .context("Failed to reload system dictionary")?;

    println!("사전 리로드 완료 (버전: {version})");
    Ok(())
}

/// Adds an entry to the user dictionary
///
/// Adds a new word with its part-of-speech tag and optional attributes
/// to the user dictionary. The dictionary is updated atomically.
///
/// # Arguments
///
/// * `dicdir` - Optional path to dictionary directory
/// * `surface` - The surface form of the word
/// * `pos` - Part-of-speech tag (e.g., "NNG", "NNP", "VV")
/// * `cost` - Word cost (lower values = higher priority, typical range: -10000 to 10000)
/// * `reading` - Optional reading/pronunciation
///
/// # Returns
///
/// Returns the new dictionary version number on success.
///
/// # Errors
///
/// Returns an error if:
/// - Dictionary cannot be loaded
/// - Invalid POS tag is provided
/// - Entry already exists (duplicate entries are allowed but warned)
///
/// # Examples
///
/// ```bash
/// # Add proper noun
/// mecab-ko dict add "ChatGPT" NNP -1000
///
/// # Add with reading
/// mecab-ko dict add "API" SL -1000 --reading "에이피아이"
/// ```
fn add_entry(
    dicdir: Option<&PathBuf>,
    surface: &str,
    pos: &str,
    cost: i16,
    reading: Option<&str>,
) -> Result<()> {
    let dict = create_hot_reload_dict(dicdir)?;

    let version = dict
        .add_entry(surface, pos, cost, reading.map(String::from))
        .context("Failed to add entry")?;

    println!(
        "엔트리 추가 완료: '{}' (품사: {}, 비용: {}, 버전: {})",
        surface, pos, cost, version
    );

    // 조회 확인
    let entries = dict.lookup(surface).context("Failed to lookup entry")?;
    if !entries.is_empty() {
        println!("추가된 엔트리:");
        for entry in entries {
            println!("  - {}\t{}\t(비용: {})", entry.surface, entry.feature, entry.cost);
        }
    }

    Ok(())
}

/// Removes entries from the user dictionary
///
/// Removes all user dictionary entries matching the specified surface form.
/// System dictionary entries are never removed.
///
/// # Arguments
///
/// * `dicdir` - Optional path to dictionary directory
/// * `surface` - The surface form to remove
///
/// # Returns
///
/// Returns the new version number and count of removed entries.
///
/// # Errors
///
/// Returns an error if dictionary operations fail.
///
/// # Examples
///
/// ```bash
/// # Remove an entry
/// mecab-ko dict remove "ChatGPT"
/// ```
fn remove_entry(dicdir: Option<&PathBuf>, surface: &str) -> Result<()> {
    let dict = create_hot_reload_dict(dicdir)?;

    let (version, removed_count) = dict
        .remove_entry(surface)
        .context("Failed to remove entry")?;

    if removed_count > 0 {
        println!(
            "엔트리 제거 완료: '{}' ({}건 제거, 버전: {})",
            surface, removed_count, version
        );
    } else {
        println!("제거할 엔트리를 찾을 수 없습니다: '{}'", surface);
    }

    Ok(())
}

/// Lists all user dictionary entries
///
/// Displays all entries in the user dictionary in a formatted table.
/// Optionally filters entries by pattern matching.
///
/// # Arguments
///
/// * `dicdir` - Optional path to dictionary directory
/// * `pattern` - Optional search pattern to filter entries
///
/// # Errors
///
/// Returns an error if dictionary cannot be loaded or exported.
///
/// # Examples
///
/// ```bash
/// # List all entries
/// mecab-ko dict list
///
/// # Search for entries containing "카카오"
/// mecab-ko dict list --pattern "카카오"
/// ```
fn list_entries(dicdir: Option<&PathBuf>, pattern: Option<&str>) -> Result<()> {
    let dict = create_hot_reload_dict(dicdir)?;
    let user_dict = dict.export_user_dict().context("Failed to export user dictionary")?;

    if user_dict.is_empty() {
        println!("사용자 사전이 비어있습니다.");
        return Ok(());
    }

    println!("사용자 사전 엔트리 ({} 건):", user_dict.len());
    println!("{:<20} {:<10} {:<10} {}", "표면형", "품사", "비용", "읽기");
    println!("{}", "-".repeat(60));

    for entry in user_dict.entries() {
        // 패턴 필터링
        if let Some(pat) = pattern {
            if !entry.surface.contains(pat) {
                continue;
            }
        }

        println!(
            "{:<20} {:<10} {:<10} {}",
            entry.surface,
            entry.pos,
            entry.cost,
            entry.reading.as_deref().unwrap_or("-")
        );
    }

    Ok(())
}

/// Exports the user dictionary to a CSV file
///
/// Creates a CSV file containing all user dictionary entries.
/// The file can be imported later to restore the dictionary state.
///
/// # Arguments
///
/// * `dicdir` - Optional path to dictionary directory
/// * `output` - Path to the output CSV file
///
/// # Errors
///
/// Returns an error if:
/// - Dictionary cannot be exported
/// - Output file cannot be created
/// - Write operation fails
///
/// # CSV Format
///
/// ```csv
/// surface,pos,cost,reading
/// 카카오톡,NNP,-1000,
/// ```
///
/// # Examples
///
/// ```bash
/// # Export to file
/// mecab-ko dict export backup.csv
/// ```
fn export_dict(dicdir: Option<&PathBuf>, output: &PathBuf) -> Result<()> {
    let dict = create_hot_reload_dict(dicdir)?;
    let user_dict = dict.export_user_dict().context("Failed to export user dictionary")?;

    user_dict
        .save_to_csv(output)
        .context("Failed to save user dictionary")?;

    println!(
        "사용자 사전 내보내기 완료: {} ({} 엔트리)",
        output.display(),
        user_dict.len()
    );

    Ok(())
}

/// Imports a user dictionary from a CSV file
///
/// Replaces the current user dictionary with entries from the specified CSV file.
/// Existing entries are preserved unless they conflict with imported entries.
///
/// # Arguments
///
/// * `dicdir` - Optional path to dictionary directory
/// * `input` - Path to the input CSV file
///
/// # Errors
///
/// Returns an error if:
/// - Input file cannot be read
/// - CSV format is invalid
/// - Dictionary import fails
///
/// # CSV Format
///
/// The input file should follow this format:
/// ```csv
/// surface,pos,cost,reading
/// 카카오톡,NNP,-1000,
/// iPhone,NNP,-1000,아이폰
/// ```
///
/// # Examples
///
/// ```bash
/// # Import from file
/// mecab-ko dict import backup.csv
/// ```
fn import_dict(dicdir: Option<&PathBuf>, input: &PathBuf) -> Result<()> {
    let dict = create_hot_reload_dict(dicdir)?;

    let mut user_dict = UserDictionary::new();
    user_dict
        .load_from_csv(input)
        .context("Failed to load user dictionary")?;

    let version = dict
        .import_user_dict(user_dict.clone())
        .context("Failed to import user dictionary")?;

    println!(
        "사용자 사전 가져오기 완료: {} ({} 엔트리, 버전: {})",
        input.display(),
        user_dict.len(),
        version
    );

    Ok(())
}

/// Shows dictionary version information
///
/// Displays the current dictionary version and optionally shows
/// the complete version history with timestamps.
///
/// # Arguments
///
/// * `dicdir` - Optional path to dictionary directory
/// * `show_history` - If true, displays complete version history
///
/// # Errors
///
/// Returns an error if dictionary cannot be loaded or version history cannot be retrieved.
///
/// # Examples
///
/// ```bash
/// # Show current version
/// mecab-ko dict version
///
/// # Show version history
/// mecab-ko dict version --history
/// ```
fn show_version(dicdir: Option<&PathBuf>, show_history: bool) -> Result<()> {
    let dict = create_hot_reload_dict(dicdir)?;
    let current_version = dict.current_version();

    println!("현재 버전: {}", current_version);

    if show_history {
        println!("\n버전 히스토리:");
        let history = dict
            .version_history()
            .context("Failed to get version history")?;

        if history.is_empty() {
            println!("  (히스토리 없음)");
        } else {
            for info in history {
                let age = info
                    .age()
                    .map(|d| {
                        if d.as_secs() < 60 {
                            format!("{}초 전", d.as_secs())
                        } else if d.as_secs() < 3600 {
                            format!("{}분 전", d.as_secs() / 60)
                        } else {
                            format!("{}시간 전", d.as_secs() / 3600)
                        }
                    })
                    .unwrap_or_else(|| "알 수 없음".to_string());

                let current_marker = if info.version == current_version {
                    " (현재)"
                } else {
                    ""
                };

                println!(
                    "  버전 {}: {} 엔트리, {}{}",
                    info.version, info.user_entry_count, age, current_marker
                );
            }
        }
    }

    Ok(())
}

/// Rolls back to a previous dictionary version
///
/// Restores the dictionary state to a specific version from the history.
/// This operation is reversible - you can rollback the rollback.
///
/// # Arguments
///
/// * `dicdir` - Optional path to dictionary directory
/// * `target_version` - The version number to rollback to
///
/// # Errors
///
/// Returns an error if:
/// - Target version does not exist
/// - Rollback operation fails
/// - Dictionary state is corrupted
///
/// # Examples
///
/// ```bash
/// # View version history first
/// mecab-ko dict version --history
///
/// # Rollback to version 5
/// mecab-ko dict rollback 5
/// ```
fn rollback_version(dicdir: Option<&PathBuf>, target_version: u64) -> Result<()> {
    let dict = create_hot_reload_dict(dicdir)?;
    let current_version = dict.current_version();

    if target_version == current_version {
        println!("이미 버전 {target_version}입니다.");
        return Ok(());
    }

    println!("버전 {current_version} → {target_version} 롤백 중...");

    dict.rollback(target_version)
        .context("Failed to rollback version")?;

    println!("롤백 완료");

    Ok(())
}

/// Clears all user dictionary entries
///
/// Removes all entries from the user dictionary, resetting it to empty state.
/// System dictionary is not affected. Prompts for confirmation unless
/// `skip_confirm` is true.
///
/// # Arguments
///
/// * `dicdir` - Optional path to dictionary directory
/// * `skip_confirm` - If true, skips confirmation prompt
///
/// # Errors
///
/// Returns an error if:
/// - Dictionary cannot be cleared
/// - User cancels operation
/// - I/O operations fail
///
/// # Examples
///
/// ```bash
/// # Clear with confirmation
/// mecab-ko dict clear
///
/// # Clear without confirmation (use with caution)
/// mecab-ko dict clear --yes
/// ```
fn clear_dict(dicdir: Option<&PathBuf>, skip_confirm: bool) -> Result<()> {
    let dict = create_hot_reload_dict(dicdir)?;
    let user_dict = dict.export_user_dict().context("Failed to export user dictionary")?;

    if user_dict.is_empty() {
        println!("사용자 사전이 이미 비어있습니다.");
        return Ok(());
    }

    if !skip_confirm {
        print!(
            "사용자 사전을 초기화하시겠습니까? ({} 엔트리 삭제) [y/N]: ",
            user_dict.len()
        );
        std::io::stdout().flush()?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("취소되었습니다.");
            return Ok(());
        }
    }

    let version = dict
        .import_user_dict(UserDictionary::new())
        .context("Failed to clear user dictionary")?;

    println!("사용자 사전 초기화 완료 (버전: {version})");

    Ok(())
}

/// Displays comprehensive dictionary information
///
/// Shows detailed information about the dictionary including path,
/// version, entry counts, and history size.
///
/// # Arguments
///
/// * `dicdir` - Optional path to dictionary directory
///
/// # Errors
///
/// Returns an error if dictionary cannot be loaded or information cannot be retrieved.
///
/// # Examples
///
/// ```bash
/// # Show dictionary information
/// mecab-ko dict info
///
/// # Show info for specific dictionary
/// mecab-ko dict info --dicdir /path/to/dict
/// ```
fn show_info(dicdir: Option<&PathBuf>) -> Result<()> {
    let dict = create_hot_reload_dict(dicdir)?;

    println!("MeCab-Ko 사전 정보");
    println!("===================");
    println!("사전 경로: {}", dict.dicdir().display());
    println!("현재 버전: {}", dict.current_version());

    let user_dict = dict.export_user_dict().context("Failed to export user dictionary")?;
    println!("사용자 사전 엔트리: {} 건", user_dict.len());

    let history = dict
        .version_history()
        .context("Failed to get version history")?;
    println!("버전 히스토리: {} 건", history.len());

    Ok(())
}

/// Creates a hot-reload dictionary instance
///
/// Helper function to create a `HotReloadDictionary` with the specified
/// or default dictionary path.
///
/// # Arguments
///
/// * `dicdir` - Optional path to dictionary directory
///
/// # Returns
///
/// Returns an `Arc<HotReloadDictionary>` for thread-safe shared access.
///
/// # Errors
///
/// Returns an error if:
/// - Dictionary path is invalid
/// - Dictionary files cannot be loaded
/// - Initialization fails
fn create_hot_reload_dict(dicdir: Option<&PathBuf>) -> Result<Arc<HotReloadDictionary>> {
    let dict = if let Some(path) = dicdir {
        HotReloadDictionary::new(path).context("Failed to create hot reload dictionary")?
    } else {
        HotReloadDictionary::new_default().context("Failed to create hot reload dictionary")?
    };

    Ok(Arc::new(dict))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dict_command_variants() {
        // 각 커맨드 타입이 올바르게 정의되었는지 확인
        let reload = DictCommand::Reload { dicdir: None };
        assert!(matches!(reload, DictCommand::Reload { .. }));

        let add = DictCommand::Add {
            surface: "테스트".to_string(),
            pos: "NNG".to_string(),
            cost: -1000,
            reading: None,
            dicdir: None,
        };
        assert!(matches!(add, DictCommand::Add { .. }));

        let remove = DictCommand::Remove {
            surface: "테스트".to_string(),
            dicdir: None,
        };
        assert!(matches!(remove, DictCommand::Remove { .. }));
    }
}
