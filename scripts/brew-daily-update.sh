#!/bin/bash

################################################################################
# Homebrew Auto-Update Script
# Production-grade automated package management
# Runs 3x daily via launchd with full error handling and cleanup
#
# Repository: https://github.com/organicnz/brew-auto-update
# License: MIT
################################################################################

set -euo pipefail  # Exit on error, undefined vars, pipe failures

# Configuration - Auto-detected with customizable overrides
readonly SCRIPT_NAME="$(basename "$0")"
readonly CURRENT_USER="${USER:-$(whoami)}"
readonly LOG_DIR="${LOG_DIR:-$HOME/Library/Logs}"
readonly LOG_FILE="${LOG_FILE:-$LOG_DIR/brew-updates.log}"
readonly ERROR_LOG="${ERROR_LOG:-$LOG_DIR/brew-updates-error.log}"
readonly LOCKFILE="${LOCKFILE:-/tmp/brew-update.lock}"
readonly LOCK_TIMEOUT="${LOCK_TIMEOUT:-7200}"  # 2 hours max runtime
readonly MAX_LOG_SIZE="${MAX_LOG_SIZE:-10485760}"  # 10MB
readonly LOG_RETENTION_DAYS="${LOG_RETENTION_DAYS:-1}"  # 24 hours only
readonly MIN_DISK_SPACE_GB="${MIN_DISK_SPACE_GB:-5}"  # Minimum free space required

# Auto-detect Homebrew installation
if [ -x "/opt/homebrew/bin/brew" ]; then
    readonly BREW_PATH="/opt/homebrew/bin/brew"
elif [ -x "/usr/local/bin/brew" ]; then
    readonly BREW_PATH="/usr/local/bin/brew"
elif command -v brew &> /dev/null; then
    readonly BREW_PATH="$(command -v brew)"
else
    readonly BREW_PATH="brew"  # Fallback to PATH
fi

# Ensure log directory exists
mkdir -p "$LOG_DIR"

################################################################################
# Logging Functions
################################################################################

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $*" | tee -a "$LOG_FILE"
}

log_error() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] ERROR: $*" | tee -a "$LOG_FILE" "$ERROR_LOG" >&2
}

log_section() {
    local msg="$1"
    log "========================================"
    log "$msg"
    log "========================================"
}

################################################################################
# Cleanup and Lock Management
################################################################################

cleanup() {
    local exit_code=$?
    
    # Kill any hung brew processes
    pkill -9 -f "brew update" 2>/dev/null || true
    pkill -9 -f "brew upgrade" 2>/dev/null || true
    pkill -9 -f "brew cleanup" 2>/dev/null || true
    pkill -9 -f "brew autoremove" 2>/dev/null || true
    
    # Remove lock
    rm -f "$LOCKFILE"
    
    if [ $exit_code -eq 0 ]; then
        log "Cleanup completed successfully"
    else
        log_error "Script exited with code $exit_code"
    fi
    
    exit $exit_code
}

trap cleanup EXIT INT TERM HUP

acquire_lock() {
    if [ -f "$LOCKFILE" ]; then
        local lock_age=$(($(date +%s) - $(stat -f %m "$LOCKFILE" 2>/dev/null || echo 0)))
        
        if [ $lock_age -gt $LOCK_TIMEOUT ]; then
            log_error "Stale lock detected (${lock_age}s old), removing..."
            rm -f "$LOCKFILE"
        else
            log "Another instance is running (lock age: ${lock_age}s), exiting..."
            exit 0
        fi
    fi
    
    echo $$ > "$LOCKFILE"
    log "Lock acquired (PID: $$)"
}

################################################################################
# Log Rotation
################################################################################

rotate_logs() {
    for logfile in "$LOG_FILE" "$ERROR_LOG"; do
        if [ -f "$logfile" ] && [ $(stat -f%z "$logfile" 2>/dev/null || echo 0) -gt $MAX_LOG_SIZE ]; then
            local timestamp=$(date '+%Y%m%d-%H%M%S')
            mv "$logfile" "${logfile}.${timestamp}"
            log "Rotated log: $(basename "$logfile")"
            
            # Compress old log
            gzip "${logfile}.${timestamp}" 2>/dev/null || true
        fi
    done
    
    # Clean old logs
    find "$LOG_DIR" -name "brew-updates*.gz" -mtime +$LOG_RETENTION_DAYS -delete 2>/dev/null || true
}

################################################################################
# Pre-flight Checks
################################################################################

check_network() {
    log "Checking network connectivity..."
    if ping -c 1 -W 2 8.8.8.8 &> /dev/null || ping -c 1 -W 2 1.1.1.1 &> /dev/null; then
        log "✓ Network is available"
        return 0
    else
        log_error "No network connectivity detected"
        return 1
    fi
}

check_disk_space() {
    log "Checking disk space..."
    local available_gb=$(df -g / | awk 'NR==2 {print $4}')
    
    if [ "$available_gb" -lt "$MIN_DISK_SPACE_GB" ]; then
        log_error "Insufficient disk space: ${available_gb}GB available (minimum ${MIN_DISK_SPACE_GB}GB required)"
        return 1
    else
        log "✓ Sufficient disk space: ${available_gb}GB available"
        return 0
    fi
}

################################################################################
# Homebrew Operations
################################################################################

check_brew_installed() {
    if ! command -v "$BREW_PATH" &> /dev/null; then
        log_error "Homebrew not found at: $BREW_PATH"
        return 1
    fi
    log "Homebrew found at: $BREW_PATH"
    log "Running as user: $CURRENT_USER"
}

