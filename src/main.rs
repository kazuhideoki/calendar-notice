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
 -
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
