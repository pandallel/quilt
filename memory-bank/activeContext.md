# Active Context

## Current Focus

The project has completed Milestone 2: "Discovery Actor Uses Scanner for Single Directory" and is now transitioning to Milestone 3: "Discovery Actor Sends Material Messages" to establish message passing between actors.

## Current Implementation Status

The codebase currently has these key components implemented:

1. **Actor System**:

   - Common actor module with Ping and Shutdown messages
   - DiscoveryActor with lifecycle management
   - QuiltOrchestrator implementing the Orchestrator pattern
   - Proper Actix/Tokio runtime integration with #[actix::main]

2. **Material Repository**:

   - Thread-safe in-memory store using `Arc<RwLock<HashMap<...>>>`
   - Material state tracking with proper validation (Discovered → Cut → Swatched → Error)
   - CRUD operations with idempotence and state transition validation

3. **Message System**:

   - Actor-specific message types for clear communication contracts
   - Typed message response handling with proper error types
   - Leveraging Actix's built-in mailbox and message handling
   - Direct actor-to-actor communication pattern

4. **Discovery System**:
   - DirectoryScanner that finds files in configured directories
   - DiscoveryActor that wraps the scanner in the actor interface
   - DiscoveryConfig for scanner parameters
   - Support for excluding patterns and hidden files

## Recent Changes

1. **Actor System Implementation**:

   - Added QuiltOrchestrator for centralized actor management
   - Configured proper lifecycle with validation and timeouts
   - Refactored main.rs to use the orchestrator pattern

2. **Pipeline Preparation**:
   - Defined message channel structure for the pipeline
   - Created channel connection interfaces
   - Added type aliases for different pipeline stages

## Active Decisions and Considerations

### Key Architectural Decisions

- Using Actix for the actor system and Tokio for async runtime
- Organizing actors in dedicated modules for better maintainability
- Using the Orchestrator pattern to manage actor lifecycle
- Using Tokio's async primitives for thread-safe access and communication
- Fixed channel capacity (100) to balance memory usage with throughput
- Minimizing message size by passing only material IDs for later pipeline stages

### Open Questions

1. **Message Passing Implementation**:

   - How to efficiently design message types between actors
   - Best approach for handling backpressure between actor stages
   - How to properly handle long-running operations in actors

2. **Persistence & Embedding**:
   - Persistence mechanism for the Material Repository
   - Which local embedding model to integrate first
   - Vector similarity algorithm balancing speed and recall

## Next Steps

### Short-term Tasks (Current Sprint)

1. Implement message passing from DiscoveryActor to CuttingActor:

   - Define proper message types for actor communication
   - Add message handling in the CuttingActor
   - Update the orchestrator to manage both actors

2. Begin CuttingActor implementation:
   - Create basic structure with message handling
   - Implement content extraction
   - Design and implement cutting strategies
