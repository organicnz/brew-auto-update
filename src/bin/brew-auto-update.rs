use brew_auto_update::utils::{self, log, log_error, send_notification, Config};
use brew_auto_update::{checks, ops};
use fs2::FileExt;
use std::fs::File;

fn main() {
    let config = Config::default();

    // Ensure log directory exists
    if let Some(parent) = config.log_file.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    log("Starting Homebrew Update", &config);

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
            log("Lock acquired", &config);
        }
        Err(_) => {
            // Check if lock is stale (implementation omitted for brevity in first pass, relying on try_lock)
            log("Another instance is running, exiting...", &config);
            std::process::exit(0);
        }
    }

    // Pre-flight checks
    if !checks::check_network(&config) {
        log_error("Aborting: No network connectivity", &config);
        send_notification("Skipped", "No network connection available");
        // Lock is released when file is closed (dropped)
        std::process::exit(1);
    }

    if !checks::check_disk_space(&config) {
        log_error("Aborting: Insufficient disk space", &config);
        send_notification("Skipped", "Insufficient disk space");
        std::process::exit(1);
    }

    if !ops::check_brew(&config) {
        std::process::exit(1);
    }

    let mut overall_success = true;

    // Update Operations
    if !ops::update_homebrew(&config) {
        overall_success = false;
    }
    if !ops::upgrade_formulae(&config) {
        overall_success = false;
    }
    if !ops::upgrade_casks(&config) {
        overall_success = false;
    }
    if !ops::update_npm(&config) {
        overall_success = false;
    }

    // Cleanup
    ops::cleanup_brew(&config);

    // Final Status
    if overall_success {
        utils::log("✓ All updates completed successfully", &config);
        send_notification("Success", "All packages updated successfully");
        std::process::exit(0);
    } else {
        utils::log("⚠ Updates completed with some errors", &config);
        send_notification("Warning", "Updates completed with some errors");
        std::process::exit(1);
    }
}
