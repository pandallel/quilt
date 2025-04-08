# Project Brief: Quilt

## Project Definition

Quilt is a local-first, modular memory and context engine designed to watch a user's work, fragment documents into meaningful pieces (swatches), embed them into a searchable memory (the swatch book), and assemble contextual spreads in response to queries.

## Core Requirements

1. **Local-first Architecture** - All operations must run on the user's machine without cloud dependencies
2. **Modular Component System** - Independent, swappable components for watching, swatching, embedding, and querying
3. **Privacy-preserving** - No data leakage or external dependencies required
4. **Actor-based Processing Pipeline** - Implement using Rust/Tokio with direct messaging between actors
5. **Material Processing Workflow**:
   - Discovery of new documents/files
   - Cutting into meaningful fragments (swatches)
   - Labeling/embedding of swatches
   - Storage in a queryable repository
   - Assembly of contextual spreads for queries

## Key Technical Components

1. **Material Repository** - Thread-safe data store for materials and their processing state
2. **Discovery Worker** - Monitors input sources for new/updated materials
3. **Cutting Worker** - Processes materials by cutting them into swatches
4. **Labeling Worker** - Executes embedding operations on swatches
5. **Vector-based Storage** - Persists embedded swatches for semantic retrieval

## Project Goals

1. Provide powerful context to LLM tools without relying on cloud infrastructure
2. Preserve user privacy by keeping all data and processing local
3. Create a flexible system that can adapt to different use cases and data types
4. Build a responsive system that can process documents incrementally and efficiently
5. Deliver a high-quality, production-ready implementation following Rust best practices
