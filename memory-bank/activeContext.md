# Active Context

## Current Focus

The project is currently implementing Milestone 2: "Discovery Actor Uses Scanner for Single Directory". We've successfully completed Milestone 1 with the actor system implementation and are now focusing on integrating the existing DirectoryScanner with the DiscoveryActor.

## Recent Changes

1. **Actor System Implementation**:

   - Integrated Actix as the actor framework for Quilt
   - Created the base actor module with common message types (Ping, Shutdown)
   - Implemented DiscoveryActor with proper lifecycle management
   - Added structured logging with env_logger
   - Set up proper Actix/Tokio runtime integration using #[actix::main]
   - Created a modular actor organization with dedicated namespaces (src/actors, src/discovery)
   - Successfully tested the actor system with basic message passing

2. **Dependency Management**:

   - Added Actix 0.13.1 to dependencies
   - Set up logging infrastructure with log 0.4.20 and env_logger 0.11.8
   - Updated thiserror to version 2.0.12
   - Ensured compatibility between all dependencies

3. **Architecture Documentation**:

   - Updated implementation plan to mark Milestone 1 as completed
   - Documented the actor system architecture
   - Updated progress tracking in memory-bank

4. **Material Repository Implementation**:

   - Completed the thread-safe in-memory store using `Arc<RwLock<HashMap<...>>>`
   - Implemented material state tracking with validation (Discovered → Cut → Swatched, with Error transitions)
   - Added CRUD operations with idempotence and state validation
   - Created comprehensive test suite for all repository functionality
   - Integrated with Tokio's async runtime for better compatibility with the actor model
   - Added Default implementation for MaterialRepository to meet Clippy standards

5. **Message Channel System Implementation**:

   - Defined the `MaterialMessage` enum with five variants (Discovered, Cut, Swatched, Error, Shutdown)
   - Implemented channel system with fixed capacity (100 messages) to provide natural backpressure
   - Created extension traits for ergonomic message handling and error management
   - Added comprehensive test suite including integration tests for pipeline message flow
   - Added detailed documentation for the channel system architecture and usage patterns
   - Optimized message size by using only material IDs for Cut and Swatched messages

6. **CI/CD and Quality Control**:
   - Implemented GitHub Actions workflow for PR validation
   - Added rustfmt configuration for consistent code formatting
   - Configured Clippy for static code analysis with custom rules
   - Created testing documentation and standards
   - Added comprehensive test utilities and examples
   - Ensured all tests pass consistently across the codebase

## Active Decisions and Considerations

### Key Architectural Decisions

- **Using Actix for the actor system** to leverage its mature actor model implementation
- **Organizing actors in dedicated modules** for better code organization and maintainability
- **Using Tokio for async runtime** underlying the actor system
- **Structured logging with env_logger** for better debugging and monitoring
- **Using direct actor-to-actor communication** via message passing
- **Treating the Repository as a standalone component** rather than an actor itself
- **Planning for worker pools** in stages that require horizontal scaling (particularly Labeling)
- **Using Tokio's async primitives** for thread-safe repository access and actor communication
- **Fixed channel capacity (100)** to balance memory usage and provide natural backpressure
- **Minimizing message size** by passing only material IDs between stages when appropriate

### Open Questions

1. **Discovery Actor Integration**:

   - How to efficiently integrate the DirectoryScanner with the DiscoveryActor?
   - What's the best way to handle configuration for directory scanning?

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

1. Integrate DirectoryScanner with DiscoveryActor:

   - Wrap DirectoryScanner in the actor interface
   - Add configuration for target directory
   - Implement material creation from scanned files

2. Add material processing logic:

   - Log discovered materials with metadata
   - Implement material state tracking

3. Connect to message channel system:

   - Utilize existing MaterialMessage enum types
   - Configure Tokio mpsc channels
   - Implement channel registration and connection

4. Expand test coverage:
   - Add integration tests for DiscoveryActor with DirectoryScanner
   - Test error handling scenarios
   - Ensure CI pipeline catches regressions

### Medium-term Goals (Next Sprint)

1. Implement the Cutting Actor to process discovered materials
2. Create the Labeling Actor skeleton
3. Add material processing pipeline with message passing
4. Begin work on the vector storage component
5. Revisit Swatch implementation with more concrete requirements
