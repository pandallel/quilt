# Active Context

## Current Focus

The project has successfully completed Milestone 2: "Discovery Actor Uses Scanner for Single Directory". We're now moving on to Milestone 3: "Discovery Actor Sends Material Messages" to establish message passing between actors in the pipeline.

## Recent Changes

1. **Discovery System Integration**:

   - Successfully integrated the DirectoryScanner with the DiscoveryActor
   - Renamed `ScanConfig` to `DiscoveryConfig` for better naming consistency
   - Enhanced logging with repository statistics showing total materials count
   - Created configuration structure for scanner parameters
   - Implemented command-line argument parsing for discovery configuration
   - Added proper error handling for directory validation and scanning
   - Added test coverage for discovery functionality

2. **Runtime Improvements**:

   - Fixed actix runtime usage by replacing `#[actix_rt::main]` with `#[actix::main]`
   - Ensured compatibility with current actix ecosystem
   - Tested actor lifecycle with proper async runtime

3. **Actor System Refactoring**:

   - Refactored main.rs to use a dedicated QuiltOrchestrator for actor coordination
   - Moved actor initialization, communication, and shutdown logic out of main.rs
   - Created a modular approach for managing the actor lifecycle with clear separation of concerns
   - Improved error handling and message flow management
   - Better aligned implementation with the actor model's best practices

4. **Actor System Implementation**:

   - Integrated Actix as the actor framework for Quilt
   - Created the base actor module with common message types (Ping, Shutdown)
   - Implemented DiscoveryActor with proper lifecycle management
   - Added structured logging with env_logger
   - Set up proper Actix/Tokio runtime integration using #[actix::main]
   - Created a modular actor organization with dedicated namespaces (src/actors, src/discovery)
   - Successfully tested the actor system with basic message passing

5. **Dependency Management**:

   - Added Actix 0.13.1 to dependencies
   - Set up logging infrastructure with log 0.4.20 and env_logger 0.11.8
   - Updated thiserror to version 2.0.12
   - Ensured compatibility between all dependencies

6. **Architecture Documentation**:

   - Updated implementation plan to mark Milestone 2 as completed
   - Documented the discovery system architecture
   - Updated progress tracking in memory-bank

7. **Material Repository Implementation**:

   - Completed the thread-safe in-memory store using `Arc<RwLock<HashMap<...>>>`
   - Implemented material state tracking with validation (Discovered → Cut → Swatched, with Error transitions)
   - Added CRUD operations with idempotence and state validation
   - Created comprehensive test suite for all repository functionality
   - Integrated with Tokio's async runtime for better compatibility with the actor model
   - Added Default implementation for MaterialRepository to meet Clippy standards

8. **Message Channel System Implementation**:

   - Defined the `MaterialMessage` enum with five variants (Discovered, Cut, Swatched, Error, Shutdown)
   - Implemented channel system with fixed capacity (100 messages) to provide natural backpressure
   - Created extension traits for ergonomic message handling and error management
   - Added comprehensive test suite including integration tests for pipeline message flow
   - Added detailed documentation for the channel system architecture and usage patterns
   - Optimized message size by using only material IDs for Cut and Swatched messages

9. **CI/CD and Quality Control**:
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
- **Enhanced logging with repository statistics** to monitor system state

### Open Questions

1. **Message Passing Strategy**:

   - How to efficiently connect DiscoveryActor to the CuttingActor?
   - What's the best approach for handling backpressure between stages?
   - How to properly serialize/deserialize materials between actors?

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

1. Implement the CuttingActor:

   - Create basic CuttingActor structure
   - Add message reception from Discovery
   - Implement basic cut creation logic
   - Add file content extraction

2. Establish channel connections between actors:

   - Create CutMessage types
   - Implement channel connections
   - Add proper error handling
   - Configure DiscoveryActor to forward materials

3. Add document splitting strategies:

   - Create text extraction pipeline
   - Implement basic cutting strategies
   - Add metadata extraction for cuts

4. Expand test coverage:
   - Add integration tests for message passing between actors
   - Test backpressure handling
   - Ensure proper error propagation

### Medium-term Goals (Next Sprint)

1. Complete the Cutting Actor implementation
2. Create the Labeling Actor skeleton
3. Add material processing pipeline with message passing
4. Begin work on the CutsRepository component
5. Revisit Swatch implementation with more concrete requirements
