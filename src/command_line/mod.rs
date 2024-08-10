#![allow(unused_variables)]
use clap::Parser;
use reqwest::StatusCode;
use std::io::{self, BufRead};

use crate::{
    google_calendar, oauth::request_access_token_by_refresh_token, repository::OAuthResponse,
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
                    "token" => println!("{:?}", OAuthResponse::from_file()),
                    "event" => {
                        let result = google_calendar::list_events().await;

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
                            Err(google_calendar::Error::Unauthorized) => {
                                let OAuthResponse { refresh_token, .. } =
                                    OAuthResponse::from_file()
                                        .expect("Failed to get OAuthResponse from file");
                                if refresh_token.is_none() {
                                    println!("Refresh token is not found");
                                    continue;
                                }
                                let result =
                                    request_access_token_by_refresh_token(refresh_token.unwrap())
                                        .await;

                                match OAuthResponse::parse_and_save(&result.unwrap()) {
                                    Ok(response) => {
                                        println!("Success to get token! by refresh token");
                                    }
                                    Err(e) => {
                                        println!("Recv error: {:?}", e.to_string());
                                    }
                                }
                            }
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
