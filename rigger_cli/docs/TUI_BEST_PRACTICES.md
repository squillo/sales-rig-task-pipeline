# TUI Development Best Practices

## Buffer Safety in Ratatui

### The Problem

Ratatui uses a fixed-size buffer to render terminal content. When you try to render content outside the buffer bounds, it panics with:

```
index outside of buffer: the area is Rect { x: 0, y: 0, width: W, height: H }
but index is (x, y)
```

This is a **runtime panic** that crashes the application - there's no graceful error handling.

### Common Causes

1. **Uncapped Dialog Heights**: Calculating dialog height as `lines.len() + 2` without checking terminal size
2. **Dynamic Content**: Error messages, user input, or LLM responses that can be arbitrarily long
3. **Small Terminals**: Users with small terminal windows (e.g., 35 lines) hit limits quickly
4. **Variable Content**: Content that changes based on runtime conditions (errors, API responses)

### The Solution: Always Use Safe Height Calculation

**MANDATORY**: All dialog rendering MUST use the `calculate_safe_dialog_height()` helper:

```rust
// ❌ UNSAFE - Can cause buffer overflow
let dialog_height = lines.len() as u16 + 2;

// ✅ SAFE - Automatically capped to available space
let dialog_height = calculate_safe_dialog_height(lines.len(), area.height);
```

### The Helper Function

Located in `src/commands/tui.rs`:

```rust
/// Calculates a safe dialog height that will never exceed the available buffer area.
///
/// This function prevents buffer overflow panics in ratatui by capping the dialog
/// height to the available space. Use this for all dialog rendering to ensure
/// buffer safety.
///
/// # Arguments
///
/// * `content_lines` - Number of content lines (without border)
/// * `available_height` - Total available height from the rendering area
///
/// # Returns
///
/// A safe height value that includes borders (+2) but never exceeds available space.
fn calculate_safe_dialog_height(content_lines: usize, available_height: u16) -> u16 {
    let desired_height = content_lines as u16 + 2; // +2 for borders
    let max_height = available_height.saturating_sub(4); // Leave 4 lines padding
    std::cmp::min(desired_height, max_height)
}
```

### Test Coverage

The helper function has comprehensive test coverage:

```rust
#[test]
fn test_calculate_safe_dialog_height_within_bounds() {
    // Small dialog fits completely
    assert_eq!(calculate_safe_dialog_height(10, 35), 12);
}

#[test]
fn test_calculate_safe_dialog_height_exceeds_bounds() {
    // Large dialog is capped to available space
    assert_eq!(calculate_safe_dialog_height(50, 35), 31);
}

#[test]
fn test_calculate_safe_dialog_height_exact_fit() {
    // Boundary condition: content exactly fills available space
    assert_eq!(calculate_safe_dialog_height(29, 35), 31);
}

#[test]
fn test_calculate_safe_dialog_height_tiny_terminal() {
    // Graceful handling of very small terminals
    assert_eq!(calculate_safe_dialog_height(20, 10), 6);
}
```

### Audit Checklist

When creating or reviewing TUI code, check:

- [ ] All `dialog_height` calculations use `calculate_safe_dialog_height()`
- [ ] No hardcoded height values that could exceed buffer bounds
- [ ] Dynamic content (errors, user input) is bounded
- [ ] Dialogs handle small terminal sizes gracefully
- [ ] Width calculations also respect buffer bounds

### Finding Unsafe Patterns

Use these grep commands to audit code:

```bash
# Find potentially unsafe height calculations
grep -n "let dialog_height = " src/commands/tui.rs | grep -v "calculate_safe_dialog_height"

# Find hardcoded dialog heights
grep -n "let dialog_height = [0-9]" src/commands/tui.rs

# Find lines.len() without safety check
grep -n "lines.len() as u16" src/commands/tui.rs
```

### Historical Fixes

**2025-11-25**: Fixed 7 vulnerable functions:
- `render_prd_processing()` - Long error messages from Ollama
- `render_wizard_complete()` - Multi-line welcome text
- `render_wizard_welcome()` - Setup wizard intro
- `render_wizard_task_tool_slots()` - Slot configuration explanation
- `render_wizard_configure_slot()` - Provider selection screen
- `render_wizard_database_configuration()` - Database setup screen
- `render_wizard_confirmation()` - Final confirmation screen

All now use `calculate_safe_dialog_height()` for guaranteed buffer safety.

### Additional Best Practices

1. **Pagination**: For very long content, implement pagination instead of scrolling
2. **Truncation**: Use `truncate_string()` helper for long single lines
3. **Wrapping**: Use ratatui's text wrapping features for long paragraphs
4. **Testing**: Test with terminal heights of 20, 35, 50 lines
5. **Error Messages**: Keep error messages concise or implement scrolling

### CI/CD Integration

Consider adding these checks to CI:

```bash
# Fail build if unsafe patterns detected
! grep -q "let dialog_height = lines.len() as u16 + 2" src/commands/tui.rs
```

### Resources

- [Ratatui Buffer Documentation](https://docs.rs/ratatui/latest/ratatui/buffer/struct.Buffer.html)
- [Terminal Size Detection](https://docs.rs/crossterm/latest/crossterm/terminal/fn.size.html)
- [Layout Constraints](https://docs.rs/ratatui/latest/ratatui/layout/enum.Constraint.html)

---

**Remember**: Buffer overflows in ratatui cause immediate panics. Prevention is the only option - there's no try/catch for this. Always use safe helper functions.
