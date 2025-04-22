# Resilience and Error Handling

This workstream focuses on improving the system's robustness, including handling stuck materials and implementing retries.

### Milestone 13: "Reconciliation Actor Implementation"

**Goal:** Implement the Reconciliation Actor to handle stuck materials and retries
**Implementation Time:** 3-4 days

1. Create Reconciliation Actor skeleton (1 day)

   - Implement Actor structure
   - Add configuration for scan interval, retry limits, timeouts per stage
   - Set up periodic triggering using `ctx.run_interval`

2. Implement Registry Scanning Logic (1-2 days)

   - Query `MaterialRegistry` (or `MaterialRepository`) for materials in intermediate states (`Cutting`, `Swatching`) longer than configured timeout
   - Implement logic to check `retry_count` against `max_retries`

3. Implement Retry and Error Handling (1 day)
   - If retries remain: update retry count/timestamp in Registry/Repository, re-publish preceding event (e.g., `MaterialDiscovered`) to shared `EventBus`
   - If max retries exceeded: update material status to `Error` in Registry/Repository
   - Add comprehensive logging and metrics for reconciliation actions (scans, retries, errors)

**Demonstration:** Running `main` shows logs from the Reconciliation Actor identifying stuck items (if any manually created), attempting retries, and eventually marking items as Error after exceeding retries.
