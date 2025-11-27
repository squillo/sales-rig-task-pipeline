//! LLM response parser for action items.
//!
//! This infrastructure component parses possibly noisy JSON responses from LLM
//! providers into structured ExtractedActionItem DTOs. It handles common schema
//! variations and provides fallback alias mapping when strict deserialization fails.
//!
//! Revision History
//! - 2025-11-23T21:00:00Z @AI: Refactor from utils/ to infrastructure/llm_parsers/ (HEXSER compliance).
//! - 2025-11-08T08:38:10Z @AI: Introduce tolerant parser shared for all adapters.

/// Parses a possibly noisy LLM response into a list of extracted action items.
///
/// This parser handles:
/// - JSON arrays embedded in text (extracts first [...] found)
/// - Field name aliases (e.g., "task" vs "title", "owner" vs "assignee")
/// - Missing optional fields with graceful degradation
/// - Normalization (e.g., assignee → first name only)
///
/// # Arguments
///
/// * `response_text` - Raw text from LLM that should contain a JSON array
///
/// # Returns
///
/// Returns a Vec of ExtractedActionItem if parsing succeeds, or an error string.
///
/// # Examples
///
/// ```
/// use task_manager::infrastructure::llm_parsers::action_item_parser::parse_action_items_tolerant;
///
/// let response = r#"[{"title": "Write docs", "assignee": "Alice"}]"#;
/// let items = parse_action_items_tolerant(response).unwrap();
/// assert_eq!(items[0].title, "Write docs");
/// ```
pub fn parse_action_items_tolerant(
    response_text: &str,
) -> std::result::Result<
    std::vec::Vec<crate::infrastructure::dtos::extracted_action_item::ExtractedActionItem>,
    std::string::String,
