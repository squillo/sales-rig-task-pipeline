//! DependencyGraph domain service for analyzing task dependencies.
//!
//! This service builds a directed graph of task dependencies and provides cycle
//! detection and topological sorting. Cycle detection ensures tasks don't have
//! circular dependencies that would prevent execution. Topological sorting
//! provides a valid execution order for tasks with dependencies.
//!
//! Revision History
//! - 2025-11-23T15:50:00Z @AI: Create DependencyGraph for Phase 2 Sprint 5 Task 2.4.

/// Directed graph of task dependencies with cycle detection and topological sorting.
///
/// DependencyGraph builds a dependency graph from a set of tasks and provides
/// methods to detect circular dependencies and compute valid execution orders.
/// The graph maps task IDs to their list of dependency task IDs.
///
/// # Algorithms
///
/// - **Cycle Detection**: Uses Depth-First Search (DFS) with a recursion stack
///   to detect back edges, which indicate cycles.
/// - **Topological Sort**: Uses DFS-based topological sorting (Kahn's algorithm variant)
///   to produce a valid execution order. Fails if cycles are present.
///
/// # Examples
///
/// ```
/// # use task_manager::domain::services::dependency_graph::DependencyGraph;
/// # use task_manager::domain::task::Task;
/// # use transcript_extractor::domain::action_item::ActionItem;
/// // Create tasks with dependencies
/// let action1 = ActionItem {
///     title: std::string::String::from("Task A"),
///     assignee: None,
///     due_date: None,
/// };
/// let mut task_a = Task::from_action_item(&action1, None);
/// task_a.id = std::string::String::from("A");
/// task_a.dependencies = std::vec![std::string::String::from("B")];
///
/// let action2 = ActionItem {
///     title: std::string::String::from("Task B"),
///     assignee: None,
///     due_date: None,
/// };
/// let mut task_b = Task::from_action_item(&action2, None);
/// task_b.id = std::string::String::from("B");
/// task_b.dependencies = std::vec![];
///
/// let tasks = std::vec![task_a, task_b];
/// let graph = DependencyGraph::new(&tasks);
///
/// // Check for cycles
/// let cycles = graph.detect_cycles();
/// assert!(cycles.is_empty(), "No cycles should be detected");
///
/// // Get execution order
/// let order = graph.topological_sort().unwrap();
/// assert_eq!(order, std::vec![std::string::String::from("B"), std::string::String::from("A")]);
/// ```
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    /// Adjacency list: maps task_id -> list of tasks it depends on
    adjacency: std::collections::HashMap<String, std::vec::Vec<String>>,
}

impl DependencyGraph {
    /// Constructs a new DependencyGraph from a list of tasks.
    ///
    /// Builds the adjacency list from each task's dependencies field.
    ///
    /// # Arguments
    ///
    /// * `tasks` - Slice of tasks to build the graph from
    ///
    /// # Examples
    ///
    /// ```
    /// # use task_manager::domain::services::dependency_graph::DependencyGraph;
    /// # use task_manager::domain::task::Task;
    /// # use transcript_extractor::domain::action_item::ActionItem;
    /// let action = ActionItem {
    ///     title: std::string::String::from("Task 1"),
    ///     assignee: None,
    ///     due_date: None,
    /// };
    /// let task = Task::from_action_item(&action, None);
    /// let tasks = std::vec![task];
    ///
    /// let graph = DependencyGraph::new(&tasks);
    /// ```
    pub fn new(tasks: &[crate::domain::task::Task]) -> Self {
        let mut adjacency = std::collections::HashMap::new();

        for task in tasks {
            adjacency.insert(task.id.clone(), task.dependencies.clone());
        }

        DependencyGraph { adjacency }
    }

