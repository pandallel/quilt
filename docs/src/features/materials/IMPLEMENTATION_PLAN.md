# Quilt Implementation Plan

## Phase 1: Core Material System

### Step 1: Basic Material Types (1-2 days) ✅

- [x] Implement basic Material struct
  - Using `time` for timestamps (modern alternative to chrono)
  - Using `cuid2` for IDs (more readable, sortable alternative to UUID)
  - Added constructor method `Material::new()`
- [x] Define MaterialStatus enum
  - Implemented states: Discovered, Valid, Invalid
  - Added error message support for Invalid state
- [x] Create MaterialFileType enum
  - Currently supporting Markdown files
  - Extensible for future file types
- [x] Write unit tests for basic types
  - Material creation tests
  - Status transition tests
  - CUID uniqueness tests
  - Timestamp ordering tests

### Step 2: Material Registry (2-3 days)

- [ ] Create MaterialRegistry struct
- [ ] Implement basic CRUD operations for materials
- [ ] Add in-memory storage for materials
- [ ] Write tests for registry operations

### Step 3: Directory Scanner (2-3 days)

- [ ] Implement basic file system operations
- [ ] Create directory scanning functionality
- [ ] Filter for markdown files
- [ ] Add tests for directory scanning

### Step 4: Material Validation (2-3 days)

- [ ] Implement MaterialValidator trait
- [ ] Create basic markdown validation rules
- [ ] Add validation status tracking
- [ ] Write validation tests

### Step 5: Event System (2-3 days)

- [ ] Define MaterialEvent enum
- [ ] Implement basic event emission
- [ ] Create event subscribers
- [ ] Add event handling tests

## Phase 2: Enhanced Features

### Step 6: Error Handling (1-2 days)

- [ ] Implement custom error types
- [ ] Add error recovery mechanisms
- [ ] Improve error reporting
- [ ] Write error handling tests

### Step 7: File Watching (2-3 days)

- [ ] Add file system watching
- [ ] Implement change detection
- [ ] Create update mechanisms
- [ ] Add file watching tests

### Step 8: Performance Optimization (2-3 days)

- [ ] Add async support
- [ ] Implement batch processing
- [ ] Optimize file operations
- [ ] Write performance tests

## Implementation Notes

### Starting Point

✅ Completed Step 1 with modern choices:

- Using `time` instead of `chrono` for better timezone handling and type safety
- Using `cuid2` instead of `uuid` for more readable, sortable IDs
- Implemented with clean module structure in `src/materials/`

### Development Approach

1. Each step will follow Test-Driven Development (TDD)
2. We'll implement the simplest working version first
3. Each feature will be developed in isolation before integration
4. Documentation will be written alongside code

### Key Principles

- Keep it simple initially
- Focus on correctness before optimization
- Write clear, idiomatic Rust code
- Maintain good test coverage

### Dependencies

Current dependencies:

- cuid2 = "0.1.2" (for unique IDs)
- time = { version = "0.3", features = ["serde", "macros"] } (for timestamps)

Future dependencies will be added as needed:

- tokio (for async)
- anyhow (for error handling)
- notify (for file watching)
- etc.

### Next Implementation Task

Moving on to Step 2: Material Registry

- Will implement CRUD operations for materials
- Need to decide on storage strategy (in-memory vs persistent)
- Consider using a trait-based approach for storage backends
