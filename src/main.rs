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
  TODO
  - イベントの正しい同期
    - 変更になったイベントを正しく更新できているか？
  - 通知
    - 通知設定をイベント同期と共に作成
    - 通知設定を参照して通知(notification_sec_from_start をもとに)
    - 通知設定の表示
    - 通知設定の更新 (オンオフ、notification_sec_from_start)
  - reporitory 整理
*/
#[tokio::main]
async fn main() {
    spawn_redirect_server();

    spawn_sync_calendar_cron();

    spawn_notification_cron();

    run_command_loop_async().await;
}
