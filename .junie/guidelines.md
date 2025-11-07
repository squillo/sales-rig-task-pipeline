AI Assistant: Code Generation & Modification Protocol

YOU WILL NEVER ALTER THIS FILE.

---

When you encounter a request for a specific library, framework, or crate, use Context7 MCP server.

### **1.Revision History Protocol**

Whenever you alter a file, you MUST add a revision history comment. This comment MUST be at the END of the documentation preamble and BEFORE any code. It MUST be in a valid Rust location. The entry MUST include:

- The date and time of the change in UTC (YYYY-MM-DDTHH:MM:SSZ).
- The user or AI that made the change (e.g., @AI).
- A brief, formal description of the change.

```rust
//! Revision History
//! - 2025-04-13T13:37:01Z @AI: Refine example comments per latest request.
//! - 2025-04-13T13:35:39Z @AI: Clean example comments, add note on test doc simplicity.
```

---

### **2\. Environment Tooling**

**Modern**: for Libraries and Frameworks ALWAYS use the latest versions

- E.g. Rust Edition 2024 is the target for Rust.

**Tools:** Use the Context7 MCP server for library and framework specific tasks.

- If an included library or framework already has a specific function of a task, use the library’s implementation. If you are ever unsure about using a library component or writing a component from scratch, you MUST ask.

**Trim:**

- In strictly statically typed language environments (i.e. Rust), it’s preferable to use MIT or Apache licensed libraries instead of writing custom code by hand.
- In dynamically typed environments (i.e. JavaScript, EcmaScript, TypeScript), use previously installed library tools where applicable, or write custom code by hand, always striving for a functional style of coding.

---

### **3\. Target Language & Overarching Goals**

**Primary Goals:** All generated code MUST be optimized for Maintainability, Reliability, Security, and Speed.

- **Maintainability:** Achieved through maximum clarity, simplicity, and modularity.
- **Reliability:** Achieved by strict adherence to the Single Responsibility Principle.
- **Security:** Achieved by avoiding `unsafe` code and ensuring robust error handling. FFI is the only exception.
- **Speed:** Refers to the efficiency of development and execution across all other goals.

---

### **4\. Operational Conventions**

**A. Instruction Hierarchy**

- Words in `ALL CAPS` are absolute, non-negotiable requirements.
- Words in **bold** are strong, evidence-based recommendations.
- Words in *italics* are suggestions.

**B. Workspace Management (Multi-Crate Projects)** Use the following conventions to manage your workspace:

- Set dependencies in the workspace crate.
- Reference dependency crates in sub-crates by their workspace e.g. `{ workspace = true }`

**C. Long-Term Task Management (Source of Record)** Tasks spanning multiple interactions MUST be managed in a separate `TASK_PLAN.md` file to maintain state and overcome context window limitations. This file is the shared Source of Record.\[1\]

When instructed to create or maintain a plan, the file MUST follow this structure:

---

## **task\_id: \<unique\_identifier\> status: \[planning | in-progress | blocked | completed\]**

# **Task:**

USE THE GUIDANCE PROVIDED FOR ALL TASK ITEMS

## **Plan**

- [ ] 1. Decompose problem into sub-tasks.
- [ ] 2. Implement sub-task A.
- [ ] 3. Test sub-task A.

## **Current Step**

- **Action:** Implementing sub-task A.
- **Details:**.

## **Blockers**

- \[List any issues preventing progress\].

**Input/Output Protocol:** When a task involves a `TASK_PLAN.md`, the full content of the file will be provided as the initial context. Your response MUST conclude with the complete, updated content of the `TASK_PLAN.md` inside a single, fenced Markdown block labeled `updated_task_plan`. No other content should follow this block.

---

### **5\. Knowledge Graph**

- Each crate/library/module MUST have a `README.md` file containing a knowledge graph of its contents which must be maintained.
- AFTER generating code for a multi-(crate/library/module) project task, you MUST update the relevant `README.md`
- BEFORE generating code for a multi-(crate/library/module) project task, you MUST consult the relevant `README.md` to understand existing patterns and architecture.

**D. Testing Mandate** Tasks that span multiple responses MUST ensure all existing and new tests pass between each iteration.

**E. State Integrity Mandate** The code MUST be in a verifiable working state (i.e., compiles and passes all tests) at the end of each group of alterations. In a multi-step operation, this means each step should result in a stable state. Any deviation that leaves the code in a non-working state is a critical failure UNLESS it is explicitly documented as a `TODO` or `BLOCKED` item in the `TASK_PLAN.md` Source of Record.

**F. Prompt Scope Sizing (H3 Resolution Analogy)** All tasks must be scoped to an appropriate "resolution" before execution. This principle is analogous to the H3 global grid system, which partitions geographies into a hierarchy of hexagonal cells of varying sizes.

- **Low-Resolution Task (Anti-Pattern):** A vague, high-level objective like "Refactor the entire auth module" is a low-resolution prompt. It covers a large conceptual area but lacks the detail required for precise, verifiable execution.
- **High-Resolution Task (Mandatory):** A large task MUST be broken down into a hierarchy of smaller, high-resolution sub-tasks, each with a single, verifiable goal.\[2\] This process of decomposition is the primary planning step.

**Rule:** For any task that cannot be completed in a single, verifiable step, the first action MUST be to generate a detailed execution plan in the `TASK_PLAN.md`. This plan represents the decomposition of the low-resolution goal into a series of high-resolution, executable sub-prompts.

* **Example of Decomposing a Low-Resolution Task:**
* **Initial Prompt:** "Refactor the auth module to use the new error handling standard."
* **Generated High-Resolution Plan:**
1. Analyze `auth/mod.rs` and its sub-modules to identify all functions that currently use `.unwrap()` or `.expect()`.
2. Modify the `LoginError` enum in `auth/types.rs` to derive `thiserror::Error`.
3. Refactor the `login` function in `auth/handlers/login.rs` to return a `Result<_, LoginError>` and use the `?` operator.
4. Write a new unit test for the `login` function that specifically triggers and verifies an authentication error.

**G. Pre-computation Self-Correction** Before generating the final code or plan, you MUST perform a self-correction pass. In your reasoning process, explicitly list the key directives from this protocol (e.g., `NO use STATEMENTS`, `STATE INTEGRITY MANDATE`) and verify that your planned output adheres to each one. This verification process must be part of your internal monologue and must be completed before you output the final result.

---

### **6\. Architectural Principle: Single Responsibility**

Code organization follows the Single Responsibility Principle, articulated through the DPAI (Domain, Ports, Adapters, Application, Infrastructure) model. This is a conceptual guide for separating concerns.

| Layer | Role | Example Code Components |
| :---- | :---- | :---- |
| **Core Domain** | Business logic, no framework or tech dependency | `Parser`, `Grammar`, `WriteAheadLog`, `ForeignFunctions` traits |
| **Ports** | Abstract interfaces to the outside world | Traits like `SnappRepository`, `WebPort` |
| **Adapters** | Concrete implementations of ports | `SqlLiteRepo`, `ActixWebPort`, REST API |
| **Application** | Coordinates domain and ports | `NCLI`, `NAPI`, command handlers |
| **Infrastructure** | External tools (databases, APIs, etc.) | Uses `axum`, `sqlx`, `tokio` |

---

## **Rust Coding Standards: Evidence-Based Clarity & Modularity**

**A. CRITICAL: Safety & Path Unambiguity**

1. **NO `unsafe` CODE:** The `unsafe` keyword is FORBIDDEN. The only exception is for Foreign Function Interface (FFI) implementations.
2. **NO `use` STATEMENTS:** All `use` statements are FORBIDDEN. This is to enforce absolute clarity and remove ambiguity for multi-agent analysis.
- All types, functions, macros, and traits MUST be referenced by their fully qualified path.
- **Correct:** `std::collections::HashMap`, `crate::my_mod::MyStruct`
- **Incorrect:** `use std::collections::HashMap;`, `use crate::my_mod::MyStruct;`
- **Exception:** Items in the Rust prelude (e.g., `Vec`, `String`, `Option`, `Result`) do not require qualification.

**B. File & Module Structure**

1. **One Logical Item Per File:** Each `.rs` file MUST contain exactly one primary logical item (`fn`, `struct`, `enum`, `type`, `const`, `static`).
- Associated `impl` blocks MUST reside in the same file as their `struct` or `enum`.
- `impl Trait for Type` blocks should reside with the `Type` definition.
2. **File Naming:** Files MUST be named using `snake_case.rs` that matches the item they contain (e.g., `my_function.rs` for `fn my_function`).
3. **Module Files (`mod.rs`):** These files are for declaration ONLY. They MUST contain only `mod` statements and should NOT contain item definitions.

**C. Documentation & Style**

1. **File Preamble:** Every `.rs` file MUST begin with file-level documentation (`//!`). The first line is a single sentence summary, followed by a blank line and 3-5 lines of expansion.
2. **Item Documentation:** All public items MUST have documentation comments (`///`) that include doc tests where applicable.
3. **Traits as Content Maps:** Traits are used to categorize functions. `trait` blocks with default function implementations are FORBIDDEN.
4. **Functional Style:** Prefer immutable data, pure functions, and iterator-based transformations (`map`, `filter`, `fold`) over imperative loops and mutable state where it enhances clarity.
5. **Brevity:** NEVER use two words when one will do. Avoid colorful language and emojis unless explicitly requested.

**D. Function & Test Structure**

1. **Function Length:** Functions MUST NOT exceed 50 lines of code (LoC), excluding signature, comments, and blank lines. Justify rare exceptions with a comment.
2. **In-File Tests:** Unit tests for an item MUST reside in the same file under a `#[cfg(test)] mod tests {... }` block. Access the item under test via `super::`.

**E. Adherence to Official API Guidelines** All code MUST conform to the Rust API Guidelines where applicable. The following are non-negotiable:

- **`C-QUESTION-MARK`:** All examples and fallible functions MUST use `?` for error propagation, not `.unwrap()` or `.expect()`.
- **`C-GOOD-ERR`:** Error types must be meaningful and well-behaved.
- **`C-CTOR`:** Constructors MUST be static, inherent methods, typically named `new`.
- **`C-STRUCT-PRIVATE`:** Struct fields MUST be private by default to ensure encapsulation. The `Public Struct Fields` rule is an exception and applies only when explicitly requested for a specific data struct.

---

### **1\. Example Ideal File: `calculate_weighted_sum.rs`**

This example adheres to all specified rules.

````rust
//! Calculates the weighted sum of a slice of numbers using provided weights.
//!
//! This function takes two slices: one for values and one for their corresponding weights.
//! It computes the sum of `value * weight` for each pair.
//! Returns an error if slices differ in length.
//! The weighted sum of empty slices is 0.0.

//! Revision History
//! - 2025-04-13T13:37:01Z @AI: Refined internal comments per latest request.
//! - 2025-04-13T13:24:57Z @AI: Convert all example comments to doc comments, add example revision history.

