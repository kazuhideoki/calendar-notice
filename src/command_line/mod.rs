#![allow(unused_variables)]
use clap::Parser;
use std::io::{self, BufRead};

use crate::{
    google_calendar::sync_events,
    oauth::{self, is_token_expired::is_token_expired, refresh_and_save_token},
    repository::{
        self,
        models::{EventFindMany, OAuthToken},
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
                    "sync" => handle_command_sync().await,
                    "list" => handle_list_notification().await,
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

async fn handle_command_sync() {
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
                    sync_events(token.unwrap()).await.unwrap_or_else(|e| {
                        eprintln!(
                            "Failed to sync events in handle_command_sync with new token: {:?}",
                            e
                        )
                    });
                } else {
                    println!("Refresh token is not found");
                    oauth::to_oauth_on_browser();
                }
            } else {
                sync_events(token.unwrap()).await.unwrap_or_else(|e| {
                    eprintln!("Failed to sync events in handle_command_sync: {:?}", e)
                });
            }
        }
        Err(e) => {
            eprintln!("Error occurred when getting latest token: {:?}", e);
        }
    }
}

async fn handle_list_notification() {
    let now = chrono::Local::now();
    let events = repository::event::find_many(EventFindMany {
        from: Some(now.to_rfc3339()),
        to: Some((now + chrono::Duration::days(2)).to_rfc3339()),
        ..Default::default()
    })
    .unwrap();

    println!("");
    println!("通知設定");
    for (event, notification) in events {
        let notified = if notification.enabled {
            format!("{}分前通知", notification.notification_sec_from_start / 60)
        } else {
            "通知なし".to_string()
        };
        println!(
            "{}: {}開始 {}",
            event.summary,
            chrono::DateTime::parse_from_rfc3339(event.start_datetime.as_str())
                .expect("Error occurred when parsing start time, handle_list_notification")
                .format("%m-%d %H:%M"),
            notified
        );
    }
}
