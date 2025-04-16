# Product Overview

> For a quick introduction to Quilt, see the [Introduction](../README.md).

## What is Quilt?

Quilt is a local-first, modular memory and context engine designed to watch a user's work, fragment documents into meaningful pieces (swatches), embed them into a searchable memory (the swatch book), and assemble contextual spreads in response to queries.

It serves as a backend component that can power various LLM applications by providing relevant context without relying on cloud infrastructure or compromising user privacy.

## Core Concepts

| Term            | Description                                                      | Technical Implementation                                                                   |
| --------------- | ---------------------------------------------------------------- | ------------------------------------------------------------------------------------------ |
| **Material**    | A raw document or file (code, notes, etc.) that Quilt processes  | Stored with metadata like file path, modification time, and processing status              |
| **Cut**         | A chunk of content extracted from a Material during processing   | Created by the cutting system using text-splitter algorithms with configurable token sizes |
| **Swatch**      | A meaningful fragment with embedded vector representation        | Contains a Cut plus its vector embedding, stored in the SwatchRepository                   |
| **Swatch Book** | The searchable memory of embedded Swatches                       | Implemented as a vector database with efficient similarity search                          |
| **Spread**      | A contextual bundle of Swatches assembled in response to a Query | Result of a similarity search plus contextual ranking and assembly                         |

## Material Processing Pipeline

Quilt processes materials through a defined pipeline implemented as a series of actor-based stages:

1. **Discovery** - Find and ingest new or updated materials (files)

   - Implemented by DiscoveryActor which monitors directories for changes
   - Supports configurable file patterns and exclusions
   - Detects new and modified files

2. **Cutting** - Split materials into meaningful chunks (cuts)

   - Implemented by CuttingActor which subscribes to MaterialDiscovered events
   - Uses text-splitter algorithms to divide content into semantic chunks
   - Produces Cuts with configurable token sizes (default: 300 tokens)
   - Stores results in CutsRepository

3. **Swatching** - Generate embeddings for cuts to create swatches

   - Implemented by SwatchingActor which subscribes to MaterialCut events
   - Uses local embedding models to create vector representations
   - Associates embeddings with cuts to form swatches
   - Stores results in SwatchRepository

4. **Storage** - Add swatches to the searchable swatch book

   - Persists swatches with SQLite and vector extensions
   - Implements efficient similarity search
   - Supports automatic indexing

5. **Retrieval** - Find and assemble related swatches into spreads based on queries
   - Uses vector similarity to find relevant swatches
   - Assembles contextual bundles with source attribution
   - Formats for consumption by LLM applications

## Key Features

1. **Local-first Architecture**

   - All processing runs locally on the user's machine
   - No cloud dependencies for core functionality
   - Complete privacy preservation
   - SQLite-based persistence with vector search capabilities

2. **Modular Component System**

   - Independent, swappable components (Repository implementations, Actors)
   - Extensible for different file types and embedding models
   - Actor-based architecture for clean separation of concerns
   - Event-driven communication for loose coupling

3. **Incremental Processing**

   - Processes documents as they change
   - Maintains up-to-date context
   - Efficient processing of incremental changes
   - Reconciliation system for handling processing errors or interruptions

4. **Semantic Search**

   - Vector-based retrieval of related content
   - Contextual understanding beyond keyword matching
   - Fast retrieval of relevant information
   - Local embedding models avoid sending content to external services

5. **Context Assembly**
   - Intelligent bundling of related swatches
   - Prioritization of most relevant context
   - Structured response format for LLM consumption
   - Configurable context size and relevance thresholds
