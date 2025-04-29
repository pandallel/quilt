# Future Milestones

Future milestones will focus on more advanced features:

1. **Embedding Generation & Vector Search**

   - Integration with embedding models (e.g., Sentence Transformers via ONNX or similar)
   - Vector storage integration (e.g., `sqlite-vec`) with `SqliteSwatchRepository`
   - Implementation of vector similarity search logic
   - Query API for semantic search

2. **Scaling and Performance**

   - Swatching Router implementation for dynamic actor scaling
     - Router actor for managing multiple Swatching Actors
     - Queue length monitoring and health checks (both internal `mpsc` and `EventBus` lag)
     - Dynamic actor pool management
   - Enhanced caching strategies
   - Load balancing and monitoring
   - Performance optimization based on usage patterns (tuning queue sizes, buffer capacities)
   - **Address Swatching Bottlenecks:**
     - **Parallelize Embedding Generation:** Explore parallelizing CPU-intensive embedding computation within the `EmbeddingService` (e.g., using `rayon`) or by spawning multiple embedding tasks.
     - **Implement Batch Swatch Saving:** Modify `SwatchRepository` and `SwatchingActor` to save generated swatches in batches, reducing database transaction overhead.
   - **Optimize Progress Reporting:** Reduce database load by making progress count queries in `MaterialRegistry` less frequent (e.g., timer-based, every N updates, or dedicated reporting task).
   - **Database Optimization:** Evaluate alternative databases (e.g., PostgreSQL) for improved concurrent write performance if SQLite becomes a bottleneck under heavy load.

3. **Cutting Enhancements**

   - Improve backpressure handling
     - Add explicit backpressure strategy when internal queue fills up
     - Implement circuit-breaking for continuous error situations
     - Add metrics for queue depth and backpressure events
   - Implement retry mechanisms
     - Add retry capability for recoverable errors
     - Implement exponential backoff strategy
     - Create configurable retry policies per error type
   - Enhance configuration
     - Make cutting parameters configurable (chunk size, overlap)
     - Allow runtime configuration updates

4. **Storage and Persistence Improvements**

   - Implement disk-based repository options for cuts and materials
   - Add streaming processing for very large files
   - Create efficient indexing strategies for large repositories
   - Implement data compression for storage efficiency
   - Add data integrity validation and repair mechanisms

5. **Enhanced Logging and Observability**

   - Implement structured logging with span contexts
   - Create comprehensive tracing for request flows
   - Add detailed performance metrics collection
   - Implement health monitoring dashboards
   - Create alerting for system issues

6. **Enhanced Text Processing**

   - Language detection
   - Text classification
   - Entity extraction

7. **Advanced Search and Queries**

   - Query language development
   - Search result ranking
   - Filter and facet implementation

8. **Enhanced Cutting Strategies**

   - Markdown content cutting
     - MarkdownCutter implementation using MarkdownSplitter
     - Format detection for Markdown content
     - Fallback to TextCutter on errors
   - Source code cutting
     - CodeCutter implementation using CodeSplitter
     - Language detection for code content
     - Specialized semantic boundary handling for code structures
   - Format-specific optimizations including language-specific tokenization and semantic boundary recognition

9. **User Interfaces**

   - Web-based dashboard
   - Search interface
   - Material management

10. **Integration APIs**
    - REST API for external access
    - Webhooks for processing events
    - Subscription mechanism for updates
