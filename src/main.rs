#![allow(unused_variables)]

mod command_line;
mod env;
mod google_calendar;
mod notification;
mod oauth;

use command_line as cmd;
use notification::run_notification_cron_thread;
use oauth::OAuthResponse;

/**
 TODO
 - cron で定期実行するもの作る
 - 書くイベントの 10分前を過ぎたら通知 -> カレンダーAppを表示
*/
#[tokio::main]
async fn main() {
    if OAuthResponse::from_file().is_none() {
        oauth::to_oauth_on_browser();
    }

    oauth::run_redirect_server();

    run_notification_cron_thread();

    cmd::wait_for_command().await;
}
