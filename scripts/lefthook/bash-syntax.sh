#!/bin/bash
# Bash syntax checker

set -e

echo "🔍 Checking bash syntax..."

has_errors=0

for file in "$@"; do
    if [[ -f "$file" ]]; then
        echo "  Checking $file..."
        if ! bash -n "$file"; then
            echo "❌ Syntax error in $file"
            has_errors=1
        fi
    fi
done

if [ $has_errors -eq 1 ]; then
    echo "❌ Bash syntax check failed"
    exit 1
fi

echo "✅ Bash syntax check passed"
exit 0
