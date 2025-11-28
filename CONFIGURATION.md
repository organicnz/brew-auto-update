# Configuration Guide

This document explains all configuration options for Homebrew Auto-Update.

## Installation-Time Configuration

Set these environment variables **before** running `./install.sh`:

### Schedule Configuration

```bash
# First run time (default: 9:00 AM)
export BREW_UPDATE_HOUR1=9
export BREW_UPDATE_MINUTE1=0

# Second run time (default: 3:00 PM)
export BREW_UPDATE_HOUR2=15
export BREW_UPDATE_MINUTE2=0

# Third run time (default: 9:00 PM)
export BREW_UPDATE_HOUR3=21
export BREW_UPDATE_MINUTE3=0
```

### System Settings

```bash
# CPU priority (0-20, higher = lower priority, default: 10)
export BREW_UPDATE_NICE_LEVEL=10

# Minimum seconds between runs (default: 300 = 5 minutes)
export BREW_UPDATE_THROTTLE_INTERVAL=300

# Maximum runtime before force kill (default: 7200 = 2 hours)
export BREW_UPDATE_EXIT_TIMEOUT=7200

# Log retention in days (default: 1 = 24 hours)
export BREW_UPDATE_LOG_RETENTION_DAYS=1

# Minimum free disk space in GB (default: 5)
export BREW_UPDATE_MIN_DISK_SPACE_GB=5
```

### Example: Custom Installation

```bash
# Run at 8 AM, 2 PM, and 10 PM
export BREW_UPDATE_HOUR1=8
export BREW_UPDATE_HOUR2=14
export BREW_UPDATE_HOUR3=22

# Keep logs for 3 days
export BREW_UPDATE_LOG_RETENTION_DAYS=3

# Require 10GB free space
export BREW_UPDATE_MIN_DISK_SPACE_GB=10

# Install
./install.sh
```

## Runtime Configuration

These variables are set in the plist and passed to the script at runtime. They can also be overridden by setting them in your shell before running the script manually.

### Script Variables

```bash
# Log directory (default: $HOME/Library/Logs)
export LOG_DIR="$HOME/Library/Logs"

# Log file path (default: $LOG_DIR/brew-updates.log)
export LOG_FILE="$LOG_DIR/brew-updates.log"

# Error log path (default: $LOG_DIR/brew-updates-error.log)
export ERROR_LOG="$LOG_DIR/brew-updates-error.log"

# Lock file path (default: /tmp/brew-update.lock)
export LOCKFILE="/tmp/brew-update.lock"

# Lock timeout in seconds (default: 7200 = 2 hours)
export LOCK_TIMEOUT=7200

# Max log file size before rotation (default: 10485760 = 10MB)
export MAX_LOG_SIZE=10485760

# Log retention days (default: 1)
export LOG_RETENTION_DAYS=1

# Minimum disk space in GB (default: 5)
export MIN_DISK_SPACE_GB=5
```

## Auto-Detected Values

These are automatically detected and don't need configuration:

- **Username**: Detected from `$USER` or `whoami`
- **Home directory**: Detected from `$HOME`
- **Homebrew path**: Auto-detected from:
  - `/opt/homebrew/bin/brew` (Apple Silicon)
  - `/usr/local/bin/brew` (Intel)
  - `brew` in PATH (fallback)

## Modifying After Installation

### Change Schedule

1. Edit the plist file:
```bash
nano ~/Library/LaunchAgents/com.$(whoami).brew-update.plist
```

2. Modify the `StartCalendarInterval` section

3. Reload:
```bash
launchctl unload ~/Library/LaunchAgents/com.$(whoami).brew-update.plist
launchctl load ~/Library/LaunchAgents/com.$(whoami).brew-update.plist
```

### Change Runtime Settings

Edit the `EnvironmentVariables` section in the plist:

```xml
<key>EnvironmentVariables</key>
<dict>
    <key>LOG_RETENTION_DAYS</key>
    <string>3</string>
    <key>MIN_DISK_SPACE_GB</key>
    <string>10</string>
</dict>
```

Then reload the agent.

## Examples

### Minimal Updates (Once Daily)

```bash
export BREW_UPDATE_HOUR1=9
export BREW_UPDATE_HOUR2=9  # Same time = effectively once
export BREW_UPDATE_HOUR3=9
./install.sh
```

### Aggressive Updates (Every 4 Hours)

```bash
export BREW_UPDATE_HOUR1=6
export BREW_UPDATE_HOUR2=12
export BREW_UPDATE_HOUR3=18
./install.sh
```

### Low Priority (Don't Slow Down System)

```bash
export BREW_UPDATE_NICE_LEVEL=19  # Lowest priority
export BREW_UPDATE_THROTTLE_INTERVAL=600  # 10 min between runs
./install.sh
```

### Keep More Logs

```bash
export BREW_UPDATE_LOG_RETENTION_DAYS=7  # Keep 1 week
./install.sh
```

## Validation

After installation, verify your configuration:

```bash
# Check schedule
launchctl print gui/$(id -u)/com.$(whoami).brew-update | grep -A 10 "calendar"

# Check environment variables
plutil -p ~/Library/LaunchAgents/com.$(whoami).brew-update.plist | grep -A 10 "EnvironmentVariables"

# View full config
cat ~/Library/LaunchAgents/com.$(whoami).brew-update.plist
```
