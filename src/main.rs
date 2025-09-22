mod command_line;
mod env;
mod google_calendar;
mod notification;
mod oauth;
mod repository;
mod schema;
mod tui;

use clap::Parser;
use google_calendar::spawn_sync_calendar_cron;
use notification::spawn_notification_cron;
use oauth::spawn_redirect_server;
use tui::show_tui;

#[derive(Parser)]
struct Cli {
    #[arg(long)]
    daemon: bool,
}

/**
TUI
- TODO events で 7日分取得しておいて、選択した日付のイベントをフィルターして表示
- ログ表示
- 予定間の時間の表現
- 詳細など情報表示

improvement..
- 各ファイルの エラーハンドリング。必要最低限のエラー定義(なければ Stringで)。呼び出し元でログor分岐
- module整理 + テスト追加
- 接続がないときに panic になってしまうことへの対応
*/
#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // シャットダウンシグナル用のチャンネルを作成
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);

    // 各タスクにシャットダウンレシーバーを渡す
    let server_handle = spawn_redirect_server(shutdown_rx.clone());
    let notification_handle = spawn_notification_cron(shutdown_rx.clone());
    let sync_handle = spawn_sync_calendar_cron(shutdown_rx.clone());

    if !cli.daemon {
        show_tui(shutdown_tx.clone());
    } else {
        // Ctrl-C を待機
        tokio::signal::ctrl_c()
            .await
            .expect("failed to listen for ctrl_c");

        println!("Shutting down...");
    }

    // シャットダウンシグナルを送信
    let _ = shutdown_tx.send(true);

    // すべてのタスクの終了を待つ
    let _ = tokio::join!(server_handle, notification_handle, sync_handle);

    println!("Shutdown complete");
}
