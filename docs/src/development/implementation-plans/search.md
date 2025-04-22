# Semantic Search

This workstream focuses on implementing vector similarity search capabilities using the generated embeddings.

### Milestone: "Implement Vector Search with `sqlite-vec`"

**Goal:** Integrate `sqlite-vec` and implement vector similarity search functionality on the stored swatch embeddings.
**Implementation Time:** ~2-3 days

1.  **Dependencies & DB Setup (`Cargo.toml`, `src/db.rs`)**:
    - Add `sqlite-vec` crate dependency.
    - Update `init_memory_db` function:
      - Load/enable the `sqlite-vec` extension.
      - Create the `vss_swatches` virtual table using `vss0` indexing the `embedding` column from the `swatches` table (use appropriate dimensions).
2.  **Update Repository (`src/swatching/sqlite_repository.rs`)**:
    - Modify `save_swatch`/`save_swatches_batch` to also insert the embedding into the `vss_swatches` virtual table.
    - Modify `delete_swatch`/`delete_swatches_by_cut_id`/`delete_swatches_by_material_id` to also remove corresponding entries from `vss_swatches`.
    - Implement the `search_similar` method using `vss_search` on the `vss_swatches` virtual table.
3.  **Unit Tests (`src/swatching/sqlite_repository.rs`)**:
    - Add/update unit tests specifically verifying the `search_similar` functionality and interactions with the virtual table.
4.  **(Optional) Basic Query Interface**: Implement a basic way to test search (e.g., a simple command-line flag or internal function) that takes query text, generates its embedding (using logic from "Swatching Actor Logic" milestone), and performs the search via the repository.

**Demonstration:** Unit tests for `SqliteSwatchRepository` pass, verifying vector search functionality. Optionally, a basic query interface allows for manual search testing.
