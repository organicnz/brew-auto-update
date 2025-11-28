#!/bin/bash
# ShellCheck linter for shell scripts

set -e

echo "🔍 Running ShellCheck..."

if ! command -v shellcheck &> /dev/null; then
    echo "⚠️  ShellCheck not installed, skipping..."
    echo "   Install with: brew install shellcheck"
    exit 0
fi

has_errors=0

for file in "$@"; do
    if [[ -f "$file" ]]; then
        echo "  Checking $file..."
        if ! shellcheck -x "$file"; then
            has_errors=1
        fi
    fi
done

if [ $has_errors -eq 1 ]; then
    echo "❌ ShellCheck found issues"
    exit 1
fi

echo "✅ ShellCheck passed"
exit 0
