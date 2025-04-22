# Text Processing (Cutting)

This workstream handles the process of splitting discovered materials into smaller text chunks (cuts).

### ✅ Milestone 6: "Material Text Cutting Implementation"

**Goal:** Implement the text-based document cutting functionality within the Cutting Actor's processor task
**Implementation Time:** ~4 days (Completed)
**Status:** ✅ Completed

1. ✅ Integrate text-splitter crate (1 day)

   - ✅ Add text-splitter dependency to Cargo.toml
   - ✅ Create TextCutter implementation using TextSplitter
   - ✅ Configure with default token sizes (target: 300, min: 150, max: 800)

2. ✅ Integrate with TextSplitter for content chunking (within Processor Task)

   - ✅ Use TextCutter to split file content into chunks (within Processor Task)
   - ✅ Implement error handling with fallback strategy (within Processor Task)
   - ✅ Handle backpressure: Listener task uses `await send()` and handles `EventBus` lag (`RecvError::Lagged`)

3. ✅ Implement State Update and Event Publishing (within Material Registry) (1 day)

   - ✅ Update material status in registry (Discovered → Cut or Discovered → Error) via `update_material_status`.
   - ✅ Implement MaterialCut event creation and publishing _within the registry_ upon successful transition to `Cut` state.
   - ✅ Implement ProcessingError event creation and publishing _within the registry_ upon transition to `Error` state, inferring the stage (`Cutting`) from the previous state.
   - ✅ Implement error reporting for failed cuts via the `Error` state transition.
   - ✅ Add metrics for cut creation (count, size distribution)

**Demonstration:** Running `main` shows "Created X cuts from Y materials using TextSplitter" with detailed metrics in logs