/// Calculates the weighted sum of two slices of f64 numbers.
///
/// This function computes the sum of `value * weight` for each corresponding
/// pair of elements in the `values` and `weights` slices. It is designed
/// to be robust and handles several edge cases, such as empty or mismatched-length slices.
///
/// # Arguments
///
/// * `values` - A slice of f64 numbers to be weighted.
/// * `weights` - A slice of f64 numbers representing the weights. MUST be the same length as `values`.
///
/// # Returns
///
/// * `Ok(f64)` containing the weighted sum if the inputs are valid.
/// * `Err(String)` if the input slices have mismatched lengths.
///
/// # Errors
///
/// Returns an error if `values` and `weights` are not of the same length.
///
/// # Examples
///
/// ```
/// // This doc test demonstrates basic usage and adherence to the fully qualified path rule.
/// // Note: In a real crate, `crate::calculate_weighted_sum` would be the path.
/// // For this standalone example, we assume it's available at the crate root.
/// fn main() {
///     let values = [1.0, 2.0, 3.0];
///     let weights = [0.5, 1.0, 2.0];
///     let result = calculate_weighted_sum(&values, &weights);
///     std::assert_eq!(result, std::result::Result::Ok(8.5));
/// }
///
/// // To run this specific doc test: `rustdoc --test calculate_weighted_sum.rs`
/// // Assuming the function is in a library, you would define it in the lib.rs
/// // and then the doc test could be run with `cargo test`.
/// pub fn calculate_weighted_sum(values: &[f64], weights: &[f64]) -> std::result::Result<f64, std::string::String> {
///     if values.len()!= weights.len() {
///         return std::result::Result::Err(std::string::String::from("Value and weight slices must have the same length."));
///     }
///     if values.is_empty() {
///         return std::result::Result::Ok(0.0);
///     }
///     let weighted_sum: f64 = values.iter().zip(weights.iter()).map(|(v, w)| v * w).sum();
///     std::result::Result::Ok(weighted_sum)
/// }
/// ```
pub fn calculate_weighted_sum(values: &[f64], weights: &[f64]) -> Result<f64, String> {
  // Validate input lengths. This is the most common failure point.
  if values.len()!= weights.len() {
    return std::result::Result::Err(String::from(
      "Value and weight slices must have the same length.",
    ));
  }
  // Handle the edge case of empty slices, defined as a valid input returning 0.0.
  if values.is_empty() {
    return std::result::Result::Ok(0.0);
  }

  // Calculate using a functional style for clarity and conciseness.
  let weighted_sum: f64 = values
    .iter()
    .zip(weights.iter())
    .map(|(v, w)| v * w)
    .sum();

  std::result::Result::Ok(weighted_sum)
}

#[cfg(test)]
mod tests {
  #[test]
  fn test_basic_weighted_sum() {
    // Test: Validates the calculation with standard positive floating-point numbers.
    // Justification: This is the primary success case and ensures the core logic is correct
    // under normal, expected conditions.
    let values = [1.0, 2.0, 3.0];
    let weights = [0.5, 1.0, 2.0];
    // Expected: (1.0 * 0.5) + (2.0 * 1.0) + (3.0 * 2.0) = 0.5 + 2.0 + 6.0 = 8.5
    let result = super::calculate_weighted_sum(&values, &weights);
    std::assert_eq!(result, std::result::Result::Ok(8.5));
  }

  #[test]
  fn test_empty_slices() {
    // Test: Ensures the function handles empty slices gracefully.
    // Justification: This is a critical edge case. The function defines the sum of an
    // empty set as 0.0, and this test verifies that specific behavior.
    let values: [f64; 0] =;
    let weights: [f64; 0] =;
    let result = super::calculate_weighted_sum(&values, &weights);
    std::assert_eq!(result, std::result::Result::Ok(0.0));
  }

  #[test]
  fn test_mismatched_lengths() {
    // Test: Verifies that the function returns an error when input slices have different lengths.
    // Justification: This is the primary failure case for input validation. The function MUST
    // reject mismatched slices to prevent incorrect calculations or panics.
    let values = [1.0, 2.0];
    let weights = [0.5];
    let result = super::calculate_weighted_sum(&values, &weights);
    std::assert!(result.is_err());
    std::assert_eq!(
      result.unwrap_err(),
      String::from("Value and weight slices must have the same length.")
    );
  }

  #[test]
  fn test_negative_values_and_weights() {
    // Test: Validates the calculation with a mix of negative and positive numbers.
    // Justification: This test ensures the function's mathematical correctness extends
    // beyond simple positive values and handles negative inputs correctly.
    let values = [-1.0, 2.0];
    let weights = [3.0, -0.5];
    // Expected: (-1.0 * 3.0) + (2.0 * -0.5) = -3.0 + -1.0 = -4.0
    let result = super::calculate_weighted_sum(&values, &weights);
    std::assert_eq!(result, std::result::Result::Ok(-4.0));
  }
}
````

### 

### **2\. Output Protocol**

All code modification tasks MUST produce their output as a single JSON array within a fenced code block. Each object in the array represents a single, atomic change and MUST conform to the following schema:

```json
[
  {
    "action": "replace",
    "file_path": "path/to/file.rs",
    "line_range": [<start_line_number>, <end_line_number>],
    "new_code": "<string containing the new code block>",
    "justification": "A brief explanation of the change, referencing the specific rule from this protocol (e.g., 'Adheres to C-QUESTION-MARK')."
  }
]
```

### 

### **3\. Running Test Commands**

In a multi-crate project where we are working on a single crate, you MUST run the tests from inside the crate's directory and ASK before running tests from root.

---

# **TypeScript Coding Standards: Evidence-Based Clarity & Modularity**

These standards adapt the core philosophy of the Rust coding standards—Maintainability, Reliability, Security, and Speed—to the TypeScript ecosystem. The goal is to enforce absolute clarity, remove ambiguity, and ensure robust, type-safe development.

## **A. CRITICAL: Type Safety & Path Unambiguity**

1. **NO `any` TYPE**: The any keyword is FORBIDDEN. It undermines the entire purpose of TypeScript. Use unknown for values of unknown type and perform type checking before use.
2. **NO `default` EXPORTS**: All exports MUST be named exports. This enforces absolute clarity on what is being imported and removes ambiguity.
  * **Correct**: `export const myService = { ... };`
  * **Incorrect**: `export default myService;`
3. **EXPLICIT NAMED IMPORTS**: All imports MUST be explicit and named. Wildcard imports are FORBIDDEN. This ensures the origin of every symbol is immediately clear.
  * **Correct**: `import { analyzeProject } from '@/services/analysisService';`
  * **Incorrect**: `import * as AnalysisService from '@/services/analysisService';`
4. **NON-RELATIVE PATH ALIASES**: All imports for project modules MUST use non-relative path aliases configured in tsconfig.json. This avoids fragile and unreadable relative paths (../../../).
  * **Correct**: `import { CoreButton } from '@/components/ui/CoreButton';`
  * **Incorrect**: `import { CoreButton } from '../../../components/ui/CoreButton';`

## **B. File & Module Structure**

1. **One Logical Item Per File**: Each .ts file SHOULD contain exactly one primary logical item (e.g., class, interface, a set of related functions).
  * Associated types or interfaces SHOULD reside in the same file as the primary item they describe.
  * File Naming: Files MUST be named using camelCase.ts or PascalCase.ts that matches the item they contain (e.g., analysisService.ts for analysisService, GraphNode.ts for class GraphNode). Svelte components are an exception and MUST use PascalCase.svelte.
2. **Module Index Files (index.ts)**: These files are for re-exporting ONLY. They MUST contain only export statements and SHOULD NOT contain original item definitions. This mirrors the mod.rs philosophy.
  * Example services/index.ts:  
    export \* from './analysisService';  
    export \* from './fileLoaderService';

## **C. Documentation & Style**

1. **File Preamble**: Every non-trivial .ts file MUST begin with a file-level JSDoc comment (/\*\* ... \*/). The first line is a single-sentence summary, followed by a blank line and a brief expansion.
2. **Item Documentation**: All exported items (functions, classes, types, interfaces) MUST have JSDoc comments (/\*\* ... \*/) that include @param, @returns, and @throws tags where applicable.
3. **Interfaces as Contracts**: Interfaces (interface) or type aliases (type) are used to define clear data structures and service contracts. They are the primary tool for ensuring type safety across the IPC boundary.
4. **Functional Style**: Prefer immutable data, pure functions, and array methods (.map, .filter, .reduce) over imperative loops and mutable state where it enhances clarity and predictability. Use read-only types (Readonly\<T\>, ReadonlyArray\<T\>) where appropriate.
5. **Brevity and Clarity**: Code SHOULD be self-documenting. Avoid unnecessary comments that merely restate what the code does. Comments should explain *why*, not *what*.

## **D. Function & Error Handling**

1. **Function Length**: Functions SHOULD NOT exceed 75 lines of code (LoC), excluding signature, JSDoc, and blank lines. Justify rare exceptions with a comment.

2. **Explicit Error Handling**: Avoid generic `throw new Error("...")`. Define custom error classes that extend Error for different failure domains (e.g., IPCError, AnalysisError).

3. **Type-Safe Results for Asynchronous Operations**: Functions that perform fallible asynchronous operations (like IPC calls) MUST return a `Promise<Result<T, E>>` where Result is a discriminated union. This makes error handling explicit and avoids try/catch blocks for expected failures.

  * **Result\<T, E\> Type Definition**:  
    `export type Result<T, E extends Error> = { ok: true; value: T } | { ok: false; error: E };`



4. **NO .unwrap() OR .expect()**: Never create utility functions like .unwrap() that throw on an error result. The consumer of the function MUST explicitly handle both the ok: true and ok: false cases.

## **E. Testing**

1. **Colocated Tests**: Unit tests for an item SHOULD reside in the same directory, named with a .test.ts or .spec.ts suffix (e.g., analysisService.test.ts).
2. **Test Clarity**: Tests MUST be clear, concise, and test only one logical condition per test block. Each test should follow the Arrange-Act-Assert pattern.

### **Example Ideal File: analysisService.ts**

This example adheres to all specified rules for a TypeScript service file.

