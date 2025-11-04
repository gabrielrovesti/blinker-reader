# Contributing to Blinker Reader

Thank you for your interest in contributing to Blinker Reader!

## Getting Started

1. Fork the repository
2. Clone your fork
3. Create a feature branch: `git checkout -b feature/my-feature`
4. Make your changes
5. Run tests: `cargo test --workspace`
6. Run lints: `cargo clippy --workspace --all-targets`
7. Format code: `cargo fmt --all`
8. Commit your changes
9. Push to your fork
10. Open a Pull Request

## Development Setup

See [README.md](README.md#getting-started) for development environment setup.

## Code Style

### Rust

- Follow standard Rust conventions
- Run `cargo fmt` before committing
- Address all `cargo clippy` warnings
- Write tests for new functionality
- Document public APIs with doc comments

### TypeScript/React

- Use TypeScript strict mode
- Follow React best practices
- Use functional components with hooks
- Write clean, readable JSX

## Testing

- Write unit tests for Rust crates
- Add integration tests for cross-crate functionality
- Test UI components where appropriate
- Ensure all tests pass before submitting PR

## Security

- Follow secure coding practices
- No unsafe code without justification
- Validate all user inputs
- Report security issues privately to [security contact]

## Commit Messages

Use clear, descriptive commit messages:

```
feat: Add EPUB table of contents parsing
fix: Correct PDF rendering on HiDPI displays
docs: Update architecture documentation
test: Add tests for library scanner
```

Prefix format:
- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation
- `test:` - Tests
- `refactor:` - Code refactoring
- `perf:` - Performance improvement
- `chore:` - Maintenance

## Pull Request Process

1. Update documentation if needed
2. Add tests for new features
3. Ensure CI passes
4. Request review from maintainers
5. Address review feedback
6. Squash commits if requested

## Code Review

All submissions require code review. We aim to:

- Provide feedback within 48 hours
- Be respectful and constructive
- Focus on code quality and security

## License

By contributing, you agree that your contributions will be licensed under the Apache-2.0 License.

## Questions?

Open a GitHub Discussion or issue for questions about contributing.
