# Repository Pattern & Transaction Management

This document outlines the Repository Pattern implementation in Quilt, focusing on data persistence strategies and transaction management.

## Repository Pattern Overview

Quilt uses the Repository Pattern to abstract data access logic and separate domain models from persistence mechanisms. This pattern provides:

- **Separation of Concerns**: Domain logic is isolated from data access logic
- **Testability**: Repositories can be mocked for unit testing
- **Flexibility**: Underlying storage can be changed without affecting domain logic
- **Consistency**: Standard interfaces for data access operations

## Repository Interfaces

Quilt defines several repository traits that specify contract interfaces for different domain models:

```rust
// Example repository trait (simplified)
#[async_trait]
pub trait MaterialRepository: Send + Sync {
    async fn register_material(&self, material: Material) -> Result<()>;
    async fn get_material_by_id(&self, material_id: &str) -> Result<Option<Material>>;
    async fn update_material_status(&self, material_id: &str, status: MaterialStatus) -> Result<()>;
    // ... other methods
}
```

## SQLite Implementations

Current implementations use SQLite for persistence with these characteristics:

- **SQLite Database**: Local file-based storage for all domain entities
- **Async API**: Using `sqlx` for async database access with Tokio
- **Connection Pooling**: Efficient connection management for concurrent operations
- **Transaction Management**: ACID-compliant operations for data integrity
- **Error Mapping**: Translation of database errors to domain-specific errors

## Transaction Management Pattern

The repository implementations use a standardized transaction management pattern with three helper methods:

### 1. `execute_in_transaction`

For raw transactions with custom error handling. Used when you need complete control over the transaction flow and error handling.

```rust
async fn execute_in_transaction<F, T>(&self, f: F) -> Result<T>
where
    F: for<'a> FnOnce(&'a mut Transaction<'_, Sqlite>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T>> + Send + 'a>> + Send + 'static,
```

### 2. `execute_query_in_transaction`

For write operations that need transaction guarantees. This provides standard error mapping for database errors to domain errors.

```rust
async fn execute_query_in_transaction<F, T>(&self, f: F) -> Result<T>
where
    F: for<'a> FnOnce(&'a mut Transaction<'_, Sqlite>) -> std::pin::Pin<Box<dyn std::future::Future<Output = std::result::Result<T, sqlx::Error>> + Send + 'a>> + Send + 'static,
```

### 3. `execute_read_query`

For read-only operations that can execute directly against the connection pool. This avoids transaction overhead for read operations.

```rust
async fn execute_read_query<F, T, E>(&self, f: F) -> Result<T>
where
    F: FnOnce(&SqlitePool) -> std::pin::Pin<Box<dyn std::future::Future<Output = std::result::Result<T, E>> + Send + '_>> + Send + 'static,
    E: std::error::Error + 'static,
```

## Benefits of the Transaction Management Pattern

- **Consistency**: All repository operations follow the same patterns for error handling
- **DRY Code**: Reduces duplication of transaction management logic
- **Clear Intent**: Method names explicitly communicate the transaction requirements
- **Testability**: Centralized transaction logic is easier to test and verify
- **Error Mapping**: Consistent translation of database errors to domain errors

## Usage Examples

### Write Operation Example

```rust
// Using execute_query_in_transaction for this write operation
self.execute_query_in_transaction(move |tx| {
    Box::pin(async move {
        sqlx::query("INSERT INTO swatches (id, cut_id, material_id, embedding) VALUES (?, ?, ?, ?)")
            .bind(&swatch_id)
            .bind(&cut_id)
            .bind(&material_id)
            .bind(&embedding_bytes)
            .execute(&mut **tx)
            .await
    })
}).await
```

### Read Operation Example

```rust
// Using execute_read_query for this read-only operation
self.execute_read_query(move |pool| {
    Box::pin(async move {
        let rows = sqlx::query("SELECT * FROM swatches WHERE material_id = ? ORDER BY created_at")
            .bind(&material_id)
            .fetch_all(pool)
            .await?;

        rows.iter()
            .map(Self::map_row_to_swatch)
            .collect::<std::result::Result<Vec<_>, _>>()
    })
}).await
```

## Error Handling

Repository operations map database errors to domain-specific errors:

```rust
// Example error mapping
match &e {
    sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
        return Err(SwatchRepositoryError::SwatchAlreadyExists(
            "Duplicate ID detected".into(),
        ));
    }
    _ => {}
}
```

## Implementation Guidelines

When implementing repository methods:

1. **Choose the right helper method**:

   - For write operations with transaction guarantees: `execute_query_in_transaction`
   - For read operations: `execute_read_query`
   - For complex custom transactions: `execute_in_transaction`

2. **Clone all data needed in the closure**:

   ```rust
   let id_for_closure = id.to_string();
   ```

3. **Use proper error mapping**:

   - Map database errors to meaningful domain errors
   - Include context in error messages
   - Log errors appropriately

4. **Consider batch operations**:
   - Use transactions for multi-entity operations
   - Balance batch size for optimal performance

## Future Considerations

- **Migration to other databases**: The repository pattern enables easier migration to other database systems
- **Distributed transactions**: For future support of distributed operations
- **Retry mechanisms**: Implementation of retry strategies for transient failures
- **Query optimization**: Specialized index usage for performance-critical operations
