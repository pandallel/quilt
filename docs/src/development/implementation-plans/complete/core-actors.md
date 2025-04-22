# Core Actors and Communication

This workstream focuses on setting up the fundamental actor structures and the event bus for communication.

### ✅ Milestone 1: "Actor System Logs Startup"

**Goal:** Establish the basic actor framework with proper initialization
**Implementation Time:** 2-3 days
**Status:** Completed

1. ✅ Setup actor framework foundation (1-2 days)

   - ✅ Define basic actor trait and message types
     - ✅ Created common message types (Ping, Shutdown) in actors module
     - ✅ Implemented actor-specific message types (StartDiscovery)
     - ✅ Ensured thread safety and proper ownership in async context
   - ✅ Create actor system initialization patterns
   - ✅ Ensure proper Actix/Tokio runtime integration
     - ✅ Used Actix runtime with #[actix::main] in main.rs
     - ✅ Added proper async actor initialization and message handling

2. ✅ Add logging infrastructure (1 day)
   - ✅ Setup structured logging with env_logger
   - ✅ Created actor lifecycle logging (start/stop/message events)

**Demonstration:** Running `main` shows "Actor system started" with proper configuration in logs, and the DiscoveryActor successfully handles messages.

### ✅ Milestone 3: "Event Bus and Material Registry Foundation"

**Goal:** Establish the core communication infrastructure
**Implementation Time:** 2-3 days
**Status:** Completed

1. ✅ Implement basic Event Bus (1 day)

   - ✅ Create central event bus using `tokio::sync::broadcast` channels
   - ✅ Implement simple event types (MaterialDiscovered event only)
   - ✅ Add logging for event publishing and subscription
   - ✅ Create basic tests that verify event transmission

2. ✅ Create Material Registry prototype (1-2 days)
   - ✅ Implement basic registry that works alongside existing Repository
   - ✅ Add minimal event publishing for material discovery
   - ✅ Create simple validation of events using logging
   - ✅ Keep the existing Repository functionality intact

**Demonstration:** Running `main` shows "Event Bus initialized" in logs and demonstrates events flowing with log messages

### ✅ Milestone 5: "Basic Cutting Actor Creation"

**Goal:** Create a minimal Cutting Actor that listens for events and sets up internal backpressure queue
**Implementation Time:** 2-3 days
**Status:** Completed

1. ✅ Create Cutting Actor skeleton (1-2 days)

   - ✅ Implement simple actor that subscribes to MaterialDiscovered events from the shared `EventBus`
   - ✅ Set up internal bounded `mpsc` channel (sender/receiver pair)
   - ✅ Spawn internal 'Listener Task': receives events from `EventBus`, filters for `MaterialDiscovered`, logs receipt, tries sending to internal `mpsc` queue (no blocking/backpressure handling yet)
   - ✅ Spawn internal 'Processor Task': receives from internal `mpsc` queue, logs receipt (no processing yet)
   - ✅ Add logging for received events on both tasks
   - ✅ Create basic actor lifecycle (start/stop) including task cleanup

2. ✅ Set up actor monitoring (1 day)
   - ✅ Add heartbeat logging for the actor
   - ✅ Implement basic health checks
   - ✅ Add subscription metrics
   - ✅ Create actor configuration structure

**Demonstration:** Running `main` shows "Cutting Actor received X MaterialDiscovered events" in logs without processing them

### ✅ Milestone 8: "Basic Swatching Actor Creation"

**Goal:** Create a minimal Swatching Actor that listens for cut events and sets up internal backpressure queue
**Implementation Time:** 2-3 days
**Status:** ✅ Completed

1. ✅ Create Swatching Actor skeleton (1-2 days)

   - ✅ Implement simple actor that subscribes to MaterialCut events from the shared `EventBus`
   - ✅ Set up internal bounded `mpsc` channel
   - ✅ Spawn internal 'Listener Task': receives events, filters for `MaterialCut`, logs receipt, tries sending to internal queue
   - ✅ Spawn internal 'Processor Task': receives from internal queue, logs receipt (no processing yet)
   - ✅ Add logging for received events on both tasks
   - ✅ Create basic actor lifecycle management including task cleanup

2. ✅ Integrate with Orchestrator
   - ✅ Add SwatchingActor to the QuiltOrchestrator initialization
   - ✅ Include in the shutdown sequence with proper order
   - ✅ Ensure event flow from CuttingActor to SwatchingActor
   - ✅ Add comprehensive tests for actor lifecycle

**Demonstration:** Running `main` shows "Swatching Actor received X MaterialCut events" in logs without processing them
