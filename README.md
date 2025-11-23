# Discord Name Parser

A Rust library and CLI tool to extract Discord user IDs and their associated usernames/display names from exported Discord chat HTML files, with powerful search capabilities.

## Features

- **Library + CLI**: Use as a library in your project or as a standalone CLI tool
- Parses HTML files exported from Discord
- Extracts User IDs, Usernames, Display Names, and Server Nicknames
- **Search API**: Search by ID or name (exact and partial/fuzzy matching)
- Supports updating an existing JSON file with new names
- Outputs deduplicated list of all names per user ID
- Recursively scans directories for HTML files

## Installation

### As a Library (for Discord bots, etc.)

Add to your `Cargo.toml`:

```toml
[dependencies]
discord_name_parser = { path = "../discord_name_parser" }
```

### As a CLI Tool

```bash
cd discord_name_parser
cargo build --release
# Binary will be in target/release/discord_name_parser
```

## Library Usage

### Basic Example

```rust
use discord_name_parser::UserDatabase;

// Load existing database
let db = UserDatabase::from_file("discord_users.json")?;

// Search by user ID
if let Some(names) = db.get_names_by_id("1315621978648084484") {
    println!("Names: {:?}", names);
}

// Search by name (exact)
let ids = db.find_ids_by_name("Person");
println!("User IDs: {:?}", ids);

// Search by name (partial, case-insensitive)
let matches = db.find_ids_by_name_like("Dude");
for (id, matching_names) in matches {
    println!("{}: {:?}", id, matching_names);
}
```

### Discord Bot Integration

```rust
use discord_name_parser::UserDatabase;

// Load once at bot startup
let db = UserDatabase::from_file("discord_users.json")?;

// In your command handler:
fn handle_whois(db: &UserDatabase, query: &str) -> String {
    let result = db.search_by_name(query);

    if !result.has_matches() {
        return format!("No users found matching '{}'", query);
    }

    // Format response with exact and partial matches
    // See examples/discord_bot_usage.rs for full implementation
}
```

### API Reference

#### `UserDatabase`

- `new()` - Create empty database
- `from_file(path)` - Load from JSON file
- `save_to_file(path)` - Save to JSON file
- `parse_directory(path)` - Parse Discord HTML exports
- `get_names_by_id(id)` - Get all names for a user ID
- `find_ids_by_name(name)` - Find IDs with exact name match
- `find_ids_by_name_like(pattern)` - Find IDs with partial match (case-insensitive)
- `search_by_name(query)` - Combined search returning `SearchResult`
- `user_count()` - Get total number of users

#### `SearchResult`

- `exact_matches: Vec<String>` - User IDs with exact matches
- `partial_matches: Vec<(String, Vec<String>)>` - User IDs with partial matches
- `has_matches()` - Check if any matches found
- `total_matches()` - Get total count

## CLI Usage

### First run (create new database):
```bash
cargo run --release -- "Server"
```

### Update existing database:
```bash
cargo run --release -- "Server" discord_users.json
```

### Scan multiple directories:
```bash
# First directory
cargo run --release -- "Server"

# Update with second directory
cargo run --release -- "Server 12-08-2025" discord_users.json

# Update with third directory
cargo run --release -- "Server (2)" discord_users.json
```

## Output Format

The tool outputs JSON in the following format:

```json
{
  "1315621978648084484": [
    "Name1",
    "Username1"
  ],
  "853343414330720302": [
    "Name2",
    "Username2"
  ]
}
```

## How It Works

1. Recursively finds all `.html` files in the specified directory
2. Uses regex patterns to extract user information blocks containing:
   - Username (Discord handle)
   - Display Name (user-set display name)
   - Server Nickname (server-specific nickname)
   - User ID (unique Discord ID)
3. Merges with existing data if provided
4. Deduplicates and sorts names for each user
5. Outputs to both stdout and `discord_users.json`

## Example

```bash
$ cargo run --release -- "../Server"
Scanning directory: ../Server
Found 14 HTML files

{
  "1315621978648084484": [
    "Name1",
    "Username1"
  ],
  ...
}

✓ Output written to discord_users.json
✓ Found 42 unique users
```
