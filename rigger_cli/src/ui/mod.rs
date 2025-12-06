//! UI components for Rigger CLI.
//!
//! Provides reusable UI components for TUI including hierarchical config
//! editor, dialog boxes, and specialized visualizations.
//!
//! Revision History
//! - 2025-12-03T09:00:00Z @AI: Create UI module for config editor (Phase 3 of CONFIG-MODERN-20251203).

pub mod config_editor;

// Re-export for convenience
pub use config_editor::{ConfigEditorState, ConfigTreeNode, FieldStatus};
