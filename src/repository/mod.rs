use std::fs;

use serde::{Deserialize, Serialize};

// やはり、oauth モジュールに OAuthResponse 構造体を移動しましょうか？
#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthResponse {
    pub access_token: String,
    pub expires_in: u64,
    pub refresh_token: Option<String>,
    scope: String,
    token_type: String,
}
impl OAuthResponse {
    pub fn from_file() -> Option<Self> {
        let file = fs::read_to_string("oauth_token_response.json").unwrap_or_else(|_| {
            fs::write("oauth_token_response.json", "{}").unwrap();
            "".to_string()
        });
        serde_json::from_str(&file).ok()
    }
    pub fn parse_and_save(data: &str) -> Result<Self, std::io::Error> {
        fs::write("oauth_token_response.json", data)?;
        let oauth_response: OAuthResponse = serde_json::from_str(data)?;
        Ok(oauth_response)
    }
}
