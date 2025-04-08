# Active Context

## Current Focus

The project is currently in the initial implementation phase, focusing on the **Core Material Processing Pipeline** (Milestone 1). We're establishing the core actor model architecture with a dedicated repository for material state management.

## Recent Changes

1. **Architecture Documentation**:

   - Defined the simplified actor model with direct messaging between workers
   - Established the role of the Material Repository as a separate thread-safe data store
   - Documented the flow of messages and state transitions

2. **Initial Implementation**:
   - Created basic project structure and documentation setup
   - Started implementing core domain models for Material and Swatch
   - Began work on the Material Repository implementation
3. **Documentation System**:
   - Set up mdBook for project documentation
   - Added admonish extension for enhanced documentation features
   - Created initial documentation structure and content

## Active Decisions and Considerations

### Key Architectural Decisions

- **Using direct actor-to-actor communication** via Tokio channels instead of a centralized dispatcher
- **Treating the Repository as a standalone component** rather than an actor itself
- **Planning for worker pools** in stages that require horizontal scaling (particularly Labeling)

### Open Questions

1. **Persistence Strategy**:
   - What persistence mechanism to use for the Material Repository?
   - How to handle recovery on startup?
2. **Embedding Integration**:

   - Which local embedding model to integrate first?
   - How to efficiently manage embedding resources?

3. **Vector Storage**:
   - What vector similarity algorithm best balances speed and recall?
   - How to efficiently store and retrieve embeddings?

## Next Steps

### Short-term Tasks (Current Sprint)

1. Complete the Material Repository implementation:
   - Implement thread-safe storage with proper state transitions
   - Add CRUD operations and idempotence checks
2. Implement the basic message channel system:
   - Define the `MaterialMessage` enum
   - Set up Tokio channels between stages
3. Create the Discovery Worker:

   - Implement basic file system watching
   - Add material detection and registration

4. Begin work on the Cutting Worker:
   - Implement basic document fragmentation
   - Define swatch content extraction

### Medium-term Goals (Next Sprint)

1. Complete all three worker implementations
2. Add basic file monitoring for real document processing
3. Implement intelligent document splitting strategies
4. Begin work on the vector storage component
