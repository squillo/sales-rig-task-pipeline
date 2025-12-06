//! Configuration management commands for Rigger CLI.
//!
//! This module provides commands for viewing, validating, editing, and migrating
//! Rigger configuration files. All commands support both project-local (.rigger/config.json)
//! and global (~/.config/rigger/config.json) configuration files.
//!
//! Revision History
//! - 2025-12-04T00:00:00Z @AI: Initial implementation for Phase 4.3 config management CLI.

/// Displays the current configuration with syntax highlighting.
///
/// Loads the configuration from the specified path (or default locations) and
/// displays it as formatted JSON with color coding for better readability.
///
/// # Arguments
///
/// * `path` - Optional path to config file. If None, searches default locations.
///
/// # Errors
///
/// Returns an error if the config file cannot be found or loaded.
pub async fn show(path: std::option::Option<&str>) -> anyhow::Result<()> {
    let config_path = resolve_config_path(path)?;

    println!("📄 Configuration: {}", config_path.display());
    println!();

    // Load and display config
    let config = rigger_core::RiggerConfig::load_with_migration(
        config_path.to_str().ok_or_else(|| anyhow::anyhow!("Invalid config path"))?
    )?;

    // Serialize to pretty JSON
    let json = serde_json::to_string_pretty(&config)?;

    // Display with basic syntax highlighting
    for line in json.lines() {
        if line.contains("\"version\"") || line.contains("\"provider\"") {
            println!("\x1b[33m{}\x1b[0m", line); // Yellow for important fields
        } else if line.contains("{") || line.contains("}") {
            println!("\x1b[36m{}\x1b[0m", line); // Cyan for braces
        } else if line.contains("\"") {
            println!("\x1b[32m{}\x1b[0m", line); // Green for strings
        } else {
            println!("{}", line);
        }
    }

    println!();
    println!("✅ Configuration loaded successfully");

    std::result::Result::Ok(())
}

/// Validates the configuration and displays any errors.
///
/// Loads the configuration, runs all validation checks, and reports any issues
/// found (missing providers, invalid URLs, missing API keys, etc.).
///
/// # Arguments
///
/// * `path` - Optional path to config file. If None, searches default locations.
///
/// # Errors
///
/// Returns an error if the config file cannot be found or loaded.
pub async fn validate(path: std::option::Option<&str>) -> anyhow::Result<()> {
    let config_path = resolve_config_path(path)?;

    println!("🔍 Validating configuration: {}", config_path.display());
    println!();

    // Load config (with migration if needed)
    let config = rigger_core::RiggerConfig::load_with_migration(
        config_path.to_str().ok_or_else(|| anyhow::anyhow!("Invalid config path"))?
    )?;

    // Run validation
    match config.validate() {
        std::result::Result::Ok(()) => {
            println!("✅ Configuration is valid!");
            println!();
            println!("📊 Summary:");
            println!("   Version: {}", config.version);
            println!("   Providers: {}", config.providers.len());
            println!("   Task slots: 6 (main, research, fallback, embedding, vision, chat_agent)");
            println!("   Database: {}", config.database.url);
        }
        std::result::Result::Err(errors) => {
            println!("❌ Configuration has {} error(s):", errors.len());
            println!();
            for (idx, error) in errors.iter().enumerate() {
                println!("  {}. {}", idx + 1, error);
            }
            println!();
            anyhow::bail!("Configuration validation failed");
        }
    }

    std::result::Result::Ok(())
}

/// Opens the configuration editor in TUI mode.
///
/// Launches the interactive TUI with the config editor pre-opened, allowing
/// users to edit configuration values with a visual interface.
///
/// # Errors
///
/// Returns an error if the TUI cannot be launched.
pub async fn edit() -> anyhow::Result<()> {
    println!("🎨 Opening configuration editor...");
    println!();

    // Launch TUI - the TUI will handle config editor navigation
    // For now, just notify user to use TUI + 'c' key
    println!("💡 Configuration editor is available in the TUI:");
    println!("   1. Run: rig tui");
    println!("   2. Press 'c' to open config editor");
    println!("   3. Navigate with ↑/↓, expand with Tab, edit with Enter");
    println!("   4. Save with 's', close with Esc");
    println!();
    println!("🚀 Launching TUI...");

    // Delegate to TUI command
    crate::commands::tui::execute().await
}

