# Testing and Linting Guide

This guide covers testing and linting procedures for Quilt development.

## Testing

Run the test suite with:

```bash
cargo test
```

### Writing Tests

Tests should be organized as follows:

- Unit tests in the same file as the code they test, within a `#[cfg(test)]` module
- Integration tests in the `src/<module>/tests/` directory
- System tests in the `tests/` directory

Example of a unit test:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_behavior() {
        // Test code here
    }
}
```

For async tests, use the `#[tokio::test]` attribute:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_async_function() {
        // Async test code here
    }
}
```

### Test Output

Tests should be quiet by default and rely on Rust's built-in test framework for reporting. Follow these guidelines:

- **Avoid `println!` statements** in tests, as they add noise to test output and make it harder to spot actual failures.
- Use descriptive test names and assertion messages instead of printing progress messages.
- For conditional test execution (e.g., skipping tests when prerequisites aren't met), use `eprintln!` instead of `println!` to output warnings.
- If debugging test behavior, use `cargo test -- --nocapture` to see output, but remove debug prints before committing.

Example of proper error output in tests:

```rust
#[test]
fn test_configuration_parsing() {
    let config = match parse_config("test.cfg") {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Skipping test due to missing test file: {}", e);
            return;
        }
    };

    assert_eq!(config.value, expected_value, "Config should parse 'value' field correctly");
}
```

## Linting and Formatting

### Rustfmt

To check your code formatting:

```bash
cargo fmt --all -- --check
```

To automatically format your code:

```bash
cargo fmt
```

The project uses a custom `rustfmt.toml` configuration to ensure consistent formatting. Note that our configuration only uses options available in stable Rust to ensure compatibility with CI/CD workflows.

### Clippy

To run clippy linting checks:

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

The project uses a custom `.clippy.toml` configuration with the following settings:

- Maximum cognitive complexity: 20
- Maximum type complexity: 500
- Maximum function arguments: 8
- Maximum lines per file: 200

## CI/CD

This project uses GitHub Actions for PR validation, which includes:

- Code formatting checks with `rustfmt`
- Linting with `clippy`
- Running the test suite

Pull requests must pass all validation checks before they can be merged.

The workflow configuration can be found in `.github/workflows/pr_validation.yml`.

### Troubleshooting CI Failures

If you encounter formatting errors in CI:

1. Ensure you're using the same rustfmt configuration as the CI pipeline
2. Run `cargo fmt` locally before pushing changes
3. If working with nightly Rust locally, verify your changes still format correctly with stable Rust
