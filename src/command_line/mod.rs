#![allow(unused_variables)]
use clap::Parser;
use std::io::{self, BufRead, Error};

use crate::{
    google_calendar,
    oauth::{
        self, is_token_expired::is_token_expired, request_access_token_by_refresh_token,
        OAuthResponse,
    },
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
    let mut stdin_lines = io::stdin().lock().lines();

    while let Some(line) = stdin_lines.next() {
        match line {
            Ok(input) => {
                if input.trim().is_empty() {
                    break;
                }
                println!("Input: {}", input);

                match input.as_str() {
                    "token" => {
                        handle_command_token();
                        continue;
                    }
                    "refresh" => {
                        handle_command_refresh().await;
                        continue;
                    }
                    "event" => {
                        handle_command_event().await;
                        continue;
                    }
                    _ => {}
                }
            }
            Err(e) => eprintln!("Error occurred when reading line: {:?}", e),
        }
    }
}

// TODO きれいに出力
fn handle_command_token() {
    let oauth_token = repository::oauth_token::find_latest();

    match oauth_token {
        Ok(Some(token)) => {
            println!("{:?}", token);
        }
        Ok(None) => {
            println!("OAuth token is not found");
        }
        Err(e) => {
            eprintln!("Error: {:?}", e);
        }
    }
}

async fn handle_command_refresh() {
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

async fn handle_command_event() {
    let token_result = repository::oauth_token::find_latest();
    match token_result {
        Ok(token) => {
            if token
                .as_ref()
                .map_or(true, |token| is_token_expired(token, chrono::Local::now()))
            {
                println!("OAuth token is not found. Please authenticate again.");
                if let Some(OAuthToken {
                    refresh_token: Some(refresh_token),
                    ..
                }) = token.clone()
                {
                    let _ = update_token(token.clone().unwrap().id, refresh_token).await;
                    let token = repository::oauth_token::find_latest().unwrap();
                    update_events(token.unwrap()).await
                } else {
                    panic!("Refresh token is not found");
                }
            } else {
                update_events(token.unwrap()).await;
            }
        }
        Err(e) => {
            eprintln!("Error occurred when getting latest token: {:?}", e);
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

async fn update_events(token: OAuthToken) {
    let OAuthToken {
        id,
        access_token,
        refresh_token,
        ..
    } = token;

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
                oauth::to_oauth_on_browser();
            }
        },
        Err(e) => eprintln!("エラー: {:?}", e),
    }
}
