#![allow(unused_variables)]

mod command_line;
mod env;
mod google_calendar;
mod notification;
mod oauth;
mod repository;
mod schema;

use command_line as cmd;
use google_calendar::run_sync_calendar_cron_thread;
use notification::run_notification_cron_thread;
use oauth::is_token_expired::is_token_expired;

/**
  TODO
  - カレンダーを db に通知設定とともに保存
    -
  - 通知
    - 通知設定を参照して通知
    - 通知設定の表示
  - reporitory 整理
*/
#[tokio::main]
async fn main() {
    oauth::run_redirect_server();

    let latest_token = repository::oauth_token::find_latest().unwrap();
    if latest_token
        .as_ref()
        .map_or(true, |token| is_token_expired(token, chrono::Local::now()))
    {
        oauth::to_oauth_on_browser();
    }

    run_sync_calendar_cron_thread();
    run_notification_cron_thread();

    cmd::wait_for_command().await;
}