```
/**
 * Provides services for interacting with the Rust core for code analysis.
 *
 * This service acts as a type-safe adapter for the Tauri IPC commands,
 * handling data serialization/deserialization and robust error mapping.
 * It ensures that all communication with the backend is explicit and predictable.
 */

import { invoke } from '@tauri-apps/api/core';
import { decode } from '@msgpack/msgpack';
import type { Graph, GraphHandle, NodeId, Metrics } from '@/types/graph';
import type { Result } from '@/types/result';
import { AnalysisError, IPCError } from '@/types/errors';

/**
 * Defines the contract for the code analysis service.
 * This interface represents the "Input Port" from the frontend's perspective.
 */
export interface IAnalysisService {
	/**
	 * Triggers the full analysis pipeline for a project at a given file path.
	 *
	 * @param path - The absolute file path to the project directory.
	 * @returns A promise resolving to a Result containing the graph handle or an error.
	 */
	analyzeProject(path: string): Promise<Result<GraphHandle, AnalysisError | IPCError>>;

	/**
	 * Retrieves the full graph data associated with a given handle.
	 *
	 * @param handle - The handle to the analyzed graph state in the Rust core.
	 * @returns A promise resolving to a Result containing the deserialized Graph or an error.
	 */
	getGraphData(handle: GraphHandle): Promise<Result<Graph, AnalysisError | IPCError>>;
}

/**
 * Triggers the full analysis pipeline for a project at a given file path.
 *
 * This function calls the `analyze_project` Tauri command, which is expected
 * to return a MessagePack-encoded byte array representing the `GraphHandle`.
 *
 * @param path - The absolute file path to the project directory.
 * @returns A promise resolving to a Result containing the graph handle or an error.
 * @throws {IPCError} - If the Tauri `invoke` call itself fails.
 */
export async function analyzeProject(path: string): Promise<Result<GraphHandle, AnalysisError | IPCError>> {
	try {
		const handle = await invoke<GraphHandle>('analyze_project', { path });
		return { ok: true, value: handle };
	} catch (e) {
		if (e instanceof Error) {
			// Map the raw invoke error to a structured application error.
			return { ok: false, error: new AnalysisError(`Project analysis failed: ${e.message}`) };
		}
		return { ok: false, error: new IPCError('An unknown IPC error occurred during analysis.') };
	}
}

/**
 * Retrieves the full graph data associated with a given handle.
 *
 * This function calls the `get_graph_data` command and is responsible for
 * deserializing the MessagePack byte array (`Uint8Array`) into the canonical
 * `Graph` TypeScript type.
 *
 * @param handle - The handle to the analyzed graph state in the Rust core.
 * @returns A promise resolving to a Result containing the deserialized Graph or an error.
 */
export async function getGraphData(handle: GraphHandle): Promise<Result<Graph, AnalysisError | IPCError>> {
	try {
		// The backend returns raw bytes, so we type the invoke result as `Uint8Array`.
		const rawGraphData = await invoke<Uint8Array>('get_graph_data', { handle });

		// Deserialize the MessagePack bytes into a structured object.
		// We cast to `unknown` first to safely type-check before casting to `Graph`.
		const decodedData = decode(rawGraphData) as unknown;

		// Here you would add runtime validation to ensure `decodedData` matches the `Graph` shape.
		// For this example, we'll assume it's valid.
		const graph = decodedData as Graph;

		return { ok: true, value: graph };
	} catch (e) {
		if (e instanceof Error) {
			return { ok: false, error: new AnalysisError(`Failed to get graph data: ${e.message}`) };
		}
		return { ok: false, error: new IPCError('An unknown IPC error occurred while fetching graph data.') };
	}
}
```


## **Claude Sonnet 4.5 Prompt Preamble: System Generation Mandate**

**Objective:** Generate a detailed, step-by-step implementation plan and critical code structure (e.g., Rust trait definitions, configuration files) for a high-performance, language-agnostic code canvas application, adhering strictly to the architectural constraints outlined below.

**Tooling Context Instruction:** For all implementation steps involving Rust, SvelteKit (Svelte 5), or Tauri, assume the execution environment is **IntelliJ with Context 7 enabled as an MCP server that you MUST use**. This ensures access to the most up-to-date and relevant library information for all crates and packages referenced in the plan, specifically tauri, rmp-serde, tree-sitter, and cytoscape.

**Key Architectural Constraints (Non-Negotiable):**

1. **Architecture:** Strict Hexagonal Architecture (Hexser pattern).
2. **Core Logic:** 100% of heavy logic (Parsing, Graph Generation, Metrics) must reside in the Rust core.
3. **Visualization Library Policy:** Minimize external JavaScript libraries. The exception is **Cytoscape.js**, which is mandatory for high-performance WebGL rendering of complex graphs.
4. **Frontend Language Policy:** All frontend code, including component logic and library interactions, must be implemented using **TypeScript** (Svelte 5 \+ TS), utilizing type definitions for all dependencies (e.g., @types/cytoscape, MessagePack deserializer types).

---

# **Expert System Design Document: Language-Agnostic Code Canvas Platform (Rust/Tauri/SvelteKit)**

## **I. System Architecture Mandate: The Hexagonal (Hexser) Blueprint**

The development of a high-performance, language-agnostic code visualization platform requires a robust architectural foundation capable of isolating complex analysis logic from presentation and infrastructure concerns. This design mandates strict adherence to the Hexagonal Architecture (Ports and Adapters), often referenced as the Hexser pattern, ensuring modularity, testability, and clear separation of responsibilities.1 By placing all "heavy logic" exclusively within the Rust core, the system minimizes the performance impact of Inter-Process Communication (IPC) and maintains the domain as the definitive source of truth.3

### **I.A. Conceptual Mapping: Hexagonal Architecture in the Tauri/Rust Stack**

In the context of a Tauri-based desktop application utilizing a Rust core and a SvelteKit frontend, the Hexagonal structure maps precisely to the technology stack. The **Application Core (Inner Hexagon)** consists solely of pure Rust code, encapsulated within a dedicated core\_domain crate. This layer defines the canonical data models (Graph, Node, Edge) and the business rules for generating and querying code dependencies.2

**Ports** are defined as Rust Traits within this core domain, establishing precise contracts for communication without specifying the underlying infrastructure implementation.

The external layers interact via **Adapters**:

1. **Driving Adapters** manage requests initiating from the external world. In this system, the SvelteKit user interface, communicating via the Tauri IPC layer, serves as the primary Driving Adapter. It translates user actions (e.g., clicking an "Analyze Project" button) into command calls directed at the Core's input ports.1
2. **Driven Adapters** fulfill the core's needs for external services. These include infrastructure implementations such as the File System I/O handler (persistence\_adapter) and the multi-language Analysis Engine (analysis\_adapter), which implements the necessary ports for parsing code and calculating metrics.1

This strict separation, facilitated by utilizing multiple Rust sub-projects or crates, ensures the core domain logic remains isolated and easily testable, irrespective of whether the application is running locally via the Tauri adapter or via a hypothetical web server.4

The architectural isolation necessitates a deliberate strategy regarding language idioms. While Hexagonal architecture can sometimes introduce an object-oriented programming (OOP) feel in Rust implementations, often involving dynamic dispatch using Box\<dyn Trait\> for ports 4, the design favors minimizing runtime overhead. When defining the Core's ports, the implementation will prioritize static dispatch through generics and trait bounds whenever feasible. Dynamic dispatch should be reserved primarily for the infrastructure composition root (the application entry point in main.rs) where concrete adapters are injected, thus avoiding unnecessary boxing within the performance-critical domain logic.

Furthermore, this architecture dictates a crucial performance strategy. Since the Core is responsible for generating and maintaining the massive Abstract Syntax Trees (ASTs) and derived graphs, it must handle the computationally intensive steps (parsing, metric calculation).5 The resulting visualization data must be pre-processed and optimized by the Core before leaving the hexagon. This prevents the performance bottleneck from occurring during the IPC transfer, ensuring that the heavy burden of analysis does not impact UI responsiveness, a fundamental requirement when dealing with large codebases.6

### **I.B. Hexagonal Component Mapping and Project Structure**

The physical organization of the project enforces the Hexagonal structure using separate Rust crates or well-defined modules. This compartmentalization ensures compile-time dependency checks, guaranteeing that infrastructure concerns do not bleed into the pure domain logic.

Table I.B.1: Hexagonal Architecture Component Mapping (Rust/Tauri/SvelteKit)

| Hexagonal Layer | Component Type | Technology Stack / Rust Crate | Key Functionality |
| :---- | :---- | :---- | :---- |
| Application Core | Domain Logic & Ports | core\_domain (Rust Crate) | Defines Entities, Value Objects, and all Port Traits (IDrivePort, IDrivenPort). |
| Driving Adapter | User Interface Interaction | SvelteKit UI / Tauri invoke calls | Translates user input into Core Port calls (e.g., Button click \-\> AnalyzeProject()). |
| Input Port | Application Interface | core\_domain::ports::input (Rust Traits) | Defines command interfaces that the Driving Adapter uses to run the core application logic. |
| Output Port | External Service Interface | core\_domain::ports::output (Rust Traits) | Defines interfaces the Core uses to request external resources (e.g., LoadFile, RunAnalysis). |
| Driven Adapters | Infrastructure Implementation | analysis\_adapter (Tree-Sitter/Metrics); tauri\_ipc\_adapter (IPC Implementation); persistence\_adapter (File I/O) | Implements the Output Port traits to connect the Core to the external world. |

## **II. The Application Core: Rust Domain Logic (core\_domain)**

The core\_domain crate is entirely infrastructure-agnostic. Its sole responsibility is to define the application's business objects and the operations that transform them.

### **II.A. Domain Modeling: The Canonical Graph Structure**

The primary data structure is the **Dependency Graph**, which models the relationships within the analyzed codebase. This structure must be carefully standardized to remain language-agnostic, allowing the system to handle diverse programming languages without modifying the core model.

Table II.A.1: Core Domain Entities and Value Objects

| Entity/Value Object | Description | Key Attributes | Source Data |
| :---- | :---- | :---- | :---- |
| **Graph** (Entity) | The collection of all analyzed nodes and edges for a project. | id, project\_name, nodes (Vec), edges (Vec). | Aggregated from Analysis Adapter. |
| **Node** (Entity) | Represents a logical code element (File, Module, Function, Struct, Class). | node\_id, type (Enum), name, language, location (File/Line), metrics (Value Object). | Parsed AST/Metrics.5 |
| **Edge** (Value Object) | Represents a directed relationship (dependency, call, reference) between two nodes. | source\_id, target\_id, type (Enum: Call, Import, Inheritance), weight. | Derived from AST relationship queries.7 |
| **Metrics** (Value Object) | Quantifiable analysis data associated with a Node. | cc (Cyclomatic Complexity), sloc (Source LOC), halstead\_effort, nargs (Arguments). | rust-code-analysis output.5 |

To achieve language agnosticism, the Core mandates normalization. The Node entity uses abstract, generalized types (e.g., Container, Executable, Reference) instead of language-specific terms like "struct" or "class." The analysis\_adapter is responsible for translating specific language structures (e.g., a Rust fn definition or a Python def statement) into these canonical Node types. This design insulates the domain from future language changes.

Furthermore, the Core must strictly maintain data authority. If the visualization layer requires a subset or refinement of the graph (e.g., filtering nodes based on high Cyclomatic Complexity), the UI must invoke a Core service method, such as GetFilteredGraph(query), rather than performing local manipulation.9 This preserves the integrity of the analysis results and prevents issues where changes to the data source are pushed back through retrieval formulas, ensuring that data modification is handled by specialized formulas/ports in the Core.9

### **II.B. Defining Input Ports (Driving the Core)**

Input Ports define the actions that the external environment (the Svelte UI via Tauri IPC) can execute against the core domain. These are represented by the CodeCanvasService trait.

**Port Trait:** CodeCanvasService (The orchestrator of domain actions)

| Method | Description | Return Type |
| :---- | :---- | :---- |
| analyze\_project(path: PathBuf) | Triggers full analysis pipeline (File loading, Parsing, Graph generation, Metrics calculation). | Result\<GraphHandle, AppError\> |
| get\_graph\_data(handle: GraphHandle, query: GraphQuery) | Retrieves a specific, potentially filtered, section of the master graph state. | Result\<Graph, AppError\> |
| save\_state(handle: GraphHandle) | Persists the current analysis state (potentially including user annotations) to disk. | Result\<(), AppError\> |
| calculate\_metrics(node\_id: NodeId) | Request computation or retrieval of detailed metrics for a specific node. | Result\<Metrics, AppError\> |

### **II.C. Defining Output Ports (Core Driving Infrastructure)**

Output Ports define the external services that the Core domain relies upon to execute its logic. These are also defined as traits within the core\_domain.

**Port Trait:** ILanguageAnalysis (The interface to parsing and metric generation)

