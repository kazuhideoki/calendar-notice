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
  機能追加
  - 通知
    - 通知設定の更新 オンオフ
    - 筋の良い通知方法

  Improve
  - 各ファイルの エラーハンドリング。必要最低限のエラー定義(なければ Stringで)。呼び出し元でログor分岐
  - module整理 + テスト追加
  - 一部 env に逃がすか？
  - Clippy(リンター) の導入
  - doc つくる
*/
#[tokio::main]
async fn main() {
    spawn_redirect_server();

    spawn_sync_calendar_cron();

    spawn_notification_cron();

    run_command_loop_async().await;
}
