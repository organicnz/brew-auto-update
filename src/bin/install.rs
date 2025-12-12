use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("==========================================");
    println!("Homebrew Auto-Update Installer (Rust Version)");
    println!("==========================================");
    println!();

    // Get current user
    let user = env::var("USER").expect("USER not set");
    let home = env::var("HOME").expect("HOME not set");

    println!("Installing for user: {}", user);
    println!("Home directory: {}", home);
    println!();

    // Check Homebrew
    println!("Checking Homebrew...");
    if Command::new("which").arg("brew").status().is_err() {
        eprintln!("✗ Homebrew not found. Please install Homebrew first.");
        std::process::exit(1);
    }
    println!("✓ Homebrew found");

    // Build Project
    println!("Building brew-auto-update binary...");
    let status = Command::new("cargo")
        .args(["build", "--release", "--bin", "brew-auto-update"])
        .status()
        .expect("Failed to execute cargo");

    if !status.success() {
        eprintln!("✗ Build failed.");
        std::process::exit(1);
    }

    // Create Scripts dir
    let scripts_dir = PathBuf::from(&home).join("Scripts");
    if !scripts_dir.exists() {
        println!("Creating {}...", scripts_dir.display());
        fs::create_dir_all(&scripts_dir).expect("Failed to create Scripts dir");
    }

    // Install Binary
    let target_bin = PathBuf::from("target/release/brew-auto-update");
    let dest_bin = scripts_dir.join("brew-update");

    println!("Installing binary to {}...", dest_bin.display());
    fs::copy(&target_bin, &dest_bin).expect("Failed to copy binary");

    // Read Plist Template
    let template_content = fs::read_to_string("config/com.USER.brew-update.plist.template")
        .expect("Failed to read plist template");

    // Replace Placeholders
    // Default config
    let hour1 = env::var("BREW_UPDATE_HOUR1").unwrap_or("9".to_string());
    let minute1 = env::var("BREW_UPDATE_MINUTE1").unwrap_or("0".to_string());
    let hour2 = env::var("BREW_UPDATE_HOUR2").unwrap_or("15".to_string());
    let minute2 = env::var("BREW_UPDATE_MINUTE2").unwrap_or("0".to_string());
    let hour3 = env::var("BREW_UPDATE_HOUR3").unwrap_or("21".to_string());
    let minute3 = env::var("BREW_UPDATE_MINUTE3").unwrap_or("0".to_string());

    // Configs
    let nice = env::var("BREW_UPDATE_NICE_LEVEL").unwrap_or("10".to_string());
    let throttle = env::var("BREW_UPDATE_THROTTLE_INTERVAL").unwrap_or("300".to_string());
    let timeout = env::var("BREW_UPDATE_EXIT_TIMEOUT").unwrap_or("7200".to_string());
    let retention = env::var("BREW_UPDATE_LOG_RETENTION_DAYS").unwrap_or("1".to_string());
    let min_disk = env::var("BREW_UPDATE_MIN_DISK_SPACE_GB").unwrap_or("5".to_string());

    let plist_content = template_content
        .replace("{{USER}}", &user)
        .replace("{{HOME}}", &home)
        .replace("{{HOUR1}}", &hour1)
        .replace("{{MINUTE1}}", &minute1)
        .replace("{{HOUR2}}", &hour2)
        .replace("{{MINUTE2}}", &minute2)
        .replace("{{HOUR3}}", &hour3)
        .replace("{{MINUTE3}}", &minute3)
        .replace("{{NICE_LEVEL}}", &nice)
        .replace("{{THROTTLE_INTERVAL}}", &throttle)
        .replace("{{EXIT_TIMEOUT}}", &timeout)
        .replace("{{LOG_RETENTION_DAYS}}", &retention)
        .replace("{{MIN_DISK_SPACE_GB}}", &min_disk);

    // Write Plist
    let plist_name = format!("com.{}.brew-update.plist", user);
    let plist_path = PathBuf::from(&home)
        .join("Library/LaunchAgents")
        .join(&plist_name);

    println!("Writing plist to {}...", plist_path.display());
    fs::write(&plist_path, plist_content).expect("Failed to write plist");

    // Reload Launch Agent
    println!("Reloading launch agent...");
    let _ = Command::new("launchctl")
        .arg("unload")
        .arg(&plist_path)
        .output();
    let status = Command::new("launchctl")
        .arg("load")
        .arg(&plist_path)
        .status()
        .expect("Failed to load plist");

    if status.success() {
        println!("✓ Launchd agent loaded");
    } else {
        eprintln!("⚠ Warning: launchctl load returned non-zero exit code");
    }

    println!();
    println!("==========================================");
    println!("Installation Complete!");
    println!("==========================================");
}
