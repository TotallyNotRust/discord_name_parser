/// Example showing how to use discord_name_parser in a Discord bot
///
/// Add to your bot's Cargo.toml:
/// ```toml
/// [dependencies]
/// discord_name_parser = { path = "../discord_name_parser" }
/// # or if published:
/// # discord_name_parser = "0.1.0"
/// ```

use discord_name_parser::UserDatabase;

fn main() {
    // Load the pre-parsed database
    let db = UserDatabase::from_file("discord_users.json")
        .expect("Failed to load user database");

    println!("Loaded database with {} users\n", db.user_count());

    // Example 1: Search by User ID
    println!("=== Example 1: Get names by User ID ===");
    let user_id = "1315621978648084484";
    if let Some(names) = db.get_names_by_id(user_id) {
        println!("User {}: {:?}", user_id, names);
    } else {
        println!("User {} not found", user_id);
    }
    println!();

    // Example 2: Exact name search
    println!("=== Example 2: Find user IDs by exact name ===");
    let name = "Jane Doe";
    let ids = db.find_ids_by_name(name);
    println!("Users with exact name '{}': {:?}", name, ids);
    for id in &ids {
        if let Some(all_names) = db.get_names_by_id(id) {
            println!("  {} -> {:?}", id, all_names);
        }
    }
    println!();

    // Example 3: Partial/fuzzy name search
    println!("=== Example 3: Find users by partial name ===");
    let pattern = "John Doe";
    let matches = db.find_ids_by_name_like(pattern);
    println!("Users matching '{}': {} results", pattern, matches.len());
    for (id, matching_names) in &matches {
        if let Some(all_names) = db.get_names_by_id(id) {
            println!("  {} (matched: {:?})", id, matching_names);
            println!("    All names: {:?}", all_names);
        }
    }
    println!();

    // Example 4: Advanced search with SearchResult
    println!("=== Example 4: Using SearchResult API ===");
    let query = "Jane Doe";
    let result = db.search_by_name(query);

    println!("Search query: '{}'", result.query);
    println!("Has matches: {}", result.has_matches());
    println!("Total matches: {}", result.total_matches());

    if !result.exact_matches.is_empty() {
        println!("\nExact matches:");
        for id in &result.exact_matches {
            if let Some(names) = db.get_names_by_id(id) {
                println!("  {} -> {:?}", id, names);
            }
        }
    }

    if !result.partial_matches.is_empty() {
        println!("\nPartial matches:");
        for (id, matched_names) in &result.partial_matches {
            if let Some(all_names) = db.get_names_by_id(id) {
                println!("  {} (matched: {:?})", id, matched_names);
                println!("    All names: {:?}", all_names);
            }
        }
    }
    println!();

    // Example 5: Case-insensitive search
    println!("=== Example 5: Case-insensitive search ===");
    let patterns = vec!["JOHN DOE", "john doe", "JoHn DoE"];
    for pattern in patterns {
        let matches = db.find_ids_by_name_like(pattern);
        println!("Pattern '{}': {} matches", pattern, matches.len());
    }
}

// Example Discord bot command handler
#[allow(dead_code)]
fn handle_whois_command(db: &UserDatabase, query: &str) -> String {
    // Try to parse as user ID first
    if query.chars().all(|c| c.is_ascii_digit()) {
        if let Some(names) = db.get_names_by_id(query) {
            return format!("User <@{}> has used these names: {}", query, names.join(", "));
        }
    }

    // Search by name
    let result = db.search_by_name(query);

    if !result.has_matches() {
        return format!("No users found matching '{}'", query);
    }

    let mut response = String::new();

    if !result.exact_matches.is_empty() {
        response.push_str("**Exact matches:**\n");
        for id in &result.exact_matches {
            if let Some(names) = db.get_names_by_id(id) {
                response.push_str(&format!("• <@{}> ({})\n", id, names.join(", ")));
            }
        }
    }

    if !result.partial_matches.is_empty() {
        if !result.exact_matches.is_empty() {
            response.push_str("\n");
        }
        response.push_str("**Partial matches:**\n");
        for (id, matched_names) in result.partial_matches.iter().take(5) {
            if let Some(all_names) = db.get_names_by_id(id) {
                response.push_str(&format!(
                    "• <@{}> matched `{}` (all names: {})\n",
                    id,
                    matched_names.join(", "),
                    all_names.join(", ")
                ));
            }
        }
        if result.partial_matches.len() > 5 {
            response.push_str(&format!("\n_...and {} more_", result.partial_matches.len() - 5));
        }
    }

    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_whois_command() {
        let mut db = UserDatabase::new();
        db.get_data().insert(
            "123456789".to_string(),
            vec!["Test User".to_string(), "test_user".to_string()]
        );

        let response = handle_whois_command(&db, "Test User");
        assert!(response.contains("123456789"));
    }
}
