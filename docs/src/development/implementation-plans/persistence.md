# Persistence and Data Structures

This workstream defines data structures (like Swatches) and handles the persistence of materials, cuts, swatches, and events, including the transition to SQLite.

### ✅ Milestone 7: "Cuts Repository Implementation"

**Goal:** Store processed cuts with event integration
**Implementation Time:** 2-3 days
**Status:** ✅ Completed

1. ✅ Implement CutsRepository (1-2 days)

   - ✅ Create in-memory storage for cuts
   - ✅ Implement CRUD operations
   - ✅ Add integration with Registry
   - ✅ Create comprehensive tests

2. ✅ Connect Cutting Actor to repository (1 day)
   - ✅ Add repository interaction in actor
   - ✅ Implement proper error handling
   - ✅ Add storage metrics and logging
   - ✅ Create validation of stored cuts

**Demonstration:** Running `main` logs "Stored X cuts in repository" with metrics on storage operations

### ✅ Milestone 7.5: SQLite Repository Implementation

#### Prerequisites: Repository Trait Refactoring

1. ✅ Rename and validate Material Repository
   - ✅ Rename `struct MaterialRepository` to `struct InMemoryMaterialRepository`.
   - ✅ Update all external usages.
   - ✅ Validate with `cargo check` and `cargo test`.
2. ✅ Add trait for Material Repository
   - ✅ Add `async-trait` dependency.
   - ✅ Define `trait MaterialRepository` in `repository.rs` using `#[async_trait]`.
   - ✅ Implement the trait for `InMemoryMaterialRepository`.
   - ✅ Validate: ensure `cargo check` and `cargo test` pass.
3. ✅ Update the Registry to use the trait
   - ✅ Update `MaterialRegistry` to use `Arc<dyn MaterialRepository>`.
   - ✅ Update dependent code and tests.
   - ✅ Validate: ensure `cargo check` and `cargo test` pass.
4. ✅ Update Actor Dependencies
   - ✅ Update `Orchestrator`, `CuttingActor`, `DiscoveryActor` initialization and tests.
   - ✅ Validate: ensure `cargo check` and `cargo test` pass.
5. ✅ Move trait definitions to mod.rs
   - ✅ Move trait/error definitions to `mod.rs`.
   - ✅ Validate: ensure `cargo check` and `cargo test` pass.
6. ✅ Apply Repository Trait Pattern to CutsRepository
   - ✅ Define `trait CutsRepository` in appropriate location.
   - ✅ Update the `CuttingActor` to use the trait.
   - ✅ Validate: ensure `cargo check` and `cargo test` pass.

#### SQLite Implementation

1. ✅ Add SQLite dependencies

   - ✅ Add `sqlx` with `sqlite`, `runtime-tokio-rustls` and `time` features to Cargo.toml.
   - ✅ Add basic initialization and tests.

2. ✅ Create database setup module

   - ✅ Implement `init_memory_db()` function in new `src/db.rs` module
   - ✅ Set up schema definition for the `materials` table
   - ✅ Add test for database initialization

3. ✅ Implement SQLite Material Repository

   - ✅ Create `src/materials/sqlite_repository.rs` for `SqliteMaterialRepository`
   - ✅ Implement `MaterialRepository` trait with SQLite backend
   - ✅ Add proper row to struct conversion. Rename `ingested_at` column/field to `created_at`. Add `status_updated_at` and `updated_at` columns/fields.
   - ✅ Utilize `sqlx`\'s `"time"` feature for automatic `OffsetDateTime` encoding/decoding (removes manual parsing/formatting).
   - ✅ Add comprehensive tests comparing with in-memory implementation

4. ✅ Update app integration

   - ✅ Add module exports in `lib.rs`
   - ✅ Modify `QuiltOrchestrator` to include SQLite support
   - ✅ Add command-line option for selecting repository type
   - ✅ Implement fallback to in-memory repository if SQLite fails

