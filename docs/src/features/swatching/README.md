# Material Swatching

This directory contains documentation about the Material Swatching system in Quilt, which transforms semantic text chunks (Cuts) into vector embeddings (Swatches) for similarity search and retrieval.

## Core Concepts

- [**Swatch**](./swatching-behaviors.md) - A vector embedding representing the semantic content of a Cut, generated using embedding models
- [**Swatching Process**](./swatching-architecture.md) - The architecture and strategies for converting Cuts into Swatches using embedding models
- [**Error Handling**](./swatching-errors.md) - How errors are detected, reported, and recovered during the swatching process

## Implementation Status

The Material Swatching system is fully implemented using an event-driven, actor-based architecture. The current implementation features:

1. **Event-Driven Processing** - Automatically processes materials in response to MaterialCut events
2. **HuggingFace Integration** - Uses the fastembed library for high-quality embeddings
3. **Repository Storage** - Persists swatches in SQLite with efficient batch operations
4. **Status Management** - Tracks material status through the swatching lifecycle
5. **Error Handling** - Provides detailed error information with automatic status updates
6. **Comprehensive Testing** - Includes unit tests, integration tests, and mock-based tests for all components
7. **Full Integration** - Connects with cutting, embedding, and event systems via dependency injection

The system has been fully tested with successful end-to-end processing of materials, including:

- Embedding generation for text content of any size
- Proper error handling and recovery
- Performance optimization for batch processing
- Status updates and event publishing

## The SwatchingActor Component

The SwatchingActor component:

- Subscribes to MaterialCut events via the event bus
- Processes cuts asynchronously using embedding models
- Stores swatches in a repository for later retrieval
- Updates material status in the registry
- Publishes MaterialSwatched events for downstream processing
- Implements error handling with detailed diagnostic information
