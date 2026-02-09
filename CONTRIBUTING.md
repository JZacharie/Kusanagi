# Contributing to Kusanagi

Thank you for your interest in contributing to Kusanagi! This document provides guidelines and information for contributors.

## 🚀 Quick Start

```bash
# 1. Fork and clone
git clone https://github.com/YOUR_USERNAME/Kusanagi.git
cd Kusanagi

# 2. Setup development environment
./scripts/dev-setup.sh

# 3. Create a branch
git checkout -b feature/your-feature

# 4. Make changes and commit
git commit -m "feat: add your feature"

# 5. Push and create PR
git push origin feature/your-feature
```

## 📋 Development Workflow

### Prerequisites

- Rust 1.70+
- Docker (optional but recommended)
- kubectl (for Kubernetes testing)

### Setup

```bash
# Install development tools
./scripts/dev-setup.sh

# Verify setup
make check
```

### Running Locally

```bash
# Development mode with hot reload
cargo watch -x run

# Or use Make
make run
```

### Testing

```bash
# Run all tests
make test

# Run unit tests only
make test-unit

# Run integration tests
make test-integration

# Generate coverage report
make coverage
```

### Code Quality

```bash
# Format code
make fmt

# Run linter
make lint

# Run all checks
make all
```

## 🏗️ Architecture

Kusanagi follows **Hexagonal Architecture (Ports and Adapters)**:

```
src/
├── domain/          # Business logic, entities, ports
├── application/     # Use cases, DTOs, mappers
├── infrastructure/  # Repository implementations
├── interfaces/      # HTTP handlers, middleware
└── legacy/          # Modules being refactored
```

### Adding New Features

1. **Domain Layer** (`src/domain/`)
   - Add entities to `entities/`
   - Add repository ports to `ports/`

2. **Application Layer** (`src/application/`)
   - Create use cases in `use_cases/`
   - Add DTOs and mappers if needed

3. **Infrastructure Layer** (`src/infrastructure/`)
   - Implement repositories

4. **Interface Layer** (`src/interfaces/`)
   - Create HTTP handlers
   - Register routes

See [ARCHITECTURE.md](src/ARCHITECTURE.md) for details.

## 📝 Commit Guidelines

We follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

### Types

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting)
- `refactor`: Code refactoring
- `test`: Test changes
- `chore`: Build/process changes

### Examples

```
feat(api): add pod filtering by status

fix(cache): resolve TTL expiration bug

docs(readme): update installation instructions

refactor(legacy): migrate security module to hexagonal
```

## 🧪 Testing Guidelines

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pod_filtering() {
        let pods = vec![
            Pod { name: "pod-1".to_string(), status: PodStatus::Running },
            Pod { name: "pod-2".to_string(), status: PodStatus::Pending },
        ];
        
        let running = filter_by_status(&pods, PodStatus::Running);
        assert_eq!(running.len(), 1);
    }
}
```

### Integration Tests

Place in `tests/` directory:

```rust
// tests/pod_api_test.rs
use kusanagi::create_app;

#[actix_web::test]
async fn test_get_pods() {
    let app = test::init_service(create_app()).await;
    let req = test::TestRequest::get().uri("/api/pods").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}
```

## 🔒 Security

- Never commit secrets or credentials
- Use environment variables for configuration
- Run `cargo audit` before submitting PRs
- Report security issues privately to maintainers

## 📊 Code Coverage

We aim for 60%+ test coverage:

```bash
# Generate coverage report
make coverage

# Open report
open coverage/index.html
```

## 🐛 Reporting Bugs

When reporting bugs, please include:

1. **Description**: Clear description of the bug
2. **Steps to Reproduce**: Minimal steps to reproduce
3. **Expected Behavior**: What should happen
4. **Actual Behavior**: What actually happens
5. **Environment**: OS, Rust version, Kubernetes version
6. **Logs**: Relevant log output

## 💡 Feature Requests

Feature requests are welcome! Please:

1. Check existing issues first
2. Describe the use case
3. Explain why it would be useful
4. Consider contributing the feature yourself!

## 🏷️ Labels

We use labels to categorize issues:

| Label | Description |
|-------|-------------|
| `good first issue` | Good for newcomers |
| `help wanted` | Extra attention needed |
| `bug` | Something isn't working |
| `enhancement` | New feature request |
| `documentation` | Documentation improvements |
| `refactoring` | Code restructuring |

## 🤝 Code of Conduct

This project follows the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct):

- Be respectful and inclusive
- Welcome newcomers
- Focus on constructive feedback
- Respect different viewpoints

## 📞 Getting Help

- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: General questions
- **Wiki**: Documentation and guides

## 🙏 Thank You!

Your contributions make Kusanagi better for everyone. We appreciate your time and effort!

---

For questions or clarification, please open an issue or discussion.
