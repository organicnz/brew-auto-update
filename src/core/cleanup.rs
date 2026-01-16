//! Comprehensive cleanup module for development environments
//! Handles Homebrew, NPM, Cargo, and system caches

use super::utils::{self, Config};
use std::path::Path;
use std::process::Command;
use std::time::Duration;

/// Comprehensive cleanup statistics
#[derive(Debug, Default)]
pub struct CleanupStats {
    pub bytes_freed: u64,
    pub files_removed: usize,
    pub dirs_removed: usize,
    pub brew_cache_freed: u64,
    pub npm_cache_freed: u64,
    pub cargo_cache_freed: u64,
    pub system_cache_freed: u64,
    pub logs_cleaned: usize,
    pub locks_removed: usize,
    pub errors: Vec<String>,
}

impl CleanupStats {
    pub fn total_mb_freed(&self) -> u64 {
        self.bytes_freed / 1_000_000
    }
}

impl std::fmt::Display for CleanupStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "📊 Cleanup Summary:")?;
        writeln!(f, "   Total freed: {}MB", self.total_mb_freed())?;
        writeln!(f, "   Files removed: {}", self.files_removed)?;
        if self.brew_cache_freed > 0 {
            writeln!(f, "   Homebrew: {}MB", self.brew_cache_freed / 1_000_000)?;
        }
        if self.npm_cache_freed > 0 {
            writeln!(f, "   NPM: {}MB", self.npm_cache_freed / 1_000_000)?;
        }
        if self.cargo_cache_freed > 0 {
            writeln!(f, "   Cargo: {}MB", self.cargo_cache_freed / 1_000_000)?;
        }
        if self.system_cache_freed > 0 {
            writeln!(f, "   System: {}MB", self.system_cache_freed / 1_000_000)?;
        }
        if self.logs_cleaned > 0 {
            writeln!(f, "   Logs rotated: {}", self.logs_cleaned)?;
        }
        Ok(())
    }
}

/// Run comprehensive system cleanup
pub fn comprehensive_cleanup(config: &Config, aggressive: bool) -> CleanupStats {
    let mut stats = CleanupStats::default();
    let home = std::env::var("HOME").unwrap_or_default();

    if home.is_empty() {
        stats
            .errors
            .push("HOME environment variable not set".to_string());
        return stats;
    }

    utils::log("🧹 Running comprehensive cleanup...", config);

    cleanup_homebrew(config, &mut stats, aggressive);
    cleanup_npm(config, &mut stats, &home);
    cleanup_cargo(config, &mut stats, &home);
    cleanup_system_caches(config, &mut stats, &home, aggressive);
    cleanup_dev_caches(config, &mut stats, &home);
    cleanup_old_logs(config, &mut stats, &home);
    cleanup_temp_files(config, &mut stats);
    cleanup_stale_locks(config, &mut stats);

    stats.bytes_freed = stats.brew_cache_freed
        + stats.npm_cache_freed
        + stats.cargo_cache_freed
        + stats.system_cache_freed;

    utils::log(
        &format!(
            "✓ Cleanup complete: {}MB freed, {} files removed",
            stats.total_mb_freed(),
            stats.files_removed
        ),
        config,
    );

    stats
}

fn cleanup_homebrew(config: &Config, stats: &mut CleanupStats, aggressive: bool) {
    utils::log("  🍺 Cleaning Homebrew...", config);

    let prune_days = if aggressive { "3" } else { "14" };
    if let Ok(output) = Command::new("brew")
        .args(["cleanup", &format!("--prune={}", prune_days), "-s"])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Some(bytes) = parse_size_from_output(&stdout) {
                stats.brew_cache_freed += bytes;
            }
        }
    }

    if let Ok(output) = Command::new("brew").arg("autoremove").output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stats.files_removed += stdout
                .lines()
                .filter(|l| l.starts_with("==> Uninstalling"))
                .count();
        }
    }

    let home = std::env::var("HOME").unwrap_or_default();
    let cache_dirs = [
        format!("{}/Library/Caches/Homebrew", home),
        format!("{}/Library/Caches/Homebrew/downloads", home),
        format!("{}/Library/Caches/Homebrew/Cask", home),
    ];

    let max_age = if aggressive { 3 } else { 14 };
    for cache_dir in &cache_dirs {
        let freed = clean_old_files(cache_dir, max_age, stats);
        stats.brew_cache_freed += freed;
    }

    let log_dir = format!("{}/Library/Logs/Homebrew", home);
    let freed = clean_old_files(&log_dir, 7, stats);
    stats.brew_cache_freed += freed;
}

fn cleanup_npm(config: &Config, stats: &mut CleanupStats, home: &str) {
    utils::log("  📦 Cleaning NPM...", config);

    if !command_exists("npm") {
        return;
    }

    let _ = Command::new("npm")
        .args(["cache", "clean", "--force"])
        .output();

    let npm_caches = [
        format!("{}/.npm/_cacache", home),
        format!("{}/.npm/_logs", home),
        format!("{}/.npm/_npx", home),
    ];

    for cache in &npm_caches {
        let freed = clean_old_files(cache, 7, stats);
        stats.npm_cache_freed += freed;
    }

    let temp_node_modules = format!("{}/tmp/node_modules", home);
    if Path::new(&temp_node_modules).exists() {
        let freed = get_dir_size(&temp_node_modules);
        if std::fs::remove_dir_all(&temp_node_modules).is_ok() {
            stats.npm_cache_freed += freed;
            stats.dirs_removed += 1;
        }
    }
}

