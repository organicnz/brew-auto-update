use regex::Regex;
use std::env;
use std::fs;
use std::process::Command;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: audit <check> <files...>");
        std::process::exit(1);
    }

    let check = &args[1];
    let files = &args[2..];
    let mut has_error = false;

    match check.as_str() {
        "plist" => {
            println!("🔍 Validating plist templates...");
            for file in files {
                if !check_plist(file) {
                    has_error = true;
                }
            }
        }
        "secrets" => {
            println!("🔍 Checking for secrets...");
            for file in files {
                if !check_secrets(file) {
                    has_error = true;
                }
            }
        }
        "markdown" => {
            println!("🔍 Checking markdown files...");
            for file in files {
                if !check_markdown(file) {
                    has_error = true;
                }
            }
        }
        _ => {
            eprintln!("Unknown check: {}", check);
            std::process::exit(1);
        }
    }

    if has_error {
        std::process::exit(1);
    }
}

fn check_plist(path: &str) -> bool {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return true, // maximize robustness, skip if not readable
    };

    println!("  Checking {}...", path);
    let mut valid = true;

    if !content.contains("{{USER}}") {
        eprintln!("❌ Missing {{USER}} placeholder in {}", path);
        valid = false;
    }
    if !content.contains("{{HOME}}") {
        eprintln!("❌ Missing {{HOME}} placeholder in {}", path);
        valid = false;
    }

    // Validate XML structure
    let temp_content = Regex::new(r"\{\{[^}]*\}\}")
        .unwrap()
        .replace_all(&content, "1");
    let temp_path = format!("/tmp/temp_plist_{}", std::process::id());
    fs::write(&temp_path, temp_content.as_bytes()).unwrap();

    let status = Command::new("plutil")
        .arg("-lint")
        .arg(&temp_path)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    let _ = fs::remove_file(&temp_path);

    if !status {
        eprintln!("❌ Invalid XML structure in {}", path);
        valid = false;
    }

    valid
}

fn check_secrets(path: &str) -> bool {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return true,
    };

    let patterns: &[(&str, &str)] = &[
        ("GitHub PAT", r"github_pat_[a-zA-Z0-9_]+"),
        ("GitHub Token", r"ghp_[a-zA-Z0-9]+"),
        ("AWS Key", r"AKIA[0-9A-Z]{16}"),
        ("Stripe SK", r"sk_live_[a-zA-Z0-9]+"),
        ("Stripe PK", r"pk_live_[a-zA-Z0-9]+"),
        (
            "Private Key",
            r"-----BEGIN (RSA|DSA|EC|OPENSSH) PRIVATE KEY-----",
        ),
        (
            "Password Assignment",
            r"password\s*=\s*['\x22][^'\x22]+['\x22]",
        ),
        (
            "API Key Assignment",
            r"api_key\s*=\s*['\x22][^'\x22]+['\x22]",
        ),
    ];

    let mut valid = true;
    for (name, pattern) in patterns.iter() {
        let re = Regex::new(pattern).unwrap();
        if re.is_match(&content) {
            eprintln!("❌ Potential secret found in {}: {}", path, name);
            valid = false;
        }
    }
    valid
}

fn check_markdown(path: &str) -> bool {
    // Check if markdownlint-cli is installed (npm)
    let status = Command::new("which").arg("markdownlint").status();
    if status.is_err() || !status.unwrap().success() {
        println!("⚠️  markdownlint not installed, skipping...");
        return true;
    }

    println!("  Checking {}...", path);
    Command::new("markdownlint")
        .arg(path)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}
