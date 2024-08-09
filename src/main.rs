#![allow(unused_variables)]
use dotenvy::dotenv;
use oauth::{handle_oauth_redirect, AUTH_REDIRECT};
use std::env;
use warp::Filter;

mod calendar;
mod oauth;

async fn http_server() {
    let routes = warp::path(AUTH_REDIRECT)
        .and(warp::query::<std::collections::HashMap<String, String>>())
        .map(handle_oauth_redirect);

    let port = 8990;
    println!("HTTP server starting at {}", port);
    warp::serve(routes).run(([127, 0, 0, 1], port)).await;
}

/**
 * TODO
 * - oatuh まわりをサーバーとしてまとめる
 * - token をグローバルで扱えるようにする
 * - カレンダーを取得する
 */
#[tokio::main]
async fn main() {
    let auth_url = "https://accounts.google.com/o/oauth2/auth?client_id=121773230254-om9bag3ku8958qmeiv2qa42ddjjfot3d.apps.googleusercontent.com&redirect_uri=http://localhost:8990/auth&response_type=code&scope=https://www.googleapis.com/auth/calendar.readonly&access_type=offline&state=random_state_string";

    // ブラウザでURLを開く
    if let Err(e) = open::that(auth_url) {
        eprintln!("Failed to open URL in browser: {}", e);
    } else {
        println!("Opened authentication URL in your default browser.");
    }
    // HTTPサーバーを起動
    tokio::spawn(async {
        http_server().await;
    });

    // メインタスクを終了させないようにする
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for ctrl+c event");
    println!("Shutting down");
}