> {
    // Try to find JSON array in response (model might include extra text)
    let json_start = response_text
        .find('[')
        .ok_or_else(|| std::string::String::from("No JSON array found in response"))?;
    let json_end = response_text
        .rfind(']')
        .ok_or_else(|| std::string::String::from("No JSON array found in response"))?;

    let json_str = &response_text[json_start..=json_end];

    // First, try strict deserialization into the expected schema.
    let strict: std::result::Result<
        std::vec::Vec<crate::infrastructure::dtos::extracted_action_item::ExtractedActionItem>,
        serde_json::Error,
    > = serde_json::from_str(json_str);
    if let std::result::Result::Ok(items) = strict {
        if items.is_empty() {
            return std::result::Result::Err(std::string::String::from("No action items found in response"));
        }
        return std::result::Result::Ok(items);
    }

    // Fallback: parse loosely and map common alias fields to the schema.
    let value: serde_json::Value = serde_json::from_str(json_str)
        .map_err(|e| std::format!("Failed to parse LLM response as JSON: {}", e))?;
    let arr = match value {
        serde_json::Value::Array(a) => a,
        _ => return std::result::Result::Err(std::string::String::from("Top-level JSON is not an array")),
    };

    let mut out: std::vec::Vec<crate::infrastructure::dtos::extracted_action_item::ExtractedActionItem> = std::vec::Vec::new();

    for (idx, item) in arr.into_iter().enumerate() {
        let obj = match item {
            serde_json::Value::Object(m) => m,
            _ => {
                // Skip non-object entries
                println!("[Parser] Skipping non-object at index {} in LLM JSON array", idx);
                continue;
            }
        };

        // Helper to extract a string value from multiple candidate keys
        fn extract_string(
            m: &serde_json::Map<std::string::String, serde_json::Value>,
            keys: &[&str],
        ) -> std::option::Option<std::string::String> {
            for k in keys {
                if let std::option::Option::Some(v) = m.get(*k) {
                    match v {
                        serde_json::Value::String(s) => {
                            if !s.trim().is_empty() {
                                return std::option::Option::Some(s.trim().to_string());
                            }
                        }
                        serde_json::Value::Number(n) => {
                            return std::option::Option::Some(n.to_string());
                        }
                        serde_json::Value::Bool(b) => {
                            return std::option::Option::Some(b.to_string());
                        }
                        _ => {}
                    }
                }
            }
            std::option::Option::None
        }

        // The model may use aliases; try a set of common variants.
        let title = extract_string(
            &obj,
            &[
                "title",
                "task",
                "action",
                "item",
                "summary",
                "description",
                "name",
            ],
        );

        // If we still don't have a title, attempt to synthesize from other fields.
        let title = match title {
            std::option::Option::Some(t) => t,
            std::option::Option::None => {
                // Combine any present fields to form a basic title; if nothing usable, skip.
                let synthesized = extract_string(&obj, &["content", "text", "details"]);
                match synthesized {
                    std::option::Option::Some(s) if !s.is_empty() => s,
                    _ => {
                        println!("[Parser] Skipping entry {}: missing required 'title' or aliases", idx);
                        continue;
                    }
                }
            }
        };

        // Assignee aliases
        let mut assignee = extract_string(
            &obj,
            &[
                "assignee",
                "owner",
                "assigned_to",
                "responsible",
                "who",
                "person",
                "assignee_name",
            ],
        );
        // Normalize assignee to first word (first name) if present
        if let std::option::Option::Some(a) = assignee.clone() {
            let first = a.split_whitespace().next().unwrap_or("").to_string();
            assignee = if first.is_empty() { std::option::Option::None } else { std::option::Option::Some(first) };
        }

        // Due date aliases (keep as-is; upstream may validate/normalize)
        let due_date = extract_string(
            &obj,
            &[
                "due_date",
                "dueDate",
                "due",
                "deadline",
                "date",
                "due_by",
            ],
        );

        out.push(crate::infrastructure::dtos::extracted_action_item::ExtractedActionItem {
            title,
            assignee,
            due_date,
        });
    }

    if out.is_empty() {
        return std::result::Result::Err(std::string::String::from(
            "Failed to parse LLM response as JSON: no valid items after alias mapping",
        ));
    }
    std::result::Result::Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_strict_json() {
        let json = r#"[{"title": "Write docs", "assignee": "Alice", "due_date": "2025-12-01"}]"#;
        let result = parse_action_items_tolerant(json);
        assert!(result.is_ok());
        let items = result.unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "Write docs");
        assert_eq!(items[0].assignee, Some("Alice".to_string()));
        assert_eq!(items[0].due_date, Some("2025-12-01".to_string()));
    }

    #[test]
    fn test_parse_with_aliases() {
        let json = r#"[{"task": "Fix bug", "owner": "Bob Smith", "deadline": "2025-11-30"}]"#;
        let result = parse_action_items_tolerant(json);
        assert!(result.is_ok());
        let items = result.unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "Fix bug");
        assert_eq!(items[0].assignee, Some("Bob".to_string())); // Normalized to first name
        assert_eq!(items[0].due_date, Some("2025-11-30".to_string()));
    }

    #[test]
    fn test_parse_with_noisy_text() {
        let response = r#"Here are the action items:
        [{"title": "Review PR", "assignee": "Charlie"}]
        Let me know if you need more!"#;
        let result = parse_action_items_tolerant(response);
        assert!(result.is_ok());
        let items = result.unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "Review PR");
    }

    #[test]
    fn test_empty_array_fails() {
        let json = "[]";
        let result = parse_action_items_tolerant(json);
        assert!(result.is_err(), "Empty array should return error");
    }

    #[test]
    fn test_missing_title_skips_entry() {
        let json = r#"[{"assignee": "David"}, {"title": "Valid task"}]"#;
        let result = parse_action_items_tolerant(json);
        assert!(result.is_ok());
        let items = result.unwrap();
        assert_eq!(items.len(), 1); // First entry skipped, second entry parsed
        assert_eq!(items[0].title, "Valid task");
    }
}
