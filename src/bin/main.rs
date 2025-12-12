use brew_auto_update::cleanup;
use brew_auto_update::utils::{self, log, log_error, send_notification, Config};
use brew_auto_update::{checks, ops, stats};
use fs2::FileExt;
use std::fs::File;

fn main() {
    let config = Config::default();

    // Ensure log directory exists
    if let Some(parent) = config.log_file.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    log("🚀 Starting Brew Auto Update", &config);
    log(
        &format!("   Version: {}", env!("CARGO_PKG_VERSION")),
        &config,
    );

    // Acquire lock
    let lock_file = match File::create(&config.lock_file) {
        Ok(f) => f,
        Err(e) => {
            log_error(&format!("Failed to create lock file: {}", e), &config);
            std::process::exit(1);
        }
    };

    match lock_file.try_lock_exclusive() {
        Ok(_) => {
            log("🔒 Lock acquired", &config);
        }
        Err(_) => {
            log("⚠ Another instance is running, exiting...", &config);
            std::process::exit(0);
        }
    }

    // Pre-flight checks
    log("\n📡 Running pre-flight checks...", &config);

    if !checks::check_network(&config) {
        log_error("Aborting: No network connectivity", &config);
        send_notification("Skipped", "No network connection available");
        std::process::exit(1);
    }

    // ========================================================================
    // PRE-TASK COMPREHENSIVE CLEANUP
    // ========================================================================
    log("\n", &config);
    let pre_cleanup_stats = cleanup::comprehensive_cleanup(&config, false);

    // Check disk space after cleanup
    if !checks::check_disk_space(&config) {
        // Try aggressive cleanup
        log("⚠ Low disk space, running aggressive cleanup...", &config);
        let aggressive_stats = cleanup::comprehensive_cleanup(&config, true);

        if !checks::check_disk_space(&config) {
            log_error(
                "Aborting: Insufficient disk space even after cleanup",
                &config,
            );
            send_notification("Skipped", "Insufficient disk space");
            std::process::exit(1);
        }

        log(
            &format!(
                "✓ Aggressive cleanup freed additional {}MB",
                aggressive_stats.total_mb_freed()
            ),
            &config,
        );
    }

    if !ops::check_brew(&config) {
        std::process::exit(1);
    }

    // Initialize stats tracking
    let mut update_stats = stats::UpdateStats::default();
    let mut overall_success = true;

    // ========================================================================
    // UPDATE OPERATIONS
    // ========================================================================
    log("\n📦 Running package updates...", &config);

    // Update Homebrew
    if !ops::update_homebrew(&config) {
        overall_success = false;
    }

    // Upgrade Formulae
    if !ops::upgrade_formulae(&config) {
        overall_success = false;
    }

    // Upgrade Casks (with detailed stats)
    update_stats.casks = ops::upgrade_casks(&config);

    // Update NPM (with invalid package detection)
    let (npm_success, invalid_npm_packages) = ops::update_npm(&config);
    if !npm_success {
        overall_success = false;
    }
    if !invalid_npm_packages.is_empty() {
        update_stats.npm.skipped = invalid_npm_packages.len();
    }

    // ========================================================================
    // POST-TASK COMPREHENSIVE CLEANUP
    // ========================================================================
    log("\n", &config);
    let post_cleanup_stats = cleanup::comprehensive_cleanup(&config, false);

    // ========================================================================
    // SUMMARY AND REPORTING
    // ========================================================================
    log("\n", &config);
    log(
        "═══════════════════════════════════════════════════════════",
        &config,
    );
    log(
        "                    UPDATE SUMMARY                         ",
        &config,
    );
    log(
        "═══════════════════════════════════════════════════════════",
        &config,
    );

    // Print update stats
    log(&format!("{}", update_stats), &config);

    // Print cleanup summary
    let total_freed_mb = pre_cleanup_stats.total_mb_freed() + post_cleanup_stats.total_mb_freed();
    let total_files = pre_cleanup_stats.files_removed + post_cleanup_stats.files_removed;

    log("\n📊 Cleanup Summary:", &config);
    log(
        &format!("   Total space freed: {}MB", total_freed_mb),
        &config,
    );
    log(&format!("   Files/dirs removed: {}", total_files), &config);

    if pre_cleanup_stats.brew_cache_freed + post_cleanup_stats.brew_cache_freed > 0 {
        log(
            &format!(
                "   Homebrew: {}MB",
                (pre_cleanup_stats.brew_cache_freed + post_cleanup_stats.brew_cache_freed)
                    / 1_000_000
            ),
            &config,
        );
    }
    if pre_cleanup_stats.npm_cache_freed + post_cleanup_stats.npm_cache_freed > 0 {
        log(
            &format!(
                "   NPM: {}MB",
                (pre_cleanup_stats.npm_cache_freed + post_cleanup_stats.npm_cache_freed)
                    / 1_000_000
            ),
            &config,
        );
    }
    if pre_cleanup_stats.cargo_cache_freed + post_cleanup_stats.cargo_cache_freed > 0 {
        log(
            &format!(
                "   Cargo: {}MB",
                (pre_cleanup_stats.cargo_cache_freed + post_cleanup_stats.cargo_cache_freed)
                    / 1_000_000
            ),
            &config,
        );
    }
    if pre_cleanup_stats.pip_cache_freed + post_cleanup_stats.pip_cache_freed > 0 {
        log(
            &format!(
                "   Pip: {}MB",
                (pre_cleanup_stats.pip_cache_freed + post_cleanup_stats.pip_cache_freed)
                    / 1_000_000
            ),
            &config,
        );
    }
    if pre_cleanup_stats.system_cache_freed + post_cleanup_stats.system_cache_freed > 0 {
        log(
            &format!(
                "   System caches: {}MB",
                (pre_cleanup_stats.system_cache_freed + post_cleanup_stats.system_cache_freed)
                    / 1_000_000
            ),
            &config,
        );
    }
    if pre_cleanup_stats.logs_cleaned + post_cleanup_stats.logs_cleaned > 0 {
        log(
            &format!(
                "   Logs rotated: {}",
                pre_cleanup_stats.logs_cleaned + post_cleanup_stats.logs_cleaned
            ),
            &config,
        );
    }

    // Report disk space
    if let Ok(output) = std::process::Command::new("df").args(["-h", "/"]).output() {
        let df_output = String::from_utf8_lossy(&output.stdout);
        for line in df_output.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                log(&format!("\n💾 Disk space available: {}", parts[3]), &config);
            }
        }
    }

    // NPM invalid packages
    if !invalid_npm_packages.is_empty() {
        log("\n⚠ NPM Invalid Packages (run to fix):", &config);
        for pkg in &invalid_npm_packages {
            log(&format!("   npm uninstall -g \"{}\"", pkg), &config);
        }
    }

    log(
        "\n═══════════════════════════════════════════════════════════",
        &config,
    );

    // Final Status and Notification
    if overall_success {
        if update_stats.casks.has_actionable_items() {
            utils::log(
                "✅ Updates completed with some items requiring attention",
                &config,
            );
            send_notification("Success", "Updates completed - some items need attention");
            std::process::exit(0);
        } else {
            utils::log("✅ All updates completed successfully!", &config);
            send_notification("Success", &format!("Updated! Freed {}MB", total_freed_mb));
            std::process::exit(0);
        }
    } else {
        utils::log("⚠️ Updates completed with some errors", &config);
        send_notification("Warning", "Updates completed with some errors");
        std::process::exit(1);
    }
}
