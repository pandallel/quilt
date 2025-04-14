# Memory Bank Migration Plan

This document outlines the steps to transition from the AI-centric Memory Bank structure to a hybrid model prioritizing human-readable documentation (`docs/`) supplemented by lean AI guidance rules (`.cursor/rules/`).

## Goal

- Establish a single source of truth for project documentation in `docs/`.
- Reduce duplication between human and AI-specific documentation.
- Integrate the AI more effectively into team workflows by guiding it to use shared resources.
- Improve maintainability of documentation.

## Phases

### Phase 1: Establish Human-Centric Documentation (`docs/`)

1.  **Create `docs/` Directory:**

    - If it doesn't exist, create a `docs/` directory at the project root.

2.  **Define Core Structure:**

    - Create necessary subdirectories within `docs/`:
      - `introduction/`
      - `architecture/` (consider adding `adr/` inside)
      - `technical/`
      - `processes/`
      - _(Adapt structure as needed for the project)_

3.  **Migrate Foundational Info:**

    - **Goals & Context:** Review `memory_bank/projectbrief.md` and `memory_bank/productContext.md`. Migrate and refine content into `docs/introduction/overview.md`, `docs/introduction/product_vision.md`.
    - **Architecture:** Review `memory_bank/systemPatterns.md`. Migrate/refine content into `docs/architecture/overview.md`. Create initial ADRs in `docs/architecture/adr/` for key decisions if applicable.
    - **Tech Stack:** Review `memory_bank/techContext.md`. Migrate/refine content into `docs/technical/stack.md` and `docs/technical/setup.md`.

4.  **Document Processes:**

    - Create/populate files in `docs/processes/` covering:
      - `collaboration.md`
      - `issue_tracking.md`
      - `code_review.md`
      - _(Add other relevant processes)_
    - Ensure `CONTRIBUTING.md` exists at the project root and covers contribution guidelines.

5.  **Update Root `README.md`:**
    - Ensure the main `README.md` provides a concise project overview and clearly links to the new `docs/` directory.

### Phase 2: Create Lean AI Guidance Rules (`.cursor/rules/`)

_(Assumes rules reside in `.cursor/rules/`. Adjust path if using `docs/ai/` or another location)._

1.  **Create `ai_guide.md`:**

    - Create `.cursor/rules/ai_guide.md`.
    - Populate with explicit pointers directing the AI to relevant sections in `docs/` for foundational knowledge (e.g., "For Project Goals, see `docs/introduction/overview.md`").

2.  **Create `ai_process_integration.md`:**

    - Create `.cursor/rules/ai_process_integration.md`.
    - Define rules for how the AI should interact with team workflows, referencing the corresponding files in `docs/processes/`, `CONTRIBUTING.md`, etc. (e.g., "Follow coding standards in `docs/technical/coding_standards.md`").

3.  **Refactor `active_context.md` (or create `current_task.md`):**
    - Rename/reuse `memory_bank/activeContext.md` as `.cursor/rules/active_context.md` (or similar).
    - **Remove** duplicated static information.
    - **Focus** content strictly on the _immediate task_: link to issue tracker, current goal, task-specific notes, relevant files for the _current session_.
    - Deprecate `memory_bank/progress.md` if its purpose is now covered by the issue tracker and the lean active context file.

### Phase 3: Deprecation and Rollout

1.  **Update AI Instructions:**

    - Modify the AI's primary instructions (e.g., custom instructions, system prompts) to:
      - Prioritize `docs/` as the source of truth.
      - Use `.cursor/rules/ai_guide.md` as the entry point for finding information.
      - Adhere to rules in `.cursor/rules/ai_process_integration.md`.
      - Use `.cursor/rules/active_context.md` for immediate task context only.

2.  **Test:**

    - Engage the AI on typical tasks.
    - Verify it correctly references `docs/` and follows process integration rules.
    - Iterate on the AI rules and instructions as needed.

3.  **Deprecate Old Memory Bank Files:**

    - Once confident in the new structure, delete the old, redundant files from the `memory_bank/` directory:
      - `projectbrief.md`
      - `productContext.md`
      - `systemPatterns.md`
      - `techContext.md`
      - `progress.md` (if replaced)
      - The `activeContext.md` _if_ it was moved/renamed, not just refactored in place.

4.  **Communicate (if applicable):**
    - Inform the team about the new documentation structure in `docs/` and the updated approach to AI guidance.