| Method | Description | Return Type | Driven Adapter |
| :---- | :---- | :---- | :---- |
| parse\_file(content: \&str, language: Language) | Generates the language-specific AST. | Result\<RawAST, AppError\> | Tree-Sitter Adapter 10 |
| generate\_graph\_data(ast: RawAST) | Transforms the AST into the canonical Core Graph structure (Nodes/Edges). | Result\<(Vec\<Node\>, Vec\<Edge\>), AppError\> | tree-sitter-graph or Custom Logic 7 |
| calculate\_code\_metrics(content: \&str) | Runs metric calculations (CC, Halstead) on the source code. | Result\<Metrics, AppError\> | rust-code-analysis Adapter 5 |

**Port Trait:** IFileLoader (The interface for infrastructure I/O)

| Method | Description | Return Type | Driven Adapter |
| :---- | :---- | :---- | :---- |
| load\_directory\_recursively(path: PathBuf) | Loads all relevant source files in a project path. | Result\<Vec\<CodeFile\>, AppError\> | FileSystem Adapter |
| stream\_file\_content(path: PathBuf, channel: Channel) | Streams large file content for real-time progress updates or binary transfer optimization. | Result\<(), AppError\> | FileSystem Adapter (Utilizing Tauri Channels) 11 |

## **III. Language-Agnostic Code Analysis Pipeline (Driven Adapter Implementation)**

The analysis\_adapter crate implements the ILanguageAnalysis Output Port, managing the complex process of turning raw source code into structured, analyzed domain entities.

### **III.A. AST Generation Adapter: Tree-Sitter and Modular Parsers**

Language agnosticism relies fundamentally on the choice of parsing technology. The system utilizes Tree-Sitter, which provides robust, incremental parsing for many languages, crucial for future scalability.5 This approach provides significant advantages over language-specific tools, such as rusty-ast 12 or dep\_graph\_rs 13, which are limited to a single codebase type.

The implementation of parse\_file dynamically selects the appropriate Tree-Sitter grammar based on the file type. The adapter abstracts the complexity of loading and querying these parsers, returning a standardized representation of the raw AST to the Core.10 This design ensures that the Core never needs to understand the specifics of, for instance, C++ header files versus JavaScript module imports; it simply receives a generic tree structure ready for relationship extraction.

### **III.B. Graph Construction Engine and Dependency Mapping**

The subsequent step, transforming the raw AST into the Core's normalized Graph structure, is the most computationally intensive aspect of the analysis pipeline. This process involves identifying logical nodes and, critically, mapping dependencies between them.

The implementation will leverage the Rust library tree-sitter-graph 14, which provides a Domain Specific Language (DSL) for constructing graphs based on Tree-Sitter query results.7 Complex queries are designed to identify semantic relationships, such as cross-file imports, function calls, and structural hierarchies, going far beyond basic internal module tracking provided by simpler tools.13

This deep structural analysis, while necessary for a comprehensive code canvas experience, inherently comes with a substantial processing cost, particularly for large projects. Since the graph generation process can take a significant amount of time, resulting in a potentially unresponsive application, the system must address this performance hurdle upfront. The required mitigation is to integrate asynchronous progress feedback. The analysis\_adapter, driven by the Core, must utilize Tauri Channels (Section IV.C) to stream real-time updates (e.g., percentage complete, current file being processed) back to the Svelte UI during the initial parsing and graph construction phases.11

### **III.C. Metrics Calculation Adapter**

The calculation of code metrics is handled by a separate section of the analysis\_adapter, specifically integrating the rust-code-analysis library.5 This library supports metrics like Cyclomatic Complexity (CC), Source Lines of Code (SLOC), Logical Lines of Code (LLOC), and Halstead metrics (effort, difficulty, bugs estimate) for multiple languages.5

The calculate\_code\_metrics method is implemented to receive the source code content and return the structured Metrics Value Object. By adhering to the ILanguageAnalysis port contract, the system ensures that the specific calculation engine can be modified or replaced (e.g., if a more efficient metrics library emerges) without requiring any changes to the Core Domain or the Graph construction logic.

## **IV. Inter-Process Communication (IPC) and Performance Adapter Design**

Performance is a defining characteristic of this application, especially considering the large graph datasets generated by analysis tools.6 The Tauri IPC layer, serving as the system's primary Driving Adapter, must be aggressively optimized to mitigate serialization overhead.

### **IV.A. The Tauri IPC Adapter (tauri\_ipc\_adapter)**

The tauri\_ipc\_adapter crate implements the Core's CodeCanvasService Input Port. It exposes public Rust functions decorated with \#\[tauri::command\] that the Svelte frontend can invoke.3

The adapter handles two critical responsibilities: routing and state management. When the Svelte UI invokes a command (e.g., invoke('analyze\_project', {... })), the adapter translates this external request into a standardized call to the concrete implementation of the CodeCanvasService. To maintain the integrity and persistence of the analyzed graph data across multiple query calls (e.g., analyze once, filter many times), the adapter must hold the master Graph state. This is typically managed in the Rust backend using managed state, such as tauri::State or an Arc\<Mutex\<...\>\> wrapped structure, ensuring the Core's state persists between asynchronous frontend requests.

### **IV.B. Advanced IPC Serialization Strategy for High Throughput**

The default Tauri IPC uses JSON serialization, which is highly readable but extremely inefficient for transferring large, complex data structures like a dependency graph containing thousands of nodes and edges.6 To achieve the requisite performance for an interactive canvas, a binary serialization format is mandatory.

The system mandates the use of **MessagePack (via rmp-serde)** for all large data transfers, such as the initial graph output and complex query results. Empirical evidence indicates that binary formats like MessagePack can be 1.2x to 2.0x faster than JSON and yield significantly smaller data payloads (e.g., reducing 240 Kb of JSON to 130 Kb of MessagePack).15

The implementation requires a bypass of Tauri's default JSON serialization. The tauri\_ipc\_adapter will explicitly serialize the Core's Graph entity into a raw byte vector (Vec\<u8\>) using rmp-serde before returning the data via the command.

Table IV.B.1: IPC Serialization Strategy for Performance

| Data Structure | Standard Tauri Format | Optimized Format (Mandate) | Implementation Approach |
| :---- | :---- | :---- | :---- |
| Simple Commands/Status | JSON (Default Tauri) | JSON (Default Tauri) | Used for small, readable metadata exchange. |
| Large AST/Graph Data (Nodes/Edges) | JSON (Slow) | MessagePack (via rmp-serde) | Crucial performance optimization. Requires Rust adapter to serialize to Vec\<u8\> and UI to deserialize.15 |
| Real-time Progress Updates | Events/JSON | Tauri Channels (tauri::ipc::Channel) | Required for non-blocking, asynchronous streaming of analysis progress or partial results.11 |

The frontend (SvelteKit) must be equipped to handle this binary data. Upon receiving the Vec\<u8\> output, the **TypeScript** layer must explicitly deserialize the MessagePack bytes using a corresponding, **TypeScript-compatible** library (e.g., @msgpack/msgpack with type definitions) before feeding the resulting object structure to the Cytoscape.js rendering engine. This ensures all frontend code, including the critical deserialization logic, is strongly typed.

### **IV.C. Asynchronous Communication and Robust Error Handling**

Heavy operations, such as the initial recursive directory load and the subsequent graph generation, must run asynchronously and provide continuous feedback to maintain a non-blocking user experience.

The system will leverage **Tauri Channels** (tauri::ipc::Channel) to facilitate non-blocking communication between the Rust backend and the Svelte frontend.11 This is used for streaming updates during long-running tasks, allowing the Rust Core, via its driven adapters, to push progress reports (e.g., file processing status) asynchronously to the UI.

For error handling, the system defines a custom AppError enum within the core\_domain. This error type implements serde::Serialize 16, ensuring that errors returned from \#\[tauri::command\] functions are explicit, structured, and contain recognizable error codes. This practice gives the Svelte frontend a reliable contract, allowing it to easily map returned errors to a corresponding TypeScript error enum for precise user feedback, saving significant effort in debugging and future refactoring.16

## **V. Presentation Layer (SvelteKit and Visualization Adapter)**

The Presentation Layer serves as the consumer of the analysis results and the interpreter of user interactions, translating them into input port commands for the Rust Core.

### **V.A. SvelteKit Setup and Tauri Compatibility**

SvelteKit is primarily designed for Server-Side Rendering (SSR). To function within the embedded Webview environment provided by Tauri, the SvelteKit project must be specifically configured to disable SSR and utilize the @sveltejs/adapter-static.1 This configuration ensures that the Svelte application compiles into purely static assets that can be bundled and served by the Tauri binary. The frontend interacts with the Rust core using the standard Tauri API (@tauri-apps/api/core) for command invocation.6 The initial SvelteKit setup will use the **TypeScript template** to ensure type safety is established from the start.1

### **V.B. Visualization Library Selection: Cytoscape.js Mandate**

For a code canvas that must handle potentially complex networks involving thousands of nodes and edges 11, the visualization library choice is critical for performance. The design strictly minimizes JavaScript libraries, but the selection mandates **Cytoscape.js** as a necessary exception. This is due to its unmatched optimization for high complexity networks, utilizing WebGL rendering, and its integrated breadth of layout algorithms (dagre, klay, cose-bilkent) essential for visualizing dependencies efficiently.2 To meet the **TypeScript compatibility** mandate, the implementation will utilize the corresponding official type definitions (e.g., @types/cytoscape), ensuring all frontend logic remains strongly typed.

### **V.C. The Visualization Adapter and Canvas Interaction Design**

The Visualization Adapter is a dedicated Svelte component implemented entirely in **TypeScript** (Svelte \+ TS) responsible for marshaling data between the IPC layer and Cytoscape.js.

Its first task is client-side data translation: receiving the MessagePack byte array from the IPC adapter, deserializing it into a **TypeScript** object, and then mapping the Core's canonical Node and Edge structures into the specific JSON format required by Cytoscape.js, ensuring end-to-end type safety.

Critically, all user interactions that relate to the underlying data model (e.g., filtering, modification, refinement) must adhere to the Core's data authority principle. The Visualization Adapter does not modify the graph state locally; instead, it translates user actions (e.g., "Hide all nodes tagged as utility") into invocations of the Core's Input Port methods (CodeCanvasService::get\_filtered\_graph). The Core performs the filtering in Rust and returns a new, optimized graph subset via MessagePack, which the adapter then renders. This closed-loop control ensures that the domain logic remains the single source of truth for the analysis results.

### **V.D. Visualization Feature Mapping**

The visualization features provided by the Svelte UI must directly correspond to the analytical capabilities exposed by the Rust core.

Table V.D.1: Feature Mapping and Core Interaction

| Visualization Feature | Rust Core Interaction (Port Method) | Data Source / Rationale |
| :---- | :---- | :---- |
| Project Structure Tree View | Initial analysis; Data retrieved via get\_graph\_data | Displays high-level Node entities (Files/Modules) for navigation. |
| Dynamic Filtering (Metric-based) | CodeCanvasService::get\_filtered\_graph(query) | Implements dynamic refinement logic entirely in the Rust Core for performance.17 |
| Node Details Pane | CodeCanvasService::calculate\_metrics(node\_id) | Populates details (CC, LOC, NARGS) using data retrieved from the rust-code-analysis adapter.5 |
| Infinite Pan/Zoom | Handled natively by Cytoscape.js / WebGL | Leverages high-performance rendering capabilities.11 |

## **VI. Synthesis and Conclusions**

The implementation plan is architecturally driven by the Hexagonal model, focusing heavily on performance and language agnosticism. The critical success factors are derived from careful component segregation and optimization of the inter-layer communication protocols.

