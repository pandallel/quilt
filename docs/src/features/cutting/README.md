# Material Cuts

This directory contains documentation about the Material Cut system in Quilt, which transforms raw documents (Materials) into meaningful, semantically coherent fragments for embedding and retrieval.

## Core Concepts

- [**Cut**](./cutting-behaviors.md) - A semantically meaningful fragment extracted from a Material, sized appropriately for embedding and retrieval
- [**Cutting Process**](./cutting-architecture.md) - The architecture and strategies for dividing Materials into semantically meaningful Cuts
- [**Error Handling**](./cutting-errors.md) - How errors are detected, reported, and recovered during the cutting process

## Implementation Status

The Material Cut system uses an event-driven, actor-based architecture. The current implementation (as of Q2 2023):

1. Uses Actix for the actor framework and message handling
2. Processes materials in response to MaterialDiscovered events from the Discovery system
3. Integrates with the Material Registry for state management and tracking
4. Supports fully asynchronous processing via Tokio runtime

## The CuttingActor Component

The CuttingActor component:

- Subscribes to MaterialDiscovered events via the event bus
- Processes materials asynchronously using the text-splitter library
- Updates material status in the registry (from Discovered → Processing → Cut)
- Publishes MaterialCut events for downstream processing by the SwatchingActor
- Implements a progressive fallback strategy for error handling with detailed diagnostic information
