//! Markdown parser for Product Requirements Documents (PRDs).
//!
//! This parser extracts structured sections from PRD markdown files including
//! Objectives, Tech Stack, and Constraints. It handles standard markdown
//! formatting with ## headers and bullet point lists.
//!
//! Revision History
//! - 2025-11-24T05:00:00Z @AI: Add project_id parameter to parse_prd_markdown for Phase 1 TUI project architecture.
//! - 2025-11-23T22:15:00Z @AI: Fix doctest example formatting (HEXSER refactoring).
//! - 2025-11-22T16:05:00Z @AI: Initial PRD markdown parser for Rigger Phase 0.

/// Parses a PRD markdown file into a structured PRD entity.
///
/// Extracts sections marked with ## Objectives, ## Tech Stack, and ## Constraints.
/// Each section's content is parsed as bullet points (lines starting with -, *, or numbers).
/// The first # header is used as the title.
///
/// # Arguments
///
/// * `project_id` - The ID of the project this PRD belongs to.
/// * `content` - The raw markdown content of the PRD file.
///
/// # Returns
///
/// A Result containing the parsed PRD on success, or an error message on failure.
///
/// # Errors
///
/// Returns an error if:
/// - No title (# header) is found
/// - Content is empty
///
/// # Examples
///
/// ```
/// use task_manager::infrastructure::markdown_parsers::prd_parser::parse_prd_markdown;
///
/// let markdown = "# Build Rigger Platform\n\n## Objectives\n- Enable AI agent task decomposition\n- Support multiple LLM providers\n\n## Tech Stack\n- Rust\n- Rig framework\n\n## Constraints\n- Must compile with Rust 2024 edition\n";
///
/// let prd = parse_prd_markdown("project-123", markdown).unwrap();
/// assert_eq!(prd.project_id, "project-123");
/// assert_eq!(prd.title, "Build Rigger Platform");
/// assert_eq!(prd.objectives.len(), 2);
/// assert_eq!(prd.tech_stack.len(), 2);
/// assert_eq!(prd.constraints.len(), 1);
/// ```
pub fn parse_prd_markdown(project_id: &str, content: &str) -> std::result::Result<crate::domain::prd::PRD, std::string::String> {
    if content.trim().is_empty() {
        return std::result::Result::Err(std::string::String::from("PRD content cannot be empty"));
    }

    let lines: std::vec::Vec<&str> = content.lines().collect();

    // Extract title (first # header)
    let title = extract_title(&lines)?;

    // Extract sections
    let objectives = extract_section(&lines, "## Objectives");
    let tech_stack = extract_section(&lines, "## Tech Stack");
    let constraints = extract_section(&lines, "## Constraints");

    std::result::Result::Ok(crate::domain::prd::PRD::new(
        project_id.to_string(),
        title,
        objectives,
        tech_stack,
        constraints,
        content.to_string(),
    ))
}

/// Extracts the title from the first # header in the markdown.
fn extract_title(lines: &[&str]) -> std::result::Result<String, std::string::String> {
    for line in lines {
        let trimmed = line.trim();
        if trimmed.starts_with("# ") && !trimmed.starts_with("##") {
            return std::result::Result::Ok(trimmed[2..].trim().to_string());
        }
    }
    std::result::Result::Err(std::string::String::from("No title (# header) found in PRD"))
}

/// Extracts bullet points or numbered items from a section.
///
/// Finds the section header and collects all lines that start with bullet
/// markers (-, *, or numbers) until the next ## header or end of document.
fn extract_section(lines: &[&str], header: &str) -> std::vec::Vec<String> {
    let mut items = std::vec::Vec::new();
    let mut in_section = false;

    for line in lines {
        let trimmed = line.trim();

        // Check if we've entered the target section
        if trimmed == header {
            in_section = true;
            continue;
        }

        // Check if we've hit another ## header (end of current section)
        if in_section && trimmed.starts_with("##") {
            break;
        }

        // Extract bullet points or numbered items
        if in_section {
            if let Some(item) = extract_list_item(trimmed) {
                items.push(item);
            }
        }
    }

    items
}

