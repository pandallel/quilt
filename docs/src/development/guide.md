# Development Guide

This guide provides information for developers working on the Sidecar project.

## Documentation

The project uses [mdBook](https://rust-lang.github.io/mdBook/) for documentation. Here are some useful commands:

```bash
# Run the documentation server
cd docs && mdbook serve

# Stop the documentation server
pkill -f "mdbook serve"

# Install admonish assets (if needed)
cd docs && mdbook-admonish install
```

## Building the Project

```bash
# Build the project
cargo build

# Run tests
cargo test

# Run with specific features
cargo run --features=feature-name
```

## Project Structure

- `src/` - Source code
- `docs/` - Documentation
- `tests/` - Integration tests
