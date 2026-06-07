# Contributing to CrossCrypt

Thank you for your interest in contributing to CrossCrypt!

## Development Setup

1. Install Rust: https://rustup.rs/
2. Clone the repository
3. Install platform dependencies:
   ```bash
   ./scripts/install-deps.sh
   ```

## Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Cross-compile
./scripts/build-all.sh --all
```

## Code Style

- Follow Rust naming conventions
- Use `cargo fmt` before committing
- Run `cargo clippy` and fix warnings
- Add tests for new functionality
- Document public APIs with rustdoc

## Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_encrypt_decrypt

# Run with output
cargo test -- --nocapture
```

## Pull Request Process

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Update documentation
6. Submit pull request

## Security

Please report security vulnerabilities to security@crosscrypt.io

## Code of Conduct

Be respectful and constructive in all interactions.
