/// Minimal example showing how to integrate with a Discord bot
///
/// Usage in your Discord bot:
/// 1. Add discord_name_parser to Cargo.toml
/// 2. Load database at startup
/// 3. Use in command handlers

use discord_name_parser::UserDatabase;

struct BotState {
    user_db: UserDatabase,
}

impl BotState {
    fn new(database_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let user_db = UserDatabase::from_file(database_path)?;
        println!("✓ Loaded {} users from database", user_db.user_count());
        Ok(Self { user_db })
    }
}

// Simulated Discord command handlers

fn cmd_whois(state: &BotState, query: &str) -> String {
    // Check if query is a user ID (all digits)
    if query.chars().all(|c| c.is_ascii_digit()) {
        if let Some(names) = state.user_db.get_names_by_id(query) {
            return format!("**User <@{}>**\nKnown names: {}", query, names.join(", "));
        }
        return format!("User ID `{}` not found in database", query);
    }

    // Search by name
    let result = state.user_db.search_by_name(query);

    if !result.has_matches() {
        return format!("❌ No users found matching `{}`", query);
    }

    let mut response = String::new();

    // Exact matches
    if !result.exact_matches.is_empty() {
        response.push_str("**Exact matches:**\n");
        for id in &result.exact_matches {
            if let Some(names) = state.user_db.get_names_by_id(id) {
                response.push_str(&format!("• <@{}> — `{}`\n", id, names.join("`, `")));
            }
        }
    }

    // Partial matches (limit to 5)
    if !result.partial_matches.is_empty() {
        if !result.exact_matches.is_empty() {
            response.push_str("\n");
        }
        response.push_str("**Partial matches:**\n");

        for (id, _matched_names) in result.partial_matches.iter().take(5) {
            if let Some(all_names) = state.user_db.get_names_by_id(id) {
                response.push_str(&format!("• <@{}> — `{}`\n", id, all_names.join("`, `")));
            }
        }

        if result.partial_matches.len() > 5 {
            response.push_str(&format!("\n_...and {} more results_", result.partial_matches.len() - 5));
        }
    }

    response
}

fn cmd_lookup(state: &BotState, user_id: &str) -> String {
    match state.user_db.get_names_by_id(user_id) {
        Some(names) => {
            format!("**<@{}>** has used these names:\n{}",
                user_id,
                names.iter().map(|n| format!("• `{}`", n)).collect::<Vec<_>>().join("\n")
            )
        }
        None => format!("No data found for user ID `{}`", user_id),
    }
}

fn cmd_search(state: &BotState, pattern: &str) -> String {
    let matches = state.user_db.find_ids_by_name_like(pattern);

    if matches.is_empty() {
        return format!("No users found matching `{}`", pattern);
    }

    let mut response = format!("**Found {} users matching `{}`:**\n", matches.len(), pattern);

    for (id, matched_names) in matches.iter().take(10) {
        response.push_str(&format!("• <@{}> — `{}`\n", id, matched_names.join("`, `")));
    }

    if matches.len() > 10 {
        response.push_str(&format!("\n_...and {} more results_", matches.len() - 10));
    }

    response
}

fn main() {
    println!("=== Discord Bot User Lookup Example ===\n");

    // Initialize bot state
    let state = BotState::new("discord_users.json")
        .expect("Failed to load database");

    println!("\nSimulating Discord commands:\n");

    // Test !whois command
    println!("Command: !whois Zoot");
    println!("{}\n", cmd_whois(&state, "Zoot"));

    // Test !whois with partial match
    println!("Command: !whois mustafa");
    println!("{}\n", cmd_whois(&state, "mustafa"));

    // Test !lookup command
    println!("Command: !lookup 1315621978648084484");
    println!("{}\n", cmd_lookup(&state, "1315621978648084484"));

    // Test !search command
    println!("Command: !search kevin");
    println!("{}\n", cmd_search(&state, "kevin"));

    // Test !whois with ID
    println!("Command: !whois 853343414330720302");
    println!("{}\n", cmd_whois(&state, "853343414330720302"));
}
