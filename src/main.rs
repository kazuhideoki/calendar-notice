mod command_line;
mod env;
mod google_calendar;
mod notification;
mod oauth;
mod repository;
mod schema;
mod tui;

use google_calendar::spawn_sync_calendar_cron;
use notification::spawn_notification_cron;
use oauth::spawn_redirect_server;
use tui::show_tui;

/**
functoin..
- TUI での操作
- 「参加する/辞退する」変更
  - 辞退したら通知もなしにする
- 初期起動時のUI調整

db..
- テーブル
  - oauth_tokens を先に持ってくれば管理楽そう

improvement..
- summary がない時もある タイトルなしの時
- 各ファイルの エラーハンドリング。必要最低限のエラー定義(なければ Stringで)。呼び出し元でログor分岐
- module整理 + テスト追加
- 接続がないときに panic になってしまうことへの対応
*/
#[tokio::main]
async fn main() {
    spawn_redirect_server();

    spawn_notification_cron();

    spawn_sync_calendar_cron();

    show_tui();
}