5. ⚠️ Known Issues

   - ~~When running with `--dir=./src`, file paths are relative to the working directory, causing errors like `Failed to read file 'materials/types.rs': No such file or directory (os error 2)`~~ **Resolved (See Below)**
   - This issue was fixed by ensuring the `DiscoveryActor` resolves relative paths to absolute paths before registering materials. The `CuttingActor` now receives absolute paths via the `MaterialDiscoveredEvent`.

6. ✅ Implement SqliteCutsRepository
   - ✅ Create schema for cuts table in `src/db.rs`
   - ✅ Implement `SqliteCutsRepository` in `src/cutting/sqlite_repository.rs`
   - ✅ Add comprehensive tests for all repository operations
   - ✅ Update `QuiltOrchestrator` to use `SqliteCutsRepository` when in SQLite mode
   - ✅ Ensure proper foreign key constraints with materials table

**Demonstration:** Running `cargo run -- --dir=./src` uses SQLite by default for both materials and cuts, while `cargo run -- --dir=./src --in-memory` uses the original in-memory stores.

### ✅ Task: Refactor Material Timestamps

**Goal:** Improve timestamp tracking for materials by renaming `ingested_at` and adding status change and general update timestamps.
**Status:** Completed

1. ✅ **Add Timestamps:**
   - ✅ Renamed `ingested_at` field/column to `created_at`.
   - ✅ Added `status_updated_at` field/column.
   - ✅ Added `updated_at` field/column.
   - ✅ Updated `Material` struct, `Material::new()`, database schema (`src/db.rs`).
2. ✅ **Implement Update Logic:**
   - ✅ Updated `InMemoryMaterialRepository::update_material_status` to set `status_updated_at` and `updated_at` to `now` on success.
   - ✅ Updated `SqliteMaterialRepository::update_material_status` SQL to set `status_updated_at` and `updated_at` to `now` on success.
   - ✅ Updated `SqliteMaterialRepository::register_material` SQL to insert all three timestamps.
3. ✅ **Leverage `sqlx` Time Feature:**
   - ✅ Added `"time"` feature to `sqlx` dependency in `Cargo.toml`.
   - ✅ Refactored `SqliteMaterialRepository` to use automatic `OffsetDateTime` encoding/decoding via `sqlx`, removing manual parsing/formatting.
   - ✅ Updated tests in both repositories.

### ✅ Milestone 9: "Define Swatch Structures & Repository Trait"

**Goal:** Define the core data structures and persistence contract for swatches.
**Implementation Time:** ~1 day
**Status:** ✅ Completed

1. ✅ Define Swatch data structures (1 day)
   - ✅ Define `Swatch` struct in `src/swatching/swatch.rs` with necessary fields (id, cut_id, material_id, embedding `Vec<f32>`, model_name, model_version, dimensions, metadata).
   - ✅ Define `SwatchRepository` trait in `src/swatching/repository.rs` with comprehensive async CRUD and search methods using `#[async_trait::async_trait]`.
   - ✅ Add appropriate error types and result type alias for repository operations.
   - ✅ Update module exports in `src/swatching/mod.rs` to expose the new types.
   - ✅ Add `serde` and `serde_json` dependencies for metadata serialization.
   - ✅ Implement comprehensive unit tests for the `Swatch` struct.

**Demonstration:** Code compiles with the new `Swatch` type and `SwatchRepository` trait definitions, including similarity search method signatures for future implementation.

### Milestone 14: "Event and Data Persistence"

**Goal:** Ensure data and events persist between application runs (Focus on event log, repository persistence is largely covered by SQLite)
**Implementation Time:** 2-3 days

1. Implement event logging (1-2 days)

   - Add event serialization
   - Create file-based event log
   - Implement event replay on startup
   - Add validation for event consistency

2. Add repository persistence (1 day)
   - ⏩ Defer file-based serialization for repositories; SQLite handles persistence.
   - Add startup/shutdown procedures for event log
   - Create recovery mechanisms for event log

**Demonstration:** Stopping and restarting `main` shows "Recovered X events..." with intact data in SQLite and replayed events.