update_homebrew() {
    log "Updating Homebrew..."
    if "$BREW_PATH" update 2>&1 | tee -a "$LOG_FILE"; then
        log "✓ Homebrew updated successfully"
        return 0
    else
        log_error "Failed to update Homebrew"
        return 1
    fi
}

upgrade_formulae() {
    log "Checking for outdated formulae..."
    local outdated_list=$("$BREW_PATH" outdated --formula 2>/dev/null)
    local outdated_count=$(echo "$outdated_list" | grep -v '^$' | wc -l | tr -d ' ')
    
    if [ "$outdated_count" -eq 0 ]; then
        log "✓ All formulae are up to date"
        return 0
    fi
    
    log "Upgrading $outdated_count formulae:"
    echo "$outdated_list" | while read -r formula; do
        [ -n "$formula" ] && log "  - $formula"
    done
    
    if "$BREW_PATH" upgrade --formula 2>&1 | grep -E "^(==>|✔|✘|Error:)" | tee -a "$LOG_FILE"; then
        log "✓ Formulae upgraded successfully"
        return 0
    else
        log_error "Failed to upgrade formulae"
        return 1
    fi
}

upgrade_casks() {
    log "Checking for outdated casks..."
    local outdated_list=$("$BREW_PATH" outdated --cask --greedy 2>/dev/null)
    local outdated_count=$(echo "$outdated_list" | grep -v '^$' | wc -l | tr -d ' ')
    
    if [ "$outdated_count" -eq 0 ]; then
        log "✓ All casks are up to date"
        return 0
    fi
    
    log "Upgrading $outdated_count casks:"
    echo "$outdated_list" | while read -r cask; do
        [ -n "$cask" ] && log "  - $cask"
    done
    
    if "$BREW_PATH" upgrade --cask --greedy 2>&1 | grep -E "^(==>|✔|✘|Error:)" | tee -a "$LOG_FILE"; then
        log "✓ Casks upgraded successfully"
        return 0
    else
        log_error "Failed to upgrade casks (some may require manual intervention)"
        return 0  # Don't fail on cask errors
    fi
}

cleanup_brew() {
    log "Running cleanup (removing versions older than 30 days)..."
    local cleanup_output=$("$BREW_PATH" cleanup --prune=30 -s 2>&1)
    
    if [ $? -eq 0 ]; then
        # Only log if something was actually cleaned
        if echo "$cleanup_output" | grep -qE "(Removing:|freed|Pruned)"; then
            echo "$cleanup_output" | grep -E "(Removing:|freed|Pruned)" | tee -a "$LOG_FILE"
            log "✓ Cleanup completed"
        else
            log "✓ Nothing to clean up"
        fi
    else
        log_error "Cleanup encountered errors"
    fi
    
    log "Removing unused dependencies..."
    local autoremove_output=$("$BREW_PATH" autoremove 2>&1)
    
    if [ $? -eq 0 ]; then
        if echo "$autoremove_output" | grep -qE "(Autoremoving|Uninstalling)"; then
            echo "$autoremove_output" | grep -E "(Autoremoving|Uninstalling|freed)" | tee -a "$LOG_FILE"
            log "✓ Autoremove completed"
        else
            log "✓ No unused dependencies"
        fi
    else
        log_error "Autoremove encountered errors"
    fi
}

health_check() {
    log "Running health check..."
    local doctor_output=$("$BREW_PATH" doctor 2>&1 || true)
    
    if echo "$doctor_output" | grep -q "ready to brew"; then
        log "✓ System health check passed"
    else
        log "⚠ Health check warnings:"
        echo "$doctor_output" | grep -v "ready to brew" | tee -a "$LOG_FILE"
    fi
}

generate_summary() {
    log_section "Update Summary"
    
    log "Installed formulae: $("$BREW_PATH" list --formula 2>/dev/null | wc -l | tr -d ' ')"
    log "Installed casks: $("$BREW_PATH" list --cask 2>/dev/null | wc -l | tr -d ' ')"
    
    local cache_size=$(du -sh "$("$BREW_PATH" --cache)" 2>/dev/null | awk '{print $1}')
    log "Cache size: ${cache_size}"
    
    local cellar_size=$(du -sh "$("$BREW_PATH" --cellar)" 2>/dev/null | awk '{print $1}')
    log "Cellar size: ${cellar_size}"
}

send_notification() {
    local status="$1"
    local message="$2"
    
    osascript -e "display notification \"$message\" with title \"Homebrew Update\" subtitle \"$status\"" 2>/dev/null || true
}

################################################################################
# Main Execution
################################################################################

main() {
    log_section "Starting Homebrew Update - $(date)"
    
    # Pre-flight checks
    rotate_logs
    acquire_lock
    
    # Check prerequisites
    check_network || {
        log_error "Aborting: No network connectivity"
        send_notification "Skipped" "No network connection available"
        exit 1
    }
    
    check_disk_space || {
        log_error "Aborting: Insufficient disk space"
        send_notification "Skipped" "Insufficient disk space"
        exit 1
    }
    
    check_brew_installed || exit 1
    
    # Track overall success
    local overall_success=true
    
    # Update operations
    update_homebrew || overall_success=false
    upgrade_formulae || overall_success=false
    upgrade_casks || overall_success=false
    
    # Cleanup operations (always run)
    cleanup_brew
    health_check
    generate_summary
    
    # Final status
    if [ "$overall_success" = true ]; then
        log_section "✓ All updates completed successfully"
        send_notification "Success" "All packages updated successfully"
        exit 0
    else
        log_section "⚠ Updates completed with some errors"
        send_notification "Warning" "Updates completed with some errors"
        exit 1
    fi
}

# Execute main function
main "$@"