/// Migrates legacy configuration to v3.0 format.
///
/// Reads a legacy config file (v0, v1, or v2 format), converts it to the modern
/// v3.0 format with full provider and task slot support, and writes it back.
///
/// # Arguments
///
/// * `path` - Optional path to legacy config. If None, uses .rigger/config.json
/// * `output` - Optional output path. If None, overwrites input file
/// * `backup` - Whether to create a backup before overwriting (default: true)
///
/// # Errors
///
/// Returns an error if migration fails or file operations fail.
pub async fn migrate(
    path: std::option::Option<&str>,
    output: std::option::Option<&str>,
    backup: bool,
) -> anyhow::Result<()> {
    let input_path = resolve_config_path(path)?;
    let output_path = if let Some(out) = output {
        std::path::PathBuf::from(out)
    } else {
        input_path.clone()
    };

    println!("🔄 Migrating configuration...");
    println!("   Input:  {}", input_path.display());
    println!("   Output: {}", output_path.display());
    println!();

    // Create backup if requested and overwriting input
    if backup && input_path == output_path {
        let backup_path = input_path.with_extension("json.backup");
        println!("💾 Creating backup: {}", backup_path.display());
        std::fs::copy(&input_path, &backup_path)?;
    }

    // Load with auto-migration
    let config = rigger_core::RiggerConfig::load_with_migration(
        input_path.to_str().ok_or_else(|| anyhow::anyhow!("Invalid input path"))?
    )?;

    // Validate migrated config
    match config.validate() {
        std::result::Result::Ok(()) => {
            println!("✅ Migration successful! Config is valid.");
        }
        std::result::Result::Err(errors) => {
            println!("⚠️  Migration completed with validation warnings:");
            for error in &errors {
                println!("   - {}", error);
            }
        }
    }

    // Write migrated config
    let json = serde_json::to_string_pretty(&config)?;
    std::fs::write(&output_path, json)?;

    println!();
    println!("✨ Configuration migrated to v3.0 format");
    println!("   Providers: {}", config.providers.len());
    println!("   Task slots: 6 (including new chat_agent)");
    println!();
    println!("📋 Next steps:");
    println!("   - Review with: rig config show");
    println!("   - Edit with: rig config edit");
    println!("   - Validate with: rig config validate");

    std::result::Result::Ok(())
}

/// Resolves the configuration file path, checking default locations.
///
/// Search order:
/// 1. Provided path (if Some)
/// 2. .rigger/config.json (project-local)
/// 3. ~/.config/rigger/config.json (global)
///
/// # Arguments
///
/// * `path` - Optional explicit path to config file
///
/// # Errors
///
/// Returns an error if no config file can be found in any default location.
fn resolve_config_path(path: std::option::Option<&str>) -> anyhow::Result<std::path::PathBuf> {
    if let Some(p) = path {
        let path_buf = std::path::PathBuf::from(p);
        if path_buf.exists() {
            return std::result::Result::Ok(path_buf);
        } else {
            anyhow::bail!("Config file not found: {}", p);
        }
    }

    // Try project-local first
    let local_path = std::path::PathBuf::from(".rigger/config.json");
    if local_path.exists() {
        return std::result::Result::Ok(local_path);
    }

    // Try global config
    if let Some(proj_dirs) = directories::ProjectDirs::from("com", "rigger", "rigger") {
        let global_path = proj_dirs.config_dir().join("config.json");
        if global_path.exists() {
            return std::result::Result::Ok(global_path);
        }
    }

    anyhow::bail!(
        "No config file found. Try:\n  \
         - Create one with: rig init\n  \
         - Or specify path with: --path <path>"
    )
}
