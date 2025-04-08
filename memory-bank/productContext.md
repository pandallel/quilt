# Product Context

## Problem Statement

LLM-powered applications need access to personal and private data to be useful, but existing solutions have significant drawbacks:

1. **Privacy Concerns**: Uploading personal documents to cloud services risks sensitive data exposure
2. **Infrastructure Dependencies**: Most context systems rely on external services, reducing reliability
3. **Poor Integration**: Most systems aren't designed to work seamlessly with existing workflows
4. **Lack of Flexibility**: Current solutions are often rigid in how they process and represent information

Quilt addresses these issues by providing a local-first, modular memory and context engine that works silently behind the scenes, processing documents into meaningful fragments without relying on external services.

## User Experience Goals

### Primary User Persona

Quilt is designed for:

- Developers building LLM-powered applications who need context management
- Privacy-conscious users who want to leverage their personal data without uploading it
- Knowledge workers who need to quickly retrieve relevant information from large document collections

### Core User Experience Principles

1. **Invisible Power**: Users shouldn't interact with Quilt directly—it works behind the scenes to support other systems
2. **Effortless Integration**: Seamlessly integrates with existing workflows and document stores
3. **Quality Results**: Produces high-quality, relevant context for queries
4. **Control & Transparency**: Users understand what data is being processed and how

### Key User Journeys

1. **Setup & Configuration**:

   - Install Quilt locally
   - Configure watched directories and processing options
   - Connect to applications that will use Quilt's context

2. **Passive Document Processing**:

   - Quilt silently watches configured directories
   - New or changed documents are automatically processed
   - Materials are cut into swatches and embedded without user intervention

3. **Context Retrieval**:
   - Applications query Quilt with natural language or specific search terms
   - Quilt assembles relevant swatches into a contextual spread
   - Applications receive structured, relevant context to power LLM interactions

## Success Metrics

1. **Processing Speed**: Documents are processed within seconds of being created or modified
2. **Retrieval Quality**: Context is highly relevant to queries, improving LLM responses
3. **Resource Efficiency**: Minimal CPU, memory, and storage usage during idle and active periods
4. **Reliability**: System operates without errors or performance degradation over time
