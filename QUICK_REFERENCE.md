# Quick Reference Card

## Installation

```toml
# In your bot's Cargo.toml
[dependencies]
discord_name_parser = { path = "../discord_name_parser" }
```

## Load Database

```rust
use discord_name_parser::UserDatabase;

let db = UserDatabase::from_file("discord_users.json")?;
```

## API Cheat Sheet

### Search by ID → Get Names
```rust
// Returns: Option<&Vec<String>>
db.get_names_by_id("1315621978648084484")
// → Some(["username", "name"])
```

### Search by Name (Exact) → Get IDs
```rust
// Returns: Vec<String>
db.find_ids_by_name("Person")
// → ["85334341430000000"]
```

### Search by Name (Fuzzy) → Get IDs + Matched Names
```rust
// Returns: Vec<(String, Vec<String>)>
db.find_ids_by_name_like("Per")  // Case-insensitive
// → [
//     ("1150000003140000003", ["Person1", "Username1"]),
//     ("1363100003300009118", ["person2"])
//    ]
```

### Advanced Search → Get Both
```rust
// Returns: SearchResult
let result = db.search_by_name("John Doe");
// result.exact_matches: Vec<String>
// result.partial_matches: Vec<(String, Vec<String>)>
// result.has_matches(): bool
// result.total_matches(): usize
```

## Discord Bot Example

```rust
// In your command handler:
fn whois_command(db: &UserDatabase, query: &str) -> String {
    // Try as ID first
    if query.chars().all(|c| c.is_ascii_digit()) {
        if let Some(names) = db.get_names_by_id(query) {
            return format!("<@{}>: {}", query, names.join(", "));
        }
    }

    // Search by name
    let result = db.search_by_name(query);
    if !result.has_matches() {
        return format!("No matches for '{}'", query);
    }

    // Format response...
    format!("Found {} matches", result.total_matches())
}
```

## Common Patterns

### Check if user exists
```rust
db.get_names_by_id(id).is_some()
```

### Get all users
```rust
db.get_all_ids()  // Vec<String>
db.user_count()   // usize
```

### Case-insensitive search
```rust
// All of these work the same:
db.find_ids_by_name_like("PERSON")
db.find_ids_by_name_like("person")
db.find_ids_by_name_like("PeRsOn")
```

## Performance

- ✅ Load once at startup
- ✅ Keep in memory (small dataset)
- ✅ All searches are fast (HashMap-based)
- ⚠️ Limit fuzzy search results (5-10 max)

## See Also

- `INTEGRATION.md` - Full Discord bot integration guide
- `examples/simple_bot.rs` - Complete command examples
- `README.md` - Full documentation
