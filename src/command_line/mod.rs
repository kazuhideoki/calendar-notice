#![allow(unused_variables)]
use clap::Parser;
use std::io::{self, BufRead};

use crate::repository::oauth_state;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None, disable_help_flag = true)]
struct Args {
    #[arg(short, long)]
    test: Option<String>,
}

pub fn wait_for_command() {
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
                    _ => {}
                }
            }
            Err(e) => eprintln!("エラー: {}", e),
        }
    }
}
