use crate::utils::{self, Config};
use std::process::Command;

pub fn check_network(config: &Config) -> bool {
    utils::log("Checking network connectivity...", config);

    let status = Command::new("ping")
        .args(["-c", "1", "-W", "2", "8.8.8.8"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if status {
        utils::log("✓ Network is available", config);
        true
    } else {
        utils::log_error("No network connectivity detected", config);
        false
    }
}

pub fn check_disk_space(config: &Config) -> bool {
    utils::log("Checking disk space...", config);

    let output = Command::new("df").args(["-g", "/"]).output();

    if let Ok(out) = output {
        let stdout = String::from_utf8_lossy(&out.stdout);
        // Parse the second line, 4th column (Available)
        for line in stdout.lines().skip(1) {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                if let Ok(available) = parts[3].parse::<i64>() {
                    let min_gb = std::env::var("MIN_DISK_SPACE_GB")
                        .unwrap_or("5".to_string())
                        .parse::<i64>()
                        .unwrap_or(5);

                    if available < min_gb {
                        utils::log_error(
                            &format!(
                                "Insufficient disk space: {}GB available (minimum {}GB required)",
                                available, min_gb
                            ),
                            config,
                        );
                        return false;
                    } else {
                        utils::log(
                            &format!("✓ Sufficient disk space: {}GB available", available),
                            config,
                        );
                        return true;
                    }
                }
            }
        }
    }

    utils::log_error("Failed to check disk space", config);
    false
}
