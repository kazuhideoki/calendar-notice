#![allow(unused_variables)]
use clap::Parser;
use std::io::{self, BufRead};

use crate::{
    google_calendar,
    oauth::{is_token_expired, request_access_token_by_refresh_token, OAuthResponse},
    repository::{
        self,
        models::{OAuthToken, OAuthTokenUpdate},
    },
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None, disable_help_flag = true)]
struct Args {
    #[arg(short, long)]
    test: Option<String>,
}

pub async fn wait_for_command() {
    // 標準入力からコマンドを読み取るループ
    let stdin = io::stdin();
    let mut stdin_lines = stdin.lock().lines();

    while let Some(line) = stdin_lines.next() {
        match line {
            Ok(input) => {
                if input.trim().is_empty() {
                    break;
                }
                println!("受け取ったコマンド: {}", input);
                // ここでコマンドを処理する

                match input.as_str() {
                    "token" => println!("{:?}", repository::oauth_token::find_latest().unwrap()),
                    "refresh" => {
                        let OAuthToken {
                            id, refresh_token, ..
                        } = repository::oauth_token::find_latest().unwrap().unwrap();

                        match refresh_token {
                            Some(refresh_token) => {
                                update_token(id, refresh_token).await;
                            }
                            None => {
                                println!("Refresh token is not found");
                            }
                        }
                    }
                    "event" => {
                        let latest_token = repository::oauth_token::find_latest().unwrap();
                        if latest_token
                            .as_ref()
                            .map_or(true, |token| is_token_expired(token))
                        {
                            println!("OAuth token is not found. Please authenticate again.");
                            continue;
                        }

                        let OAuthToken {
                            id,
                            access_token,
                            refresh_token,
                            ..
                        } = latest_token.unwrap();

                        let result = google_calendar::list_events(access_token).await;

                        match result {
                            Ok(events) => {
                                println!(
                                    "{:?}",
                                    events
                                        .items
                                        .iter()
                                        .map(|item| &item.summary)
                                        .collect::<Vec<&String>>()
                                );
                            }
                            Err(google_calendar::Error::Unauthorized) => match refresh_token {
                                Some(refresh_token) => {
                                    update_token(id, refresh_token).await;
                                }
                                None => {
                                    println!("Refresh token is not found");
                                }
                            },
                            Err(e) => eprintln!("エラー: {:?}", e),
                        }
                    }
                    _ => {}
                }
            }
            Err(e) => eprintln!("エラー: {}", e),
        }
    }
}

async fn update_token(id: String, refresh_token: String) {
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
