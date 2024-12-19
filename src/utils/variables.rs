// Constant for the subfolder where source files will be stored
pub const SUB_FOLDER: &str = "sources";

// Constant for the GitHub URL of the SUI project repository
// This URL points to the official repository for the Sui blockchain framework
pub const SUI_PROJECT: &str = "https://github.com/MystenLabs/sui.git";

// Constant for the specific subdirectory within the SUI project where the main framework code is located
// This path refers to the location of the Sui framework within the repository
pub const SUI_PROJECT_SUB_DIR: &str = "crates/sui-framework/packages/sui-framework";

// Struct representing the details of a token, typically used in smart contract contexts
// This struct holds various properties that define the characteristics of a token
pub struct TokenDetails {
    // The number of decimal places the token can represent
    // This is important for defining the precision of token amounts
    pub decimals: u8,

    // The symbol or ticker for the token (e.g., "BTC", "ETH")
    // This helps identify the token in user interfaces and on exchanges
    pub symbol: String,

    // The name of the token (e.g., "Bitcoin", "Ethereum")
    // This is a human-readable representation of the token's identity
    pub name: String,

    // A short description of the token's purpose or functionality
    // This is typically displayed on user interfaces to give context to the token
    pub description: String,

    // Boolean indicating whether the token is frozen (i.e., if it is temporarily non-transferable)
    // This could be used to implement additional security measures or functionality within a smart contract
    pub is_frozen: bool,
}
