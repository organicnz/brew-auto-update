#!/bin/bash

################################################################################
# Homebrew Auto-Update Installer
# Installs the automated Homebrew update system
################################################################################

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "=========================================="
echo "Homebrew Auto-Update Installer"
echo "=========================================="
echo ""

# Get current user
CURRENT_USER=$(whoami)
USER_HOME=$(eval echo ~$CURRENT_USER)

# Configuration (customize via environment variables)
HOUR1="${BREW_UPDATE_HOUR1:-9}"
MINUTE1="${BREW_UPDATE_MINUTE1:-0}"
HOUR2="${BREW_UPDATE_HOUR2:-15}"
MINUTE2="${BREW_UPDATE_MINUTE2:-0}"
HOUR3="${BREW_UPDATE_HOUR3:-21}"
MINUTE3="${BREW_UPDATE_MINUTE3:-0}"
NICE_LEVEL="${BREW_UPDATE_NICE_LEVEL:-10}"
THROTTLE_INTERVAL="${BREW_UPDATE_THROTTLE_INTERVAL:-300}"
EXIT_TIMEOUT="${BREW_UPDATE_EXIT_TIMEOUT:-7200}"
LOG_RETENTION_DAYS="${BREW_UPDATE_LOG_RETENTION_DAYS:-1}"
MIN_DISK_SPACE_GB="${BREW_UPDATE_MIN_DISK_SPACE_GB:-5}"

echo "Installing for user: $CURRENT_USER"
echo "Home directory: $USER_HOME"
echo "Schedule: ${HOUR1}:$(printf '%02d' $MINUTE1), ${HOUR2}:$(printf '%02d' $MINUTE2), ${HOUR3}:$(printf '%02d' $MINUTE3)"
echo ""

# Check if Homebrew is installed
if ! command -v brew &> /dev/null; then
    echo -e "${RED}✗ Homebrew not found${NC}"
    echo "Please install Homebrew first: https://brew.sh"
    exit 1
fi
echo -e "${GREEN}✓ Homebrew found${NC}"

# Create Scripts directory if it doesn't exist
SCRIPTS_DIR="$USER_HOME/Scripts"
if [ ! -d "$SCRIPTS_DIR" ]; then
    echo "Creating $SCRIPTS_DIR..."
    mkdir -p "$SCRIPTS_DIR"
fi
echo -e "${GREEN}✓ Scripts directory ready${NC}"

# Copy the script
echo "Installing brew-daily-update.sh..."
cp scripts/brew-daily-update.sh "$SCRIPTS_DIR/"
chmod +x "$SCRIPTS_DIR/brew-daily-update.sh"
echo -e "${GREEN}✓ Script installed${NC}"

# Create launchd plist
PLIST_NAME="com.$CURRENT_USER.brew-update.plist"
PLIST_PATH="$USER_HOME/Library/LaunchAgents/$PLIST_NAME"

echo "Creating launchd configuration..."
sed -e "s|{{USER}}|$CURRENT_USER|g" \
    -e "s|{{HOME}}|$USER_HOME|g" \
    -e "s|{{HOUR1}}|$HOUR1|g" \
    -e "s|{{MINUTE1}}|$MINUTE1|g" \
    -e "s|{{HOUR2}}|$HOUR2|g" \
    -e "s|{{MINUTE2}}|$MINUTE2|g" \
    -e "s|{{HOUR3}}|$HOUR3|g" \
    -e "s|{{MINUTE3}}|$MINUTE3|g" \
    -e "s|{{NICE_LEVEL}}|$NICE_LEVEL|g" \
    -e "s|{{THROTTLE_INTERVAL}}|$THROTTLE_INTERVAL|g" \
    -e "s|{{EXIT_TIMEOUT}}|$EXIT_TIMEOUT|g" \
    -e "s|{{LOG_RETENTION_DAYS}}|$LOG_RETENTION_DAYS|g" \
    -e "s|{{MIN_DISK_SPACE_GB}}|$MIN_DISK_SPACE_GB|g" \
    com.USER.brew-update.plist.template > "$PLIST_PATH"
echo -e "${GREEN}✓ Launchd plist created${NC}"

# Unload if already loaded
launchctl unload "$PLIST_PATH" 2>/dev/null || true

# Load the agent
echo "Loading launchd agent..."
launchctl load "$PLIST_PATH"
echo -e "${GREEN}✓ Launchd agent loaded${NC}"

# Verify
if launchctl list | grep -q "brew-update"; then
    echo -e "${GREEN}✓ Installation verified${NC}"
else
    echo -e "${YELLOW}⚠ Warning: Could not verify installation${NC}"
fi

echo ""
echo "=========================================="
echo "Installation Complete!"
echo "=========================================="
echo ""
echo "The script will run automatically at:"
echo "  • ${HOUR1}:$(printf '%02d' $MINUTE1)"
echo "  • ${HOUR2}:$(printf '%02d' $MINUTE2)"
echo "  • ${HOUR3}:$(printf '%02d' $MINUTE3)"
echo ""
echo "Logs location:"
echo "  • Main: ~/Library/Logs/brew-updates.log"
echo "  • Errors: ~/Library/Logs/brew-updates-error.log"
echo ""
echo "Management commands:"
echo "  • Manual run: ~/Scripts/brew-daily-update.sh"
echo "  • View logs: tail -f ~/Library/Logs/brew-updates.log"
echo "  • Check status: launchctl list | grep brew-update"
echo "  • Disable: launchctl unload $PLIST_PATH"
echo "  • Enable: launchctl load $PLIST_PATH"
echo ""
echo "Running initial test..."
sleep 2
launchctl start "com.$CURRENT_USER.brew-update"
echo -e "${GREEN}✓ Test run initiated${NC}"
echo ""
echo "Check logs: tail -f ~/Library/Logs/brew-updates.log"
