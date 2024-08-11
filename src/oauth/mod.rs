#![allow(unused_variables)]
mod oauth_secret;

use rand::{distributions::Alphanumeric, Rng};
use std::collections::HashMap;
use std::fs;
use warp::Filter;

use serde::{Deserialize, Serialize};

use crate::oauth::oauth_secret::OAuthSecret;

const PORT: u16 = 8990;
const BASE_URL: &str = "http://localhost";
const AUTH_REDIRECT_PATH: &str = "auth";

use warp::reject::Reject;

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
    pub fn parse(data: &str) -> Result<Self, std::io::Error> {
        let oauth_response: OAuthResponse = serde_json::from_str(data)?;
        Ok(oauth_response)
    }
    pub fn save_to_file(&self) -> Result<(), std::io::Error> {
        let previous = Self::from_file().unwrap();
        let updated = OAuthResponse {
            access_token: self.access_token.clone(),
            expires_in: self.expires_in,
            refresh_token: previous.refresh_token.or(self.refresh_token.clone()),
            scope: self.scope.clone(),
            token_type: self.token_type.clone(),
        };
        fs::write(
            "oauth_token_response.json",
            serde_json::to_string(&updated)?,
        )?;
        Ok(())
    }
}

#[derive(Debug)]
struct ReqwestError;

impl Reject for ReqwestError {}

impl From<reqwest::Error> for ReqwestError {
    fn from(_: reqwest::Error) -> Self {
        ReqwestError
    }
}

pub fn to_oauth_on_browser() {
    let oauth_url = format!("https://accounts.google.com/o/oauth2/auth?client_id=121773230254-om9bag3ku8958qmeiv2qa42ddjjfot3d.apps.googleusercontent.com&redirect_uri={BASE_URL}:{PORT}/{AUTH_REDIRECT_PATH}&response_type=code&scope=https://www.googleapis.com/auth/calendar.readonly&access_type=offline&state=random_state_string");

    open::that(oauth_url).expect("Failed to open URL in browser");
}

pub fn run_redirect_server() {
    tokio::spawn(async {
        let routes = warp::path(AUTH_REDIRECT_PATH)
            .and(warp::query::<std::collections::HashMap<String, String>>())
            .and_then(handle_oauth_redirect);

        println!("HTTP server starting at {}", PORT);
        warp::serve(routes).run(([127, 0, 0, 1], PORT)).await;
    });
}

async fn handle_oauth_redirect(
    params: std::collections::HashMap<String, String>,
) -> Result<impl warp::Reply, warp::Rejection> {
    if OAuthResponse::from_file().is_some() {
        return Ok("Completed".to_string());
    }

    // 認証コードを取得時
    if let Some(code) = params.get("code").cloned() {
        let result = request_access_token_by_redirect(code).await;

        match OAuthResponse::parse(&result.unwrap()) {
            Ok(response) => {
                response
                    .save_to_file()
                    .expect("Failed to save OAuthResponse to file");
                println!("Success to get token!");
            }
            Err(e) => {
                println!("Recv error: {:?}", e.to_string());
            }
        }

        Ok("Ok handle_oauth_redirect".to_string())
    } else {
        // TODO エラー処理
        Ok("Failed to get code".to_string())
    }
}

async fn request_access_token_by_redirect(code: String) -> Result<String, reqwest::Error> {
    let OAuthSecret {
        client_id,
        client_secret,
        token_uri,
    } = OAuthSecret::get_from_file("oauth_secret.json")
        .unwrap_or_else(|e| panic!("Failed to get OAuthSecret from file: {}", e));

    // ステップ5 https://developers.google.com/identity/protocols/oauth2/native-app?hl=ja#uwp
    let mut body = HashMap::new();
    body.insert("code", code);
    body.insert("client_id", client_id);
    body.insert("client_secret", client_secret);
    body.insert("grant_type", "authorization_code".to_string());
    body.insert("code_challenge", generate_code_challenge());
    body.insert(
        "redirect_uri",
        format!("{BASE_URL}:{PORT}/{AUTH_REDIRECT_PATH}",).to_string(),
    );

    let client = reqwest::Client::new();
    let result = client.post(token_uri).form(&body).send().await;
    let result = match result {
        Ok(result) => result.text().await,
        Err(e) => {
            println!("Failed to get token: {:?}", e);
            return Ok("Failed to get token".to_string());
        }
    };

    result
}
pub async fn request_access_token_by_refresh_token(
    refresh_token: String,
) -> Result<String, reqwest::Error> {
    let OAuthSecret {
        client_id,
        client_secret,
        token_uri,
    } = OAuthSecret::get_from_file("oauth_secret.json")
        .unwrap_or_else(|e| panic!("Failed to get OAuthSecret from file: {}", e));

    let mut body = HashMap::new();
    body.insert("client_id", client_id);
    body.insert("client_secret", client_secret);
    body.insert("refresh_token", refresh_token);
    body.insert("grant_type", "refresh_token".to_string());

    let client = reqwest::Client::new();
    let result = client.post(token_uri).form(&body).send().await;
    let result = match result {
        Ok(result) => result.text().await,
        Err(e) => {
            println!("Failed to get token: {:?}", e);
            return Ok("Failed to get token".to_string());
        }
    };

    result
}

fn generate_code_challenge() -> String {
    let verifier: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(60) // 最小長 43 文字
        .map(char::from)
        .collect();
    verifier
}
