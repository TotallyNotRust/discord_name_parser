# Integration Guide for Discord Bots

This guide shows how to integrate `discord_name_parser` into your Discord bot.

## Quick Start

### 1. Add to Your Project

In your bot's `Cargo.toml`:

```toml
[dependencies]
discord_name_parser = { path = "../discord_name_parser" }
# Or use git:
# discord_name_parser = { git = "https://github.com/yourusername/discord_name_parser" }
```

### 2. Load Database at Startup

```rust
use discord_name_parser::UserDatabase;

// In your bot initialization code:
let user_db = UserDatabase::from_file("discord_users.json")
    .expect("Failed to load user database");

println!("Loaded {} users", user_db.user_count());
```

### 3. Store in Bot State

Most Discord bot frameworks use some kind of state/context pattern:

#### Serenity Example

```rust
use serenity::prelude::TypeMapKey;
use discord_name_parser::UserDatabase;

struct UserDbContainer;
impl TypeMapKey for UserDbContainer {
    type Value = UserDatabase;
}

// In your main():
{
    let mut data = client.data.write().await;
    data.insert::<UserDbContainer>(user_db);
}

// In your command handler:
let data = ctx.data.read().await;
let db = data.get::<UserDbContainer>().unwrap();
```

#### Poise Example

```rust
struct Data {
    user_db: UserDatabase,
}

// In your main():
let framework = poise::Framework::builder()
    .options(poise::FrameworkOptions {
        commands: vec![whois()],
        ..Default::default()
    })
    .setup(|ctx, _ready, framework| {
        Box::pin(async move {
            Ok(Data {
                user_db: UserDatabase::from_file("discord_users.json")?,
            })
        })
    })
    .build();
```

## Command Examples

### Basic Lookup Command

```rust
#[poise::command(slash_command, prefix_command)]
async fn lookup(
    ctx: Context<'_>,
    #[description = "User ID to lookup"] user_id: String,
) -> Result<(), Error> {
    let db = &ctx.data().user_db;

    let response = match db.get_names_by_id(&user_id) {
        Some(names) => {
            format!("User <@{}> has used these names:\n{}",
                user_id,
                names.iter().map(|n| format!("• `{}`", n)).collect::<Vec<_>>().join("\n")
            )
        }
        None => format!("No data found for user ID `{}`", user_id),
    };

    ctx.say(response).await?;
    Ok(())
}
```

### Advanced Search Command

```rust
#[poise::command(slash_command, prefix_command)]
async fn whois(
    ctx: Context<'_>,
    #[description = "Username or ID to search"] query: String,
) -> Result<(), Error> {
    let db = &ctx.data().user_db;

    // Check if query is a user ID
    if query.chars().all(|c| c.is_ascii_digit()) {
        if let Some(names) = db.get_names_by_id(&query) {
            let response = format!("**User <@{}>**\nKnown names: {}",
                query, names.join(", "));
            ctx.say(response).await?;
            return Ok(());
        }
    }

    // Search by name
    let result = db.search_by_name(&query);

    if !result.has_matches() {
        ctx.say(format!("❌ No users found matching `{}`", query)).await?;
        return Ok(());
    }

    let mut response = String::new();

    if !result.exact_matches.is_empty() {
        response.push_str("**Exact matches:**\n");
        for id in &result.exact_matches {
            if let Some(names) = db.get_names_by_id(id) {
                response.push_str(&format!("• <@{}> — `{}`\n", id, names.join("`, `")));
            }
        }
    }

    if !result.partial_matches.is_empty() {
        if !result.exact_matches.is_empty() {
            response.push_str("\n");
        }
        response.push_str("**Partial matches:**\n");
        for (id, _) in result.partial_matches.iter().take(5) {
            if let Some(names) = db.get_names_by_id(id) {
                response.push_str(&format!("• <@{}> — `{}`\n", id, names.join("`, `")));
            }
        }
        if result.partial_matches.len() > 5 {
            response.push_str(&format!("\n_...and {} more_", result.partial_matches.len() - 5));
        }
    }

    ctx.say(response).await?;
    Ok(())
}
```

### Fuzzy Search Command

```rust
#[poise::command(slash_command, prefix_command)]
async fn search(
    ctx: Context<'_>,
    #[description = "Name pattern to search"] pattern: String,
) -> Result<(), Error> {
    let db = &ctx.data().user_db;
    let matches = db.find_ids_by_name_like(&pattern);

    if matches.is_empty() {
        ctx.say(format!("No users found matching `{}`", pattern)).await?;
        return Ok(());
    }

    let mut response = format!("**Found {} users matching `{}`:**\n", matches.len(), pattern);

    for (id, matched_names) in matches.iter().take(10) {
        response.push_str(&format!("• <@{}> — `{}`\n", id, matched_names.join("`, `")));
    }

    if matches.len() > 10 {
        response.push_str(&format!("\n_...and {} more_", matches.len() - 10));
    }

    ctx.say(response).await?;
    Ok(())
}
```

## API Quick Reference

### Search by ID
```rust
// Returns Option<&Vec<String>>
db.get_names_by_id("1000000000648084484")
```

### Search by Exact Name
```rust
// Returns Vec<String> of user IDs
db.find_ids_by_name("Person")
```

### Search by Partial Name (fuzzy)
```rust
// Returns Vec<(String, Vec<String>)> - (user_id, matching_names)
db.find_ids_by_name_like("pers")  // Case-insensitive
```

### Combined Search
```rust
// Returns SearchResult with both exact and partial matches
let result = db.search_by_name("John Doe");
if result.has_matches() {
    // Handle exact_matches and partial_matches
}
```

## Performance Tips

1. **Load once at startup** - Don't reload the database on every command
2. **Keep in memory** - The database is small and searching is fast
3. **Limit results** - For partial matches, limit display to 5-10 results
4. **Use exact match first** - Check for exact matches before fuzzy search

## Updating the Database

To add new Discord exports to the database:

```bash
# Parse new HTML exports and merge with existing database
cargo run --release -- "New Discord Export Folder" discord_users.json
```

Then restart your bot to reload the updated database.

## Examples

See the `examples/` directory for complete working examples:
- `discord_bot_usage.rs` - Detailed API usage examples
- `simple_bot.rs` - Minimal bot command handlers

Run them with:
```bash
cargo run --example simple_bot
cargo run --example discord_bot_usage
```
