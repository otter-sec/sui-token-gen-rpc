use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CreateRequest {
    pub decimals: u8,
    pub name: String,
    pub symbol: String,
    pub description: String,
    pub is_frozen: bool,
    pub environment: String,
}

#[derive(Debug, Serialize)]
pub struct CreateResponse {
    pub token: String,
    pub move_toml: String,
    pub test_token: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifyUrlRequest {
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifyRequest {
    pub content: String,
}
