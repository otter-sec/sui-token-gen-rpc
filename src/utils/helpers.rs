use rand::{distributions::Alphanumeric, Rng};
use regex::Regex;
use std::{fs, path::Path};

use crate::utils::{errors::TokenGenErrors, variables::TokenDetails};

// Function to validate if the given URL is a valid GitHub or GitLab repository URL
pub fn is_valid_repository_url(url: &str) -> Result<bool, TokenGenErrors> {
    // Regular expression pattern to match GitHub and GitLab URLs
    let repository_url_pattern = r"^https?://(www\.)?(github|gitlab)\.com/[\w\-]+/[\w\-]+/?$";
    let re = Regex::new(repository_url_pattern).expect("Invalid pattern");

    // Check if the URL matches the pattern
    re.is_match(url)
        .then_some(true) // Return true if the URL matches
        .ok_or(TokenGenErrors::InvalidGitUrl) // Return error if the URL doesn't match
}

// Function to sanitize the name by removing any non-alphanumeric characters
pub fn sanitize_name(name: &str) -> String {
    // Filter and collect only alphanumeric characters from the name
    name.chars()
        .filter(|c| c.is_alphanumeric())
        .collect::<String>()
}

// Function to sanitize the repository name by removing invalid characters and appending a random string
pub fn sanitize_repo_name_with_random(repo_name: &str) -> String {
    // Sanitize the repository name to remove path traversal characters
    let sanitized_name = sanitize_repo_name(repo_name);

    // Generate a random 8-character alphanumeric string
    let random_suffix: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(8)
        .map(char::from)
        .collect();

    // Append the random string to the sanitized repository name
    format!("{}_{}", sanitized_name, random_suffix)
}

// Function to filter out only whitespace and empty lines and return an error if comments are found
pub fn filter_whitespace_and_empty_lines(content: &str) -> Result<String, TokenGenErrors> {
    let mut result = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.starts_with("/*") || trimmed.starts_with("#") {
            return Err(TokenGenErrors::ContractModified);
        }
        result.push(trimmed.to_lowercase());
    }

    Ok(result.join("\n"))
}

pub fn get_environment_from_toml(content: &str) -> Option<String> {
    content
        .lines()
        .map(str::trim) // Trim whitespace from each line
        .find_map(|line| {
            if line.starts_with("rev =") {
                line.split('=')
                    .nth(1) // Get the value after '='
                    .map(str::trim) // Remove spaces
                    .map(|s| s.trim_matches('"')) // Remove surrounding quotes
                    .and_then(|s| s.split('/').next_back()) // Get last part after '/'
                    .map(String::from) // Convert to String
            } else {
                None
            }
        })
}

// Function to extract token details (decimals, symbol, name, description, is_frozen) from a contract content
pub fn get_token_info(content: &str) -> TokenDetails {
    let mut name = String::new();
    let mut symbol = String::new();
    let mut description = String::new();
    let mut decimals = 0;
    let mut is_frozen = false;

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("///") {
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if parts.len() == 2 {
                let key = parts[0].trim_start_matches("///").trim().to_lowercase();
                let value = parts[1].trim();
                match key.as_str() {
                    "name" => name = value.to_string(),
                    "symbol" => symbol = value.to_string(),
                    "description" => description = value.to_string(),
                    "decimals" => decimals = value.parse().unwrap_or(0),
                    "is_frozen" => is_frozen = value.parse().unwrap_or(false),
                    _ => {}
                }
            }
        }
    }

    TokenDetails {
        decimals,
        symbol,
        name,
        description,
        is_frozen,
    }
}

// Function to sanitize the repository name by removing path traversal sequences
// This ensures the resulting name is safe for use as a directory name.
pub fn sanitize_repo_name(repo_name: &str) -> String {
    let mut sanitized = repo_name.replace("..", ""); // Remove any ".."
    sanitized = sanitized.replace("/", ""); // Remove "/"
    sanitized = sanitized.replace("\\", ""); // Remove "\"

    // Ensure no remaining "." attempts
    sanitized = sanitized.replace(".", "");

    // Ensure it's not empty after sanitization
    if sanitized.is_empty() {
        return "default_repo".to_string();
    }

    sanitized
}

// Function to check if the cloned contract exists at the specified path, and remove it if it does
pub fn check_cloned_contract(path: &Path) -> Result<(), TokenGenErrors> {
    if path.exists() && path.is_dir() {
        // If the directory exists, remove it
        fs::remove_dir_all(path).map_err(|e| TokenGenErrors::FileIoError(e.to_string()))?;
    }
    Ok(())
}

// Struct that ensures the cloned contract is cleaned up when the operation is finished or fails
pub struct CleanupGuard<'a> {
    pub path: &'a Path,
}

impl Drop for CleanupGuard<'_> {
    fn drop(&mut self) {
        // Attempt to clean up the cloned contract when the guard is dropped
        if let Err(e) = check_cloned_contract(self.path) {
            // If cleaning up fails, log the error
            eprintln!("Failed to clean cloned contract: {:?}", e);
        }
    }
}

pub fn extract_module_and_coin(input: &str) -> Option<(String, String, bool)> {
    let module_re = Regex::new(r"module\s+([a-f0-9]+)\.([a-zA-Z0-9_]+)\s+\{").ok()?;
    let coin_re = Regex::new(r"coin::create_currency<([a-zA-Z0-9_]+)>").ok()?;
    let freeze_re = Regex::new(r"transfer::public_freeze_object").ok()?;
    let share_re = Regex::new(r"transfer::public_share_object").ok()?;

    let module_caps = module_re.captures(input)?;
    let coin_caps = coin_re.captures(input)?;

    let coin_type = coin_caps[1].to_string();
    let is_frozen = freeze_re.is_match(input) && !share_re.is_match(input);

    Some((module_caps[2].to_string(), coin_type, is_frozen))
}