fn cleanup_cargo(config: &Config, stats: &mut CleanupStats, home: &str) {
    utils::log("  🦀 Cleaning Cargo...", config);

    if !command_exists("cargo") {
        return;
    }

    let registry_cache = format!("{}/.cargo/registry/cache", home);
    let freed = clean_old_files(&registry_cache, 30, stats);
    stats.cargo_cache_freed += freed;

    let registry_src = format!("{}/.cargo/registry/src", home);
    let freed = clean_old_files(&registry_src, 30, stats);
    stats.cargo_cache_freed += freed;

    let git_checkouts = format!("{}/.cargo/git/checkouts", home);
    let freed = clean_old_files(&git_checkouts, 30, stats);
    stats.cargo_cache_freed += freed;
}

fn cleanup_system_caches(config: &Config, stats: &mut CleanupStats, home: &str, aggressive: bool) {
    utils::log("  💻 Cleaning system caches...", config);

    let max_age = if aggressive { 3 } else { 14 };

    let cache_dirs = [
        format!("{}/Library/Caches/com.apple.dt.Xcode", home),
        format!("{}/Library/Caches/org.swift.swiftpm", home),
        format!("{}/Library/Caches/com.googlecode.iterm2", home),
        format!("{}/Library/Caches/com.spotify.client", home),
        format!("{}/Library/Caches/com.microsoft.VSCode", home),
        format!("{}/Library/Caches/Google", home),
        format!("{}/.cache", home),
    ];

    for cache_dir in &cache_dirs {
        if Path::new(cache_dir).exists() {
            let freed = clean_old_files(cache_dir, max_age, stats);
            stats.system_cache_freed += freed;
        }
    }

    let derived_data = format!("{}/Library/Developer/Xcode/DerivedData", home);
    if Path::new(&derived_data).exists() {
        let freed = clean_old_files(&derived_data, max_age, stats);
        stats.system_cache_freed += freed;
    }

    let device_support = format!("{}/Library/Developer/Xcode/iOS DeviceSupport", home);
    if Path::new(&device_support).exists() {
        let freed = clean_old_dirs(&device_support, 60, stats);
        stats.system_cache_freed += freed;
    }
}

fn cleanup_dev_caches(config: &Config, stats: &mut CleanupStats, home: &str) {
    utils::log("  🛠️  Cleaning dev tool caches...", config);

    let vscode_caches = [
        format!("{}/Library/Application Support/Code/Cache", home),
        format!("{}/Library/Application Support/Code/CachedData", home),
        format!("{}/Library/Application Support/Code/CachedExtensions", home),
        format!("{}/Library/Application Support/Code/logs", home),
    ];

    for cache in &vscode_caches {
        let freed = clean_old_files(cache, 14, stats);
        stats.system_cache_freed += freed;
    }

    let gradle_cache = format!("{}/.gradle/caches", home);
    let freed = clean_old_files(&gradle_cache, 30, stats);
    stats.system_cache_freed += freed;

    let cocoapods_cache = format!("{}/Library/Caches/CocoaPods", home);
    let freed = clean_old_files(&cocoapods_cache, 14, stats);
    stats.system_cache_freed += freed;

    if command_exists("yarn") {
        let _ = Command::new("yarn").args(["cache", "clean"]).output();
    }

    if command_exists("pnpm") {
        let _ = Command::new("pnpm").args(["store", "prune"]).output();
    }

    let bun_cache = format!("{}/.bun/install/cache", home);
    let freed = clean_old_files(&bun_cache, 14, stats);
    stats.system_cache_freed += freed;

    if command_exists("docker") {
        let _ = Command::new("docker")
            .args(["image", "prune", "-f"])
            .output();
    }
}

fn cleanup_old_logs(config: &Config, stats: &mut CleanupStats, home: &str) {
    utils::log("  📋 Cleaning old logs...", config);

    let log_dirs = [format!("{}/Library/Logs", home)];

    for log_dir in &log_dirs {
        if Path::new(log_dir).exists() {
            clean_old_log_files(log_dir, 7, stats);
        }
    }

    if let Some(log_parent) = config.log_file.parent() {
        clean_old_log_files(&log_parent.to_string_lossy(), 7, stats);
    }
}

