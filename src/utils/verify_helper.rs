use anyhow::Result;
use git2::Repository;
use reqwest::Client;
use std::env;
use std::{fs, io, path::Path};
use url::Url;

use crate::utils::helpers::sanitize_name;
use crate::utils::{
    errors::TokenGenErrors,
    generation::{generate_move_toml, generate_token},
    helpers::{
        check_cloned_contract, filter_whitespace_and_empty_lines, get_environment_from_toml,
        get_token_info, is_valid_repository_url, sanitize_repo_name_with_random, CleanupGuard,
    },
    variables::{TokenDetails, SUB_FOLDER},
};

use super::sui_helper::{build_sui, fetch_coin_metadata, fetch_sui_object, get_rpc_url};
use super::variables::DEFAULT_ENVIRONMENT;
use super::{helpers::extract_module_and_coin, sui_helper::SuiObjectData};

// This function verifies a token using the repository URL provided by the user.
// Steps involved:
// 1. Validate the URL and extract the repository name.
// 2. Clone the repository locally and ensure it is in a valid state.
// 3. Locate and read the first available .move file.
// 4. Extract token details from the file and compare them with the expected contract.
// 5. Perform cleanup operations and return the verified .move file name.
pub async fn verify_token_using_url(url: &str) -> Result<String, TokenGenErrors> {
    // Validate the URL and sanitize the repository name
    let repo_name = validate_url(url)?;

    let clone_path = Path::new(&repo_name);

    // Ensure the cloned contract is in a valid state (e.g., check for necessary files)
    check_cloned_contract(clone_path)?;

    // Initialize a cleanup guard to handle cleaning up after the repository clone operation
    let _cleanup_guard = CleanupGuard { path: clone_path };

    // Get the current working directory
    let current_dir = env::current_dir().map_err(|e| TokenGenErrors::FileIoError(e.to_string()))?;

    // Clone the repository from the given URL
    Repository::clone(url, clone_path).map_err(|e| TokenGenErrors::GitError(e.to_string()))?;

    // Define the expected folder path containing the .move files in the cloned repository
    let sources_path: String = format!("{}/{}/{}", current_dir.display(), repo_name, SUB_FOLDER);

    let toml_path: String = format!("{}/{}/Move.toml", current_dir.display(), repo_name);

    // Convert the folder path into a Path reference
    let sources_path_ref: &Path = Path::new(&sources_path);

    let toml_path_ref: &Path = Path::new(&toml_path);

    // Ensure the folder containing the .move files exists
    if !sources_path_ref.exists() || !sources_path_ref.is_dir() || !toml_path_ref.exists() {
        return Err(TokenGenErrors::ClonedRepoNotFound);
    }

    let toml_content =
        read_file(toml_path_ref).map_err(|e| TokenGenErrors::FileIoError(e.to_string()))?;

    // Read the directory entries to locate the .move files
    let entries =
        fs::read_dir(sources_path_ref).map_err(|e| TokenGenErrors::FileIoError(e.to_string()))?;

    // Initialize a string to store the content of the first .move file
    let mut current_content = String::new();

    // Initialize a string to store the name of the first .move file
    let mut verifying_file_name = String::new();

    // Iterate through the directory entries to find a .move file
    for entry in entries {
        let entry = entry.map_err(|e| TokenGenErrors::FileIoError(e.to_string()))?;
        let path = entry.path();

        // Only process files with the `.move` extension and valid filenames
        if path.is_file()
            && path.extension().is_some_and(|e| e == "move")
            && path
                .file_stem()
                .and_then(|s| s.to_str())
                .map_or(true, |s| !s.contains('.'))
        // Ensure filename does not contain a dot
        {
            // Read the content of the `.move` file
            current_content =
                read_file(&path).map_err(|e| TokenGenErrors::FileIoError(e.to_string()))?;
            if let Some(file_name) = path.file_name().and_then(|f| f.to_str()) {
                verifying_file_name = file_name.to_string();
            }
            break; // Exit the loop after finding the first valid .move file
        }
    }

    // Return an error if no `.move` file or .toml file was found
    if current_content.is_empty() || toml_content.is_empty() {
        return Err(TokenGenErrors::InvalidPathNoMoveFiles);
    }

    // Compare the contract content extracted from the .move file with the generated token contract
    compare_contract_content(current_content, toml_content)?;

    // Ensure the cloned repository is in a good state before cleanup
    check_cloned_contract(clone_path)?;
    Ok(verifying_file_name) // Return verified .move file name if no issues were found
}

// Helper function to read the content of a file into a string
pub fn read_file(file_path: &Path) -> io::Result<String> {
    fs::read_to_string(file_path)
}

