use crate::utils::{self, Config};
use std::process::Command;

pub fn check_brew(config: &Config) -> bool {
    if Command::new("which")
        .arg("brew")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        return true;
    }

    // Check common paths
    let paths = ["/opt/homebrew/bin/brew", "/usr/local/bin/brew"];
    for path in paths {
        if std::path::Path::new(path).exists() {
            return true;
        }
    }

    utils::log_error("Homebrew not found", config);
    false
}

pub fn update_homebrew(config: &Config) -> bool {
    utils::log("Updating Homebrew...", config);
    match Command::new("brew").arg("update").status() {
        Ok(status) if status.success() => {
            utils::log("✓ Homebrew updated successfully", config);
            true
        }
        _ => {
            utils::log_error("Failed to update Homebrew", config);
            false
        }
    }
}

pub fn upgrade_formulae(config: &Config) -> bool {
    utils::log("Checking for outdated formulae...", config);

    // Count outdated
    let outdated_count = Command::new("brew")
        .args(["outdated", "--formula"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).lines().count())
        .unwrap_or(0);

    if outdated_count == 0 {
        utils::log("✓ All formulae are up to date", config);
        return true;
    }

    utils::log(&format!("Upgrading {} formulae...", outdated_count), config);
    match Command::new("brew").args(["upgrade", "--formula"]).status() {
        Ok(status) if status.success() => {
            utils::log("✓ Formulae upgraded successfully", config);
            true
        }
        _ => {
            utils::log_error("Failed to upgrade formulae", config);
            false
        }
    }
}

pub fn upgrade_casks(config: &Config) -> bool {
    utils::log("Checking for outdated casks...", config);

    let outdated_count = Command::new("brew")
        .args(["outdated", "--cask", "--greedy"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).lines().count())
        .unwrap_or(0);

    if outdated_count == 0 {
        utils::log("✓ All casks are up to date", config);
        return true;
    }

    utils::log(&format!("Upgrading {} casks...", outdated_count), config);
    match Command::new("brew")
        .args(["upgrade", "--cask", "--greedy"])
        .status()
    {
        Ok(status) if status.success() => {
            utils::log("✓ Casks upgraded successfully", config);
            true
        }
        _ => {
            utils::log_error("Failed to upgrade casks", config);
            // Don't fail overall success for casks
            true
        }
    }
}

pub fn cleanup_brew(config: &Config) {
    utils::log("Running cleanup...", config);
    let _ = Command::new("brew")
        .args(["cleanup", "--prune=30", "-s"])
        .status();
    let _ = Command::new("brew").arg("autoremove").status();
    utils::log("✓ Cleanup completed", config);
}

pub fn update_npm(config: &Config) -> bool {
    if Command::new("command")
        .args(["-v", "npm"])
        .status()
        .is_err()
        && Command::new("which").arg("npm").status().is_err()
    {
        return true; // Skip if no npm
    }

    utils::log("Updating global npm packages...", config);
    let outdated = Command::new("npm")
        .args(["outdated", "-g", "--parseable", "--depth=0"])
        .output();

    if let Ok(out) = outdated {
        if out.stdout.is_empty() {
            utils::log("✓ All global npm packages are up to date", config);
            return true;
        }
    }

    match Command::new("npm").args(["update", "-g"]).status() {
        Ok(status) if status.success() => {
            utils::log("✓ Global npm packages updated successfully", config);
            true
        }
        _ => {
            utils::log_error("Failed to update global npm packages", config);
            false
        }
    }
}
