use std::{
    collections::HashMap,
    sync::{mpsc, Mutex, OnceLock},
};

use oauth_secret::OAuthSecret;
use rand::{distributions::Alphanumeric, Rng};
use tokio::runtime::Runtime;

mod oauth_secret;

pub const BASE_PATH: &str = "http://localhost:8990";
pub const AUTH_REDIRECT: &str = "auth";

fn authentication_code() -> &'static std::sync::Mutex<String> {
    static AUTHENTICATION_CODE: OnceLock<Mutex<String>> = OnceLock::new();
    AUTHENTICATION_CODE.get_or_init(|| Mutex::new("".to_string()))
}

pub fn handle_oauth_redirect(params: std::collections::HashMap<String, String>) -> String {
    if !authentication_code().lock().unwrap().is_empty() {
        return "Completed".to_string();
    }
    // 認証コードを取得時
    if let Some(code) = params.get("code").cloned() {
        let OAuthSecret {
            client_id,
            client_secret,
            token_uri,
        } = OAuthSecret::get_from_file("oauth_secret.json")
            .unwrap_or_else(|e| panic!("Failed to get OAuthSecret from file: {}", e));
        *authentication_code().lock().unwrap() = code.to_string();

        let (tx, rx) = mpsc::channel();

        let rt = Runtime::new().unwrap();
        rt.spawn(async move {
            println!("Requesting access token...");

            // ステップ5 https://developers.google.com/identity/protocols/oauth2/native-app?hl=ja#uwp
            let mut body = HashMap::new();
            body.insert("code", code);
            body.insert("client_id", client_id);
            body.insert("client_secret", client_secret);
            body.insert("grant_type", "authorization_code".to_string());
            body.insert("code_challenge", generate_code_challenge());
            body.insert(
                "redirect_uri",
                format!("{}/{}", BASE_PATH, AUTH_REDIRECT).to_string(),
            );

            let client = reqwest::Client::new();
            let result = client.post(token_uri).form(&body).send().await;

            let result_body = result.unwrap().text().await.unwrap();

            tx.send(result_body).expect("Failed to send result");
        });

        match rx.recv() {
            Ok(response) => {
                println!("🔵Access Token Ok Response: {:?}", response);
            }
            Err(e) => {
                println!("Recv error: {:?}", e.to_string());
            }
        }

        format!("Code!, {}!", *authentication_code().lock().unwrap())
    } else if let Some(code) = params.get("access_token").cloned() {
        println!("🔵 Access Token redirected, {}!", &code);
        "Ok handle_oauth_redirect".to_string()
    } else {
        "Code not found".to_string()
    }
}

fn generate_code_challenge() -> String {
    let verifier: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(60) // 最小長 43 文字
        .map(char::from)
        .collect();
    verifier
}