    /// Detects cycles in the dependency graph using DFS.
    ///
    /// Returns a list of cycles, where each cycle is represented as a vector
    /// of task IDs forming the cycle. Returns empty vector if no cycles exist.
    ///
    /// # Algorithm
    ///
    /// Uses Depth-First Search with three states per node:
    /// - Unvisited (white)
    /// - In recursion stack (gray) - currently being processed
    /// - Fully processed (black)
    ///
    /// A back edge (from gray to gray) indicates a cycle.
    ///
    /// # Returns
    ///
    /// Vector of cycles, where each cycle is a vector of task IDs.
    ///
    /// # Examples
    ///
    /// ```
    /// # use task_manager::domain::services::dependency_graph::DependencyGraph;
    /// # use task_manager::domain::task::Task;
    /// # use transcript_extractor::domain::action_item::ActionItem;
    /// // Create a cycle: A -> B -> C -> A
    /// let action_a = ActionItem {
    ///     title: std::string::String::from("Task A"),
    ///     assignee: None,
    ///     due_date: None,
    /// };
    /// let mut task_a = Task::from_action_item(&action_a, None);
    /// task_a.id = std::string::String::from("A");
    /// task_a.dependencies = std::vec![std::string::String::from("B")];
    ///
    /// let action_b = ActionItem {
    ///     title: std::string::String::from("Task B"),
    ///     assignee: None,
    ///     due_date: None,
    /// };
    /// let mut task_b = Task::from_action_item(&action_b, None);
    /// task_b.id = std::string::String::from("B");
    /// task_b.dependencies = std::vec![std::string::String::from("C")];
    ///
    /// let action_c = ActionItem {
    ///     title: std::string::String::from("Task C"),
    ///     assignee: None,
    ///     due_date: None,
    /// };
    /// let mut task_c = Task::from_action_item(&action_c, None);
    /// task_c.id = std::string::String::from("C");
    /// task_c.dependencies = std::vec![std::string::String::from("A")];
    ///
    /// let tasks = std::vec![task_a, task_b, task_c];
    /// let graph = DependencyGraph::new(&tasks);
    ///
    /// let cycles = graph.detect_cycles();
    /// assert_eq!(cycles.len(), 1, "Should detect one cycle");
    /// assert!(cycles[0].contains(&std::string::String::from("A")));
    /// assert!(cycles[0].contains(&std::string::String::from("B")));
    /// assert!(cycles[0].contains(&std::string::String::from("C")));
    /// ```
    pub fn detect_cycles(&self) -> std::vec::Vec<std::vec::Vec<String>> {
        let mut visited = std::collections::HashSet::new();
        let mut rec_stack = std::collections::HashSet::new();
        let mut path = std::vec::Vec::new();
        let mut cycles = std::vec::Vec::new();

        for node in self.adjacency.keys() {
            if !visited.contains(node) {
                self.dfs_detect_cycle(node, &mut visited, &mut rec_stack, &mut path, &mut cycles);
            }
        }

        cycles
    }

