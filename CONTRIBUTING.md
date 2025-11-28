# Contributing to CorgiTerm

Thank you for your interest in contributing to CorgiTerm! This document provides guidelines for contributing to the project.

## Code of Conduct

Be kind, respectful, and constructive. We're all here to build something great together.

## Getting Started

1. **Fork the repository** on GitHub
2. **Clone your fork** locally
3. **Set up the development environment**

```bash
# Install dependencies (Fedora)
sudo dnf install gtk4-devel libadwaita-devel

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone your fork
git clone https://github.com/YOUR_USERNAME/corgiterm
cd corgiterm

# Build
cargo build

# Run tests
cargo test --workspace
```

## Development Workflow

### Branches

- `main` - Stable release branch
- `develop` - Active development
- `feature/*` - New features
- `fix/*` - Bug fixes

### Making Changes

1. Create a branch from `develop`:
   ```bash
   git checkout develop
   git pull origin develop
   git checkout -b feature/my-feature
   ```

2. Make your changes following our style guidelines

3. Write tests for new functionality

4. Run the full test suite:
   ```bash
   cargo test --workspace
   cargo fmt --all -- --check
   cargo clippy --workspace
   ```

5. Commit with a clear message:
   ```bash
   git commit -m "feat(ui): add natural language input widget"
   ```

6. Push and create a Pull Request

### Commit Messages

We follow [Conventional Commits](https://www.conventionalcommits.org/):

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only
- `style`: Formatting, missing semicolons, etc.
- `refactor`: Code change that neither fixes a bug nor adds a feature
- `perf`: Performance improvement
- `test`: Adding missing tests
- `chore`: Build process or auxiliary tools

Examples:
```
feat(ai): add Claude provider support
fix(terminal): handle unicode combining characters
docs: update installation instructions
refactor(config): simplify theme loading
```

## Code Style

### Rust

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Write doc comments for public APIs

```rust
/// A terminal session within a project.
///
/// # Example
///
/// ```
/// let session = Session::new("dev", PathBuf::from("/home/user/project"));
/// session.start(TerminalSize::default())?;
/// ```
pub struct Session {
    // ...
}
```

### Documentation

- Use clear, concise language
- Include code examples where helpful
- Keep README up to date with changes

## Testing

### Running Tests

```bash
# All tests
cargo test --workspace

# Specific crate
cargo test -p corgiterm-core

# With output
cargo test -- --nocapture

# Integration tests
cargo test --test integration
```

### Writing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let session = Session::new("test", PathBuf::from("/tmp"));
        assert_eq!(session.name, "test");
    }
}
```

## Architecture

### Crate Structure

| Crate | Purpose |
|-------|---------|
| `corgiterm-core` | Terminal emulation, PTY, sessions |
| `corgiterm-ui` | GTK4/libadwaita interface |
| `corgiterm-ai` | AI provider integration |
| `corgiterm-config` | Configuration management |
| `corgiterm-plugins` | Plugin system |

### Adding a Feature

1. Determine which crate(s) need changes
2. Add configuration options in `corgiterm-config`
3. Implement core logic in `corgiterm-core` or feature crate
4. Add UI in `corgiterm-ui`
5. Write tests
6. Update documentation

## Pull Request Process

1. Update the README.md if needed
2. Update documentation for any API changes
3. Add tests for new functionality
4. Ensure CI passes (tests, fmt, clippy)
5. Request review from maintainers
6. Address review feedback
7. Squash commits if requested

### PR Title Format

```
feat(scope): Short description
fix(scope): Short description
docs: Short description
```

## Issues

### Bug Reports

Include:
- CorgiTerm version
- Operating system and version
- Steps to reproduce
- Expected vs actual behavior
- Logs if applicable

### Feature Requests

Include:
- Clear description of the feature
- Use case / motivation
- Possible implementation approach

## Questions?

- Open a [Discussion](https://github.com/corgiterm/corgiterm/discussions)
- Join our community chat (coming soon)

---

Thank you for contributing! 🐕
