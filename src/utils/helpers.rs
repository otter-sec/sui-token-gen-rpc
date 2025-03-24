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
        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("#") {
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
    // Initialize default values for token properties
    let mut decimals = 0;
    let mut symbol = String::new();
    let mut name = String::new();
    let mut description = String::new();
    let mut is_frozen = false;

    // Counter to track occurrences of "witness"
    let mut witness_count = 0;
    let mut i = 0;

    // Split the content into lines and process them
    for line in content.lines() {
        i += 1;
        let trimmed_line = line.trim();
        if trimmed_line.contains("witness") && i > 3 {
            witness_count += 1;
            if witness_count == 2 {
                let token_info = parse_token(trimmed_line);
                decimals = token_info.decimals;
                symbol = token_info.symbol;
                name = token_info.name;
                description = token_info.description;
            }
        }

        // Check if the token is frozen
        if trimmed_line.contains("transfer::public_freeze_object(metadata);") {
            is_frozen = true;
        }
    }
    // Return the TokenDetails struct
    TokenDetails {
        decimals,
        symbol,
        name,
        description,
        is_frozen,
    }
}

#[derive(Debug)]
struct Token {
    decimals: u8,
    symbol: String,
    name: String,
    description: String,
}

fn parse_token(input: &str) -> Token {
    // Initialize variables to hold extracted values
    let mut decimals: u8 = 0;
    let mut idx = 0;
    let chars: Vec<char> = input.chars().collect();

    // Skip "witness, "
    idx += 9;

    // Extract decimals (assume it's a single number)
    while idx < chars.len() && chars[idx].is_ascii_digit() {
        decimals = decimals * 10 + (chars[idx] as u8 - b'0');
        idx += 1;
    }

    // Skip ", "
    idx += 2;

    // Extract symbol
    let (s, next_idx) = extract_b_string(input, idx);
    let symbol = s; // Assign directly here
    idx = next_idx;

    // Skip ", "
    idx += 2;

    // Extract name
    let (n, next_idx) = extract_b_string(input, idx);
    let name = n; // Assign directly here
    idx = next_idx;

    // Skip ", "
    idx += 2;

    // Extract description
    let (d, _) = extract_b_string(input, idx);
    let description = d; // Assign directly here

    // Return the parsed struct
    Token {
        decimals,
        symbol,
        name,
        description,
    }
}

// Helper function to extract a substring enclosed in b"..." notation
fn extract_b_string(input: &str, start_idx: usize) -> (String, usize) {
    let mut result = String::new();
    let mut idx = start_idx;

    // Skip the 'b"' prefix
    idx += 2;

    // Extract characters until the closing '"'
    while idx < input.len() && input.as_bytes()[idx] != b'"' {
        result.push(input.chars().nth(idx).unwrap());
        idx += 1;
    }

    // Skip the closing '"'
    idx += 1;

    (result, idx)
}

// Function to sanitize the repository name by removing path traversal sequences
// This ensures the resulting name is safe for use as a directory name.
pub fn sanitize_repo_name(repo_name: &str) -> String {
    // Replace path traversal characters with an empty string
    repo_name
        .replace("..", "")
        .replace("/", "")
        .replace("\\", "")
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
