# Material Cuts

This directory contains documentation about the Material Cut system in Quilt.

## Core Concepts

- [**Cut**](./cutting-behaviors.md) - The fundamental cut fragment from a Material
- [**Cutting Process**](./cutting-architecture.md) - How Materials are cut into meaningful pieces
- [**Error Handling**](./cutting-errors.md) - How errors are handled during cutting

## Implementation Status

The Material Cut system uses an event-driven, actor-based architecture. The current implementation:

1. Uses Actix for the actor system
2. Processes materials based on MaterialDiscovered events
3. Integrates with the Material Registry for state management
4. Supports asynchronous processing via Tokio

The CuttingActor:

- Subscribes to material discovery events
- Processes materials asynchronously
- Updates material status in the registry
- Publishes events for downstream actors
- Handles failures gracefully with detailed error reporting
