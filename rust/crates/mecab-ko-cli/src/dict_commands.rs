//! # 사전 관리 CLI 커맨드
//!
//! 실시간 사전 업데이트를 위한 CLI 인터페이스를 제공합니다.
//!
//! ## 커맨드
//!
//! - `mecab-ko dict reload` - 사전 리로드
//! - `mecab-ko dict add <surface> <pos> [cost]` - 엔트리 추가
//! - `mecab-ko dict remove <surface>` - 엔트리 제거
//! - `mecab-ko dict list` - 사용자 사전 목록
//! - `mecab-ko dict export <file>` - 사용자 사전 내보내기
//! - `mecab-ko dict import <file>` - 사용자 사전 가져오기
//! - `mecab-ko dict version` - 버전 정보
//! - `mecab-ko dict rollback <version>` - 버전 롤백

use anyhow::{Context, Result};
use clap::{Args as ClapArgs, Subcommand};
use mecab_ko_dict::{HotReloadDictionary, UserDictionary};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;

/// 사전 관리 커맨드
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

/// 사전 커맨드 실행
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

/// 사전 리로드
fn reload_dict(dicdir: Option<&PathBuf>) -> Result<()> {
    println!("사전 리로드 중...");

    let dict = create_hot_reload_dict(dicdir)?;
    let version = dict
        .reload_system_dict()
        .context("Failed to reload system dictionary")?;

    println!("사전 리로드 완료 (버전: {version})");
    Ok(())
}

/// 엔트리 추가
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

/// 엔트리 제거
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

/// 사용자 사전 목록 표시
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

/// 사용자 사전 내보내기
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

/// 사용자 사전 가져오기
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

/// 버전 정보 표시
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

/// 버전 롤백
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

/// 사용자 사전 초기화
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

/// 사전 정보 표시
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

/// 핫 리로드 사전 생성
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
