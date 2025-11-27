# Baseline: Deterministic Adapter Behavior

**Date:** 2025-11-23
**Phase:** Phase 1 Sprint 1 (Validation & Testing)
**Purpose:** Document current behavior before Rig upgrade in Sprint 2

## Executive Summary

This document captures the baseline behavior of deterministic adapters used in the Rigger orchestration pipeline. These adapters return fixed/dummy data for testing and validation purposes. Sprint 2 will upgrade them to use Rig's LLM-based extraction with schema enforcement.

## Test Configuration

- **Test File:** `task_orchestrator/tests/integration_end_to_end_flow.rs`
- **Test Name:** `test_baseline_deterministic_adapter_output`
- **Model:** llama3.1 (used in adapter constructors, not actual LLM calls)
- **Test Type:** short_answer
- **Task Input:** Minimal task with title "Baseline test task"

## Current Adapter Implementations

### OllamaEnhancementAdapter

**Location:** `task_orchestrator/src/adapters/ollama_enhancement_adapter.rs`

**Behavior:**
- Returns a single Enhancement with deterministic content
- Enhancement ID format: `enh-{task_id}-{timestamp_millis}`
- Enhancement type: "rewrite" (hardcoded)
- Content format: `[model:{model_name}] Enhanced: {task_title}`
- Example output:
  ```
  Enhancement {
    enhancement_id: "enh-2105d8ba-4608-4b3c-a1de-7e8ee0f1239e-1763907599685",
    task_id: "2105d8ba-4608-4b3c-a1de-7e8ee0f1239e",
    timestamp: 2025-11-23 14:19:59.685699 UTC,
    enhancement_type: "rewrite",
    content: "[model:llama3.1] Enhanced: Baseline test task"
  }
  ```

**Issues/Limitations:**
- Content is not meaningful (just echoes task title)
- Enhancement type is always "rewrite" (should vary based on task needs)
- Only returns single enhancement (real adapter should return multiple suggestions)

### OllamaComprehensionTestAdapter

**Location:** `task_orchestrator/src/adapters/ollama_comprehension_test_adapter.rs`

**Behavior:**
- Returns a single ComprehensionTest with deterministic question
- Test ID format: `ct-{task_id}-{timestamp_millis}`
- Question format: `[{model_name}] Task goal?` (shortened to ≤80 chars to pass CheckTestResultNode)
- Options: None (regardless of test_type)
- Correct answer: "N/A" (placeholder)
- Example output:
  ```
  ComprehensionTest {
    test_id: "ct-2105d8ba-4608-4b3c-a1de-7e8ee0f1239e-1763907599685",
    task_id: "2105d8ba-4608-4b3c-a1de-7e8ee0f1239e",
    timestamp: 2025-11-23 14:19:59.685749 UTC,
    test_type: "short_answer",
    question: "[llama3.1] Task goal?",
    options: None,
    correct_answer: "N/A"
  }
  ```

**Issues/Limitations:**
- Question is generic, not task-specific
- Question length constraint (≤80 chars) required to avoid infinite loop in CheckTestResultNode
- Multiple choice tests don't populate options array
- Correct answer is placeholder, not derived from task content

## Orchestration Flow Validation

### Graph Execution Order

The baseline test confirms the graph executes in correct order:

1. **SemanticRouterNode** → Routes task based on context (currently always routes to "enhance")
2. **EnhanceTaskNode** → Calls OllamaEnhancementAdapter, adds enhancements to task
3. **GenerateComprehensionTestNode** → Calls OllamaComprehensionTestAdapter, adds tests to task
4. **CheckTestResultNode** → Validates test quality (uses question.len() ≤ 80 heuristic)
5. **END** → Returns enhanced task with status OrchestrationComplete

### State Threading

✅ **VALIDATED:** Task state is correctly preserved and enhanced through the pipeline:
- Task ID remains unchanged
- Original fields (title, assignee, due_date, source_transcript_id, source_prd_id) preserved
- New fields (enhancements, comprehension_tests) populated
- Status transitions from Todo → OrchestrationComplete

### Status Progression

| Stage | Status |
|-------|--------|
| Initial | Todo |
| After Orchestration | OrchestrationComplete |

## Data Structure Validation

### Enhancement Domain Type

