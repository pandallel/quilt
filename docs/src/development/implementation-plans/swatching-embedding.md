# Swatching and Embedding

This workstream focuses on the logic within the Swatching Actor to generate embeddings for text cuts and create Swatch objects.

### Milestone: "Implement SQLite Swatch Repository (Persistence Only)" [COMPLETED]

**Goal:** Implement the basic SQLite persistence layer for swatches, storing embeddings as `BLOB`s. **Vector search will be handled in a later milestone.**
**Implementation Time:** ~1-2 days

1.  **Define Schema (`src/db.rs`)**: Define `swatches` table schema with columns for ID, cut ID, material ID, embedding (`BLOB`), model name, model version, created_at, dimensions, metadata, etc. Include foreign keys to `materials` and `cuts` (with cascade delete). Update `init_memory_db` to create this table.
2.  **Implement Repository (`src/swatching/sqlite_repository.rs`)**:
    - Create `SqliteSwatchRepository` struct holding a `SqlitePool`.
    - Implement the `SwatchRepository` trait.
    - Handle `Vec<f32>` <=> `BLOB` serialization/deserialization for the `embedding` column. Converting Vec<f32> ↔ BLOB via bytemuck or bincode is straightforward and zero‑copy if you choose.
    - Implement basic CRUD methods (`save_swatch`, `get_swatch_by_id`, `delete_swatch`, etc.).
    - **Stub out** the `search_similar` method (e.g., return `Err(OperationFailed)` or an empty `Vec`).
3.  **Unit Tests (`src/swatching/sqlite_repository.rs`)**:
    - Add comprehensive unit tests for all implemented CRUD operations.
    - Include a test confirming `search_similar` is correctly stubbed/unimplemented.

**Demonstration:** Unit tests for `SqliteSwatchRepository` pass, verifying CRUD operations and embedding serialization/deserialization. `search_similar` is confirmed as unimplemented.

### Milestone: "Integrate Swatch Repository into Actor System" [COMPLETED]

**Goal:** Connect the implemented `SqliteSwatchRepository` to the `QuiltOrchestrator` and `SwatchingActor`.
**Implementation Time:** ~0.5 - 1 day

1.  **Update `QuiltOrchestrator`**: Initialize `SqliteSwatchRepository` (e.g., in `with_sqlite`) and store it (e.g., `Option<Arc<dyn SwatchRepository>>`).
2.  **Update `SwatchingActor`**: Modify constructor to accept and store `Arc<dyn SwatchRepository>`.
3.  **Update `QuiltOrchestrator`**: Inject the stored repository into `SwatchingActor` during `initialize_actors`.
4.  **Update Tests**: Modify `SwatchingActor` tests to provide a mock or real repository instance.

**Demonstration:** Application compiles and runs. `QuiltOrchestrator` successfully injects the `SqliteSwatchRepository` instance into `SwatchingActor`. Tests for `SwatchingActor` pass.

### Milestone: "Implement Swatching Actor Logic & Event" [COMPLETED]

**Goal:** Connect the `SwatchingActor` to retrieve cuts, generate **actual embeddings**, create swatches, store them via the repository, update registry status, and publish the `MaterialSwatched` event.  
**Implementation Time:** ~3–4 days

---

## Component Roles & Responsibilities

| Component             | Responsibility                                                                                         |
| --------------------- | ------------------------------------------------------------------------------------------------------ |
| **CutsRepository**    | Data access: fetch raw `Cut` objects from storage.                                                     |
| **EmbeddingService**  | (New) Encapsulate embedding logic: given text, return `Vec<f32>` via `fastembed` + HuggingFace model.  |
| **SwatchRepository**  | Data access: persist created `Swatch` objects to storage.                                              |
| **SwatchingActor**    | Orchestrator for a single "swatch" job: coordinates fetching, embedding, swatch creation, persistence. |
| **MaterialRegistry**  | Update material status (Swatched / Error) and publish `MaterialSwatched` event.                        |
| **QuiltOrchestrator** | Top-level actor supervisor: injects dependencies and routes material IDs to `SwatchingActor`.          |

---

## Implementation Plan

