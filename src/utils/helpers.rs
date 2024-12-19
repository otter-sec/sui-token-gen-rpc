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

// Function to filter out comments, empty lines, and unnecessary whitespace from the content
pub fn filter_token_content(content: &str) -> String {
    content
        .lines() // Split content into lines
        .filter_map(|line| {
            let trimmed = line.trim(); // Trim whitespace from each line

            // Skip empty lines or lines starting/ending with comments
            if trimmed.is_empty()
                || trimmed.starts_with("///")
                || trimmed.starts_with("//")
                || trimmed.ends_with("///")
                || trimmed.ends_with("//")
            {
                None
            } else {
                Some(trimmed) // Keep non-empty, non-comment lines
            }
        })
        .collect::<Vec<&str>>() // Collect filtered lines into a vector
        .join("") // Join the lines into a single string
}

// Function to extract token details (decimals, symbol, name, description, is_frozen) from a contract content
pub fn get_token_info(content: &str) -> TokenDetails {
    // Initialize default values for token properties
    let mut decimals = 0;
    let mut symbol = String::new();
    let mut name = String::new();
    let mut description = String::new();
    let mut is_frozen = false;

    let mut tokens = content.split_whitespace().peekable(); // Split content into tokens for parsing

    while let Some(token) = tokens.next() {
        if token.contains("witness") {
            // Parse the arguments for witness-related information
            let mut args = Vec::new();
            let mut char = String::new();
            for arg in tokens.by_ref() {
                if arg.ends_with(");") || arg.ends_with(")") || arg.ends_with("option::none(),") {
                    // Capture arguments ending with specific characters and break
                    let trimmed = char.trim_end_matches(&[')', ';'][..]).to_string();
                    args.push(trimmed);
                    break;
                }

                if arg.starts_with("b\"") {
                    // Capture and trim byte string arguments
                    let trimmed = char
                        .trim_end_matches(',')
                        .trim_start_matches(" b\"")
                        .to_string();
                    args.push(trimmed);
                    char.clear();
                }

                if char.is_empty() {
                    char = arg.trim_end_matches("\",").to_string();
                } else {
                    char.push(' ');
                    char.push_str(arg.trim_end_matches("\","));
                }
            }

            // If enough arguments are found, assign them to token properties
            if args.len() >= 4 {
                decimals = args[0].trim().parse().unwrap_or(0); // Parse decimals
                symbol = args[1]
                    .trim_start_matches("b\"")
                    .trim_end_matches("\"")
                    .to_string();
                name = args[2]
                    .trim_start_matches("b\"")
                    .trim_end_matches("\"")
                    .to_string();
                description = args[3]
                    .trim_start_matches("b\"")
                    .trim_end_matches("\"")
                    .to_string();
            }
        } else if token.contains("transfer::public_freeze_object") {
            // Set the frozen state if found
            is_frozen = true;
        }
    }

    // Return a TokenDetails struct with the extracted values
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

#[cfg(test)]
mod tests {
    use super::*;

    // Test to ensure that a valid repository path remains unmodified
    #[test]
    fn test_safe_path_valid() {
        let valid_target = "sui-token";
        // Assert that the valid target path is not altered
        assert_eq!(sanitize_repo_name(&valid_target), valid_target);
    }

    // Test to ensure that invalid paths are sanitized correctly
    #[test]
    fn test_safe_path_invalid() {
        let invalid_target = "../etc/psswd";
        // Assert that the invalid target path is sanitized by removing path traversal components
        assert_eq!(sanitize_repo_name(&invalid_target), "etcpsswd");
    }
}
