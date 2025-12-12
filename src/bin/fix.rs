use std::io::{self, Write};
use std::process::{Command, Stdio};

fn main() {
    println!("==========================================");
    println!("      Brew Auto-Update Fix Tool           ");
    println!("==========================================");
    println!();

    // 1. Clean up broken/missing casks
    println!("[1/4] Cleaning up broken/missing casks...");
    let casks_to_remove = [
        "airtable",
        "amazon-chime",
        "asana",
        "bitwarden",
        "blender",
        "brave-browser",
        "canva",
        "chatgpt",
        "cleanmymac",
        "clickup",
        "evernote",
        "firefox",
        "freecad",
        "geekbench",
        "google-chrome",
        "imazing",
        "losslesscut",
        "pgadmin4",
        "wondershare-uniconverter",
        "rustdesk",
        "xmind",
        "speedify",
        "youtube-to-mp3",
        "outline-manager",
        "ubiquiti-unifi-controller",
    ];

    for cask in casks_to_remove {
        // Check if installed
        let status = Command::new("brew")
            .args(["list", "--cask", cask])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();

        if let Ok(s) = status {
            if s.success() {
                print!("  - Uninstalling {}... ", cask);
                io::stdout().flush().unwrap();

                let uninstall_status = Command::new("brew")
                    .args(["uninstall", "--cask", cask])
                    .stdout(Stdio::null())
                    .stderr(Stdio::inherit())
                    .status();

                match uninstall_status {
                    Ok(us) if us.success() => println!("✓"),
                    _ => println!("✗ (Failed)"),
                }
            } else {
                println!("  - {} already removed.", cask);
            }
        }
    }

    println!();

    // 2. Fix Flutter
    println!("[2/4] Fixing Flutter (Invalid definition)...");
    let flutter_check = Command::new("brew")
        .args(["list", "--cask", "flutter"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    if let Ok(s) = flutter_check {
        if s.success() {
            print!("  - Uninstalling flutter... ");
            io::stdout().flush().unwrap();
            let status = Command::new("brew")
                .args(["uninstall", "--cask", "flutter"])
                .stdout(Stdio::null())
                .stderr(Stdio::inherit())
                .status();

            match status {
                Ok(us) if us.success() => println!("✓"),
                _ => println!("✗ (Failed)"),
            }
        } else {
            println!("  - Flutter already removed.");
        }
    }

    println!();

    // 3. Fix NPM
    println!("[3/4] Fixing Invalid NPM Package...");
    print!("  - Removing invalid package... ");
    io::stdout().flush().unwrap();
    let npm_status = Command::new("npm")
        .args(["uninstall", "-g", "@anthropic-ai/.claude-code-2DTsDk1V"])
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .status();

    match npm_status {
        Ok(s) if s.success() => println!("✓"),
        _ => println!("✗ (Failed or not present)"),
    }

    print!("  - Cleaning npm cache... ");
    io::stdout().flush().unwrap();
    let cache_status = Command::new("npm")
        .args(["cache", "clean", "--force"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    match cache_status {
        Ok(s) if s.success() => println!("✓"),
        _ => println!("✗"),
    }

    println!();

    // 4. Final Housekeeping
    println!("[4/4] Final Housekeeping...");

    print!("  - Running brew cleanup... ");
    io::stdout().flush().unwrap();
    let cleanup = Command::new("brew")
        .args(["cleanup", "-s"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    if cleanup.map(|s| s.success()).unwrap_or(false) {
        println!("✓");
    } else {
        println!("✗");
    }

    print!("  - Running brew autoremove... ");
    io::stdout().flush().unwrap();
    let autoremove = Command::new("brew")
        .arg("autoremove")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    if autoremove.map(|s| s.success()).unwrap_or(false) {
        println!("✓");
    } else {
        println!("✗");
    }

    println!("  - Checking brew doctor...");
    let doctor = Command::new("brew").arg("doctor").status();

    match doctor {
        Ok(s) if !s.success() => println!("  ⚠ brew doctor reported issues (see above)"),
        _ => {}
    }

    println!();
    println!("==========================================");
    println!("Fix Complete!");

    let df = Command::new("df").args(["-h", "/"]).output();
    if let Ok(out) = df {
        println!("Current Disk Space:");
        println!("{}", String::from_utf8_lossy(&out.stdout));
    }
    println!("==========================================");
}
