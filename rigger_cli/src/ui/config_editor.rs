//! Hierarchical configuration editor for Rigger v3.0 config.
//!
//! Provides tree-based navigation and editing of provider and task slot
//! configurations. Supports expand/collapse sections, API key status
//! indicators, field validation, and save/load to rigger_core::RiggerConfig.
//!
//! Revision History
//! - 2025-12-03T09:10:00Z @AI: Create hierarchical config editor (Phase 3 of CONFIG-MODERN-20251203).

/// Tree node representing a section, item, or field in the config editor.
///
/// Supports hierarchical structure with expand/collapse, editing different
/// field types, and API key status display.
#[derive(Debug, Clone)]
pub enum ConfigTreeNode {
    /// Section header (Providers, Task Slots, Database, Performance, TUI)
    Section {
        name: String,
        expanded: bool,
        children: std::vec::Vec<ConfigTreeNode>,
    },
    /// Provider entry (ollama, openai, anthropic, etc.)
    Provider {
        key: String,
        expanded: bool,
        children: std::vec::Vec<ConfigTreeNode>,
    },
    /// Task slot entry (main, research, fallback, etc.)
    TaskSlot {
        name: String,
        expanded: bool,
        children: std::vec::Vec<ConfigTreeNode>,
    },
    /// String field (editable)
    StringField {
        label: String,
        value: String,
        path: FieldPath,
    },
    /// Boolean field (toggleable)
    BoolField {
        label: String,
        value: bool,
        path: FieldPath,
    },
    /// Number field (editable)
    NumberField {
        label: String,
        value: u64,
        path: FieldPath,
    },
    /// Readonly status field (API key status, etc.)
    StatusField {
        label: String,
        status: FieldStatus,
    },
}

/// Field path for updating values in the config.
///
/// Identifies which field in the RiggerConfig structure to update.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldPath {
    /// Provider field: provider_key, field_name
    Provider(String, String),
    /// Task slot field: slot_name, field_name
    TaskSlot(String, String),
    /// Database field: field_name
    Database(String),
    /// Performance field: field_name
    Performance(String),
    /// TUI field: field_name
    Tui(String),
}

/// Field status for readonly fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldStatus {
    /// API key is available
    ApiKeyPresent,
    /// API key is missing
    ApiKeyMissing,
    /// API key not required
    ApiKeyNotRequired,
    /// Validation passed
    Valid,
    /// Validation failed with error message
    Invalid(String),
}

/// Editor state managing selection, expansion, and editing.
pub struct ConfigEditorState {
    /// The underlying config being edited
    config: rigger_core::RiggerConfig,
    /// Full tree structure (stored for rebuild)
    tree: std::vec::Vec<ConfigTreeNode>,
    /// Flattened list of visible tree nodes (for efficient rendering/navigation)
    visible_nodes: std::vec::Vec<(ConfigTreeNode, usize)>,  // (node, depth)
    /// Currently selected node index
    selected_index: usize,
    /// Currently editing field (if any)
    editing: std::option::Option<EditingState>,
    /// Whether config has unsaved changes
    dirty: bool,
}

/// Editing state for a field.
#[derive(Clone)]
struct EditingState {
    /// Field path being edited
    path: FieldPath,
    /// Edit buffer
    buffer: String,
}

impl ConfigEditorState {
    /// Create a new config editor state from a RiggerConfig.
    pub fn from_config(config: &rigger_core::RiggerConfig) -> Self {
        let tree = Self::build_tree(config);
        let visible_nodes = Self::flatten_tree(&tree);

        Self {
            config: config.clone(),
            tree,
            visible_nodes,
            selected_index: 0,
            editing: std::option::Option::None,
            dirty: false,
        }
    }

    /// Get the underlying config (with any unsaved edits applied).
    pub fn get_config(&self) -> &rigger_core::RiggerConfig {
        &self.config
    }

    /// Rebuild visible nodes from tree (after expansion or edits).
    fn rebuild_visible(&mut self) {
        self.visible_nodes = Self::flatten_tree(&self.tree);
        // Clamp selection to valid range
        if self.selected_index >= self.visible_nodes.len() && !self.visible_nodes.is_empty() {
            self.selected_index = self.visible_nodes.len() - 1;
        }
    }