### **VI.A. Core Rust Traits and Implementation Crates Summary**

The successful separation of concerns hinges on correctly defining and implementing the necessary Rust traits (Ports) within dedicated crates (Adapters). This structure enforces low coupling and high testability for the core domain.

Core Rust Traits and Implementation Crates (Summary)

| Layer | Trait (Port) | Implementation Crate (Adapter) | Rust Type |
| :---- | :---- | :---- | :---- |
| Driving Input | CodeCanvasService | tauri\_ipc\_adapter | Concrete Struct implementing CodeCanvasService Trait |
| Driven Output | ILanguageAnalysis | analysis\_adapter | Concrete Struct implementing ILanguageAnalysis Trait |
| Driven Output | IFileLoader | persistence\_adapter | Concrete Struct implementing IFileLoader Trait |
| Domain Orchestration | N/A | core\_domain | Entities, Value Objects, Core Business Logic |

### **VI.B. Nuanced Conclusions and Actionable Recommendations**

1. **Mandatory IPC Optimization:** The size and complexity of code dependency graphs mandate deviating from Tauri's default JSON IPC. The implementation must allocate resources to rigorously develop and test the MessagePack serialization/deserialization routines across the Rust and Svelte layers, as this optimization is the single most important factor for achieving a smooth, responsive user experience during data transfer.15
2. **Centralized Data Authority:** All complex data manipulation, including filtering and calculated metric retrieval, must remain within the Rust core\_domain.17 This practice ensures that the Rust core maintains the canonical truth of the analysis results, providing consistency and preventing the fragmented data state often observed when complex operations are pushed down to the UI layer.17
3. **Language Agnostic Strategy:** Tree-Sitter is the correct choice for future-proofing the system. The upfront effort in defining normalized domain entities (Nodes, Edges) and crafting robust, multi-language Tree-Sitter queries is essential.14 This complexity must be managed by treating the analysis\_adapter as a distinct module with its own comprehensive testing suite, validating that the adapter correctly maps various language constructs to the Core's generalized model.
4. **Managing Long-Running Tasks:** Given the computational intensity of deep AST analysis and graph generation, the integration of Tauri Channels is non-negotiable. Streaming progress updates back to the Svelte UI ensures user confidence and prevents the application from appearing frozen during lengthy initial processing.11
5. **Architectural Clarity:** By adhering to the Hexagonal principle and utilizing Rust crates to enforce boundaries, the project avoids the common pitfalls noted in non-idiomatic Hexagonal implementations. 4 This clear separation of domain logic from infrastructure (I/O, IPC, parsing engine) will significantly lower the maintenance burden and improve the modularity for future feature additions.
6. **TypeScript-First Frontend:** The commitment to TypeScript must be enforced across the entire SvelteKit/Tauri frontend. This includes utilizing the SvelteKit TypeScript template and installing @types packages for every external JavaScript dependency (e.g., Cytoscape.js, MessagePack deserializer) to maintain strong type safety across the IPC boundary.

### **VI.C. Visual Framework**
Svelte will use the Skeleton UI framework with tailwind for the frontend. Use Context7 MCP server for the latest version information. When adding a new basic component, we should check to see if Skeleton has the compoent first before writing one custom.