fn cleanup_temp_files(config: &Config, stats: &mut CleanupStats) {
    utils::log("  🗑️  Cleaning temp files...", config);

    let temp_dirs = [
        "/tmp".to_string(),
        "/private/tmp".to_string(),
        std::env::var("TMPDIR").unwrap_or_default(),
    ];

    for temp_dir in &temp_dirs {
        if temp_dir.is_empty() || !Path::new(temp_dir).exists() {
            continue;
        }

        if let Ok(entries) = std::fs::read_dir(temp_dir) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        if let Ok(elapsed) = modified.elapsed() {
                            if elapsed.as_secs() > 24 * 60 * 60 {
                                let size = if metadata.is_file() {
                                    metadata.len()
                                } else if metadata.is_dir() {
                                    get_dir_size(&entry.path().to_string_lossy())
                                } else {
                                    0
                                };

                                let removed = if metadata.is_file() {
                                    std::fs::remove_file(entry.path()).is_ok()
                                } else if metadata.is_dir() {
                                    std::fs::remove_dir_all(entry.path()).is_ok()
                                } else {
                                    false
                                };

                                if removed {
                                    stats.files_removed += 1;
                                    stats.system_cache_freed += size;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn cleanup_stale_locks(config: &Config, stats: &mut CleanupStats) {
    utils::log("  🔒 Cleaning stale locks...", config);

    let lock_patterns = [("/tmp", "homebrew"), ("/tmp", "brew-"), ("/tmp", ".npm-")];

    for (dir, pattern) in &lock_patterns {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.contains(pattern) && (name.ends_with(".lock") || name.contains("lock")) {
                    if let Ok(metadata) = entry.metadata() {
                        if let Ok(modified) = metadata.modified() {
                            if let Ok(elapsed) = modified.elapsed() {
                                if elapsed.as_secs() > 2 * 60 * 60
                                    && std::fs::remove_file(entry.path()).is_ok()
                                {
                                    stats.locks_removed += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn command_exists(cmd: &str) -> bool {
    Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn parse_size_from_output(output: &str) -> Option<u64> {
    for line in output.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        for part in parts {
            if let Some(bytes) = parse_size_string(part) {
                return Some(bytes);
            }
        }
    }
    None
}

fn parse_size_string(s: &str) -> Option<u64> {
    let s = s.trim();
    if s.ends_with("GB") || s.ends_with("G") {
        let num_str = s.trim_end_matches(|c: char| c.is_alphabetic());
        num_str
            .parse::<f64>()
            .ok()
            .map(|n| (n * 1_000_000_000.0) as u64)
    } else if s.ends_with("MB") || s.ends_with("M") {
        let num_str = s.trim_end_matches(|c: char| c.is_alphabetic());
        num_str
            .parse::<f64>()
            .ok()
            .map(|n| (n * 1_000_000.0) as u64)
    } else if s.ends_with("KB") || s.ends_with("K") {
        let num_str = s.trim_end_matches(|c: char| c.is_alphabetic());
        num_str.parse::<f64>().ok().map(|n| (n * 1_000.0) as u64)
    } else {
        None
    }
}

fn get_dir_size(path: &str) -> u64 {
    let mut size: u64 = 0;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    size += metadata.len();
                } else if metadata.is_dir() {
                    size += get_dir_size(&entry.path().to_string_lossy());
                }
            }
        }
    }
    size
}

fn clean_old_files(dir: &str, max_age_days: u64, stats: &mut CleanupStats) -> u64 {
    let mut freed: u64 = 0;
    let max_age = Duration::from_secs(max_age_days * 24 * 60 * 60);

    if !Path::new(dir).exists() {
        return 0;
    }

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if let Ok(modified) = metadata.modified() {
                    if let Ok(elapsed) = modified.elapsed() {
                        if elapsed > max_age {
                            let size = if metadata.is_file() {
                                metadata.len()
                            } else if metadata.is_dir() {
                                get_dir_size(&entry.path().to_string_lossy())
                            } else {
                                0
                            };

                            let removed = if metadata.is_file() {
                                std::fs::remove_file(entry.path()).is_ok()
                            } else if metadata.is_dir() {
                                std::fs::remove_dir_all(entry.path()).is_ok()
                            } else {
                                false
                            };

                            if removed {
                                freed += size;
                                stats.files_removed += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    freed
}

fn clean_old_dirs(dir: &str, max_age_days: u64, stats: &mut CleanupStats) -> u64 {
    let mut freed: u64 = 0;
    let max_age = Duration::from_secs(max_age_days * 24 * 60 * 60);

    if !Path::new(dir).exists() {
        return 0;
    }

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_dir() {
                    if let Ok(modified) = metadata.modified() {
                        if let Ok(elapsed) = modified.elapsed() {
                            if elapsed > max_age {
                                let size = get_dir_size(&entry.path().to_string_lossy());
                                if std::fs::remove_dir_all(entry.path()).is_ok() {
                                    freed += size;
                                    stats.dirs_removed += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    freed
}

fn clean_old_log_files(dir: &str, max_age_days: u64, stats: &mut CleanupStats) {
    let max_age = Duration::from_secs(max_age_days * 24 * 60 * 60);

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".log") || name.ends_with(".log.old") || name.ends_with(".log.gz") {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        if let Ok(modified) = metadata.modified() {
                            if let Ok(elapsed) = modified.elapsed() {
                                if elapsed > max_age && std::fs::remove_file(entry.path()).is_ok() {
                                    stats.logs_cleaned += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
