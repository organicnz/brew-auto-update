# CLAUDE.md - Project Context for AI Assistants

## Project Overview

**brew-auto-update** is a production-grade automated Homebrew and NPM package management system for macOS. Written in Rust for safety and performance, it runs via launchd (3x daily) with intelligent pre-flight checks, comprehensive cleanup, and robust error handling.

## Architecture

```
src/
├── lib.rs                    # Library exports
├── bin/
│   ├── main.rs               # Main update binary (brew-auto-update)
│   ├── install.rs            # Automated installer
│   ├── setup.rs              # Development setup (lefthook, tools)
│   ├── audit.rs              # System audit tool
│   └── fix.rs                # Fix/repair utility
└── core/
    ├── mod.rs                # Module exports
    ├── ops.rs                # Core operations (update, upgrade, quarantine)
    ├── cleanup.rs            # Comprehensive cleanup system
    ├── stats.rs              # Update statistics tracking
    ├── checks.rs             # Pre-flight checks (network, disk)
    └── utils.rs              # Logging, notifications, config
```

## Key Features

### Robust Cask Handling
- **Timeout protection**: 30-minute limit per cask upgrade prevents hangs
- **App recovery**: Automatically reinstalls apps disrupted during failed upgrades
- **Quarantine removal**: Removes `com.apple.quarantine` from cask apps and formula binaries post-upgrade
- **Cask ignore list**: Skip problematic casks (Battle.net, VMware Fusion, etc.)
- **Categorized skip tracking**: Manual installers, running apps, auth required, timeouts

### Comprehensive Cleanup System
- **Homebrew**: Cache, downloads, old versions, unused dependencies
- **NPM**: Cache, logs, npx cache, temp node_modules
- **Cargo**: Registry cache, src, git checkouts
- **System caches**: Xcode, VS Code, CocoaPods, Gradle, Docker
- **Aggressive mode**: Triggered when disk space is low

### Pre-flight Checks
- Network connectivity verification
- Disk space validation (5GB minimum, aggressive cleanup if needed)
- Homebrew installation detection

### Safety Features
- Exclusive file lock prevents concurrent runs
- Graceful error handling with categorized reporting
- Desktop notifications on completion
- Separate error log file

## Build Commands

```bash
# Build all binaries
cargo build --release

# Run the main updater
cargo run --release --bin brew-auto-update

# Install the system (compiles + installs plist)
cargo run --release --bin install

# Setup development environment
cargo run --bin setup

# Run tests
cargo test

# Format code
cargo fmt

# Lint
cargo clippy
```

## Important Constants

Located in `src/core/ops.rs`:
- `CASK_UPGRADE_TIMEOUT_SECS`: 1800 (30 minutes per cask)
- `IGNORED_CASKS`: Apps to skip (manual installers, auth issues)

## Quarantine Removal

Two functions handle Gatekeeper quarantine removal:
- `remove_all_quarantine()`: Scans installed casks and removes quarantine from `/Applications/*.app`
- `remove_all_formula_quarantine()`: Scans `{homebrew_prefix}/bin` and removes quarantine from CLI binaries

## Code Patterns

### Logging
```rust
use brew_auto_update::utils::{log, log_error, Config};
let config = Config::default();
log("Message", &config);
log_error("Error message", &config);
```

### Running Commands with Timeout
```rust
use crate::ops::run_with_timeout;
match run_with_timeout(Command::new("brew").args(["upgrade", "--cask", name]), 1800) {
    Ok(output) if output.status.success() => { /* success */ }
    Err(e) if e.contains("Timeout") => { /* timed out */ }
    _ => { /* other error */ }
}
```

### Cleanup Stats
```rust
use brew_auto_update::cleanup::{comprehensive_cleanup, CleanupStats};
let stats: CleanupStats = comprehensive_cleanup(&config, aggressive);
println!("Freed: {}MB", stats.total_mb_freed());
```

## Configuration

Runtime config via environment:
- `HOME`: Required for log/cache paths
- Log files: `~/Library/Logs/brew-updates.log`, `brew-updates-error.log`
- Lock file: `/tmp/brew-update.lock`

Schedule (via launchd plist): 9 AM, 3 PM, 9 PM

## Testing

```bash
cargo test                    # Run all tests
cargo test --bin main         # Test main binary only
lefthook run pre-commit       # Run all pre-commit checks
```

## Dependencies

- `chrono`: Timestamp formatting
- `fs2`: File locking
- `users`: Username detection
- `regex`: Pattern matching
- `serde`/`serde_json`: JSON parsing (brew info)
- `signal-hook`: Signal handling
- `wait-timeout`: Command timeout support

## macOS-Specific Notes

- Uses launchd (not cron) for scheduling
- Quarantine removal requires `xattr -dr com.apple.quarantine`
- Desktop notifications via `osascript`
- Supports both Intel (`/usr/local/bin/brew`) and Apple Silicon (`/opt/homebrew/bin/brew`)