// Function to compare the content of the user's contract with the expected token contract content.
// This ensures that the user's contract matches the token generated based on the extracted details.
pub fn compare_contract_content(
    current_content: String,
    toml_content: String,
) -> Result<(), TokenGenErrors> {
    // Filter the content of the user's contract to remove only whitespace and empty lines
    let cleaned_current_content: String = filter_whitespace_and_empty_lines(&current_content)?;

    // Filter the content of toml
    let cleaned_toml_content: String = filter_whitespace_and_empty_lines(&toml_content)?;

    // Extract token details (decimals, symbol, name, etc.) from the filtered content
    let details: TokenDetails = get_token_info(&cleaned_current_content);

    // Generate the expected token contract content based on the extracted details
    let expected_content: String = generate_token(
        details.decimals,            // Number of decimals for the token
        details.symbol.clone(),      // Symbol for the token
        details.name.to_string(),    // Name of the token
        details.description.clone(), // Description of the token
        details.is_frozen,           // Whether the token is frozen or not
        false,                       // Indicating if the contract should be tested (false here)
    );

    let environment = get_environment_from_toml(&cleaned_toml_content)
        .unwrap_or_else(|| DEFAULT_ENVIRONMENT.to_string());

    // Generate the Move.toml
    let expected_toml_content =
        generate_move_toml(sanitize_name(&details.name), environment.to_string());

    // Filter the generated token content to remove only whitespace and empty lines
    let cleaned_expected_content: String = filter_whitespace_and_empty_lines(&expected_content)?;

    // Filter the generated toml content to remove only whitespace and empty lines
    let cleaned_expected_toml_content: String =
        filter_whitespace_and_empty_lines(&expected_toml_content)?;

    // Compare the cleaned-up current contract content with the newly generated contract content
    if cleaned_current_content != cleaned_expected_content
        || cleaned_toml_content != cleaned_expected_toml_content
    {
        return Err(TokenGenErrors::ContractModified); // Return an error if the contracts do not match
    }

    Ok(()) // Return success if the contract content matches
}

// Function to validate the URL provided by the user.
// It checks if the URL is well-formed, verifies if it points to a valid repository,
// and sanitizes the repository name for local use.
fn validate_url(url: &str) -> Result<String, TokenGenErrors> {
    // Parse the URL to check if it is a valid URL
    Url::parse(url).map_err(|_| TokenGenErrors::InvalidUrl)?;

    // Check if the URL corresponds to a valid repository URL
    is_valid_repository_url(url)?;

    // Extract the repository name from the URL
    let name = url
        .trim_end_matches(".git") // Remove the `.git` suffix from the URL
        .rsplit('/') // Split the URL by slashes and take the last part (repository name)
        .next()
        .ok_or(TokenGenErrors::InvalidRepo)?;

    // Sanitize the repository name to ensure it is safe for local use
    Ok(sanitize_repo_name_with_random(name))
}

// Function to verify a deployed token contract by comparing it with a locally generated version
pub async fn verify_address(address: String, environment: String) -> Result<(), TokenGenErrors> {
    // Fetch the RPC URL for the given environment
    let rpc_url = get_rpc_url(&environment)?;
    let client = Client::new();

    // Fetch on-chain object data from Sui blockchain
    let object_data: SuiObjectData = fetch_sui_object(&client, rpc_url, &address).await?;

    // Extract module name, coin type, and freeze status from the contract source
    let (module_name, coin_type, is_frozen) = extract_module_and_coin(&object_data.disassembled)
        .ok_or(TokenGenErrors::ExtractionError)?;

    // Retrieve the deployed bytecode from the module map
    let deployed_byte_code = object_data
        .module_map
        .get(&module_name)
        .ok_or(TokenGenErrors::ExtractionError)?
        .to_string();

    // Construct the full coin type path in the format `address::module_name::coin_type`
    let full_coin_type = format!("{}::{}::{}", address, module_name, coin_type);

    // Fetch the metadata of the coin from the blockchain
    let metadata = fetch_coin_metadata(&client, rpc_url, &full_coin_type).await?;

    // Build a local version of the token contract with the extracted details
    let local_byte_code = build_sui(&metadata, &environment, is_frozen, &address)?;

    // Trim unnecessary characters from the deployed bytecode
    let deployed_byte_code_clean = deployed_byte_code.trim_matches('"');

    // Compare the deployed token contract with the locally built token contract
    (deployed_byte_code_clean == local_byte_code)
        .then_some(())
        .ok_or(TokenGenErrors::ContractModified)
}
