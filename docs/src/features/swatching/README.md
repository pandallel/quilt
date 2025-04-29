# Material Swatching

This directory contains documentation about the Material Swatching system in Quilt, which transforms semantic text chunks (Cuts) into vector embeddings (Swatches) for similarity search and retrieval.

## Core Concepts

- [**Swatch**](./swatching-behaviors.md) - A vector embedding representing the semantic content of a Cut, generated using embedding models
- [**Swatching Process**](./swatching-architecture.md) - The architecture and strategies for converting Cuts into Swatches using embedding models
- [**Error Handling**](./swatching-errors.md) - How errors are detected, reported, and recovered during the swatching process

## Implementation Status

The Material Swatching system uses an event-driven, actor-based architecture. The current implementation:

1. Uses Actix for the actor framework and message handling
2. Processes cuts in response to MaterialCut events from the Cutting system
3. Uses the HuggingFace fastembed library for generating embeddings
4. Stores embeddings in SQLite for persistence and retrieval
5. Supports fully asynchronous processing via Tokio runtime

## The SwatchingActor Component

The SwatchingActor component:

- Subscribes to MaterialCut events via the event bus
- Processes cuts asynchronously using embedding models
- Stores swatches in a repository for later retrieval
- Updates material status in the registry
- Publishes MaterialSwatched events for downstream processing
- Implements error handling with detailed diagnostic information