/// Extracts the content from a list item (bullet or numbered).
///
/// Recognizes:
/// - `- item` (dash)
/// - `* item` (asterisk)
/// - `1. item` (numbered)
/// - `1) item` (numbered with parenthesis)
fn extract_list_item(line: &str) -> std::option::Option<String> {
    let trimmed = line.trim();

    // Skip empty lines
    if trimmed.is_empty() {
        return std::option::Option::None;
    }

    // Match bullet points
    if trimmed.starts_with("- ") {
        return std::option::Option::Some(trimmed[2..].trim().to_string());
    }
    if trimmed.starts_with("* ") {
        return std::option::Option::Some(trimmed[2..].trim().to_string());
    }

    // Match numbered lists (e.g., "1. " or "1) ")
    if let Some(dot_pos) = trimmed.find(". ") {
        if let Some(first_char) = trimmed.chars().next() {
            if first_char.is_numeric() && dot_pos < 4 {
                // Verify all chars before dot are numeric
                if trimmed[..dot_pos].chars().all(|c| c.is_numeric()) {
                    return std::option::Option::Some(trimmed[dot_pos + 2..].trim().to_string());
                }
            }
        }
    }

    if let Some(paren_pos) = trimmed.find(") ") {
        if let Some(first_char) = trimmed.chars().next() {
            if first_char.is_numeric() && paren_pos < 4 {
                if trimmed[..paren_pos].chars().all(|c| c.is_numeric()) {
                    return std::option::Option::Some(trimmed[paren_pos + 2..].trim().to_string());
                }
            }
        }
    }

    std::option::Option::None
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_full_prd() {
        // Test: Validates full PRD parsing with all sections.
        // Justification: Ensures parser handles complete PRD documents correctly.
        let markdown = r#"
# Build Rigger Platform

## Objectives
- Enable AI agent task decomposition
- Support multiple LLM providers
- Integrate with Cursor and Windsurf

## Tech Stack
- Rust (2024 edition)
- Rig framework
- HEXSER architecture

## Constraints
- Must compile with Rust 2024 edition
- No unsafe code except for FFI
        "#;

        let prd = super::parse_prd_markdown("test-project-123", markdown).unwrap();

        std::assert_eq!(prd.project_id, "test-project-123");
        std::assert_eq!(prd.title, "Build Rigger Platform");
        std::assert_eq!(prd.objectives.len(), 3);
        std::assert_eq!(prd.objectives[0], "Enable AI agent task decomposition");
        std::assert_eq!(prd.tech_stack.len(), 3);
        std::assert_eq!(prd.tech_stack[0], "Rust (2024 edition)");
        std::assert_eq!(prd.constraints.len(), 2);
    }

    #[test]
    fn test_parse_prd_with_missing_sections() {
        // Test: Validates parser handles PRDs with missing optional sections.
        // Justification: Not all PRDs will have every section populated.
        let markdown = r#"
# Minimal Project

## Objectives
- Build something great
        "#;

        let prd = super::parse_prd_markdown("test-project-456", markdown).unwrap();

        std::assert_eq!(prd.project_id, "test-project-456");
        std::assert_eq!(prd.title, "Minimal Project");
        std::assert_eq!(prd.objectives.len(), 1);
        std::assert_eq!(prd.tech_stack.len(), 0);
        std::assert_eq!(prd.constraints.len(), 0);
    }

    #[test]
    fn test_parse_prd_with_numbered_lists() {
        // Test: Validates parser handles numbered list format.
        // Justification: PRDs may use numbered lists instead of bullets.
        let markdown = r#"
# Numbered List Project

## Objectives
1. First objective
2. Second objective
3) Third objective with paren

## Tech Stack
1. Technology one
2. Technology two
        "#;

        let prd = super::parse_prd_markdown("test-project-789", markdown).unwrap();

        std::assert_eq!(prd.objectives.len(), 3);
        std::assert_eq!(prd.objectives[2], "Third objective with paren");
        std::assert_eq!(prd.tech_stack.len(), 2);
    }

    #[test]
    fn test_parse_prd_with_asterisk_bullets() {
        // Test: Validates parser handles asterisk bullet format.
        // Justification: Markdown supports both - and * for bullets.
        let markdown = r#"
# Asterisk Project

## Objectives
* First item with asterisk
* Second item with asterisk
        "#;

        let prd = super::parse_prd_markdown("test-project-abc", markdown).unwrap();

        std::assert_eq!(prd.objectives.len(), 2);
        std::assert_eq!(prd.objectives[0], "First item with asterisk");
    }

    #[test]
    fn test_parse_prd_no_title_fails() {
        // Test: Validates parser returns error when no title found.
        // Justification: Title is required for PRD identification.
        let markdown = r#"
## Objectives
- No title in this document
        "#;

        let result = super::parse_prd_markdown("test-project-def", markdown);
        std::assert!(result.is_err());
    }

    #[test]
    fn test_parse_empty_prd_fails() {
        // Test: Validates parser returns error for empty content.
        // Justification: Empty content is invalid input.
        let result = super::parse_prd_markdown("test-project-ghi", "");
        std::assert!(result.is_err());
    }

    #[test]
    fn test_extract_list_item_variants() {
        // Test: Validates list item extraction handles all formats.
        // Justification: Ensures robust parsing of different list styles.
        std::assert_eq!(
            super::extract_list_item("- Dash bullet"),
            std::option::Option::Some(std::string::String::from("Dash bullet"))
        );
        std::assert_eq!(
            super::extract_list_item("* Asterisk bullet"),
            std::option::Option::Some(std::string::String::from("Asterisk bullet"))
        );
        std::assert_eq!(
            super::extract_list_item("1. Numbered item"),
            std::option::Option::Some(std::string::String::from("Numbered item"))
        );
        std::assert_eq!(
            super::extract_list_item("42) Paren numbered"),
            std::option::Option::Some(std::string::String::from("Paren numbered"))
        );
        std::assert_eq!(
            super::extract_list_item("Not a list item"),
            std::option::Option::None
        );
        std::assert_eq!(
            super::extract_list_item(""),
            std::option::Option::None
        );
    }

    #[test]
    fn test_prd_preserves_raw_content() {
        // Test: Validates raw content is preserved in parsed PRD.
        // Justification: Raw content needed for debugging and reference.
        let markdown = "# Test\n\n## Objectives\n- Item";
        let prd = super::parse_prd_markdown("test-project-jkl", markdown).unwrap();

        std::assert_eq!(prd.raw_content, markdown);
    }
}
