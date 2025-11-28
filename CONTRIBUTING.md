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
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Test thoroughly on macOS
5. Commit with clear messages (`git commit -m 'Add amazing feature'`)
6. Push to your fork (`git push origin feature/amazing-feature`)
7. Open a Pull Request

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