---
Expert System Design Document (Revision 2.0): The Interactive Code Canvas PlatformI. System Architecture Mandate: The Evolved Hexagonal BlueprintThis document presents a revised architectural mandate for the Language-Agnostic Code Canvas Platform. It builds upon the foundational principles of the initial design, adapting and extending them to support a significantly more dynamic and interactive user experience. The primary directive is to evolve the system from a static dependency visualizer into a fully interactive "infinite canvas" environment, inspired by the capabilities of modern developer tools.1 This evolution necessitates a corresponding evolution of the architectural blueprint. The commitment to the Hexagonal Architecture (Ports and Adapters), or the "Hexser" pattern, is not only reaffirmed but is now considered even more critical to managing the increased complexity of the system's new requirements.3I.A. Reaffirming the Hexagonal Foundation for an Interactive CanvasThe original mandate for a strict separation of concerns remains the cornerstone of this design. The core domain, encapsulated within the core_domain Rust crate, must remain pure and agnostic of all external infrastructure. In the initial plan, this primarily meant isolation from the user interface, file system, and parsing libraries.3 The revised scope introduces new, complex infrastructure dependencies, most notably the need to interact with version control systems (specifically Git) and to process a continuous stream of fine-grained user interactions from the canvas.4 A new, critical requirement is that the application must be deployable as both a native desktop application and a web application, necessitating a flexible architecture that can support multiple delivery mechanisms without duplicating core logic.5The Hexagonal Architecture provides the necessary structural integrity to accommodate these new features without corrupting the core's logical purity. By defining explicit ports (Rust traits) for these new responsibilities, the system can integrate sophisticated functionality like real-time diff visualization and deep symbol-level analysis through dedicated, interchangeable adapters. This same principle allows for the creation of multiple, distinct "driving" adapters—one for the Tauri desktop environment and another for a web server—that both interact with the exact same core application logic.6 This ensures that the central business logic—the rules for modeling and querying code structures—remains independently testable and insulated from the implementation details of how Git status is fetched or how a user's request is received, whether via Tauri's IPC or an HTTP request. This architectural discipline is paramount for maintaining long-term scalability and preventing the tight coupling that often plagues complex, interactive applications.3I.B. Conceptual Mapping for Real-Time AnalysisThe conceptual mapping of the architecture to the technology stack is updated to reflect the system's new, highly interactive and dual-platform nature. The SvelteKit user interface remains the primary user-facing component, but it will communicate with the backend via one of two Driving Adapters depending on the environment. For the desktop application, it will use Tauri's Inter-Process Communication (IPC) layer. For the web application, it will use standard HTTP requests to a web server backend.7 In both cases, the adapter's role is transformed. It no longer initiates a few high-level, batch-oriented commands like analyze_project. Instead, it drives a continuous stream of granular requests that correspond directly to user actions on the canvas, such as "get references for this specific function token," "open all files that import this module," or "apply a Dagre layout to the currently selected nodes".3On the driven side of the hexagon, the responsibilities of the existing adapters are expanded, and a new, critical adapter is introduced. The analysis_adapter, which implements the ILanguageAnalysis port, must now go beyond file-level dependency mapping to perform deep, symbol-level analysis, identifying function calls, variable references, and class implementations.4 To fulfill the requirement for visualizing real-time code changes, a new VersionControlAdapter is mandated. This adapter will implement a new IVersionControl output port, abstracting all interactions with the local project's Git repository. This clear separation ensures that the core domain can request information about code changes without any knowledge of the underlying version control system's command-line interface or library APIs.4The transition from a "load and view" application to a dynamic "infinite canvas" is not merely a feature addition; it fundamentally alters the system's temporal nature and state management model. The original architecture implied a largely stateless, transactional process: a user selects a project, the system performs a complete analysis, and a static graph is returned for display. The interactive canvas model, by contrast, requires the management of a long-lived, continuously evolving session state. The user's actions—panning, zooming, expanding nodes, filtering views, and observing live Git changes—all modify this state over time.This has a profound architectural ripple effect. The core_domain can no longer be stateless; it must be designed to hold and manage the canonical state of the entire analyzed codebase, including the user's current view configuration (e.g., node positions, zoom level, visible elements). Consequently, the CodeCanvasService input port must be expanded to include methods not just for data retrieval but for state manipulation and mutation. This, in turn, elevates the role of the driving adapters (tauri_ipc_adapter and the new web_api_adapter). They are no longer just simple command routers; they become the stateful guardians of this interactive session on the Rust backend, ensuring that the complex application state persists and remains consistent across numerous asynchronous requests from the frontend.3I.C. Evolved Component Mapping and Project StructureThe physical organization of the project, enforced through a Rust workspace with multiple crates, must be updated to reflect these new architectural components. A new crate, version_control_adapter, will be created to house the implementation of the IVersionControl port. To support the web application target, another new crate, web_api_adapter, will be created to house the web server implementation.8 This compartmentalization ensures that dependencies (e.g., git2-rs for version control, axum for the web server) are confined to the infrastructure layer and cannot leak into the pure core_domain, a guarantee enforced by the Rust compiler. The following table provides a revised, high-level overview of the system's structure and the expanded responsibilities of each component.Table I.C.1: Revised Hexagonal Architecture Component MappingHexagonal LayerComponent TypeTechnology Stack / Rust CrateKey Functionality (Revised)Application CoreDomain Logic & Portscore_domainDefines expanded entities (Symbol, ChangeSet), Value Objects, and all Port Traits.Driving AdapterDesktop UI InteractionSvelteKit UI / Tauri invoke calls / tauri_ipc_adapterTranslates granular user interactions into Core Port calls via Tauri IPC.Driving AdapterWeb UI InteractionSvelteKit UI / HTTP fetch / web_api_adapter (Axum)Exposes Core Port methods as a RESTful API for the web-based client.Driven AdaptersInfrastructure Implementationanalysis_adapter (Tree-Sitter), persistence_adapter, version_control_adapter (git2-rs)Implements Output Port traits for deep symbol analysis, I/O, and real-time Git status/diffing.II. The Application Core: The Interactive Domain Model (core_domain)The most critical evolution of the system design resides within the core_domain crate. The core domain model must be expanded significantly to represent code at a much finer granularity. The original file-level dependency graph, while still valuable, now serves as a foundational layer upon which a richer, more detailed symbol-level graph is constructed. This enhanced model is the prerequisite for delivering the deep code-tracing and interactive exploration features that define the Code Canvas experience.4II.A. Evolving the Canonical Graph: From Files to SymbolsThe original domain model, consisting of Graph, Node, and Edge entities, was sufficient for visualizing high-level file import relationships.3 However, it lacks the expressive power needed to trace a function call from its invocation to its definition, or to identify all usages of a specific variable across a project—core functionalities of the target application.4To address this, new domain entities must be introduced to model code at the symbol level. The existing Node entity will be refined to represent only code containers, such as files, modules, or classes. A new Symbol entity will be introduced to represent the granular code elements within those containers, such as functions, variables, parameters, and type definitions. This distinction is crucial for building a multi-layered mental model of the codebase. Relationships between these new Symbol entities will be captured by a SymbolEdge value object, representing interactions like function calls, variable references, or interface implementations. Furthermore, to support real-time change visualization, a ChangeSet entity is introduced to model the set of uncommitted changes within the workspace as reported by the version control system. The following table defines the new and modified data structures that form the heart of the interactive application.Table II.A.1: Extended Domain Entities for Interactive AnalysisEntity/Value ObjectDescriptionKey AttributesSource DataGraph (Entity)The collection of all analyzed nodes, edges, and symbols.id, project_name, nodes, edges, symbols.Aggregated from Analysis Adapter.Node (Entity)A code container (File, Module, Class).node_id, type, name, path, child_symbols (Vec<SymbolId>).Parsed AST.Edge (Value Object)A directed relationship between two Nodes (e.g., file import).source_id, target_id, type (Import).Derived from AST.Symbol (Entity)A specific code element (Function, Variable, Parameter, TypeDef).symbol_id, parent_node_id, type, name, location, metrics.Deep Tree-Sitter queries.SymbolEdge (Value Object)A directed relationship between two Symbols.source_id, target_id, type (Call, Reference, Implementation).Deep Tree-Sitter queries.ChangeSet (Entity)Represents the set of uncommitted changes in the workspace.id, modified_files, added_files, deleted_files.Version Control Adapter.II.B. Expanded Input Ports: Driving the Interactive CoreThe primary input port, defined by the CodeCanvasService trait, serves as the formal API through which all external driving adapters interact with the application's core logic. This port must be expanded with a suite of new methods to support the rich, interactive features demonstrated by the Code Canvas VS Code extension.4 These methods are no longer coarse-grained, batch operations; they are fine-grained queries and commands that enable the UI to feel responsive and intelligent. This expanded trait definition makes the system's capabilities concrete and serves as a clear contract between the frontend and the core domain.Table II.B.1: Expanded CodeCanvasService Input Port TraitMethodDescriptionReturn Typeanalyze_project(path: PathBuf)(Unchanged) Triggers full analysis pipeline.Result<GraphHandle, AppError>get_graph_data(...)(Unchanged) Retrieves a specific, potentially filtered, section of the master graph state.Result<Graph, AppError>get_symbol_references(symbol_id: SymbolId)Retrieves all symbols that reference or are referenced by the given symbol, enabling call hierarchy tracing.Result<Vec<SymbolEdge>, AppError>get_importers_of_file(node_id: NodeId)Retrieves all file nodes that import the given file node, enabling interactive exploration of dependencies.Result<Vec<Node>, AppError>get_local_changes()Retrieves the current set of uncommitted Git changes for visualization on the canvas.Result<ChangeSet, AppError>apply_layout_algorithm(nodes: Vec<NodeId>, algorithm: LayoutType)Requests the core to calculate new positions for a set of nodes, offloading heavy computation from the UI.Result<HashMap<NodeId, Position>, AppError>II.C. New and Expanded Output Ports: Core Driving Enhanced InfrastructureTo maintain architectural purity, the core domain's new dependency on a version control system must be strictly abstracted behind an output port. This prevents the core from being coupled to a specific implementation like Git. A new IVersionControl output port trait will be defined within the core_domain for this purpose. This is a critical architectural decision that enhances testability—allowing for the use of mock adapters in unit tests—and improves future flexibility.Concurrently, the existing ILanguageAnalysis port must be expanded to support the new requirement of symbol-level analysis. The original generate_graph_data method will be replaced with a more comprehensive generate_symbol_graph method. This new method will be responsible for returning the full, multi-layered data structure, including not only nodes and edges but also the newly defined symbols and symbol edges.Table II.C.1: New IVersionControl Output Port TraitMethodDescriptionReturn TypeDriven Adapterget_status()Fetches the current Git status of the workspace (modified, new, deleted files).Result<VersionControlStatus, AppError>version_control_adapterget_diff_for_file(path: PathBuf)Fetches the diff content for a specific uncommitted file.Result<String, AppError>version_control_adapterIII. Advanced Analysis and Infrastructure AdaptersThis section details the implementation strategies for the driven adapters, the components responsible for interacting with the external world—such as the file system, language parsers, and version control systems—on behalf of the application core. The complexity of these adapters increases significantly to support the system's enhanced analytical capabilities.III.A. The Multi-Faceted Analysis Adapter: Implementing Symbol-Level IntelligenceThe analysis_adapter crate, which implements the ILanguageAnalysis port, becomes one of the most complex components in the system. Its responsibility grows from parsing file-level imports to conducting deep, semantic analysis of the source code itself.To achieve this, the adapter will employ advanced Tree-Sitter queries.3 These queries will be far more intricate than those required for simple dependency mapping. They must be carefully crafted for each supported language to precisely identify function definitions, function calls, variable declarations, class instantiations, and references. This deep structural analysis is the technical foundation for features like "trace application flow" and "uncover relationships between functions".4 The adapter will be responsible for executing these queries against the ASTs produced by Tree-Sitter and translating the results into the canonical Symbol and SymbolEdge entities defined in the core_domain.A full, symbol-level analysis of a large codebase on every application start is computationally infeasible. Such a process could take several minutes for a non-trivial project, leading to an unacceptable user experience where the user is forced to wait before they can begin exploring their code. Therefore, a persistent caching mechanism is not merely an optimization; it is a core, non-negotiable requirement for the system.The implementation strategy must be built around an intelligent cache. On the first analysis of a project, the analysis_adapter will perform the full, computationally expensive parse and graph generation. The resulting complete graph, including all nodes, symbols, and edges, will then be serialized (likely using MessagePack for efficiency) and stored on disk within a dedicated cache directory (e.g., .codecanvas/cache) inside the user's project.On all subsequent launches of the application for that project, the startup sequence will be different. The adapter will first load the entire graph from the cached file. It will then invoke the IVersionControl adapter to get a list of all files that have been modified, added, or deleted since the cache was last written. With this information, it can perform a much faster incremental analysis, re-parsing only the changed files and any other files that directly depend on them. This incremental approach is the only viable strategy for achieving the near-instantaneous load times that users expect from a modern desktop application.III.B. The Version Control Adapter: A Port for Real-Time Git IntegrationThe new version_control_adapter crate is responsible for implementing the IVersionControl output port. Its purpose is to provide a clean, abstracted interface to the project's underlying version control system.The implementation will be built using the git2-rs Rust crate, which provides safe, idiomatic bindings to the highly performant libgit2 C library. This approach is vastly superior to shelling out to the git command-line executable, as it is more robust, more performant, and avoids parsing unstructured text output. The adapter's primary function is to act as a mediator, translating the low-level data structures and status flags provided by git2-rs into the clean, domain-specific ChangeSet and VersionControlStatus entities defined in the core_domain. This encapsulation is a textbook example of the Adapter pattern, ensuring the core logic remains completely decoupled from the specific details of Git repository management. This adapter will enable the application to query for uncommitted changes and display them in real-time on the canvas, a key feature for helping developers understand the impact of their current work.4IV. High-Throughput IPC for a Real-Time CanvasThe Inter-Process Communication (IPC) strategy must be revisited and adapted to support the more "chatty" and varied communication patterns required by a real-time, interactive canvas. The system will handle both massive, bulk data transfers and frequent, small, low-latency interactions, and the IPC layer must be optimized for both scenarios.IV.A. The Tauri IPC Adapter: Orchestrating Interactive CommandsThe tauri_ipc_adapter crate remains the concrete implementation of the CodeCanvasService input port for the desktop target. It exposes Rust functions to the frontend via the #[tauri::command] macro.3 Its role as a command dispatcher becomes more complex as it must now handle the wider and more granular set of commands defined in the expanded CodeCanvasService trait, such as get_symbol_references and apply_layout_algorithm.4 As the guardian of the application's long-lived state, this adapter will manage the primary AppState struct, likely wrapped in an Arc<Mutex<...>> to ensure thread-safe access across concurrent frontend requests.IV.B. Advanced Serialization for Hybrid Payloads (Bulk and Real-Time)The original mandate to use a binary serialization format for large data transfers remains valid and is, in fact, even more critical given the richer domain model. The default JSON serialization used by Tauri is unacceptably slow and verbose for transferring a full dependency graph that may contain tens of thousands of nodes, symbols, and edges. The use of MessagePack, via the rmp-serde crate, is mandatory for these bulk transfers. Empirical data shows that binary formats like MessagePack can be significantly faster and produce much smaller data payloads than JSON, which is a crucial optimization for ensuring a responsive UI during the initial load.3However, a single serialization format is no longer optimal for all communication patterns. The system now has two distinct types of IPC traffic:Bulk Data Transfer: The initial graph load, which is a massive, one-time transfer of data from the backend to the frontend.Interactive Commands: Frequent, small request/response cycles for actions like fetching symbol references or importers, where the payload is typically very small.These two patterns have different optimization priorities. For the bulk transfer, payload size and raw deserialization speed are the dominant factors, making MessagePack the clear choice. For the small, interactive commands, the latency is dominated by the round-trip time of the IPC call itself, not the serialization/deserialization of a few dozen bytes of data. Using MessagePack for these small payloads introduces significant frontend complexity (handling ArrayBuffers, managing a separate deserializer) for a negligible performance benefit. Furthermore, using JSON for these small payloads simplifies development and debugging, as the data is human-readable in network logs and console outputs.Therefore, the optimal solution is a hybrid serialization strategy. The tauri_ipc_adapter will selectively serialize its responses based on the command. For commands that return the large graph data (get_graph_data), the adapter will manually serialize the response to a Vec<u8> using rmp-serde and return the raw bytes. For all other commands that return small, structured data, the adapter will return the Rust struct directly, allowing Tauri to handle the serialization to JSON automatically. This nuanced approach optimizes for both performance and developer experience.IV.C. The Dual-Target Driving Adapter Strategy: Supporting Desktop and WebTo meet the requirement of deploying as both a desktop and web application, the system will implement two distinct driving adapters, both of which will implement the CodeCanvasService input port trait. This ensures that the core application logic remains identical and completely unaware of the context in which it is being run.5tauri_ipc_adapter (Desktop): This adapter, detailed above, will serve the desktop application. It integrates with the Tauri runtime, exposing the core service methods as #[tauri::command] functions that are invoked from the SvelteKit frontend via the Tauri IPC bridge.10web_api_adapter (Web): This new adapter will be created in its own crate to serve the web application. It will be implemented as a standalone web server using the Axum framework.8 Each method in the CodeCanvasService trait will be mapped to a corresponding HTTP endpoint (e.g., a RESTful API). For example, the get_symbol_references method will be exposed as a GET /api/symbols/{id}/references endpoint. This adapter will be responsible for handling HTTP requests, deserializing payloads, calling the appropriate core service method, and serializing the response back to JSON. The Axum server will also be configured to serve the static SvelteKit frontend assets, providing a single, self-contained binary for web deployment.11 This approach maximizes code reuse, as the entire core_domain and all driven adapters are shared between the two deployment targets.V. The Presentation Layer: An Interactive SvelteKit CanvasThe presentation layer, built with SvelteKit and TypeScript, is responsible for rendering the interactive canvas and translating all user interactions into the appropriate commands for the Rust core. It must be architected to handle large datasets efficiently and provide a fluid, non-blocking user experience across both desktop and web environments.V.A. The Visualization Adapter: From Static Graphs to an Interactive EnvironmentThe core of the user interface will be a dedicated Svelte component, the "Visualization Adapter." This component's primary responsibility is to manage the lifecycle of the Cytoscape.js instance, which is mandated for its high-performance WebGL rendering engine, essential for handling complex networks with thousands of elements.3This adapter will orchestrate all communication with the backend. On initialization, it will invoke the analyze_project and get_graph_data commands. It will be responsible for handling the hybrid IPC response: if it receives a binary payload (MessagePack), it will use a TypeScript-compatible library like @msgpack/msgpack to deserialize the data before feeding it into Cytoscape.js.Crucially, the adapter will attach a comprehensive set of event listeners to the Cytoscape.js canvas. A click on a node representing a Symbol, for instance, will not be handled locally. Instead, it will trigger a call to the backend via the appropriate communication channel (Tauri IPC or HTTP fetch).4 When the backend returns the list of related symbols and edges, the adapter will then dynamically update the Cytoscape.js graph, perhaps by highlighting the related elements or adding new ones to the canvas. This "closed-loop" interaction model ensures that the Rust core remains the single source of truth for all code analysis data, a principle central to the Hexagonal Architecture.3V.A.1. Conditional Backend CommunicationThe SvelteKit frontend must be environment-aware to communicate with the correct backend. This will be achieved through a dedicated API client module that uses conditional logic based on environment variables.In the Tauri Environment: The frontend can detect it's running within Tauri (e.g., by checking for the window.__TAURI__ object). In this mode, the API client will use the invoke function from the @tauri-apps/api package to call commands on the tauri_ipc_adapter.10In the Web Environment: When not running in Tauri, the application is a standard web app. The API client will use the browser's native fetch API to make HTTP requests to the REST endpoints exposed by the web_api_adapter (e.g., fetch('/api/graph_data')).7This conditional logic will be abstracted away from the UI components, which will simply call functions like api.getSymbolReferences(id) without needing to know the underlying communication mechanism. The SvelteKit build process will be configured with @sveltejs/adapter-static to produce a single-page application (SPA) build, which can be either bundled into the Tauri binary or served by the Axum web server.12V.B. Canvas Interaction and Layout ManagementThe UI will provide the standard set of infinite canvas controls, including panning (e.g., by holding the spacebar and dragging), zooming (e.g., with the scroll wheel), and a navigational minimap.4 These features will be implemented primarily using the native capabilities of Cytoscape.js.A key interactive feature is dynamic layout management. The UI will feature a toolbar or respond to keyboard shortcuts (Shift + 1, 2, 3...) that allow the user to apply various graph layout algorithms (e.g., hierarchical, force-directed) to the currently visible nodes.4 Executing these complex layout calculations in the browser's single JavaScript thread could easily freeze the UI, especially for large graphs. To prevent this, the user's action will trigger a backend call to the apply_layout_algorithm command. The computationally expensive process of calculating new positions for hundreds or thousands of nodes will be offloaded to the highly performant, multi-threaded Rust backend. The backend will return a simple map of node IDs to their new (x, y) coordinates, which the frontend can then apply to the Cytoscape.js elements in a smooth, animated transition.V.C. Feature Mapping and Core Interaction (Revised)The following table provides a clear, end-to-end mapping of user-facing features to their corresponding interactions with the Rust core. This serves as a definitive guide for both frontend and backend development, ensuring that the API surface exposed by the core directly supports the required user experience.Table V.C.1: Revised Feature Mapping and Core InteractionVisualization FeatureUser ActionRust Core Interaction (Port Method)Data Source / RationaleToken Reference HighlightingClick on a function name within a file node.CodeCanvasService::get_symbol_referencesHighlights the full call hierarchy, fulfilling a core interactive feature of the Code Canvas extension.4Real-time Git Diff VisualizationEdit and save a file in the IDE.CodeCanvasService::get_local_changes (triggered by file system events).Overlays change indicators (e.g., color highlights) on file nodes, providing immediate visual feedback on uncommitted work.4Interactive Node ExpansionClick an "expand imports" icon on a file node.CodeCanvasService::get_importers_of_file or get_imported_files.Dynamically adds related file nodes to the canvas, allowing the user to explore the codebase's structure organically.4Dynamic Layout ApplicationPress Shift+2 with nodes selected.CodeCanvasService::apply_layout_algorithmOrganizes the canvas using backend-computed layouts, ensuring UI responsiveness by offloading heavy computation.4VI. Synthesis and Strategic Implementation RoadmapThis revised design document outlines a sophisticated, high-performance, and architecturally sound platform for interactive code visualization. The evolution from a static graph viewer to a dynamic canvas introduces significant complexity, but the strict adherence to the Hexagonal Architecture provides a robust framework for managing that complexity. The success of the project hinges on the correct implementation of several key architectural decisions and a strategic, phased approach to development.VI.A. Core Architectural Decisions and Their ImplicationsThe following five decisions are the most critical pillars of this revised architecture, each with significant implications for the development process:Adopting a Symbol-Level Domain Model: This is the most profound change from the original plan. It enables the deep, semantic code analysis required for an advanced tool, but at the cost of a substantial increase in the complexity of the analysis_adapter. This adapter becomes a project in itself, requiring deep expertise in language parsing and AST manipulation.Introducing a Version Control Port/Adapter: This decision ensures the core domain remains decoupled from the infrastructure of Git. It is an architecturally pure solution that enhances testability and maintainability, allowing the version control logic to be developed and tested in isolation.Mandating an Analysis Cache: This is a non-negotiable performance requirement. Acknowledging that a full, initial analysis will be slow, the architecture mandates a persistent caching and incremental update strategy. This is essential for providing the fast, responsive experience users expect from a desktop application when working with large codebases.Specifying a Hybrid IPC Strategy: This decision reflects a nuanced understanding of performance optimization. It avoids the dogmatic application of a single solution, instead choosing the right tool for the right job: high-efficiency binary serialization for bulk data and simple, debuggable JSON for low-latency interactive commands.Adopting a Multi-Adapter Strategy for Dual Deployment: This decision formalizes the requirement to support both desktop and web platforms by implementing two distinct driving adapters (Tauri IPC and Axum Web API) against a single, shared application core. This leverages the primary strength of the Hexagonal Architecture to maximize code reuse and maintain a clean separation of concerns between the core logic and its delivery mechanisms.5VI.B. Nuanced Conclusions and Actionable RecommendationsThe complexity of the system has increased by an order of magnitude. The development team must recognize that the analysis_adapter and its associated caching layer are the most technically challenging and highest-risk components of the entire system. A significant portion of the project's resources and testing efforts must be allocated here to ensure correctness, performance, and scalability across multiple programming languages.To manage this complexity and mitigate risk, a phased implementation roadmap is strongly recommended. This approach allows the team to build upon a stable, working foundation at each stage, delivering value incrementally and validating architectural assumptions along the way.Proposed Phased Implementation Roadmap:Phase 1 (Core Scaffolding and File-Level Graph): The initial focus should be on establishing the complete, end-to-end architectural skeleton. This involves implementing the expanded domain model and port definitions in the core_domain, building the necessary adapters (persistence_adapter, a basic analysis_adapter), and creating the SvelteKit frontend with Cytoscape.js. Crucially, this phase includes scaffolding both the tauri_ipc_adapter and the web_api_adapter to ensure the dual-target architecture is established from the outset. The goal of this phase is to replicate the functionality of the original plan—successfully analyzing a project and rendering a static, file-level dependency graph—on both the desktop and web targets.Phase 2 (Deep Analysis and Interactivity): With the foundation in place, the team can now focus on the most complex feature set. This phase involves heavily investing in the analysis_adapter to implement the deep, symbol-level analysis using advanced Tree-Sitter queries. Concurrently, the frontend team will implement the interactive features that consume this new data, such as clicking on a symbol to highlight its references and call hierarchy, as described in the Code Canvas feature set.4Phase 3 (Real-Time Integration and Refinement): The final phase integrates the real-time aspects of the application. The version_control_adapter will be implemented using git2-rs. The system will be enhanced to listen for file system changes, trigger calls to the get_local_changes command, and visualize these changes on the canvas. This phase also includes implementing the dynamic layout algorithms and other quality-of-life features like the minimap and advanced filtering controls. This phased approach de-risks the project by tackling the most fundamental architectural challenges first and building progressively more sophisticated features upon a proven, stable core.VI.C. Technology Stack Versioning MandateTo ensure the highest levels of performance, security, and long-term maintainability, this project mandates the use of the latest stable versions of all core frameworks and libraries. Building on a modern foundation is not merely a preference but a strategic imperative to leverage critical optimizations, access new features, and avoid the technical debt associated with outdated dependencies.This mandate specifically includes, but is not limited to:Svelte 5: The adoption of Svelte 5 is critical for leveraging its new, more granular reactivity model ("runes"), which is expected to simplify state management and improve rendering performance in the complex, data-intensive visualization adapter.Tauri 2.8.5 (or latest): The project must stay current with the Tauri framework to benefit from ongoing improvements in binary size, IPC performance, and access to new operating system-level APIs that can enhance the application's capabilities.Latest Rust Crates: All Rust dependencies, particularly performance-critical ones like git2-rs, tree-sitter, and rmp-serde, must be kept at their latest stable versions to incorporate performance enhancements and security patches.Latest Node.js Dependencies: Frontend dependencies, especially Cytoscape.js and its associated layout algorithm libraries, must be kept up-to-date to ensure the application benefits from the latest rendering optimizations and bug fixes.A policy of continuous dependency review and updating must be integrated into the development lifecycle. This proactive approach ensures the application remains performant, secure, and aligned with the rapid evolution of the web and systems development ecosystems.VI.D. Runtime Environment Mandate: DenoFor the SvelteKit frontend development environment and the web application deployment target, the project mandates the use of the Deno runtime. This decision is driven by Deno's focus on security, developer experience, and its modern, Rust-based architecture, which aligns strategically with the project's backend technology.Key justifications for this mandate include:Integrated Toolchain: Deno provides a comprehensive suite of built-in tools, including a linter (deno lint), formatter (deno fmt), and task runner (deno task).13 This eliminates the need for configuring and managing a disparate collection of third-party dependencies (like ESLint, Prettier) common in the Node.js ecosystem, leading to a simpler, more consistent development experience.Security by Default: Unlike Node.js, Deno executes code in a secure sandbox by default. Access to the file system, network, and environment variables must be explicitly granted via permissions flags.13 This security-first posture is a critical advantage, reducing the risk of supply chain attacks and unintended side effects, both during local development and in production.Native TypeScript and Modern Module Support: Deno supports TypeScript out of the box without requiring additional build steps or configuration, which integrates seamlessly with SvelteKit's TypeScript-first approach.13 Its reliance on standard ES modules and ability to import directly from URLs simplifies dependency management.15Simplified Deployment: The SvelteKit ecosystem provides a dedicated Deno adapter that optimizes the application build for Deno-compatible platforms, such as Deno Deploy.16 This streamlines the process of deploying the web version of the application to modern edge hosting environments.Strategic Alignment with Rust: Deno is built with Rust and utilizes the V8 JavaScript engine via high-quality Rust bindings (rusty_v8).19 This creates a more cohesive technology ecosystem for the project. Leveraging a Rust-based runtime for the frontend toolchain provides philosophical alignment and potential for deeper integration with the Rust-based backend in the future.Adopting Deno standardizes the frontend toolchain, enhances the security posture of the web deployment, and aligns the entire project around a modern, performant, and security-conscious technology stack.VI.D.1. SvelteKit and Tauri Configuration for DenoTo ensure seamless integration between SvelteKit, Tauri, and the Deno runtime for the desktop application, a specific configuration is required. The primary goal is to produce a Single-Page Application (SPA) that Tauri can bundle.Install Static Adapter: The first step is to add the SvelteKit static adapter, which is necessary for generating a client-side, static build that Tauri can serve. This is done using the Deno package manager:Bashdeno add -D npm:@sveltejs/adapter-static
Update Tauri Configuration: The tauri.conf.json file must be updated to use Deno's task runner for development and build commands. The frontendDist path must also point to the output directory generated by the static adapter.
```json
{
  "build": {
    "beforeDevCommand": "deno task dev",
    "beforeBuildCommand": "deno task build",
    "devUrl": "http://localhost:5173",
    "frontendDist": "../build"
  }
}
```
Update SvelteKit Configuration: The svelte.config.js file must be configured to use the newly installed @sveltejs/adapter-static. A fallback page is specified to ensure all routes are directed to index.html, which is standard for SPA routing.JavaScript

