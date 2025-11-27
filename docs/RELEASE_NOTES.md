# Release Notes

Generated: 2025-11-18 11:48 (local)

## Phase 6: task_orchestrator graph-flow integration (Completed)

Highlights:
- Unified graph runtime using `graph_flow` (rs-graph-llm) with `FlowRunner`.
- Orchestrator graph assembled with conditional routing and loop.
- Task shims for router, enhancement, comprehension test, and result check.
- Optional SQLite session persistence behind `sqlite_persistence` feature, with smoke test.
- Documentation improvements: crate README, visual flow (docs/FLOW.md), project dashboard (docs/PROJECT_STATUS.md).

Quality Assurance:
- task_orchestrator (default): 37 unit tests passed; 2 doc tests passed.
- task_orchestrator (+sqlite_persistence): 39 unit tests passed; 2 doc tests passed.
- Workspace root: all tests passed.

Usage Guidance:
- End-to-end helper: `task_orchestrator::use_cases::run_task_with_flow::run_task_with_flow(model, test_type, task)`.
- Configure SQLite persistence by enabling the `sqlite_persistence` feature and setting `TASK_ORCHESTRATOR_SQLITE_URL` (defaults to `sqlite::memory:`).

Next Steps:
- Optional: Introduce an `Orchestrator` facade for simplified construction and usage patterns.
- Optional: Expand persistence scenarios (file-backed SQLite) and add metrics.
