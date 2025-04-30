use reqwest::Client;
use serde::Deserialize;
use serde_json::json;
use std::{error::Error, fs, path::PathBuf, process::Command};
use toml_edit::{value, DocumentMut};

use crate::utils::errors::TokenGenErrors;

use super::{
    generation::{self, ContractGenerator},
    helpers::sanitize_name,
    variables::{SUB_FOLDER, TEST_FOLDER},
};

// Struct to store metadata about a coin
#[derive(Debug, Deserialize)]
pub struct CoinMetadata {
    name: String,
    decimals: u8,
    description: String,
    symbol: String,
}

// Struct to store Sui object data fetched from the blockchain
#[derive(Debug)]
pub struct SuiObjectData {
    pub module_map: serde_json::Value,
    pub disassembled: String,
}

// Function to fetch object data from the Sui blockchain
pub async fn fetch_sui_object(
    client: &Client,
    rpc_url: &str,
    address: &str,
) -> Result<SuiObjectData, TokenGenErrors> {
    let request_json = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "sui_getObject",
        "params": [address, {
            "showType": true,
            "showOwner": true,
            "showPreviousTransaction": true,
            "showDisplay": false,
            "showContent": true,
            "showBcs": true,
            "showStorageRebate": true
        }]
    });

    let response = client
        .post(rpc_url)
        .header("Content-Type", "application/json")
        .json(&request_json)
        .send()
        .await
        .map_err(|_| TokenGenErrors::FetchObjectError("Network error".to_string()))?;

    if !response.status().is_success() {
        return Err(TokenGenErrors::FetchObjectError(
            "Error fetching module data".to_string(),
        ));
    }

    let json_response: serde_json::Value = response
        .json()
        .await
        .map_err(|_| TokenGenErrors::DecodeObjectError)?;

    // Extract module map and disassembled contract code
    let module_map = json_response["result"]["data"]["bcs"]["moduleMap"].clone();
    let disassembled = json_response["result"]["data"]["content"]["disassembled"].to_string();

    Ok(SuiObjectData {
        module_map,
        disassembled,
    })
}

// Function to fetch metadata for a specific coin type
pub async fn fetch_coin_metadata(
    client: &Client,
    rpc_url: &str,
    coin_type: &str,
) -> Result<CoinMetadata, TokenGenErrors> {
    let request_body = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "suix_getCoinMetadata",
        "params": [coin_type]
    });

    let response = client
        .post(rpc_url)
        .json(&request_body)
        .send()
        .await
        .map_err(|e| TokenGenErrors::RpcError(e.to_string()))?;

    if !response.status().is_success() {
        return Err(TokenGenErrors::RpcError(format!(
            "Failed to fetch metadata, status: {}",
            response.status()
        )));
    }

    let json_response: serde_json::Value = response
        .json()
        .await
        .map_err(|e| TokenGenErrors::RpcError(e.to_string()))?;

    let result = json_response
        .get("result")
        .ok_or(TokenGenErrors::InvalidMetadata)?;

    // Parse coin metadata
    Ok(CoinMetadata {
        decimals: result.get("decimals").and_then(|v| v.as_u64()).unwrap_or(0) as u8,
        name: result
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        symbol: result
            .get("symbol")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        description: result
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
    })
}

// Function to get RPC URL based on the environment
pub fn get_rpc_url(environment: &str) -> Result<&'static str, TokenGenErrors> {
    match environment {
        "devnet" => Ok("https://fullnode.devnet.sui.io:443"),
        "testnet" => Ok("https://fullnode.testnet.sui.io:443"),
        "mainnet" => Ok("https://fullnode.mainnet.sui.io:443"),
        _ => Err(TokenGenErrors::InvalidAddress),
    }
}

/// Prepares the project folder and generates required files for the Sui token contract.
fn prepare_sui_contract(
    metadata: &CoinMetadata,
    environment: &str,
    is_frozen: bool,
) -> Result<PathBuf, Box<dyn Error>> {
    // Sanitize the name to create a valid folder name for storing the token.
    let package_name: String = sanitize_name(&metadata.name);
    let project_folder: String = package_name.to_lowercase();
    let current_dir = std::env::current_dir().map_err(|_| TokenGenErrors::CurrentDirectoryError)?;
    let base_folder_path = current_dir.join(&project_folder);

    // Ensure the folder path is valid and convertible to a string.
    let base_folder = base_folder_path
        .to_str()
        .ok_or(TokenGenErrors::PathConversionError)?;

    // Generate the token content and test token content using utility functions.
    let token_content: String = generation::generate_token(
        metadata.decimals,
        metadata.symbol.clone(),
        metadata.name.clone(),
        metadata.description.clone(),
        is_frozen,
        false,
    );
    let test_token_content: String = generation::generate_token(
        metadata.decimals,
        metadata.symbol.clone(),
        metadata.name.clone(),
        metadata.description.clone(),
        is_frozen,
        true,
    );

    // Generate the Move.toml configuration file for the token.
    let move_toml_content =
        generation::generate_move_toml(package_name.to_string(), environment.to_string());

    // Create contract generator instance and populate contract files.
    let contract_generator = ContractGenerator::new(base_folder.to_string());
    contract_generator.create_base_folder()?;
    contract_generator.create_move_toml(&move_toml_content)?;
    contract_generator.create_contract_file(&metadata.name, &token_content, SUB_FOLDER)?;
    contract_generator.create_contract_file(&metadata.name, &test_token_content, TEST_FOLDER)?;

    Ok(base_folder_path)
}

/// Builds the Sui token contract and returns the compiled bytecode.
pub fn build_sui(
    metadata: &CoinMetadata,
    environment: &str,
    is_frozen: bool,
    address: &str,
) -> Result<String, Box<dyn Error>> {
    // Step 1: Prepare the contract files and folders.
    let token_dir = prepare_sui_contract(metadata, environment, is_frozen)?;

    // Step 2: Change module address in Move.toml
    change_module_address(token_dir.join("Move.toml"), address)?;

    // Step 3: Execute `sui move build` command.
    let mut cmd = Command::new("/root/.cargo/bin/sui");
    cmd.arg("move");
    cmd.arg("build");
    cmd.arg("--dump-bytecode-as-base64");
    cmd.current_dir(&token_dir);

    let output = cmd.output()?;
    if !output.status.success() {
        return Err(format!(
            "Sui move build failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    // Step 4: Extract bytecode from the build output.
    let stdout_str = String::from_utf8(output.stdout)?;
    let json: serde_json::Value = serde_json::from_str(&stdout_str)?;
    let bytecode = json["modules"]
        .get(0)
        .and_then(|m| m.as_str())
        .ok_or("Failed to extract bytecode from modules")?
        .to_string();

    // Step 5: Clean up generated files.
    fs::remove_dir_all(&token_dir)?;

    Ok(bytecode)
}

// Function to update the Move.toml module address
pub fn change_module_address(toml_path: PathBuf, address: &str) -> Result<(), Box<dyn Error>> {
    let cargo_toml = fs::read_to_string(&toml_path)?;
    let mut doc = cargo_toml.parse::<DocumentMut>()?;

    let package_name = doc["package"]["name"].as_str().unwrap_or("").to_string();
    doc["addresses"][&package_name] = value(address);

    fs::write(toml_path, doc.to_string())?;
    Ok(())
}