    /// Build the config tree from a RiggerConfig.
    fn build_tree(config: &rigger_core::RiggerConfig) -> std::vec::Vec<ConfigTreeNode> {
        let mut tree = std::vec::Vec::new();

        // Providers section
        let provider_children = config.providers.iter().map(|(key, provider)| {
            let mut fields = std::vec::Vec::new();

            // Provider type (readonly)
            fields.push(ConfigTreeNode::StringField {
                label: String::from("Type"),
                value: provider.provider_type.to_string(),
                path: FieldPath::Provider(key.clone(), String::from("type")),
            });

            // Base URL
            fields.push(ConfigTreeNode::StringField {
                label: String::from("Base URL"),
                value: provider.base_url.clone(),
                path: FieldPath::Provider(key.clone(), String::from("base_url")),
            });

            // API key status
            let api_key_status = if provider.api_key_env.is_none() {
                FieldStatus::ApiKeyNotRequired
            } else if provider.has_api_key() {
                FieldStatus::ApiKeyPresent
            } else {
                FieldStatus::ApiKeyMissing
            };

            fields.push(ConfigTreeNode::StatusField {
                label: String::from("API Key"),
                status: api_key_status,
            });

            // Timeout
            fields.push(ConfigTreeNode::NumberField {
                label: String::from("Timeout (seconds)"),
                value: provider.timeout_seconds,
                path: FieldPath::Provider(key.clone(), String::from("timeout_seconds")),
            });

            // Max retries
            fields.push(ConfigTreeNode::NumberField {
                label: String::from("Max Retries"),
                value: provider.max_retries as u64,
                path: FieldPath::Provider(key.clone(), String::from("max_retries")),
            });

            // Default model
            fields.push(ConfigTreeNode::StringField {
                label: String::from("Default Model"),
                value: provider.default_model.clone(),
                path: FieldPath::Provider(key.clone(), String::from("default_model")),
            });

            ConfigTreeNode::Provider {
                key: key.clone(),
                expanded: false,
                children: fields,
            }
        }).collect();

        tree.push(ConfigTreeNode::Section {
            name: String::from("Providers"),
            expanded: true,
            children: provider_children,
        });

        // Task Slots section
        let task_slot_children = [
            ("Main", &config.task_slots.main),
            ("Research", &config.task_slots.research),
            ("Fallback", &config.task_slots.fallback),
            ("Embedding", &config.task_slots.embedding),
            ("Vision", &config.task_slots.vision),
            ("Chat Agent", &config.task_slots.chat_agent),
        ].iter().map(|(name, slot)| {
            let mut fields = std::vec::Vec::new();

            // Provider
            fields.push(ConfigTreeNode::StringField {
                label: String::from("Provider"),
                value: slot.provider.clone(),
                path: FieldPath::TaskSlot(name.to_string(), String::from("provider")),
            });

            // Model
            fields.push(ConfigTreeNode::StringField {
                label: String::from("Model"),
                value: slot.model.clone(),
                path: FieldPath::TaskSlot(name.to_string(), String::from("model")),
            });

            // Enabled
            fields.push(ConfigTreeNode::BoolField {
                label: String::from("Enabled"),
                value: slot.enabled,
                path: FieldPath::TaskSlot(name.to_string(), String::from("enabled")),
            });

            // Description (readonly for now)
            fields.push(ConfigTreeNode::StringField {
                label: String::from("Description"),
                value: slot.description.clone(),
                path: FieldPath::TaskSlot(name.to_string(), String::from("description")),
            });

            // Streaming (if present)
            if let std::option::Option::Some(streaming) = slot.streaming {
                fields.push(ConfigTreeNode::BoolField {
                    label: String::from("Streaming"),
                    value: streaming,
                    path: FieldPath::TaskSlot(name.to_string(), String::from("streaming")),
                });
            }

            ConfigTreeNode::TaskSlot {
                name: name.to_string(),
                expanded: false,
                children: fields,
            }
        }).collect();

        tree.push(ConfigTreeNode::Section {
            name: String::from("Task Slots"),
            expanded: true,
            children: task_slot_children,
        });

        // Database section
        let db_children = std::vec![
            ConfigTreeNode::StringField {
                label: String::from("URL"),
                value: config.database.url.clone(),
                path: FieldPath::Database(String::from("url")),
            },
            ConfigTreeNode::BoolField {
                label: String::from("Auto Vacuum"),
                value: config.database.auto_vacuum,
                path: FieldPath::Database(String::from("auto_vacuum")),
            },
            ConfigTreeNode::NumberField {
                label: String::from("Pool Size"),
                value: config.database.pool_size as u64,
                path: FieldPath::Database(String::from("pool_size")),
            },
        ];

        tree.push(ConfigTreeNode::Section {
            name: String::from("Database"),
            expanded: false,
            children: db_children,
        });

        // Performance section
        let perf_children = std::vec![
            ConfigTreeNode::BoolField {
                label: String::from("Enable Metrics"),
                value: config.performance.enable_metrics,
                path: FieldPath::Performance(String::from("enable_metrics")),
            },
            ConfigTreeNode::StringField {
                label: String::from("Metrics File"),
                value: config.performance.metrics_file.clone(),
                path: FieldPath::Performance(String::from("metrics_file")),
            },
            ConfigTreeNode::BoolField {
                label: String::from("Cache Embeddings"),
                value: config.performance.cache_embeddings,
                path: FieldPath::Performance(String::from("cache_embeddings")),
            },
            ConfigTreeNode::NumberField {
                label: String::from("Max Concurrent Tasks"),
                value: config.performance.max_concurrent_tasks as u64,
                path: FieldPath::Performance(String::from("max_concurrent_tasks")),
            },
        ];

        tree.push(ConfigTreeNode::Section {
            name: String::from("Performance"),
            expanded: false,
            children: perf_children,
        });

        // TUI section
        let tui_children = std::vec![
            ConfigTreeNode::StringField {
                label: String::from("Theme"),
                value: config.tui.theme.clone(),
                path: FieldPath::Tui(String::from("theme")),
            },
            ConfigTreeNode::StringField {
                label: String::from("Layout"),
                value: config.tui.layout.clone(),
                path: FieldPath::Tui(String::from("layout")),
            },
            ConfigTreeNode::NumberField {
                label: String::from("Auto Refresh (ms)"),
                value: config.tui.auto_refresh_interval_ms,
                path: FieldPath::Tui(String::from("auto_refresh_interval_ms")),
            },
            ConfigTreeNode::BoolField {
                label: String::from("Show Notifications"),
                value: config.tui.show_notifications,
                path: FieldPath::Tui(String::from("show_notifications")),
            },
        ];

        tree.push(ConfigTreeNode::Section {
            name: String::from("TUI"),
            expanded: false,
            children: tui_children,
        });

        tree
    }

