mod oauth_secret;

use rand::{distributions::Alphanumeric, Rng};
use std::{collections::HashMap, sync::mpsc};
use tokio::runtime::Runtime;
use warp::Filter;

use crate::{oauth::oauth_secret::OAuthSecret, repository::OAuthResponse};

const PORT: u16 = 8990;
const BASE_URL: &str = "http://localhost";
const AUTH_REDIRECT_PATH: &str = "auth";

pub fn to_oauth_on_browser() {
    let oauth_url = format!("https://accounts.google.com/o/oauth2/auth?client_id=121773230254-om9bag3ku8958qmeiv2qa42ddjjfot3d.apps.googleusercontent.com&redirect_uri={BASE_URL}:{PORT}/{AUTH_REDIRECT_PATH}&response_type=code&scope=https://www.googleapis.com/auth/calendar.readonly&access_type=offline&state=random_state_string");

    open::that(oauth_url).expect("Failed to open URL in browser");
}

pub fn run_redirect_server() {
    // HTTPサーバーを起動
    tokio::spawn(async {
        let routes = warp::path(AUTH_REDIRECT_PATH)
            .and(warp::query::<std::collections::HashMap<String, String>>())
            .map(handle_oauth_redirect);

        println!("HTTP server starting at {}", PORT);
        warp::serve(routes).run(([127, 0, 0, 1], PORT)).await;
    });
}

// TODO リクエスト部分を Result にして整理
fn handle_oauth_redirect(params: std::collections::HashMap<String, String>) -> String {
    if OAuthResponse::from_file().is_some() {
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

        let (tx, rx) = mpsc::channel();

        let rt = Runtime::new().unwrap();
        rt.spawn(async move {
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

            let result_body = result.unwrap().text().await.unwrap();

            tx.send(result_body).expect("Failed to send result");
        });

        // match serde_json::from_str::<OAuthResponse>(&rx.recv().expect(
        match OAuthResponse::parse_and_save(&rx.recv().expect("Failed to recv")) {
            Ok(response) => {
                println!("🔵Access Token Ok Response: {:?}", response);
            }
            Err(e) => {
                println!("Recv error: {:?}", e.to_string());
            }
        }

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
