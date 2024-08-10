use std::fs;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthResponse {
    pub access_token: String,
    pub expires_in: u64,
    pub refresh_token: String,
    scope: String,
    token_type: String,
}
impl OAuthResponse {
    pub fn from_file() -> Option<Self> {
        let file = fs::read_to_string("oauth_token.json").unwrap();
        serde_json::from_str(&file).ok()
    }
    pub fn parse_and_save(data: &str) -> Result<Self, std::io::Error> {
        fs::write("oauth_token.json", data)?;
        let oauth_response: OAuthResponse = serde_json::from_str(data)?;
        Ok(oauth_response)
    }
}
