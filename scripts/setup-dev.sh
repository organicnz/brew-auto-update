#!/bin/bash
# Development environment setup

set -e

echo "🔧 Setting up development environment..."

# Check if Homebrew is installed
if ! command -v brew &> /dev/null; then
    echo "❌ Homebrew not found. Please install: https://brew.sh"
    exit 1
fi

# Install lefthook
if ! command -v lefthook &> /dev/null; then
    echo "📦 Installing lefthook..."
    brew install lefthook
else
    echo "✅ lefthook already installed"
fi

# Install optional tools
echo ""
echo "📦 Installing optional development tools..."

# ShellCheck
if ! command -v shellcheck &> /dev/null; then
    echo "  Installing shellcheck..."
    brew install shellcheck
else
    echo "  ✅ shellcheck already installed"
fi

# markdownlint (optional)
if ! command -v markdownlint &> /dev/null; then
    echo "  ⚠️  markdownlint not installed (optional)"
    echo "     Install with: npm install -g markdownlint-cli"
fi

# Install lefthook hooks
echo ""
echo "🪝 Installing git hooks..."
lefthook install

echo ""
echo "✅ Development environment ready!"
echo ""
echo "Available commands:"
echo "  lefthook run pre-commit  - Run pre-commit checks"
echo "  lefthook run pre-push    - Run pre-push tests"
echo "  scripts/lefthook/test.sh - Run full test suite"