**Fields populated:**
- ✅ enhancement_id (String, format: "enh-{task_id}-{timestamp}")
- ✅ task_id (String, links to parent task)
- ✅ timestamp (chrono::DateTime<Utc>)
- ✅ enhancement_type (String, currently hardcoded "rewrite")
- ✅ content (String, deterministic dummy data)

**Fields missing:**
- ❌ confidence_score (not in current schema)
- ❌ model_metadata (not in current schema)

### ComprehensionTest Domain Type

**Fields populated:**
- ✅ test_id (String, format: "ct-{task_id}-{timestamp}")
- ✅ task_id (String, links to parent task)
- ✅ timestamp (chrono::DateTime<Utc>)
- ✅ test_type (String, matches requested type)
- ✅ question (String, deterministic dummy data)
- ✅ options (Option<Vec<String>>, None for short_answer)
- ✅ correct_answer (String, placeholder "N/A")

**Fields missing:**
- ❌ difficulty_level (not in current schema)
- ❌ expected_response_format (not in current schema)

## Known Issues

### Issue 1: Infinite Loop with Long Questions

**Discovered:** 2025-11-23
**Root Cause:** CheckTestResultNode uses `question.len() ≤ 80` heuristic. Questions >80 chars route back to enhancement, creating infinite loop.

**Fix Applied:** Shortened deterministic question from:
```rust
"[model:{}] What is the main deliverable for '{}'?"  // >80 chars for long task titles
```
to:
```rust
"[{}] Task goal?"  // 20 chars
```

**Future Consideration:** Real LLM-based adapter will need length validation or different quality heuristic.

### Issue 2: Non-Meaningful Content

**Status:** Expected behavior for deterministic adapters
**Resolution:** Will be addressed in Sprint 2 when upgrading to Rig Extractor

## Test Coverage

### Integration Tests Passing

- ✅ `test_end_to_end_orchestration_flow_in_memory` - Main flow validation
- ✅ `test_multiple_tasks_through_flow` - State isolation validation
- ✅ `test_task_state_threading_through_nodes` - State preservation validation
- ✅ `test_baseline_deterministic_adapter_output` - Baseline behavior documentation

### Feature Flag Tests

- ✅ `test_end_to_end_orchestration_flow_with_sqlite` - SQLite persistence (requires `sqlite_persistence` feature)

## Expected Changes in Sprint 2

When upgrading to Rig-powered adapters with LLM integration:

### OllamaEnhancementAdapter Upgrade
1. Add JsonSchema derives to Enhancement domain type
2. Use Rig Extractor with schema enforcement
3. Generate meaningful enhancement suggestions based on task analysis
4. Vary enhancement_type (rewrite, clarify, decompose, etc.)
5. Return multiple enhancement options
6. Implement JSON repair strategy for malformed LLM outputs

### OllamaComprehensionTestAdapter Upgrade
1. Add JsonSchema derives to ComprehensionTest domain type
2. Use Rig Extractor with schema enforcement
3. Generate task-specific questions analyzing task content
4. Populate options array for multiple_choice tests
5. Derive correct answers from task requirements
6. Add tolerant parsing fallback for edge cases

### Quality Improvements
1. Replace question length heuristic with semantic quality check
2. Add confidence scoring to enhancements
3. Add difficulty levels to comprehension tests
4. Implement proper error handling and retry logic

## Baseline Reference Data

For regression testing, these are the exact outputs from the baseline test:

```
Task: Baseline test task (ID: 2105d8ba-4608-4b3c-a1de-7e8ee0f1239e)

Enhancement:
  ID: enh-2105d8ba-4608-4b3c-a1de-7e8ee0f1239e-1763907599685
  Type: rewrite
  Content: [model:llama3.1] Enhanced: Baseline test task

ComprehensionTest:
  ID: ct-2105d8ba-4608-4b3c-a1de-7e8ee0f1239e-1763907599685
  Type: short_answer
  Question: [llama3.1] Task goal?
  Options: None
  Correct Answer: N/A
```

## Conclusion

The deterministic adapters successfully validate that:
1. Orchestration graph executes in correct order
2. State threading preserves task properties
3. Domain types have correct field structures
4. Session storage (in-memory and SQLite) works correctly
5. Status progression flows through pipeline

These adapters provide a stable foundation for upgrading to real LLM integration in Sprint 2 while maintaining test reliability and fast execution times.