    /// Helper function for DFS-based cycle detection.
    fn dfs_detect_cycle(
        &self,
        node: &String,
        visited: &mut std::collections::HashSet<String>,
        rec_stack: &mut std::collections::HashSet<String>,
        path: &mut std::vec::Vec<String>,
        cycles: &mut std::vec::Vec<std::vec::Vec<String>>,
    ) {
        visited.insert(node.clone());
        rec_stack.insert(node.clone());
        path.push(node.clone());

        if let std::option::Option::Some(neighbors) = self.adjacency.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    self.dfs_detect_cycle(neighbor, visited, rec_stack, path, cycles);
                } else if rec_stack.contains(neighbor) {
                    // Found a cycle - extract cycle from path
                    let cycle_start_index = path.iter().position(|n| n == neighbor).unwrap_or(0);
                    let cycle = path[cycle_start_index..].to_vec();
                    cycles.push(cycle);
                }
            }
        }

        path.pop();
        rec_stack.remove(node);
    }

    /// Computes a topological sort of the dependency graph.
    ///
    /// Returns a vector of task IDs in an order where all dependencies
    /// appear before the tasks that depend on them. This provides a valid
    /// execution order.
    ///
    /// # Algorithm
    ///
    /// Uses DFS-based topological sorting:
    /// 1. Perform DFS from each unvisited node
    /// 2. Add nodes to result in post-order (after visiting all dependencies)
    /// 3. Reverse the result to get topological order
    ///
    /// # Returns
    ///
    /// - `Ok(Vec<String>)` - Valid execution order if no cycles exist
    /// - `Err(String)` - Error message if cycles are detected
    ///
    /// # Examples
    ///
    /// ```
    /// # use task_manager::domain::services::dependency_graph::DependencyGraph;
    /// # use task_manager::domain::task::Task;
    /// # use transcript_extractor::domain::action_item::ActionItem;
    /// // Create linear dependency: A depends on B, B depends on C
    /// let action_a = ActionItem {
    ///     title: std::string::String::from("Task A"),
    ///     assignee: None,
    ///     due_date: None,
    /// };
    /// let mut task_a = Task::from_action_item(&action_a, None);
    /// task_a.id = std::string::String::from("A");
    /// task_a.dependencies = std::vec![std::string::String::from("B")];
    ///
    /// let action_b = ActionItem {
    ///     title: std::string::String::from("Task B"),
    ///     assignee: None,
    ///     due_date: None,
    /// };
    /// let mut task_b = Task::from_action_item(&action_b, None);
    /// task_b.id = std::string::String::from("B");
    /// task_b.dependencies = std::vec![std::string::String::from("C")];
    ///
    /// let action_c = ActionItem {
    ///     title: std::string::String::from("Task C"),
    ///     assignee: None,
    ///     due_date: None,
    /// };
    /// let mut task_c = Task::from_action_item(&action_c, None);
    /// task_c.id = std::string::String::from("C");
    /// task_c.dependencies = std::vec![];
    ///
    /// let tasks = std::vec![task_a, task_b, task_c];
    /// let graph = DependencyGraph::new(&tasks);
    ///
    /// let order = graph.topological_sort().unwrap();
    /// // C must come before B, B must come before A
    /// let c_pos = order.iter().position(|id| id == "C").unwrap();
    /// let b_pos = order.iter().position(|id| id == "B").unwrap();
    /// let a_pos = order.iter().position(|id| id == "A").unwrap();
    /// assert!(c_pos < b_pos);
    /// assert!(b_pos < a_pos);
    /// ```
    pub fn topological_sort(&self) -> std::result::Result<std::vec::Vec<String>, String> {
        // Check for cycles first
        let cycles = self.detect_cycles();
        if !cycles.is_empty() {
            return std::result::Result::Err(std::format!(
                "Cannot perform topological sort: graph contains {} cycle(s)",
                cycles.len()
            ));
        }

        // Build reverse adjacency list for topological sort
        let reverse_adj = self.build_reverse_adjacency();

        let mut visited = std::collections::HashSet::new();
        let mut result = std::vec::Vec::new();

        for node in self.adjacency.keys() {
            if !visited.contains(node) {
                self.dfs_topological(node, &mut visited, &mut result, &reverse_adj);
            }
        }

        result.reverse();
        std::result::Result::Ok(result)
    }

    /// Helper function for DFS-based topological sort.
    fn dfs_topological(
        &self,
        node: &String,
        visited: &mut std::collections::HashSet<String>,
        result: &mut std::vec::Vec<String>,
        reverse_adj: &std::collections::HashMap<String, std::vec::Vec<String>>,
    ) {
        visited.insert(node.clone());

        if let std::option::Option::Some(dependents) = reverse_adj.get(node) {
            for dependent in dependents {
                if !visited.contains(dependent) {
                    self.dfs_topological(dependent, visited, result, reverse_adj);
                }
            }
        }

        result.push(node.clone());
    }

    /// Builds a reverse adjacency list (task -> tasks that depend on it).
    fn build_reverse_adjacency(&self) -> std::collections::HashMap<String, std::vec::Vec<String>> {
        let mut reverse_adj: std::collections::HashMap<String, std::vec::Vec<String>> = std::collections::HashMap::new();

        // Initialize all nodes in reverse adjacency
        for node in self.adjacency.keys() {
            reverse_adj.entry(node.clone()).or_insert_with(std::vec::Vec::new);
        }

        // Build reverse edges: if A depends on B, add edge B -> A
        for (node, dependencies) in &self.adjacency {
            for dep in dependencies {
                reverse_adj.entry(dep.clone())
                    .or_insert_with(std::vec::Vec::new)
                    .push(node.clone());
            }
        }

        reverse_adj
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_no_dependencies() {
        // Test: Validates graph with no dependencies (disconnected nodes).
        // Justification: Ensures basic graph construction works for independent tasks.
        let action_a = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Task A"),
            assignee: std::option::Option::None,
            due_date: std::option::Option::None,
        };
        let mut task_a = crate::domain::task::Task::from_action_item(&action_a, std::option::Option::None);
        task_a.id = std::string::String::from("A");

        let action_b = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Task B"),
            assignee: std::option::Option::None,
            due_date: std::option::Option::None,
        };
        let mut task_b = crate::domain::task::Task::from_action_item(&action_b, std::option::Option::None);
        task_b.id = std::string::String::from("B");

        let tasks = std::vec![task_a, task_b];
        let graph = DependencyGraph::new(&tasks);

        let cycles = graph.detect_cycles();
        std::assert!(cycles.is_empty(), "No cycles should exist in disconnected graph");

        let order = graph.topological_sort().unwrap();
        std::assert_eq!(order.len(), 2, "Should have 2 tasks in order");
    }

    #[test]
    fn test_graph_linear_dependency() {
        // Test: Validates topological sort for linear dependency chain A -> B -> C.
        // Justification: Ensures basic topological ordering works correctly.
        let action_a = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Task A"),
            assignee: std::option::Option::None,
            due_date: std::option::Option::None,
        };
        let mut task_a = crate::domain::task::Task::from_action_item(&action_a, std::option::Option::None);
        task_a.id = std::string::String::from("A");
        task_a.dependencies = std::vec![std::string::String::from("B")];

        let action_b = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Task B"),
            assignee: std::option::Option::None,
            due_date: std::option::Option::None,
        };
        let mut task_b = crate::domain::task::Task::from_action_item(&action_b, std::option::Option::None);
        task_b.id = std::string::String::from("B");
        task_b.dependencies = std::vec![std::string::String::from("C")];

        let action_c = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Task C"),
            assignee: std::option::Option::None,
            due_date: std::option::Option::None,
        };
        let mut task_c = crate::domain::task::Task::from_action_item(&action_c, std::option::Option::None);
        task_c.id = std::string::String::from("C");
        task_c.dependencies = std::vec![];

        let tasks = std::vec![task_a, task_b, task_c];
        let graph = DependencyGraph::new(&tasks);

        let cycles = graph.detect_cycles();
        std::assert!(cycles.is_empty(), "No cycles in linear dependency");

        let order = graph.topological_sort().unwrap();
        std::assert_eq!(order, std::vec![
            std::string::String::from("C"),
            std::string::String::from("B"),
            std::string::String::from("A")
        ], "Order should be C, B, A");
    }

    #[test]
    fn test_graph_detects_simple_cycle() {
        // Test: Validates cycle detection for A -> B -> A.
        // Justification: Ensures DFS-based cycle detection catches simple cycles.
        let action_a = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Task A"),
            assignee: std::option::Option::None,
            due_date: std::option::Option::None,
        };
        let mut task_a = crate::domain::task::Task::from_action_item(&action_a, std::option::Option::None);
        task_a.id = std::string::String::from("A");
        task_a.dependencies = std::vec![std::string::String::from("B")];

        let action_b = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Task B"),
            assignee: std::option::Option::None,
            due_date: std::option::Option::None,
        };
        let mut task_b = crate::domain::task::Task::from_action_item(&action_b, std::option::Option::None);
        task_b.id = std::string::String::from("B");
        task_b.dependencies = std::vec![std::string::String::from("A")];

        let tasks = std::vec![task_a, task_b];
        let graph = DependencyGraph::new(&tasks);

        let cycles = graph.detect_cycles();
        std::assert_eq!(cycles.len(), 1, "Should detect 1 cycle");
        std::assert!(cycles[0].contains(&std::string::String::from("A")), "Cycle should contain A");
        std::assert!(cycles[0].contains(&std::string::String::from("B")), "Cycle should contain B");

        let sort_result = graph.topological_sort();
        std::assert!(sort_result.is_err(), "Topological sort should fail with cycle");
    }

    #[test]
    fn test_graph_detects_three_node_cycle() {
        // Test: Validates cycle detection for A -> B -> C -> A.
        // Justification: Ensures cycle detection works for longer cycles (3+ nodes).
        let action_a = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Task A"),
            assignee: std::option::Option::None,
            due_date: std::option::Option::None,
        };
        let mut task_a = crate::domain::task::Task::from_action_item(&action_a, std::option::Option::None);
        task_a.id = std::string::String::from("A");
        task_a.dependencies = std::vec![std::string::String::from("B")];

        let action_b = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Task B"),
            assignee: std::option::Option::None,
            due_date: std::option::Option::None,
        };
        let mut task_b = crate::domain::task::Task::from_action_item(&action_b, std::option::Option::None);
        task_b.id = std::string::String::from("B");
        task_b.dependencies = std::vec![std::string::String::from("C")];

        let action_c = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Task C"),
            assignee: std::option::Option::None,
            due_date: std::option::Option::None,
        };
        let mut task_c = crate::domain::task::Task::from_action_item(&action_c, std::option::Option::None);
        task_c.id = std::string::String::from("C");
        task_c.dependencies = std::vec![std::string::String::from("A")];

        let tasks = std::vec![task_a, task_b, task_c];
        let graph = DependencyGraph::new(&tasks);

        let cycles = graph.detect_cycles();
        std::assert_eq!(cycles.len(), 1, "Should detect 1 cycle");
        std::assert!(cycles[0].len() >= 3, "Cycle should have at least 3 nodes");
    }

    #[test]
    fn test_graph_disconnected_components() {
        // Test: Validates handling of multiple disconnected subgraphs.
        // Justification: Ensures graph algorithms work with independent task groups.
        let action_a = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Task A"),
            assignee: std::option::Option::None,
            due_date: std::option::Option::None,
        };
        let mut task_a = crate::domain::task::Task::from_action_item(&action_a, std::option::Option::None);
        task_a.id = std::string::String::from("A");
        task_a.dependencies = std::vec![std::string::String::from("B")];

        let action_b = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Task B"),
            assignee: std::option::Option::None,
            due_date: std::option::Option::None,
        };
        let mut task_b = crate::domain::task::Task::from_action_item(&action_b, std::option::Option::None);
        task_b.id = std::string::String::from("B");
        task_b.dependencies = std::vec![];

        // Separate component: C -> D
        let action_c = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Task C"),
            assignee: std::option::Option::None,
            due_date: std::option::Option::None,
        };
        let mut task_c = crate::domain::task::Task::from_action_item(&action_c, std::option::Option::None);
        task_c.id = std::string::String::from("C");
        task_c.dependencies = std::vec![std::string::String::from("D")];

        let action_d = transcript_extractor::domain::action_item::ActionItem {
            title: std::string::String::from("Task D"),
            assignee: std::option::Option::None,
            due_date: std::option::Option::None,
        };
        let mut task_d = crate::domain::task::Task::from_action_item(&action_d, std::option::Option::None);
        task_d.id = std::string::String::from("D");
        task_d.dependencies = std::vec![];

        let tasks = std::vec![task_a, task_b, task_c, task_d];
        let graph = DependencyGraph::new(&tasks);

        let cycles = graph.detect_cycles();
        std::assert!(cycles.is_empty(), "No cycles in disconnected components");

        let order = graph.topological_sort().unwrap();
        std::assert_eq!(order.len(), 4, "Should have all 4 tasks");

        // B must come before A, D must come before C
        let b_pos = order.iter().position(|id| id == "B").unwrap();
        let a_pos = order.iter().position(|id| id == "A").unwrap();
        let d_pos = order.iter().position(|id| id == "D").unwrap();
        let c_pos = order.iter().position(|id| id == "C").unwrap();

        std::assert!(b_pos < a_pos, "B should come before A");
        std::assert!(d_pos < c_pos, "D should come before C");
    }
}
