use clap::Parser;
use std::io::{self, BufRead};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None, disable_help_flag = true)]
struct Args {
    #[arg(short, long)]
    connection: Option<String>,
    #[arg(short, long)]
    help: bool,
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
            }
            Err(e) => eprintln!("エラー: {}", e),
        }
    }
}
