# Swatching and Embedding

This workstream focuses on the logic within the Swatching Actor to generate embeddings for text cuts and create Swatch objects.

### Milestone: "Implement SQLite Swatch Repository (Persistence Only)"

**Goal:** Implement the basic SQLite persistence layer for swatches, storing embeddings as `BLOB`s. **Vector search will be handled in a later milestone.**
**Implementation Time:** ~1-2 days

1.  **Define Schema (`src/db.rs`)**: Define `swatches` table schema with columns for ID, cut ID, material ID, embedding (`BLOB`), model name, model version, created_at, dimensions, metadata, etc. Include foreign keys to `materials` and `cuts` (with cascade delete). Update `init_memory_db` to create this table.
2.  **Implement Repository (`src/swatching/sqlite_repository.rs`)**:
    - Create `SqliteSwatchRepository` struct holding a `SqlitePool`.
    - Implement the `SwatchRepository` trait.
    - Handle `Vec<f32>` <=> `BLOB` serialization/deserialization for the `embedding` column.
    - Implement basic CRUD methods (`save_swatch`, `get_swatch_by_id`, `delete_swatch`, etc.).
    - **Stub out** the `search_similar` method (e.g., return `Err(OperationFailed)` or an empty `Vec`).
3.  **Unit Tests (`src/swatching/sqlite_repository.rs`)**:
    - Add comprehensive unit tests for all implemented CRUD operations.
    - Include a test confirming `search_similar` is correctly stubbed/unimplemented.

**Demonstration:** Unit tests for `SqliteSwatchRepository` pass, verifying CRUD operations and embedding serialization/deserialization. `search_similar` is confirmed as unimplemented.

### Milestone: "Integrate Swatch Repository into Actor System"

**Goal:** Connect the implemented `SqliteSwatchRepository` to the `QuiltOrchestrator` and `SwatchingActor`.
**Implementation Time:** ~0.5 - 1 day

1.  **Update `QuiltOrchestrator`**: Initialize `SqliteSwatchRepository` (e.g., in `with_sqlite`) and store it (e.g., `Option<Arc<dyn SwatchRepository>>`).
2.  **Update `SwatchingActor`**: Modify constructor to accept and store `Arc<dyn SwatchRepository>`.
3.  **Update `QuiltOrchestrator`**: Inject the stored repository into `SwatchingActor` during `initialize_actors`.
4.  **Update Tests**: Modify `SwatchingActor` tests to provide a mock or real repository instance.

**Demonstration:** Application compiles and runs. `QuiltOrchestrator` successfully injects the `SqliteSwatchRepository` instance into `SwatchingActor`. Tests for `SwatchingActor` pass.

### Milestone: "Implement Swatching Actor Logic & Event"

**Goal:** Connect the `SwatchingActor` to retrieve cuts, generate **actual embeddings**, create swatches, store them via the repository, update registry status, and publish the `MaterialSwatched` event.
**Implementation Time:** ~3-4 days

1.  **Inject `CutsRepository`**: Update `SwatchingActor` and `QuiltOrchestrator` to inject the `CutsRepository` dependency into `SwatchingActor`.
2.  **Implement Embedding Generation**: Choose and integrate an embedding strategy (e.g., `rust-bert`, ONNX). Implement logic in the actor's processor task to generate embeddings (`Vec<f32>`) for `Cut` content.
3.  **Implement Swatching Logic**: Modify `SwatchingActor`'s processor task:
    - Retrieve `Cut`s using the injected `CutsRepository`.
    - Generate `Vec<f32>` embeddings for the cuts.
    - Create `Swatch` instances with the generated embeddings.
    - Save `Swatch` instances using the injected `SwatchRepository`.
    - Call `MaterialRegistry` to update material status to `Swatched` (or `Error`).
4.  **Implement `MaterialSwatched` Event**: Define the event variant in `QuiltEvent`. Update `MaterialRegistry` to publish the event. Add/update integration tests covering the full actor flow.

**Demonstration:** Running `main` shows "Created X swatches with embeddings..." logs. Material status updates to `Swatched`. `MaterialSwatched` events are published. Integration tests verify embedding creation and storage in the database.