    /// Flatten the tree to a list of visible nodes with their depths.
    fn flatten_tree(tree: &[ConfigTreeNode]) -> std::vec::Vec<(ConfigTreeNode, usize)> {
        let mut result = std::vec::Vec::new();
        Self::flatten_tree_recursive(tree, 0, &mut result);
        result
    }

    fn flatten_tree_recursive(
        nodes: &[ConfigTreeNode],
        depth: usize,
        result: &mut std::vec::Vec<(ConfigTreeNode, usize)>,
    ) {
        for node in nodes {
            result.push((node.clone(), depth));

            match node {
                ConfigTreeNode::Section { expanded, children, .. }
                | ConfigTreeNode::Provider { expanded, children, .. }
                | ConfigTreeNode::TaskSlot { expanded, children, .. } => {
                    if *expanded {
                        Self::flatten_tree_recursive(children, depth + 1, result);
                    }
                }
                _ => {}
            }
        }
    }

    /// Move selection up.
    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Move selection down.
    pub fn move_down(&mut self) {
        if self.selected_index < self.visible_nodes.len().saturating_sub(1) {
            self.selected_index += 1;
        }
    }

    /// Toggle expansion of current node.
    pub fn toggle_expand(&mut self) {
        if let Some((node, _)) = self.visible_nodes.get(self.selected_index).cloned() {
            let node_id = Self::get_node_id(&node);
            if Self::toggle_node_in_tree(&mut self.tree, &node_id) {
                self.rebuild_visible();
            }
        }
    }

    /// Get a unique identifier for a node (for finding it in the tree).
    fn get_node_id(node: &ConfigTreeNode) -> String {
        match node {
            ConfigTreeNode::Section { name, .. } => std::format!("section:{}", name),
            ConfigTreeNode::Provider { key, .. } => std::format!("provider:{}", key),
            ConfigTreeNode::TaskSlot { name, .. } => std::format!("taskslot:{}", name),
            _ => String::new(),
        }
    }

