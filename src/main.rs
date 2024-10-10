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
TUI
- ログ表示
- 予定間の時間の表現
- 次の日の予定を表示
- 詳細など情報表示

improvement..
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
