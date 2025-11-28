#!/bin/bash
# Run full test suite

set -e

echo "🧪 Running test suite..."

# Test 1: Bash syntax for all scripts
echo "  Testing bash syntax..."
for script in *.sh scripts/lefthook/*.sh; do
    if [[ -f "$script" ]]; then
        bash -n "$script" || exit 1
    fi
done

# Test 2: Verify template placeholders
echo "  Testing template placeholders..."
if [[ -f "com.USER.brew-update.plist.template" ]]; then
    required_vars=("{{USER}}" "{{HOME}}" "{{HOUR1}}" "{{MINUTE1}}" "{{NICE_LEVEL}}")
    for var in "${required_vars[@]}"; do
        if ! grep -q "$var" com.USER.brew-update.plist.template; then
            echo "❌ Missing required placeholder: $var"
            exit 1
        fi
    done
fi

# Test 3: Verify installer can generate valid plist
echo "  Testing plist generation..."
temp_plist=$(mktemp)
sed -e "s|{{USER}}|testuser|g" \
    -e "s|{{HOME}}|/Users/testuser|g" \
    -e "s|{{HOUR1}}|9|g" \
    -e "s|{{MINUTE1}}|0|g" \
    -e "s|{{HOUR2}}|15|g" \
    -e "s|{{MINUTE2}}|0|g" \
    -e "s|{{HOUR3}}|21|g" \
    -e "s|{{MINUTE3}}|0|g" \
    -e "s|{{NICE_LEVEL}}|10|g" \
    -e "s|{{THROTTLE_INTERVAL}}|300|g" \
    -e "s|{{EXIT_TIMEOUT}}|7200|g" \
    -e "s|{{LOG_RETENTION_DAYS}}|1|g" \
    -e "s|{{MIN_DISK_SPACE_GB}}|5|g" \
    com.USER.brew-update.plist.template > "$temp_plist"

if ! plutil -lint "$temp_plist" &> /dev/null; then
    echo "❌ Generated plist is invalid"
    rm "$temp_plist"
    exit 1
fi
rm "$temp_plist"

# Test 4: Check for required files
echo "  Checking required files..."
required_files=(
    "README.md"
    "LICENSE"
    "brew-daily-update.sh"
    "install.sh"
    "com.USER.brew-update.plist.template"
)

for file in "${required_files[@]}"; do
    if [[ ! -f "$file" ]]; then
        echo "❌ Missing required file: $file"
        exit 1
    fi
done

echo "✅ All tests passed"
exit 0
