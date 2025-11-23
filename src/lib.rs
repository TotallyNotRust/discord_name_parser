use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Represents the user database loaded from Discord exports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDatabase {
    /// Map of user ID to their associated names
    data: HashMap<String, Vec<String>>,
}

impl UserDatabase {
    /// Create a new empty user database
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    /// Load user database from a JSON file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let data: HashMap<String, Vec<String>> = serde_json::from_str(&content)?;
        Ok(Self { data })
    }

    /// Save user database to a JSON file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(&self.data)?;
        fs::write(path, json)?;
        Ok(())
    }

    /// Parse Discord HTML exports from a directory and add to database
    pub fn parse_directory<P: AsRef<Path>>(&mut self, directory: P) -> Result<usize, Box<dyn std::error::Error>> {
        let html_files = find_html_files(directory.as_ref());
        let file_count = html_files.len();

        let mut user_map: HashMap<String, HashSet<String>> = self.data
            .iter()
            .map(|(id, names)| (id.clone(), names.iter().cloned().collect()))
            .collect();

        for file_path in html_files {
            parse_html_file(&file_path, &mut user_map)?;
        }

        // Convert HashSet back to Vec
        self.data = user_map
            .into_iter()
            .map(|(id, names)| {
                let mut names_vec: Vec<String> = names.into_iter().collect();
                names_vec.sort();
                names_vec.dedup();
                (id, names_vec)
            })
            .collect();

        Ok(file_count)
    }

    /// Parse HTML content directly from a string (useful for uploaded files)
    pub fn parse_html_content(&mut self, html_content: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut user_map: HashMap<String, HashSet<String>> = self.data
            .iter()
            .map(|(id, names)| (id.clone(), names.iter().cloned().collect()))
            .collect();

        parse_html_content_string(html_content, &mut user_map)?;

        // Convert HashSet back to Vec
        self.data = user_map
            .into_iter()
            .map(|(id, names)| {
                let mut names_vec: Vec<String> = names.into_iter().collect();
                names_vec.sort();
                names_vec.dedup();
                (id, names_vec)
            })
            .collect();

        Ok(())
    }

    /// Parse multiple HTML files from content strings (useful for batch uploads)
    pub fn parse_html_contents(&mut self, html_contents: &[String]) -> Result<usize, Box<dyn std::error::Error>> {
        let mut user_map: HashMap<String, HashSet<String>> = self.data
            .iter()
            .map(|(id, names)| (id.clone(), names.iter().cloned().collect()))
            .collect();

        for content in html_contents {
            parse_html_content_string(content, &mut user_map)?;
        }

        // Convert HashSet back to Vec
        self.data = user_map
            .into_iter()
            .map(|(id, names)| {
                let mut names_vec: Vec<String> = names.into_iter().collect();
                names_vec.sort();
                names_vec.dedup();
                (id, names_vec)
            })
            .collect();

        Ok(html_contents.len())
    }

    /// Get all names associated with a user ID (exact match)
    pub fn get_names_by_id(&self, user_id: &str) -> Option<&Vec<String>> {
        self.data.get(user_id)
    }

    /// Find user IDs by exact name match
    pub fn find_ids_by_name(&self, name: &str) -> Vec<String> {
        self.data
            .iter()
            .filter(|(_, names)| names.iter().any(|n| n == name))
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Find user IDs by partial name match (case-insensitive)
    pub fn find_ids_by_name_like(&self, pattern: &str) -> Vec<(String, Vec<String>)> {
        let pattern_lower = pattern.to_lowercase();
        self.data
            .iter()
            .filter(|(_, names)| {
                names.iter().any(|n| n.to_lowercase().contains(&pattern_lower))
            })
            .map(|(id, names)| {
                // Return only the matching names
                let matching_names: Vec<String> = names
                    .iter()
                    .filter(|n| n.to_lowercase().contains(&pattern_lower))
                    .cloned()
                    .collect();
                (id.clone(), matching_names)
            })
            .collect()
    }

    /// Search for users by name (returns both exact and partial matches)
    pub fn search_by_name(&self, query: &str) -> SearchResult {
        let exact_matches = self.find_ids_by_name(query);
        let partial_matches = if exact_matches.is_empty() {
            self.find_ids_by_name_like(query)
        } else {
            Vec::new()
        };

        SearchResult {
            query: query.to_string(),
            exact_matches,
            partial_matches,
        }
    }

    /// Get total number of users in database
    pub fn user_count(&self) -> usize {
        self.data.len()
    }

    /// Get all user IDs
    pub fn get_all_ids(&self) -> Vec<String> {
        self.data.keys().cloned().collect()
    }

    /// Get the entire data map (for advanced use cases)
    pub fn get_data(&self) -> &HashMap<String, Vec<String>> {
        &self.data
    }
}

