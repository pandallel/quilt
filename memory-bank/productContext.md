# Product Context

## Problem Statement

LLM-powered applications need access to personal and private data to be useful, but existing solutions have significant drawbacks:

1. **Privacy Concerns**: Uploading personal documents to cloud services risks sensitive data exposure
2. **Infrastructure Dependencies**: Most context systems rely on external services
3. **Poor Integration**: Most systems aren't designed to work with existing workflows
4. **Lack of Flexibility**: Current solutions are often rigid in how they process information

Quilt addresses these issues by providing a local-first, modular memory and context engine that processes documents into meaningful fragments without relying on external services.

## User Experience Goals

### Primary User Persona

- Developers building LLM-powered applications who need context management
- Privacy-conscious users who want to leverage personal data without uploading it
- Knowledge workers who need to quickly retrieve relevant information from document collections

### Core User Experience Principles

1. **Invisible Power**: Works behind the scenes to support other systems
2. **Effortless Integration**: Seamlessly integrates with existing workflows
3. **Quality Results**: Produces high-quality, relevant context for queries
4. **Control & Transparency**: Users understand what data is being processed

### Key User Journeys

1. **Setup & Configuration**:

   - Install Quilt locally
   - Configure watched directories
   - Connect to applications using Quilt's context

2. **Passive Document Processing**:

   - Quilt silently watches configured directories
   - New or changed documents are automatically processed
   - Materials are cut into swatches and embedded

3. **Context Retrieval**:
   - Applications query Quilt with natural language
   - Quilt assembles relevant swatches into contextual spreads
   - Applications receive structured context for LLM interactions

## Success Metrics

1. **Processing Speed**: Documents processed within seconds of creation/modification
2. **Retrieval Quality**: Context is highly relevant to queries
3. **Resource Efficiency**: Minimal CPU, memory, and storage usage
4. **Reliability**: System operates without errors or performance degradation
