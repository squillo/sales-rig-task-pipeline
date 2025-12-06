//! Example demonstrating Rigger v3.0 configuration.
//!
//! Shows how to create, validate, and use the new config system with
//! multiple providers and task slots.

fn main() {
    // Create default config
    let config = rigger_core::RiggerConfig::default();

    std::println!("📦 Rigger Config v{}\n", config.version);

    // Show providers
    std::println!("🔌 Providers:");
    for (name, provider) in &config.providers {
        std::println!("  - {}: {} ({})",
            name,
            provider.provider_type,
            provider.base_url
        );
        std::println!("    API Key: {}", provider.get_masked_api_key());
    }

    // Show task slots
    std::println!("\n🔧 Task Slots:");
    for (name, slot) in [
        ("Main", &config.task_slots.main),
        ("Research", &config.task_slots.research),
        ("Fallback", &config.task_slots.fallback),
        ("Embedding", &config.task_slots.embedding),
        ("Vision", &config.task_slots.vision),
        ("Chat Agent", &config.task_slots.chat_agent),
    ] {
        let status = if slot.enabled { "✓" } else { "✗" };
        std::println!("  {} {}: {}/{}", status, name, slot.provider, slot.model);
    }

    // Validate config
    std::println!("\n✅ Validation:");
    match config.validate() {
        std::result::Result::Ok(()) => std::println!("  Config is valid!"),
        std::result::Result::Err(errors) => {
            std::println!("  Found {} error(s):", errors.len());
            for error in errors {
                std::println!("    - {}", error);
            }
        }
    }

    // Serialize to JSON
    std::println!("\n📄 JSON Output:");
    let json = serde_json::to_string_pretty(&config).unwrap();
    std::println!("{}", json);
}
