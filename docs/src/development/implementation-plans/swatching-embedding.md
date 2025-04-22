# Swatching and Embedding

This workstream focuses on the logic within the Swatching Actor to generate embeddings for text cuts and create Swatch objects.

### Milestone 10: "Implement SQLite Swatch Repository"

**Goal:** Implement the SQLite persistence layer for swatches, anticipating future `sqlite-vec` use.
**Implementation Time:** ~2 days

1. Implement SQLite Repository (2 days)
   - Define `swatches` table schema in `src/db.rs`, including a field for embeddings (e.g., `BLOB`) and foreign key to `materials`. Update DB initialization.
   - Implement `SqliteSwatchRepository` struct implementing the `SwatchRepository` trait using `sqlx`.
   - Add comprehensive unit tests for `SqliteSwatchRepository` operations.
   - Update `QuiltOrchestrator` to initialize and manage the `SqliteSwatchRepository` (unaffected by `--in-memory` flag). Inject the `Arc<dyn SwatchRepository>` into `SwatchingActor` upon creation.

**Demonstration:** Unit tests for `SqliteSwatchRepository` pass. Application runs, injecting the repository into `SwatchingActor`.

### Milestone 11: "Implement Swatching Actor Logic & Event"

**Goal:** Connect the Swatching Actor to retrieve cuts, generate **actual embeddings**, create swatches, store them via the repository, update registry status, and publish the `MaterialSwatched` event.
**Implementation Time:** ~3-4 days (Increased due to embedding integration)

1. Implement Embedding Generation (1-2 days)
   - Choose and integrate an embedding strategy (e.g., `rust-bert`, ONNX runtime with a sentence transformer model).
   - Implement logic within the `SwatchingActor`'s processor task to generate embeddings for retrieved `Cut`s.
2. Implement Swatching Logic (Integration) (1-2 days)
   - Modify `SwatchingActor`'s processor task:
     - Retrieve `Cut`s using injected `CutsRepository`.
     - Use the implemented embedding generation logic.
     - Create `Swatch` instances with actual embeddings.
     - Save `Swatch` instances using injected `SwatchRepository`.
     - Call `MaterialRegistry` to update material status to `Swatched` (or `Error`).
3. Implement MaterialSwatched Event (1 day)
   - Define `MaterialSwatched` event variant in `QuiltEvent` enum.
   - Update `MaterialRegistry` to publish `MaterialSwatched` event upon successful transition to `Swatched` state.
   - Add/update integration tests for the `SwatchingActor` covering cut retrieval, embedding generation, swatch saving, registry update, and event publication.

**Demonstration:** Running `main` shows "Created X swatches with embeddings..." logs, material status updates to `Swatched`, and `MaterialSwatched` events are published. Integration tests verify embedding creation and storage.