```ts


import adapter from '@sveltejs/adapter-static';

import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),
  kit: {
    adapter: adapter({
      fallback: 'index.html',
    }),
  },
};

export default config;
```

Disable Server-Side Rendering (SSR): Finally, SSR must be explicitly disabled for the entire application. This is critical because Tauri APIs, which rely on the global window object, are only available in the browser context (the WebView) and will not be accessible during a server-side build process. Disabling SSR ensures that all code runs exclusively on the client side. This is achieved by creating a root layout file at src/routes/+layout.ts with the following content:TypeScriptexport const ssr = false;
This configuration ensures that the SvelteKit application is correctly bundled as a pure SPA, allowing it to function correctly within the Tauri desktop environment while being developed and managed with the Deno toolchain.


# In-Source Testing Protocol for TypeScript/Svelte Components

Add this section to **Part II: Language-Specific Protocols** after **§10. Example Ideal TypeScript File Set**.

---

## **11. In-Source Testing for TypeScript/Svelte Components**

### **A. CRITICAL: Co-Located Testing Mandate**

For Svelte 5 components and TypeScript modules, in-source testing is the REQUIRED pattern. Tests MUST reside in the same file as the code they validate, using Vitest's in-source testing feature.

**Benefits:**
- **Co-location:** Tests and implementation are maintained together, eliminating file switching overhead.
- **Access to Internals:** Tests can validate private functions and state without requiring artificial exports.
- **Encourages Testing:** Having tests immediately visible when editing code increases test coverage.
- **Tree-Shakeable:** The `svelteTesting()` Vite plugin and conditional compilation ensure test code is excluded from production builds via dead code elimination.

