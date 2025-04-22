# Semantic Search

This workstream focuses on implementing vector similarity search capabilities using the generated embeddings.

### Milestone 12: "Implement Basic Semantic Search"

**Goal:** Implement basic vector similarity search using the stored swatches and embeddings.
**Implementation Time:** ~2-3 days

1. Integrate Vector Search Extension (1 day)
   - Integrate `sqlite-vec` or a similar vector search extension with the SQLite setup (`src/db.rs`).
   - Update the `swatches` table schema and `SqliteSwatchRepository` (from M10) to store and index the embedding vectors correctly for search.
2. Implement Search Logic (1-2 days)
   - Add a function or method (e.g., in `SqliteSwatchRepository` or a new query service) to perform vector similarity search.
   - Implement a basic query interface (e.g., a new command-line argument or internal function) that takes query text, generates its embedding, and performs the search.
   - Add tests for the vector search functionality.

**Demonstration:** Running the application with a search query performs semantic search and returns relevant swatch IDs/content based on vector similarity.