impl Default for UserDatabase {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of a name search operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub query: String,
    /// User IDs with exact name matches
    pub exact_matches: Vec<String>,
    /// User IDs with partial matches (ID, matching names)
    pub partial_matches: Vec<(String, Vec<String>)>,
}

impl SearchResult {
    /// Check if any matches were found
    pub fn has_matches(&self) -> bool {
        !self.exact_matches.is_empty() || !self.partial_matches.is_empty()
    }

    /// Get total number of matches
    pub fn total_matches(&self) -> usize {
        self.exact_matches.len() + self.partial_matches.len()
    }
}

// Internal helper functions

fn find_html_files(directory: &Path) -> Vec<PathBuf> {
    WalkDir::new(directory)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension()
                .and_then(|s| s.to_str())
                .map(|s| s.eq_ignore_ascii_case("html"))
                .unwrap_or(false)
        })
        .map(|e| e.path().to_path_buf())
        .collect()
}

fn parse_html_file(
    file_path: &PathBuf,
    user_map: &mut HashMap<String, HashSet<String>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let content = fs::read_to_string(file_path)?;
    parse_html_content_string(&content, user_map)
}

fn parse_html_content_string(
    content: &str,
    user_map: &mut HashMap<String, HashSet<String>>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Regex to match user information blocks
    let user_block_regex1 = Regex::new(
        r#"(?s)title="Username:\s*([^\n"]+)\s*Display Name:\s*([^\n"]+)(?:\s*Server Nickname:\s*([^\n"]+))?\s*User ID:\s*(\d+)"#
    )?;

    let user_block_regex2 = Regex::new(
        r#"(?s)title="Username:\s*([^\n"]+)(?:\s*Server Nickname:\s*([^\n"]+))?\s*User ID:\s*(\d+)"#
    )?;

    // Method 1: Try to capture complete blocks (Pattern 1 - with Display Name)
    for cap in user_block_regex1.captures_iter(content) {
        let username = cap.get(1).map(|m| m.as_str().trim()).unwrap_or("");
        let display_name = cap.get(2).map(|m| m.as_str().trim()).unwrap_or("");
        let server_nickname = cap.get(3).map(|m| m.as_str().trim());
        let user_id = cap.get(4).map(|m| m.as_str().trim()).unwrap_or("");

        if !user_id.is_empty() {
            let entry = user_map.entry(user_id.to_string()).or_insert_with(HashSet::new);

            if !username.is_empty() {
                entry.insert(username.to_string());
            }
            if !display_name.is_empty() {
                entry.insert(display_name.to_string());
            }
            if let Some(nick) = server_nickname {
                if !nick.is_empty() {
                    entry.insert(nick.to_string());
                }
            }
        }
    }

    // Method 1b: Try to capture blocks without Display Name (Pattern 3)
    for cap in user_block_regex2.captures_iter(content) {
        let username = cap.get(1).map(|m| m.as_str().trim()).unwrap_or("");
        let server_nickname = cap.get(2).map(|m| m.as_str().trim());
        let user_id = cap.get(3).map(|m| m.as_str().trim()).unwrap_or("");

        if !user_id.is_empty() {
            let entry = user_map.entry(user_id.to_string()).or_insert_with(HashSet::new);

            if !username.is_empty() {
                entry.insert(username.to_string());
            }
            if let Some(nick) = server_nickname {
                if !nick.is_empty() {
                    entry.insert(nick.to_string());
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_exact() {
        let mut db = UserDatabase::new();
        db.data.insert("123".to_string(), vec!["Alice".to_string(), "alice123".to_string()]);
        db.data.insert("456".to_string(), vec!["Bob".to_string()]);

        let result = db.find_ids_by_name("Alice");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "123");
    }

    #[test]
    fn test_search_like() {
        let mut db = UserDatabase::new();
        db.data.insert("123".to_string(), vec!["Alice".to_string(), "alice123".to_string()]);
        db.data.insert("456".to_string(), vec!["Bob".to_string()]);
        db.data.insert("789".to_string(), vec!["AliceWonderland".to_string()]);

        let result = db.find_ids_by_name_like("alice");
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_get_names_by_id() {
        let mut db = UserDatabase::new();
        db.data.insert("123".to_string(), vec!["Alice".to_string(), "alice123".to_string()]);

        let names = db.get_names_by_id("123");
        assert!(names.is_some());
        assert_eq!(names.unwrap().len(), 2);
    }
}
