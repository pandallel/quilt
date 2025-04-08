# Active Context

## Current Focus

The project is currently in the initial implementation phase, focusing on the **Core Material Processing Pipeline** (Milestone 1). We've established the Material Repository and are now moving to implement the message channel system and worker components.

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

3. **Documentation System**:
   - Set up mdBook for project documentation
   - Added admonish extension for enhanced documentation features
   - Created initial documentation structure and content
   - Updated implementation plan to track progress

## Active Decisions and Considerations

### Key Architectural Decisions

- **Using direct actor-to-actor communication** via Tokio channels instead of a centralized dispatcher
- **Treating the Repository as a standalone component** rather than an actor itself
- **Planning for worker pools** in stages that require horizontal scaling (particularly Labeling)
- **Using Tokio's async primitives** for thread-safe repository access and actor communication

### Open Questions

1. **Message Channel System**:

   - What channel capacity is optimal for balancing memory usage and throughput?
   - How should error handling in message processing be implemented?

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

1. Implement the basic message channel system:

   - Define the `MaterialMessage` enum with all necessary variants
   - Set up Tokio channels between processing stages
   - Implement channel creation and management

2. Create the Discovery Worker:

   - Implement basic file system watching
   - Add material detection and registration
   - Connect to the Material Repository

3. Begin work on the Cutting Worker:

   - Implement basic document fragmentation
   - Define swatch content extraction
   - Create state transitions from Discovered to Cut

4. Implement the Swatch data structure:
   - Define the core model for document fragments
   - Create metadata associations between swatches and materials
   - Implement utilities for swatch creation and manipulation

### Medium-term Goals (Next Sprint)

1. Complete the three worker implementations
2. Add basic file monitoring for real document processing
3. Implement intelligent document splitting strategies
4. Begin work on the vector storage component
