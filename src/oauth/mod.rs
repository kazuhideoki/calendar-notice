#![allow(unused_variables)]
pub mod is_token_expired;
mod oauth_secret;

use rand::{distributions::Alphanumeric, Rng};
use std::{collections::HashMap, thread, time::Duration};
use warp::Filter;

use serde::{Deserialize, Serialize};

use crate::{
    google_calendar::sync_events,
    oauth::oauth_secret::OAuthSecret,
    repository::{
        self,
        models::{OAuthToken, OAuthTokenUpdate},
    },
};

const PORT: u16 = 8990;
const BASE_URL: &str = "http://localhost";
const AUTH_REDIRECT_PATH: &str = "auth";

use warp::reject::Reject;

#[derive(Debug, Serialize, Deserialize)]
pub struct OAuthResponse {
    pub access_token: String,
    pub expires_in: u64,
    pub refresh_token: Option<String>,
    pub scope: String,
    pub token_type: String,
}
impl OAuthResponse {
    pub fn parse(data: &str) -> Result<Self, std::io::Error> {
        let oauth_response: OAuthResponse = serde_json::from_str(data)?;
        Ok(oauth_response)
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
    println!("Open browser to get OAuth token...");

    thread::sleep(Duration::from_secs(2));

    let oauth_url = format!("https://accounts.google.com/o/oauth2/auth?client_id=121773230254-om9bag3ku8958qmeiv2qa42ddjjfot3d.apps.googleusercontent.com&redirect_uri={BASE_URL}:{PORT}/{AUTH_REDIRECT_PATH}&response_type=code&scope=https://www.googleapis.com/auth/calendar.readonly&access_type=offline&state=random_state_string");

    open::that(oauth_url).expect("Failed to open URL in browser");
}

pub fn spawn_redirect_server() {
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
    if let Some(code) = params.get("code").cloned() {
        let result = request_access_token_by_redirect(code).await;

        match OAuthResponse::parse(&result.unwrap()) {
            Ok(response) => {
                let oauth_token = OAuthToken {
                    id: uuid::Uuid::new_v4().to_string(),
                    access_token: response.access_token.clone(),
                    expires_in: Some(response.expires_in.to_string()),
                    refresh_token: response.refresh_token.clone(),
                    scope: Some(response.scope.clone()),
                    token_type: Some(response.token_type.clone()),
                    created_at: chrono::Local::now().to_rfc3339(),
                    updated_at: chrono::Local::now().to_rfc3339(),
                };
                let _ = repository::oauth_token::create(oauth_token.clone());
                println!("Success to get token!");

                sync_events(oauth_token).await.unwrap_or_else(|e| {
                    println!("Failed to sync events in handle_oauth_redirect: {:?}", e);
                });
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
    match result {
        Ok(result) => Ok(result
            .text()
            .await
            .unwrap_or_else(|e| format!("Failed to get token: {:?}", e))),
        Err(e) => Err(e),
    }
}
async fn request_access_token_by_refresh_token(
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
    match result {
        Ok(result) => Ok(result
            .text()
            .await
            .unwrap_or_else(|e| format!("Failed to get token by refresh token: {:?}", e))),
        Err(e) => Err(e),
    }
}

pub async fn refresh_and_save_token(id: String, refresh_token: String) {
    let result = request_access_token_by_refresh_token(refresh_token).await;

    let oauth_token_response = OAuthResponse::parse(&result.unwrap());
    match oauth_token_response {
        Ok(response) => {
            let token_update = OAuthTokenUpdate {
                access_token: Some(response.access_token),
                expires_in: Some(response.expires_in.to_string()),
                refresh_token: response.refresh_token,
                scope: Some(response.scope),
                token_type: Some(response.token_type),
                updated_at: chrono::Local::now().to_rfc3339(),
            };
            let _ = repository::oauth_token::update(id, token_update);
            println!("Success to get token! by refresh token");
        }
        Err(e) => {
            println!("Recv error: {:?}", e.to_string());
        }
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