    /// Recursively find and toggle a node in the tree. Returns true if found.
    fn toggle_node_in_tree(nodes: &mut [ConfigTreeNode], target_id: &str) -> bool {
        for node in nodes.iter_mut() {
            match node {
                ConfigTreeNode::Section { name, expanded, children } => {
                    if std::format!("section:{}", name) == target_id {
                        *expanded = !*expanded;
                        return true;
                    }
                    if Self::toggle_node_in_tree(children, target_id) {
                        return true;
                    }
                }
                ConfigTreeNode::Provider { key, expanded, children } => {
                    if std::format!("provider:{}", key) == target_id {
                        *expanded = !*expanded;
                        return true;
                    }
                    if Self::toggle_node_in_tree(children, target_id) {
                        return true;
                    }
                }
                ConfigTreeNode::TaskSlot { name, expanded, children } => {
                    if std::format!("taskslot:{}", name) == target_id {
                        *expanded = !*expanded;
                        return true;
                    }
                    if Self::toggle_node_in_tree(children, target_id) {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }

    /// Start editing the currently selected field.
    pub fn start_editing(&mut self) {
        if let Some((node, _)) = self.visible_nodes.get(self.selected_index) {
            match node {
                ConfigTreeNode::StringField { value, path, .. } => {
                    self.editing = std::option::Option::Some(EditingState {
                        path: path.clone(),
                        buffer: value.clone(),
                    });
                }
                ConfigTreeNode::NumberField { value, path, .. } => {
                    self.editing = std::option::Option::Some(EditingState {
                        path: path.clone(),
                        buffer: value.to_string(),
                    });
                }
                _ => {}
            }
        }
    }

    /// Cancel editing.
    pub fn cancel_editing(&mut self) {
        self.editing = std::option::Option::None;
    }

    /// Commit editing changes.
    pub fn commit_editing(&mut self) {
        // Clone editing state to avoid borrow checker issues
        let editing_clone = self.editing.clone();
        if let Some(editing_state) = editing_clone {
            // Apply the edit to the config
            if self.apply_edit(&editing_state.path, &editing_state.buffer) {
                // Rebuild tree from updated config
                self.tree = Self::build_tree(&self.config);
                self.rebuild_visible();
                self.dirty = true;
            }
        }
        self.editing = std::option::Option::None;
    }

    /// Apply an edit to the config. Returns true if successful.
    fn apply_edit(&mut self, path: &FieldPath, value: &str) -> bool {
        match path {
            FieldPath::Provider(provider_key, field_name) => {
                if let Some(provider) = self.config.providers.get_mut(provider_key) {
                    match field_name.as_str() {
                        "base_url" => {
                            provider.base_url = value.to_string();
                            return true;
                        }
                        "default_model" => {
                            provider.default_model = value.to_string();
                            return true;
                        }
                        "timeout_seconds" => {
                            if let std::result::Result::Ok(num) = value.parse::<u64>() {
                                provider.timeout_seconds = num;
                                return true;
                            }
                        }
                        "max_retries" => {
                            if let std::result::Result::Ok(num) = value.parse::<usize>() {
                                provider.max_retries = num;
                                return true;
                            }
                        }
                        _ => {}
                    }
                }
            }
            FieldPath::TaskSlot(slot_name, field_name) => {
                let slot = match slot_name.as_str() {
                    "Main" => Some(&mut self.config.task_slots.main),
                    "Research" => Some(&mut self.config.task_slots.research),
                    "Fallback" => Some(&mut self.config.task_slots.fallback),
                    "Embedding" => Some(&mut self.config.task_slots.embedding),
                    "Vision" => Some(&mut self.config.task_slots.vision),
                    "Chat Agent" => Some(&mut self.config.task_slots.chat_agent),
                    _ => None,
                };

                if let Some(slot) = slot {
                    match field_name.as_str() {
                        "provider" => {
                            slot.provider = value.to_string();
                            return true;
                        }
                        "model" => {
                            slot.model = value.to_string();
                            return true;
                        }
                        "description" => {
                            slot.description = value.to_string();
                            return true;
                        }
                        _ => {}
                    }
                }
            }
            FieldPath::Database(field_name) => {
                match field_name.as_str() {
                    "url" => {
                        self.config.database.url = value.to_string();
                        return true;
                    }
                    "pool_size" => {
                        if let std::result::Result::Ok(num) = value.parse::<usize>() {
                            self.config.database.pool_size = num;
                            return true;
                        }
                    }
                    _ => {}
                }
            }
            FieldPath::Performance(field_name) => {
                match field_name.as_str() {
                    "metrics_file" => {
                        self.config.performance.metrics_file = value.to_string();
                        return true;
                    }
                    "max_concurrent_tasks" => {
                        if let std::result::Result::Ok(num) = value.parse::<usize>() {
                            self.config.performance.max_concurrent_tasks = num;
                            return true;
                        }
                    }
                    _ => {}
                }
            }
            FieldPath::Tui(field_name) => {
                match field_name.as_str() {
                    "theme" => {
                        self.config.tui.theme = value.to_string();
                        return true;
                    }
                    "layout" => {
                        self.config.tui.layout = value.to_string();
                        return true;
                    }
                    "auto_refresh_interval_ms" => {
                        if let std::result::Result::Ok(num) = value.parse::<u64>() {
                            self.config.tui.auto_refresh_interval_ms = num;
                            return true;
                        }
                    }
                    _ => {}
                }
            }
        }
        false
    }

    /// Add character to edit buffer.
    pub fn edit_push(&mut self, c: char) {
        if let Some(editing) = &mut self.editing {
            editing.buffer.push(c);
        }
    }

    /// Remove character from edit buffer.
    pub fn edit_pop(&mut self) {
        if let Some(editing) = &mut self.editing {
            editing.buffer.pop();
        }
    }

    /// Toggle boolean field.
    pub fn toggle_bool(&mut self) {
        if let Some((node, _)) = self.visible_nodes.get(self.selected_index).cloned() {
            if let ConfigTreeNode::BoolField { path, .. } = node {
                if self.toggle_bool_in_config(&path) {
                    // Rebuild tree from updated config
                    self.tree = Self::build_tree(&self.config);
                    self.rebuild_visible();
                    self.dirty = true;
                }
            }
        }
    }

    /// Toggle a boolean field in the config. Returns true if successful.
    fn toggle_bool_in_config(&mut self, path: &FieldPath) -> bool {
        match path {
            FieldPath::TaskSlot(slot_name, field_name) => {
                let slot = match slot_name.as_str() {
                    "Main" => Some(&mut self.config.task_slots.main),
                    "Research" => Some(&mut self.config.task_slots.research),
                    "Fallback" => Some(&mut self.config.task_slots.fallback),
                    "Embedding" => Some(&mut self.config.task_slots.embedding),
                    "Vision" => Some(&mut self.config.task_slots.vision),
                    "Chat Agent" => Some(&mut self.config.task_slots.chat_agent),
                    _ => None,
                };

                if let Some(slot) = slot {
                    match field_name.as_str() {
                        "enabled" => {
                            slot.enabled = !slot.enabled;
                            return true;
                        }
                        "streaming" => {
                            slot.streaming = Some(!slot.streaming.unwrap_or(false));
                            return true;
                        }
                        _ => {}
                    }
                }
            }
            FieldPath::Database(field_name) => {
                if field_name == "auto_vacuum" {
                    self.config.database.auto_vacuum = !self.config.database.auto_vacuum;
                    return true;
                }
            }
            FieldPath::Performance(field_name) => {
                match field_name.as_str() {
                    "enable_metrics" => {
                        self.config.performance.enable_metrics = !self.config.performance.enable_metrics;
                        return true;
                    }
                    "cache_embeddings" => {
                        self.config.performance.cache_embeddings = !self.config.performance.cache_embeddings;
                        return true;
                    }
                    _ => {}
                }
            }
            FieldPath::Tui(field_name) => {
                if field_name == "show_notifications" {
                    self.config.tui.show_notifications = !self.config.tui.show_notifications;
                    return true;
                }
            }
            _ => {}
        }
        false
    }

    /// Check if currently editing.
    pub fn is_editing(&self) -> bool {
        self.editing.is_some()
    }

    /// Get visible nodes for rendering.
    pub fn visible_nodes(&self) -> &[(ConfigTreeNode, usize)] {
        &self.visible_nodes
    }

    /// Get selected index.
    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    /// Get edit buffer if editing.
    pub fn edit_buffer(&self) -> std::option::Option<&str> {
        self.editing.as_ref().map(|e| e.buffer.as_str())
    }

    /// Check if config has unsaved changes.
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Clear dirty flag after successful save.
    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }
}