### 1. Define & Wire Dependencies (Day 1) [COMPLETED]

1. ~~Add `EmbeddingService` interface~~ [DONE]
   ```rust
   pub trait EmbeddingService {
     fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError>;
   }
   ```
2. ~~Implement `HfEmbeddingService` calling `fastembed` + HF model URL.~~ [DONE]
3. ~~Refactor DI in `QuiltOrchestrator` to instantiate and pass:~~ [DONE]
   - `CutsRepository`
   - `EmbeddingService`
   - `SwatchRepository`
   - `MaterialRegistry`

### 2. Fetch & Embed Cuts (Day 2) [COMPLETED]

1. ~~In `SwatchingActor.process(material_id)`:~~ [DONE]
   - ~~`let cuts = cuts_repo.get_by_material(material_id)?;`~~
   - ~~For each `cut`:~~
     ```rust
     let embedding = embedding_service.embed(&cut.content)?;
     ```

### 3. Create & Persist Swatches (Day 2–3) [COMPLETED]

1. ~~Map each `Cut` → `Swatch`:~~ [DONE]
   ```rust
   struct Swatch {
     cut_id: Uuid,
     embedding: Vec<f32>,
     created_at: DateTime<Utc>,
   }
   ```
2. ~~Save via `swatch_repo.save(&swatch)?;`~~ [DONE]

### 4. Update Registry & Publish Event (Day 3) [COMPLETED]

1. ~~On success:~~ [DONE]
   ```rust
   material_registry.mark_swatched(material_id)?;
   ```
2. ~~On failure:~~ [DONE]
   ```rust
   material_registry.mark_error(material_id, error_info)?;
   ```
3. ~~New event variant in `QuiltEvent`:~~ [DONE]
   ```rust
   enum QuiltEvent {
     MaterialSwatched { material_id: Uuid, timestamp: DateTime<Utc> },
     // …
   }
   ```

### 5. Integration & Test Coverage (Day 4) [COMPLETED]

1. **Unit tests** [DONE]

   - ~~Mock `EmbeddingService`, `CutsRepository`, `SwatchRepository` using mockall.~~
   - ~~Test happy path & embedding failures.~~
   - ~~Tests should be quiet by default (avoid println statements) and rely on assertions.~~
   - ~~Use descriptive test names and assertion messages for better failure reporting.~~
   - ~~For tests requiring special case handling (e.g., model loading failures), use `eprintln!` for warnings.~~

2. **Integration tests** [DONE]

   - ~~In-memory DB: run actor end-to-end, assert cuts retrieved, swatches persisted, event emitted.~~
   - ~~Test all expected failure modes and verify proper error propagation.~~

3. **Test robustness** [DONE]
   - ~~Ensure tests handle edge cases: empty text, very long text, special characters.~~
   - ~~Include tests for retry mechanisms and concurrent operations.~~

---

## Completion Status

The SwatchingActor implementation has been completed, including:

1. **Event-driven processing**: The actor listens for MaterialCut events and processes them asynchronously.
2. **Embedding generation**: The actor uses the EmbeddingService to generate vector embeddings for cut content.
3. **Swatch creation and storage**: The actor creates Swatch objects and persists them via the SwatchRepository.
4. **Status management**: The actor updates material status in the registry (Swatched or Error).
5. **Event publishing**: The actor publishes MaterialSwatched events upon successful processing.
6. **Error handling**: The actor properly handles errors during processing and updates material status accordingly.
7. **Testing**: Comprehensive unit and integration tests verify the functionality of all components.

All components are properly integrated with the dependency injection system and tests are passing.

## Next Steps

1. **Vector search optimization**: Implement efficient approximate nearest-neighbor search algorithms.
2. **Advanced embedding strategies**: Support for different embedding models and strategies.
3. **Metrics and monitoring**: Add tracking of embedding quality, processing time, and error rates.
4. **Performance tuning**: Optimize batch processing, caching, and parallel embedding generation.

**Demonstration:** Running `main` shows "Created X swatches with embeddings..." logs. Material status updates to `Swatched`. `MaterialSwatched` events are published. Integration tests verify embedding creation and storage in the database.