### **B. Required Configuration**

#### **1. Dependencies**

The following devDependencies MUST be installed:

```json
{
  "devDependencies": {
    "@testing-library/svelte": "^5.2.3",
    "@testing-library/user-event": "^14.5.2",
    "@testing-library/jest-dom": "^6.6.3",
    "@vitest/ui": "^2.1.8",
    "jsdom": "^25.0.1",
    "vitest": "^2.1.8"
  }
}
```

#### **2. Vite Configuration**

The `vite.config.ts` MUST include the following configuration:

```typescript
/// <reference types="vitest" />
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';
import { svelteTesting } from '@testing-library/svelte/vite';

export default defineConfig(({ mode }) => ({
  plugins: [sveltekit(), svelteTesting()],
  test: {
    includeSource: ['src/**/*.{js,ts,svelte}'],
    environment: 'jsdom',
    globals: true
  },
  define: mode === 'production' ? {
    'import.meta.vitest': 'undefined'
  } : {}
}));
```

**Key Configuration Elements:**
- `svelteTesting()` plugin: Required for proper test code elimination in production builds.
- `includeSource`: Specifies which files can contain in-source tests.
- `environment: 'jsdom'`: Provides browser-like environment for component testing.
- `define`: Replaces `import.meta.vitest` with `undefined` in production, enabling tree-shaking.

#### **3. Test Scripts**

The `package.json` MUST include these scripts:

```json
{
  "scripts": {
    "test": "vitest",
    "test:ui": "vitest --ui"
  }
}
```

### **C. In-Source Test Structure for Svelte Components**

#### **1. The Module Context Pattern**

Tests MUST be placed in a `<script context="module">` block at the END of the Svelte component file, after all component logic and markup.

**Complete Example Structure:**

```svelte
<script lang="ts">
  /**
   * Component documentation and revision history here.
   */
  
  // Component logic
  function formatPath(path: string): string {
    const parts = path.split('/');
    if (parts.length <= 2) return path;
    return `.../${parts.slice(-2).join('/')}`;
  }
  
  export let isVisible: boolean = true;
</script>

<!-- Component markup -->
<div>
  {#if isVisible}
    <p>{formatPath(somePath)}</p>
  {/if}
</div>

<style>
  /* Component styles */
</style>

<!-- In-Source Tests -->
<script context="module">
  import { describe, it, expect } from 'vitest';
  import { render, screen } from '@testing-library/svelte';
  import '@testing-library/jest-dom/vitest';
  import MyComponent from './MyComponent.svelte';

  if (import.meta.vitest) {
    describe('MyComponent.svelte', () => {
      describe('formatPath utility', () => {
        /**
         * Test: Validates that short paths are returned unchanged.
         * Justification: Edge case handling for minimal path structures.
         */
        it('should return short paths unchanged', () => {
          const formatPath = (path: string): string => {
            const parts = path.split('/');
            if (parts.length <= 2) return path;
            return `.../${parts.slice(-2).join('/')}`;
          };

          expect(formatPath('file.ts')).toBe('file.ts');
          expect(formatPath('src/file.ts')).toBe('src/file.ts');
        });

        /**
         * Test: Validates that long paths are shortened to show only last two segments.
         * Justification: Primary use case for displaying deeply nested file paths.
         */
        it('should shorten long paths correctly', () => {
          const formatPath = (path: string): string => {
            const parts = path.split('/');
            if (parts.length <= 2) return path;
            return `.../${parts.slice(-2).join('/')}`;
          };

          expect(formatPath('src/lib/components/MyComponent.svelte'))
            .toBe('.../components/MyComponent.svelte');
        });
      });

      describe('component rendering', () => {
        /**
         * Test: Validates that the component renders when visible.
         * Justification: Ensures conditional rendering logic functions correctly.
         */
        it('should render when isVisible is true', () => {
          render(MyComponent, { props: { isVisible: true } });
          expect(screen.getByText('Expected Content')).toBeInTheDocument();
        });

        /**
         * Test: Validates that the component does not render when hidden.
         * Justification: Confirms the inverse of the visibility condition.
         */
        it('should not render when isVisible is false', () => {
          render(MyComponent, { props: { isVisible: false } });
          expect(screen.queryByText('Expected Content')).not.toBeInTheDocument();
        });
      });
    });
  }
</script>
```

#### **2. Non-Negotiable Requirements**

1. **MUST use `<script context="module">`**: This ensures the test code runs in module scope, separate from component instance scope.
2. **MUST wrap all tests in `if (import.meta.vitest)`**: This conditional guard ensures tests only execute during test runs and are eliminated from production builds.
3. **MUST import the component itself**: Import the component from its own file path (e.g., `import MyComponent from './MyComponent.svelte'`) for rendering tests.
4. **MUST import jest-dom matchers**: Include `import '@testing-library/jest-dom/vitest'` to enable matchers like `toBeInTheDocument()`.
5. **MUST document each test**: Every test MUST include a JSDoc comment explaining what it validates and why it's important.

### **D. Test Categories and Patterns**

#### **1. Utility Function Tests**

For private utility functions within components, recreate the function logic in the test block to validate its behavior:

```typescript
describe('utility functions', () => {
  /**
   * Test: Validates the calculation logic for edge case inputs.
   * Justification: Ensures mathematical correctness for boundary conditions.
   */
  it('should handle edge case correctly', () => {
    // Recreate the function logic to test it
    const calculateValue = (input: number): number => {
      return input * 2 + 1;
    };

    expect(calculateValue(0)).toBe(1);
    expect(calculateValue(-5)).toBe(-9);
  });
});
```

**Rationale:** This approach tests the logic without requiring the function to be exported, maintaining encapsulation.

#### **2. Component Rendering Tests**

Keep rendering tests simple and focused. Test the component's primary rendering behaviors and conditional logic:

```typescript
describe('component rendering', () => {
  /**
   * Test: Validates that the primary content renders with valid props.
   * Justification: Core functionality test ensuring the component displays correctly.
   */
  it('should render primary content', () => {
    render(MyComponent, { 
      props: { 
        title: 'Test Title',
        isVisible: true 
      } 
    });
    expect(screen.getByText('Test Title')).toBeInTheDocument();
  });

  /**
   * Test: Validates that optional content is hidden when disabled.
   * Justification: Confirms conditional rendering based on props.
   */
  it('should hide optional content when disabled', () => {
    render(MyComponent, { 
      props: { 
        showOptional: false 
      } 
    });
    expect(screen.queryByTestId('optional-section')).not.toBeInTheDocument();
  });
});
```

**Best Practices:**
- Use `screen.getByText()` for assertions that expect the element to exist.
- Use `screen.queryByText()` for assertions that expect the element NOT to exist (returns `null` instead of throwing).
- Keep component tests focused on behavior, not implementation details.

#### **3. Test Documentation Standard**

Every test MUST include a JSDoc comment with two elements:

```typescript
/**
 * Test: [One sentence describing what this test validates]
 * Justification: [One sentence explaining why this test is important or what failure mode it prevents]
 */
it('should do something specific', () => {
  // test implementation
});
```

**Example:**
```typescript
/**
 * Test: Validates that the error message appears when validation fails.
 * Justification: Ensures users receive immediate feedback on invalid input.
 */
it('should display error message on validation failure', () => {
  // test code
});
```

### **E. Running Tests**

#### **Command Reference**

```bash
# Run all tests once (CI mode)
npm test

# Run tests in watch mode (development)
npm test -- --watch

# Run tests with interactive UI
npm run test:ui

# Run tests for a specific component
npm test -- ComponentName

# Run tests with coverage
npm test -- --coverage
```

#### **Expected Output Format**

```
✓ src/lib/components/MyComponent.svelte (4 tests) 126ms
  ✓ MyComponent.svelte > utility functions (2 tests)
    ✓ should handle edge case correctly
    ✓ should process valid input
  ✓ MyComponent.svelte > component rendering (2 tests)
    ✓ should render primary content
    ✓ should hide optional content when disabled

Test Files  1 passed (1)
     Tests  4 passed (4)
```

### **F. Production Build Verification**

The in-source testing setup MUST NOT include test code in production builds. Verify this by:

1. Running a production build: `npm run build`
2. Inspecting the bundle size: Test code should not increase bundle size
3. Checking the build output: No test-related imports should appear in bundled files

The `svelteTesting()` plugin automatically handles test code elimination through Vite's dead code elimination when `import.meta.vitest` is replaced with `undefined` in production mode.

### **G. Troubleshooting**

#### **Issue: Tests Not Found**

**Cause:** `includeSource` configuration is incorrect or missing.

**Solution:** Ensure `vite.config.ts` includes:
```typescript
test: {
  includeSource: ['src/**/*.{js,ts,svelte}']
}
```

#### **Issue: "Invalid Chai property: toBeInTheDocument"**

**Cause:** jest-dom matchers are not imported.

**Solution:** Add to the test module script:
```typescript
import '@testing-library/jest-dom/vitest';
```

#### **Issue: Component Import Fails**

**Cause:** Incorrect import path in the test block.

**Solution:** Always import the component from the same file using a relative path:
```typescript
import MyComponent from './MyComponent.svelte';
```

### **H. Integration with Revision History**

When adding or modifying in-source tests, the component's Revision History MUST be updated:

```typescript
/**
 * Component description.
 *
 * Revision History
 * - 2025-10-10T22:20:00Z @AI: Add in-source tests for utility functions and rendering.
 * - 2025-10-10T15:00:00Z @AI: Initial component implementation.
 */
```

### **I. Reference Implementation**

See `frontend/src/lib/components/GitStatusPanel.svelte` for a complete, working example of in-source testing that demonstrates:
- Utility function tests (path formatting)
- Component rendering tests (visibility conditions)
- Proper test documentation
- Correct import statements and conditional compilation

---

**End of In-Source Testing Protocol**
