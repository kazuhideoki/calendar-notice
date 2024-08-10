#![allow(unused_variables)]
use clap::Parser;
use std::io::{self, BufRead};

use crate::{google_calendar, repository::oauth_state};

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
                    "token" => println!("{:?}", oauth_state().lock().unwrap()),
                    "event" => {
                        let events = google_calendar::list_events()
                            .await
                            .unwrap_or_else(|e| panic!("Failed to list events :{:?}", e));

                        println!(
                            "{:?}",
                            events
                                .items
                                .iter()
                                .map(|item| &item.summary)
                                .collect::<Vec<&String>>()
                        );
                    }
                    _ => {}
                }
            }
            Err(e) => eprintln!("エラー: {}", e),
        }
    }
}
