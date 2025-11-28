#!/bin/bash
# Plist template validator

set -e

echo "🔍 Validating plist templates..."

has_errors=0

for file in "$@"; do
    if [[ -f "$file" ]]; then
        echo "  Checking $file..."
        
        # Check for required placeholders
        if ! grep -q "{{USER}}" "$file"; then
            echo "❌ Missing {{USER}} placeholder in $file"
            has_errors=1
        fi
        
        if ! grep -q "{{HOME}}" "$file"; then
            echo "❌ Missing {{HOME}} placeholder in $file"
            has_errors=1
        fi
        
        # Try to validate XML structure (with placeholders replaced)
        temp_file=$(mktemp)
        sed -e 's/{{[^}]*}}/placeholder/g' "$file" > "$temp_file"
        
        if ! plutil -lint "$temp_file" &> /dev/null; then
            echo "❌ Invalid XML structure in $file"
            has_errors=1
        fi
        
        rm "$temp_file"
    fi
done

if [ $has_errors -eq 1 ]; then
    echo "❌ Plist validation failed"
    exit 1
fi

echo "✅ Plist validation passed"
exit 0
