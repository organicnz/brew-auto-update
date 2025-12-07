# Homebrew Auto-Update System

## Production-Ready Setup Complete ✓

### System Architecture

**Dual Execution Paths:**
- **Automated**: launchd runs 3x daily (9 AM, 3 PM, 9 PM)
- **Manual**: Automator app or direct script execution

### Key Features

#### 1. Robust Error Handling
- Exit on error with `set -euo pipefail`
- Comprehensive trap for cleanup on EXIT/INT/TERM/HUP
- Timeout protection (2-hour max runtime)
- Graceful failure handling

#### 2. Lock Management
- Prevents concurrent executions
- Detects and removes stale locks (>2 hours)
- Safe multi-instance protection

#### 3. Process Cleanup
- Kills hung brew processes on exit
- Removes lock files automatically
- Cleans up child processes

#### 4. Logging System
- Main log: `~/Library/Logs/brew-updates.log`
- Error log: `~/Library/Logs/brew-updates-error.log`
- Automatic log rotation at 10MB
- 30-day log retention with compression
- Timestamped entries

#### 5. Update Operations
- Update Homebrew itself
- Upgrade all formulae
- Upgrade all casks (with --greedy flag)
- Update global NPM packages
- Cleanup old versions (30-day retention)
- Autoremove unused dependencies
- Health check with brew doctor

#### 6. Performance Optimizations
- Low priority I/O (LowPriorityIO)
- Nice level 10 (background priority)
- Throttle interval (300s between rapid runs)
- Process type: Background

### Files

```
~/Scripts/brew-daily-update.sh          # Main update script
~/Library/LaunchAgents/com.USER.brew-update.plist  # launchd config
~/Library/Logs/brew-updates.log         # Main log
~/Library/Logs/brew-updates-error.log   # Error log
/tmp/brew-update.lock                   # Lock file
```

### Management Commands

```bash
# Check status
launchctl list | grep brew-update

# View logs
tail -f ~/Library/Logs/brew-updates.log

# Manual run
~/Scripts/brew-daily-update.sh

# Reload schedule
launchctl unload ~/Library/LaunchAgents/com.USER.brew-update.plist
launchctl load ~/Library/LaunchAgents/com.USER.brew-update.plist

# Disable automatic runs
launchctl unload ~/Library/LaunchAgents/com.USER.brew-update.plist

# Enable automatic runs
launchctl load ~/Library/LaunchAgents/com.USER.brew-update.plist

# Test run immediately
launchctl start com.USER.brew-update
```

### Schedule

- **Morning**: 9:00 AM
- **Afternoon**: 3:00 PM  
- **Evening**: 9:00 PM

### Notifications

Desktop notifications on completion:
- Success: "All packages updated successfully"
- Warning: "Updates completed with some errors"

### Best Practices Implemented

✓ Idempotent operations
✓ Atomic lock file handling
✓ Comprehensive error logging
✓ Resource-friendly (low priority)
✓ Self-healing (stale lock removal)
✓ Timeout protection
✓ Log rotation and retention
✓ Process cleanup on exit
✓ Network-aware operations
✓ Graceful degradation

### Troubleshooting

**If updates aren't running:**
```bash
# Check if loaded
launchctl list | grep brew-update

# Check for errors
tail ~/Library/Logs/brew-update-stderr.log

# Verify script permissions
ls -la ~/Scripts/brew-daily-update.sh
```

**If lock file is stuck:**
```bash
# Remove manually (script auto-removes stale locks >2 hours)
rm -f /tmp/brew-update.lock
```

**View next scheduled run:**
```bash
launchctl print gui/$(id -u)/com.USER.brew-update | grep next
```

### System Requirements

- macOS with Homebrew installed
- Bash shell
- launchd (built-in)
- Write access to ~/Library/Logs

### Security

- Runs as user (not root)
- No sudo required
- Sandboxed to user environment
- Safe PATH configuration
