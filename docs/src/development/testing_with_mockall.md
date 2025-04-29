# Testing with Mockall in Quilt

This guide explains how to use Mockall for testing in the Quilt project, based on our experience implementing tests for the `SwatchingActor`.

## Overview

Quilt uses [Mockall](https://docs.rs/mockall/latest/mockall/) for creating mock implementations of traits in tests. This allows for unit testing components in isolation without requiring the actual implementations of their dependencies.

## Setting Up a Trait for Mocking

To make a trait mockable, add the `#[cfg_attr(test, automock)]` attribute:

```rust
use async_trait::async_trait;

#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait MyRepository: Send + Sync + Debug + 'static {
    async fn get_item(&self, id: &str) -> Result<Option<Item>, RepositoryError>;
    // ... other methods
}
```

Important notes:

- Import `mockall::automock` with `#[cfg(test)]` to only include it in test builds
- Place `#[cfg_attr(test, automock)]` before the trait definition
- The trait should include `async_trait` for async methods

## Using Mocks in Tests

When a trait is set up for automocking, Mockall generates a `Mock<TraitName>` struct:

```rust
#[actix::test]
async fn test_my_actor() {
    // Create mock instances
    let mut mock_repository = MockMyRepository::new();

    // Set up expectations
    mock_repository
        .expect_get_item()
        .with(mockall::predicate::eq("test-id"))
        .returning(|_| Ok(Some(test_item.clone())));

    // Create the component using the mock
    let actor = MyActor::new(
        "test-actor",
        Arc::new(mock_repository),
    );

    // Test the actor
    let response = actor.send(MyMessage { id: "test-id".to_string() }).await;
    assert!(response.is_ok());
}
```

## Testing Actors with Mockall

When testing actors, use the `#[actix::test]` attribute instead of `#[tokio::test]`:

```rust
#[actix::test]
async fn test_actor_with_mocks() {
    // ... test code ...
}
```

This is crucial because Actix actors use a special runtime context that isn't provided by the standard `#[tokio::test]`.

## Setting Up Expectations

Configure mock behavior using the expect methods:

```rust
// Expect a specific argument
mock.expect_method()
    .with(mockall::predicate::eq(expected_value))
    .returning(|_| Ok(result));

// Accept any argument
mock.expect_method()
    .with(mockall::predicate::always())
    .returning(|_| Ok(result));

// Return based on the argument
mock.expect_method()
    .returning(|arg| {
        // Logic based on arg
        Ok(processed_result)
    });

// Return different values on successive calls
mock.expect_method()
    .times(1)
    .returning(|_| Ok(first_result));
mock.expect_method()
    .times(1)
    .returning(|_| Ok(second_result));
```

## Common Mockall Patterns

### Returning a Clone

When returning a value that doesn't implement Copy:

```rust
// Create the value outside the closure
let result_to_return = SomeStruct::new();

// Use move and clone in the returning closure
mock.expect_method()
    .returning(move |_| Ok(result_to_return.clone()));
```

### Checking Arguments

Use predicates to verify arguments:

```rust
// Exact value match
mock.expect_method()
    .with(mockall::predicate::eq(expected_value))
    .returning(|_| Ok(result));

// Function-based match
mock.expect_method()
    .with(mockall::predicate::function(|arg: &str| arg.starts_with("test-")))
    .returning(|_| Ok(result));
```

### Testing Error Cases

Test how your code handles errors from dependencies:

```rust
// Return an error
mock.expect_method()
    .returning(|_| Err(MyError::NotFound));

// Test that the error is handled correctly
let result = component.send(Message).await;
assert!(result.is_err());
```

## Tips for Effective Mockall Testing

1. **Keep tests focused**: Test one specific behavior at a time
2. **Verify all expectations**: Mockall automatically verifies expectations when the mock is dropped
3. **Set realistic expectations**: Only mock the methods that will actually be called
4. **Use descriptive test names**: Clearly state what behavior is being tested
5. **Test error paths**: Make sure your code properly handles errors from dependencies
6. **Use separate mocks for separate tests**: Avoid reusing mock instances between test cases

## Troubleshooting Common Issues

### "Cannot find mock in this scope"

If you see errors like `failed to resolve: could not find 'mock' in mockall`, make sure you're:

- Using the correct import: `use mockall::predicate;`
- Using the automatically generated mock type: `MockMyRepository` instead of manually creating your own

### "Method is not a member of trait"

If you're implementing a method in a mock that doesn't exist in the trait:

- Check for typos in method names
- Make sure the trait and mock are in sync
- Remove any methods not in the trait

### "Cannot move out of borrowed content"

When returning values from mocks:

- Use `clone()` for values that don't implement Copy
- Use a `move` closure to capture values from outer scope: `.returning(move |_| ...)`

### "Lifetime parameters do not match trait declaration"

When using async traits with mocks:

- Make sure to use `#[async_trait]` on both the trait and its implementation
- Import `async_trait::async_trait` correctly

## Further Reading

- [Mockall Documentation](https://docs.rs/mockall/latest/mockall/)
- [Actix Testing Guide](https://actix.rs/docs/testing/)
- [Async Trait Documentation](https://docs.rs/async-trait/latest/async_trait/)
