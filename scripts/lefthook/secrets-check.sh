#!/bin/bash
# Check for accidentally committed secrets

set -e

echo "🔍 Checking for secrets..."

has_errors=0

# Patterns to check for
patterns=(
    "github_pat_[a-zA-Z0-9_]+"
    "ghp_[a-zA-Z0-9]+"
    "AKIA[0-9A-Z]{16}"
    "sk_live_[a-zA-Z0-9]+"
    "pk_live_[a-zA-Z0-9]+"
    "-----BEGIN (RSA|DSA|EC|OPENSSH) PRIVATE KEY-----"
    "password\s*=\s*['\"][^'\"]+['\"]"
    "api_key\s*=\s*['\"][^'\"]+['\"]"
)

for file in "$@"; do
    if [[ -f "$file" ]]; then
        for pattern in "${patterns[@]}"; do
            if grep -qiE "$pattern" "$file"; then
                echo "❌ Potential secret found in $file"
                echo "   Pattern: $pattern"
                has_errors=1
            fi
        done
    fi
done

if [ $has_errors -eq 1 ]; then
    echo "❌ Secrets check failed - remove sensitive data before committing"
    exit 1
fi

echo "✅ No secrets detected"
exit 0
