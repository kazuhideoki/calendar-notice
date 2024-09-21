#![allow(unused_variables)]

mod command_line;
mod env;
mod google_calendar;
mod notification;
mod oauth;
mod repository;
mod schema;

use command_line::run_command_loop_async;
use google_calendar::spawn_sync_calendar_cron;
use notification::spawn_notification_cron;
use oauth::spawn_redirect_server;

/**
functoin..
- zoom リンクを取得、自動起動
- TUI での操作
- 「参加する/辞退する」変更
  - 辞退したら通知もなしにする

improvement..
- 各ファイルの エラーハンドリング。必要最低限のエラー定義(なければ Stringで)。呼び出し元でログor分岐
- module整理 + テスト追加
*/
#[tokio::main]
async fn main() {
    spawn_redirect_server();

    spawn_sync_calendar_cron();

    spawn_notification_cron();

    run_command_loop_async().await;
}
