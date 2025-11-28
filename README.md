# Homebrew Auto-Update

Production-grade automated Homebrew package management for macOS. Runs 3x daily via launchd with intelligent pre-flight checks, differential logging, and graceful error handling.

## Features

✅ **Smart Pre-flight Checks**
- Network connectivity verification
- Disk space validation (minimum 5GB)
- Homebrew installation check

✅ **Comprehensive Updates**
- Update Homebrew itself
- Upgrade all formulae
- Upgrade all casks (with --greedy flag)
- Cleanup old versions (30-day retention)
- Autoremove unused dependencies

✅ **Robust Error Handling**
- Lock file prevents concurrent runs
- Automatic stale lock removal (>2 hours)
- Process cleanup on exit/interrupt
- Graceful failure handling

✅ **Intelligent Logging**
- Differential logging (only logs changes)
- Automatic log rotation at 10MB
- 24-hour log retention
- Timestamped entries
- Separate error log

✅ **System Integration**
- Runs 3x daily (9 AM, 3 PM, 9 PM)
- Low priority I/O and CPU
- Desktop notifications on completion
- Health checks and summaries

## Installation

### Quick Install (Fully Automated)

```bash
# Clone the repository
git clone https://github.com/organicnz/brew-auto-update.git
cd brew-auto-update

# Run the installer (no prompts, fully automatic)
./install.sh
```

The installer automatically:
- Detects your username
- Detects Homebrew installation path
- Creates necessary directories
- Installs and configures everything
- Runs an initial test

### Manual Install

1. **Copy the script:**
```bash
mkdir -p ~/Scripts
cp scripts/brew-daily-update.sh ~/Scripts/
chmod +x ~/Scripts/brew-daily-update.sh
```

2. **Install the launchd plist:**
```bash
# Replace USER with your username
sed "s/USER/$(whoami)/g" com.organic.brew-update.plist > ~/Library/LaunchAgents/com.$(whoami).brew-update.plist

# Load the agent
launchctl load ~/Library/LaunchAgents/com.$(whoami).brew-update.plist
```

3. **Verify installation:**
```bash
launchctl list | grep brew-update
```

## Configuration

### Installation-Time Variables

Customize installation by setting these before running `./install.sh`:

```bash
# Schedule (default: 9 AM, 3 PM, 9 PM)
export BREW_UPDATE_HOUR1=8
export BREW_UPDATE_MINUTE1=0
export BREW_UPDATE_HOUR2=14
export BREW_UPDATE_MINUTE2=30
export BREW_UPDATE_HOUR3=20
export BREW_UPDATE_MINUTE3=0

# System settings
export BREW_UPDATE_NICE_LEVEL=10              # CPU priority (0-20, higher = lower priority)
export BREW_UPDATE_THROTTLE_INTERVAL=300      # Min seconds between runs
export BREW_UPDATE_EXIT_TIMEOUT=7200          # Max runtime (2 hours)
export BREW_UPDATE_LOG_RETENTION_DAYS=1       # Keep logs for 24 hours
export BREW_UPDATE_MIN_DISK_SPACE_GB=5        # Minimum free space required

# Then install
./install.sh
```

### Runtime Variables

These can be set in the script or plist environment:

```bash
export LOG_DIR="$HOME/Library/Logs"           # Log directory
export LOG_RETENTION_DAYS=1                    # Keep logs for 24 hours
export MIN_DISK_SPACE_GB=5                     # Minimum free space required
export LOCK_TIMEOUT=7200                       # Max runtime (2 hours)
export MAX_LOG_SIZE=10485760                   # Log rotation size (10MB)
```

### Schedule

Edit the plist file to change run times. Default schedule:
- 9:00 AM
- 3:00 PM
- 9:00 PM

```xml
<key>StartCalendarInterval</key>
<array>
    <dict>
        <key>Hour</key>
        <integer>9</integer>
        <key>Minute</key>
        <integer>0</integer>
    </dict>
    <!-- Add more time slots as needed -->
</array>
```

## Usage

### Automatic Runs
Once installed, the script runs automatically on schedule. No action needed!

### Manual Run
```bash
~/Scripts/brew-daily-update.sh
```

### View Logs
```bash
# Main log
tail -f ~/Library/Logs/brew-updates.log

# Error log
tail -f ~/Library/Logs/brew-updates-error.log

# Launchd output
tail -f ~/Library/Logs/brew-update-stdout.log
```

### Management Commands

```bash
# Check status
launchctl list | grep brew-update

# Disable automatic runs
launchctl unload ~/Library/LaunchAgents/com.$(whoami).brew-update.plist

# Enable automatic runs
launchctl load ~/Library/LaunchAgents/com.$(whoami).brew-update.plist

# Trigger immediate run
launchctl start com.$(whoami).brew-update

# View next scheduled run
launchctl print gui/$(id -u)/com.$(whoami).brew-update | grep next
```

## Troubleshooting

### Updates aren't running
```bash
# Check if loaded
launchctl list | grep brew-update

# Check for errors
tail ~/Library/Logs/brew-update-stderr.log

# Verify script permissions
ls -la ~/Scripts/brew-daily-update.sh
```

### Lock file stuck
```bash
# Remove manually (script auto-removes stale locks >2 hours)
rm -f /tmp/brew-update.lock
```

### No network notification
The script will skip updates and notify you if no network is available. This is normal behavior.

## Uninstallation

```bash
# Unload the agent
launchctl unload ~/Library/LaunchAgents/com.$(whoami).brew-update.plist

# Remove files
rm ~/Library/LaunchAgents/com.$(whoami).brew-update.plist
rm ~/Scripts/brew-daily-update.sh
rm -rf ~/Library/Logs/brew-update*
```

## Requirements

- macOS 10.14 or later
- Homebrew installed
- Bash shell
- Write access to ~/Library/Logs

## Security

- Runs as user (not root)
- No sudo required
- Sandboxed to user environment
- Safe PATH configuration

## Development

### Setup Development Environment

```bash
# Clone the repo
git clone https://github.com/organicnz/brew-auto-update.git
cd brew-auto-update

# Setup development tools (lefthook, shellcheck, etc.)
./scripts/setup-dev.sh
```

### Pre-commit Hooks

Lefthook runs automatically on commit:
- ShellCheck linting
- Bash syntax validation
- Plist template validation
- Secrets detection
- Markdown linting

Run manually:
```bash
lefthook run pre-commit
```

## Contributing

Contributions welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT License - see LICENSE file for details

## Author

Created for automated Homebrew maintenance on macOS systems.

## Changelog

### v1.0.0
- Initial release
- Network connectivity check
- Disk space validation
- Differential logging
- Automatic log rotation
- 3x daily scheduling
- Desktop notifications
