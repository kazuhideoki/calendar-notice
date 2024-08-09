use std::{fs::File, io::BufReader, path};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct OAuthSecret {
    pub client_id: String,
    pub client_secret: String,
    pub token_uri: String,
}
impl OAuthSecret {
    pub fn get_from_file(path: &str) -> Result<Self, String> {
        let file =
            File::open(path).map_err(|e| format!("Failed to open oauth_secret.json: {}", e))?;
        let reader = BufReader::new(file);
        let data: Self = serde_json::from_reader(reader)
            .map_err(|e| format!("Failed to parse oauth_secret.json: {}", e))?;
        Ok(data)
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, io::Write};

    use super::*;

    fn setup() -> &'static str {
        // テスト用のJSONデータを一時ファイルに書き込む
        let file_path = "temp_oauth_secret.json";
        let test_json = r#"
  {
      "client_id": "test_client_id",
      "client_secret": "test_client_secret",
      "token_uri": "https://example.com/token"
  }
  "#;

        let mut file = File::create(file_path).expect("Failed to create test file");
        file.write(test_json.as_bytes())
            .expect("Failed to write test data to file");

        file_path
    }
    fn teardown(file_path: &str) {
        // テスト後に一時ファイルを削除
        fs::remove_file(file_path).expect("Failed to delete test file");
    }

    #[test]
    fn get_from_file_with_valid_path() {
        let file_path = setup();

        let result = OAuthSecret::get_from_file(file_path);

        assert!(result.is_ok());
        let oauth_secret = result.unwrap();
        assert_eq!(oauth_secret.client_id, "test_client_id");
        assert_eq!(oauth_secret.client_secret, "test_client_secret");
        assert_eq!(oauth_secret.token_uri, "https://example.com/token");

        teardown(file_path);
    }
    #[test]
    fn get_from_file_with_invalid_path() {
        let file_path = "invalid_path";

        let result = OAuthSecret::get_from_file(file_path);

        assert!(result.is_err());
    }
}
