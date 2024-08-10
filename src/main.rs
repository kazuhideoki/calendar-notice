#![allow(unused_variables)]

mod command_line;
mod env;
mod google_calendar;
mod oauth;
mod repository;

use command_line as cmd;
use repository::OAuthResponse;

/**
 * TODO
 * - トークンの有効期限が切れたら refresh する
 */
#[tokio::main]
async fn main() {
    if OAuthResponse::from_file().is_none() {
        oauth::to_oauth_on_browser();
    }

    oauth::run_redirect_server();

    cmd::wait_for_command().await;
}
