# Contributing to WRAITH Protocol

## Development Setup

1. Install Rust 1.75+ via [rustup](https://rustup.rs/)
2. Clone the repository
3. Run `cargo build` to compile

## Workflow

1. Fork the repository
2. Create a feature branch from `develop`
3. Make your changes
4. Run tests: `cargo test --workspace`
5. Run lints: `cargo clippy --workspace -- -D warnings`
6. Format code: `cargo fmt --all`
7. Submit a pull request

## Code Style

- Follow Rust standard conventions
- Use `rustfmt` for formatting (config in `rustfmt.toml`)
- Address all `clippy` warnings
- Write doc comments for public APIs
- Add tests for new functionality

## Commit Messages

Use conventional commit format:

```
feat: add new feature
fix: correct bug in module
docs: update documentation
refactor: restructure code
test: add tests
chore: update dependencies
```

## Security

If you discover a security vulnerability, please report it privately.
Do not open a public issue for security concerns.
