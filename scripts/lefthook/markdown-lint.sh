#!/bin/bash
# Markdown linter

set -e

echo "🔍 Checking markdown files..."

if ! command -v markdownlint &> /dev/null; then
    echo "⚠️  markdownlint not installed, skipping..."
    echo "   Install with: npm install -g markdownlint-cli"
    exit 0
fi

has_errors=0

for file in "$@"; do
    if [[ -f "$file" ]]; then
        echo "  Checking $file..."
        if ! markdownlint "$file"; then
            has_errors=1
        fi
    fi
done

if [ $has_errors -eq 1 ]; then
    echo "❌ Markdown lint failed"
    exit 1
fi

echo "✅ Markdown lint passed"
exit 0
