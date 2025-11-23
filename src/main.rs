use discord_name_parser::UserDatabase;
use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <directory_path> [existing_json_file]", args[0]);
        eprintln!("Example: {} \"F7 Duel Community\" existing_users.json", args[0]);
        process::exit(1);
    }

    let directory = &args[1];
    let existing_file = if args.len() >= 3 {
        Some(&args[2])
    } else {
        None
    };

    // Load existing database or create new
    let mut db = if let Some(file_path) = existing_file {
        match UserDatabase::from_file(file_path) {
            Ok(db) => {
                println!("Loaded existing database with {} users", db.user_count());
                db
            }
            Err(e) => {
                eprintln!("Warning: Could not load existing file: {}", e);
                eprintln!("Creating new database...");
                UserDatabase::new()
            }
        }
    } else {
        UserDatabase::new()
    };

    println!("Scanning directory: {}", directory);

    // Parse HTML files
    match db.parse_directory(directory) {
        Ok(file_count) => {
            println!("Found {} HTML files", file_count);
        }
        Err(e) => {
            eprintln!("Error parsing directory: {}", e);
            process::exit(1);
        }
    }

    // Output JSON
    let json = serde_json::to_string_pretty(db.get_data()).expect("Failed to serialize to JSON");
    println!("\n{}", json);

    // Save to file
    if let Err(e) = db.save_to_file("discord_users.json") {
        eprintln!("Error saving to file: {}", e);
        process::exit(1);
    }

    println!("\n✓ Output written to discord_users.json");
    println!("✓ Found {} unique users", db.user_count());
}
