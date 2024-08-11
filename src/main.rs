#![allow(unused_variables)]

mod command_line;
mod env;
mod google_calendar;
mod oauth;

use command_line as cmd;
use oauth::OAuthResponse;

/**
* TODO
* - parse_and_save
*   - 読み込みと save は分ける
*   - 保存し直すのは access_token と expires_in だけ
*/
#[tokio::main]
async fn main() {
    if OAuthResponse::from_file().is_none() {
        oauth::to_oauth_on_browser();
    }

    oauth::run_redirect_server();

    cmd::wait_for_command().await;
}
