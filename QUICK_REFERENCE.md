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
// → Some(["ilovecats002", "love spreader🥰"])
```

### Search by Name (Exact) → Get IDs
```rust
// Returns: Vec<String>
db.find_ids_by_name("Zoot")
// → ["853343414330720302"]
```

### Search by Name (Fuzzy) → Get IDs + Matched Names
```rust
// Returns: Vec<(String, Vec<String>)>
db.find_ids_by_name_like("must")  // Case-insensitive
// → [
//     ("1150079713147093093", ["SerMustafa", "sermustafa1"]),
//     ("1363151203348779118", ["Mustafa"])
//    ]
```

### Advanced Search → Get Both
```rust
// Returns: SearchResult
let result = db.search_by_name("kevin");
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
db.find_ids_by_name_like("ZOOT")
db.find_ids_by_name_like("zoot")
db.find_ids_by_name_like("ZoOt")
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
