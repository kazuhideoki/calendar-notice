#![allow(unused_variables)]
use clap::Parser;
use diesel::{query_dsl::methods::OrderDsl, ExpressionMethods, OptionalExtension, RunQueryDsl};
use std::io::{self, BufRead};

use crate::{
    db::establish_connection,
    google_calendar,
    oauth::{request_access_token_by_refresh_token, OAuthResponse},
    repository::models::OAuthToken,
    schema::oauth_tokens,
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
                    "token" => println!("{:?}", OAuthResponse::from_db()),
                    "event" => {
                        let mut conn = establish_connection();
                        let latest_token = oauth_tokens::table
                            .order(oauth_tokens::created_at.desc())
                            .first::<OAuthToken>(&mut conn)
                            .optional()
                            .expect("Error loading oauth token");
                        if latest_token.is_none() {
                            // TODO token 切れチェック
                            // TODO Unauthorized が返ってきたら再認証する
                            println!("OAuth token is not found. Please authenticate again.");
                            continue;
                        }

                        let result =
                            google_calendar::list_events(latest_token.unwrap().access_token).await;

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
                                let OAuthResponse { refresh_token, .. } = OAuthResponse::from_db()
                                    .expect("Failed to get OAuthResponse from file");
                                if refresh_token.is_none() {
                                    println!("Refresh token is not found");
                                    continue;
                                }
                                let result =
                                    request_access_token_by_refresh_token(refresh_token.unwrap())
                                        .await;

                                let oauth_token_response = OAuthResponse::parse(&result.unwrap());
                                match oauth_token_response {
                                    Ok(response) => {
                                        response
                                            .save_to_db()
                                            .expect("Failed to save OAuthResponse to file");
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
