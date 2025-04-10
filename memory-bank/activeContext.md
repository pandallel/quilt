# Active Context

## Current Focus

The project has completed Milestone 2: "Discovery Actor Uses Scanner for Single Directory" and is now transitioning to Milestone 3: "Event Bus and Material Registry Foundation" to establish the core event-driven communication infrastructure.

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

1. **Architecture Refinement**:

   - Updated architecture to use an event-driven approach with a Material Registry
   - Separated Material Registry (state management) from Material Repository (persistence)
   - Defined an Event Bus using Tokio broadcast channels
   - Created a clearer separation of concerns throughout the architecture

2. **Implementation Plan Adjustments**:
   - Restructured implementation plan to build the event-driven architecture incrementally
   - Created focused milestones with clear demonstrations for validation
   - Separated actor creation from processing logic for more tangible progress

## Active Decisions and Considerations

### Key Architectural Decisions

- Using Actix for actor lifecycle management and Tokio for concurrency primitives
- Implementing Event Bus using `tokio::sync::broadcast` channels
- Organizing actors around event subscription rather than direct message passing
- Using Registry as the central coordinator for state changes and event publishing
- Following idiomatic Rust patterns for concurrency safety
- Implementing atomic operations for state changes with event publishing

### Open Questions

1. **Actor Recovery and Supervision**:

   - How should actors handle failures and restart?
   - What's the proper supervisor hierarchy for our actors?
   - Should we implement circuit breakers for external dependencies?

2. **Event Ordering and Consistency**:

   - How do we ensure event ordering when needed?
   - Should we implement event versioning or sequence numbers?
   - How do we handle event replay for recovery scenarios?

3. **Backpressure Strategy**:

   - What are the optimal channel capacities for different event types?
   - How should actors handle backpressure when processing is slow?
   - Should we implement different priorities for different event types?

4. **State Recovery**:
   - How do we handle Registry state recovery after crashes?
   - Should we implement event sourcing for state reconstruction?
   - What's the strategy for state snapshots?

## Next Steps

### Short-term Tasks (Current Sprint)

1. Implement basic Event Bus:

   - Create central event bus using `tokio::sync::broadcast` channels
   - Implement simple event types (MaterialDiscovered event only)
   - Add logging for event publishing and subscription
   - Create basic tests that verify event transmission

2. Create Material Registry prototype:
   - Implement basic registry that works alongside existing Repository
   - Add minimal event publishing for material discovery
   - Create simple validation of events using logging
   - Keep the existing Repository functionality intact
