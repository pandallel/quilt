# Quilt Architecture – Lightweight Prototype

## Overview

Quilt is a local-first, modular memory and context engine designed to observe your work, process documents into meaningful pieces (swatches), embed them into a local searchable memory (swatch book), and assemble contextual spreads for queries. This document outlines a minimal, lightweight architecture that focuses on message flow, state management, and basic concurrency without delving into the internal work of individual actors.

---

## Architectural Decisions

### 1. Message Flow & Actor Communication

#### Channel Setup

- **Tokio mpsc Channels:**  
  Tokio does not provide an implicit default channel capacity—you must specify it when creating a channel. For this prototype, a moderate capacity (e.g., 100 messages) is chosen to balance simplicity and sufficient message buffering. This capacity should support our expected workload without overcomplicating the design.

#### Reliability

- **Fire-and-forget Messaging:**  
  The design uses a fire-and-forget messaging approach between actors (Discovery → Cutting → Labeling). This means that each actor sends its message to the next stage without waiting for an explicit acknowledgment.

---

### 2. State Management with the Material Repository

#### In-Memory Store

- **Thread-Safe Repository:**  
  An in-memory repository will be implemented using a thread-safe data structure (for example, an `Arc<RwLock<HashMap<...>>>`). This setup is acceptable for tracking the state of materials in this lightweight version.

#### Idempotence

- **State Checks Before Processing:**  
  Before processing a material at any stage, the actor must check the material's current state in the repository. This simple mechanism ensures that a material's state is updated only once per stage, preventing duplicate processing.

---

### 3. Concurrency & Backpressure

#### Worker Count

- **Single Worker per Stage:**  
  To keep the design minimal, start with one worker for each stage: one Discovery worker, one Cutting worker, and one Labeling worker. This meets initial workload expectations without adding complexity via dynamic scaling.

#### Backpressure

- **Bounded Channel Behavior in Tokio:**  
  Using bounded Tokio channels naturally introduces backpressure: when a channel reaches its capacity, send operations will block or yield errors, thereby throttling upstream processes. This inherent mechanism is sufficient for our prototype, and no additional explicit backpressure handling is needed.

---

## Minimal Workflow Summary

1. **Discovery Stage:**

   - Monitors specified folders for new or updated materials.
   - Registers new materials in the in-memory repository.
   - Sends a `Discovered` message through a Tokio mpsc channel (with a capacity of around 100 messages).

2. **Cutting Stage:**

   - Listens for `Discovered` messages.
   - Retrieves the corresponding material from the repository and checks its current state.
   - Processes the material into swatches and updates the state to _Cut_.
   - Forwards a `Cut` message to the Labeling stage.

3. **Labeling Stage:**
   - Listens for `Cut` messages.
   - Retrieves the material, verifies the state transition, and processes labeling/embedding operations.
   - Updates the material’s state to _Swatched_.
   - Continues with fire-and-forget messaging for simplicity.

---

## Conclusion

This lightweight architectural approach for Quilt emphasizes a straightforward message flow via fire-and-forget communication, basic in-memory state management with idempotence checks, and minimal concurrency with inherent backpressure using Tokio’s bounded channels. It provides a clear, modular foundation that can be expanded as requirements evolve.
