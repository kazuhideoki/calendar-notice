#![allow(unused_variables)]

mod command_line;
mod env;
mod google_calendar;
mod oauth;
mod repository;

use command_line as cmd;
use repository::oauth_state;

/**
 * TODO
 * - カレンダーを取得する
 * - token を file に永続化
 * - file に token があればそれを使う
 * - トークンの有効期限が切れたら refresh する
 */
#[tokio::main]
async fn main() {
    if oauth_state().lock().unwrap().is_none() {
        oauth::to_oauth_on_browser();
    }

    oauth::run_redirect_server();

    cmd::wait_for_command().await;
}
