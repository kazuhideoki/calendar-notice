#![allow(unused_variables)]
use clap::Parser;
use std::io::{self, BufRead, Error};

use crate::{
    google_calendar,
    oauth::{self, is_token_expired::is_token_expired, refresh_and_save_token, OAuthResponse},
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

pub async fn run_command_loop_async() {
    let mut stdin_lines = io::stdin().lock().lines();

    while let Some(line) = stdin_lines.next() {
        match line {
            Ok(input) => {
                if input.trim().is_empty() {
                    break;
                }
                println!("Input: {}", input);

                match input.as_str() {
                    "token" => handle_command_token(),
                    "refresh" => handle_command_refresh().await,
                    "event" => handle_command_event().await,
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
            refresh_and_save_token(id, refresh_token).await;
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
                    let _ = refresh_and_save_token(token.clone().unwrap().id, refresh_token).await;
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
                refresh_and_save_token(id, refresh_token).await;
            }
            None => {
                println!("Refresh token is not found");
                oauth::to_oauth_on_browser();
            }
        },
        Err(e) => eprintln!("エラー: {:?}", e),
    }
}
