# Active Context

## Current Focus

The project is currently in the initial implementation phase, focusing on the **Core Material Processing Pipeline** (Milestone 1). We've established the Material Repository and completed the message channel system. The next focus is implementing the worker components.

## Recent Changes

1. **Architecture Documentation**:

   - Defined the simplified actor model with direct messaging between workers
   - Established the role of the Material Repository as a separate thread-safe data store
   - Documented the flow of messages and state transitions

2. **Material Repository Implementation**:

   - Completed the thread-safe in-memory store using `Arc<RwLock<HashMap<...>>>`
   - Implemented material state tracking with validation (Discovered → Cut → Swatched, with Error transitions)
   - Added CRUD operations with idempotence and state validation
   - Created comprehensive test suite for all repository functionality
   - Integrated with Tokio's async runtime for better compatibility with the actor model
   - Added Default implementation for MaterialRepository to meet Clippy standards

3. **Message Channel System Implementation**:

   - Defined the `MaterialMessage` enum with five variants (Discovered, Cut, Swatched, Error, Shutdown)
   - Implemented channel system with fixed capacity (100 messages) to provide natural backpressure
   - Created extension traits for ergonomic message handling and error management
   - Added comprehensive test suite including integration tests for pipeline message flow
   - Added detailed documentation for the channel system architecture and usage patterns
   - Optimized message size by using only material IDs for Cut and Swatched messages

4. **Material Data Structure**:

   - Reviewed existing Material struct implementation and found it sufficient for current needs
   - Deferred Swatch data structure implementation to a later milestone
   - Updated implementation plan to reflect these decisions

5. **Documentation System**:

   - Set up mdBook for project documentation
   - Added admonish extension for enhanced documentation features
   - Created initial documentation structure and content
   - Updated implementation plan to track progress

6. **CI/CD and Quality Control**:
   - Implemented GitHub Actions workflow for PR validation
   - Added rustfmt configuration for consistent code formatting
   - Configured Clippy for static code analysis with custom rules
   - Created testing documentation and standards
   - Added comprehensive test utilities and examples
   - Ensured all tests pass consistently across the codebase

## Active Decisions and Considerations

### Key Architectural Decisions

- **Using direct actor-to-actor communication** via Tokio channels instead of a centralized dispatcher
- **Treating the Repository as a standalone component** rather than an actor itself
- **Planning for worker pools** in stages that require horizontal scaling (particularly Labeling)
- **Using Tokio's async primitives** for thread-safe repository access and actor communication
- **Deferring Swatch implementation** until we have more concrete requirements from the Cutting Worker
- **Fixed channel capacity (100)** to balance memory usage and provide natural backpressure
- **Minimizing message size** by passing only material IDs between stages when appropriate

### Open Questions

1. **Worker Implementation**:

   - How to efficiently implement worker loops with proper error handling?
   - What's the best approach for graceful shutdown propagation?

2. **Persistence Strategy**:

   - What persistence mechanism to use for the Material Repository?
   - How to handle recovery on startup?

3. **Embedding Integration**:

   - Which local embedding model to integrate first?
   - How to efficiently manage embedding resources?

4. **Vector Storage**:
   - What vector similarity algorithm best balances speed and recall?
   - How to efficiently store and retrieve embeddings?

## Next Steps

### Short-term Tasks (Current Sprint)

1. Implement the minimal actor framework:

   - Create the Discovery Worker with message handling loop
   - Implement the Cutting Worker with basic processing
   - Create the Labeling Worker skeleton
   - Add graceful shutdown propagation

2. Create the Discovery Worker:

   - Implement basic file system watching
   - Add material detection and registration
   - Connect to the Material Repository

3. Begin work on the Cutting Worker:

   - Implement basic document fragmentation
   - Define swatch content extraction
   - Create state transitions from Discovered to Cut

4. Expand test coverage:
   - Add more integration tests for worker components
   - Implement property-based testing for critical components
   - Ensure CI pipeline catches regressions
   - Standardize test patterns across the codebase

### Medium-term Goals (Next Sprint)

1. Complete the three worker implementations
2. Add basic file monitoring for real document processing
3. Implement intelligent document splitting strategies
4. Begin work on the vector storage component
5. Revisit Swatch implementation with more concrete requirements
