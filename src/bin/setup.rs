use std::process::Command;

fn main() {
    println!("🔧 Setting up development environment...");

    // Check Homebrew
    if Command::new("which").arg("brew").status().is_err() {
        eprintln!("❌ Homebrew not found. Please install: https://brew.sh");
        std::process::exit(1);
    }

    // Install lefthook
    if Command::new("which").arg("lefthook").status().is_err() {
        println!("📦 Installing lefthook...");
        let status = Command::new("brew")
            .arg("install")
            .arg("lefthook")
            .status()
            .unwrap();
        if !status.success() {
            eprintln!("Failed to install lefthook");
        }
    } else {
        println!("✅ lefthook already installed");
    }

    println!();
    println!("📦 Installing optional development tools...");

    // Check Cargo
    if Command::new("which").arg("cargo").status().is_err() {
        println!("  ⚠️  Rust/Cargo not found. Check https://rust-lang.org");
    } else {
        println!("  ✅ Cargo found. Installing clippy/fmt...");
        let _ = Command::new("rustup")
            .args(["component", "add", "clippy", "rustfmt"])
            .status();
    }

    // Install hooks
    println!();
    println!("🪝 Installing git hooks...");
    let status = Command::new("lefthook").arg("install").status().unwrap();
    if !status.success() {
        eprintln!("Failed to install git hooks");
    }

    println!();
    println!("✅ Development environment ready!");
    println!();
    println!("Available commands:");
    println!("  lefthook run pre-commit  - Run pre-commit checks");
    println!("  lefthook run pre-push    - Run pre-push tests");
    println!("  scripts/lefthook/test.sh - Run full test suite");
}
