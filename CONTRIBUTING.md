# Contributing to Homebrew Auto-Update

Thank you for your interest in contributing! This document provides guidelines for contributing to this project.

## How to Contribute

### Reporting Issues

- Check existing issues before creating a new one
- Provide detailed information:
  - macOS version
  - Homebrew version (`brew --version`)
  - Error messages or logs
  - Steps to reproduce

### Suggesting Features

- Open an issue with the `enhancement` label
- Describe the feature and its use case
- Explain why it would be useful

### Pull Requests

1. Fork the repository
2. Clone and setup development environment:
   ```bash
   git clone https://github.com/YOUR_USERNAME/brew-auto-update.git
   cd brew-auto-update
   ./scripts/setup-dev.sh
   ```
3. Create a feature branch (`git checkout -b feature/amazing-feature`)
4. Make your changes
5. Run tests: `scripts/lefthook/test.sh`
6. Commit (lefthook will run pre-commit checks automatically)
7. Push to your fork (`git push origin feature/amazing-feature`)
8. Open a Pull Request

### Git Hooks

This project uses [Lefthook](https://github.com/evilmartians/lefthook) for git hooks:

**Pre-commit:**
- ShellCheck linting
- Bash syntax validation
- Plist template validation
- Secrets detection
- Markdown linting

**Pre-push:**
- Full test suite

Run manually:
```bash
lefthook run pre-commit
lefthook run pre-push
```

## Development Guidelines

### Code Style

- Follow existing bash script conventions
- Use meaningful variable names
- Add comments for complex logic
- Keep functions focused and small

### Testing

Before submitting:
- Test on a clean macOS installation if possible
- Verify the script runs without errors
- Check that launchd integration works
- Ensure logs are created correctly

### Documentation

- Update README.md for new features
- Add inline comments for complex code
- Update CHANGELOG in README.md

## Questions?

Open an issue with the `question` label.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
