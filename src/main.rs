#![allow(unused_variables)]

mod command_line;
mod oauth;

use command_line as cmd;

/**
 * TODO
 * - コマンドラインを受け付けるようにする ✅
 * - token をグローバルで扱えるようにする  -> コマンドライン
 * - カレンダーを取得する
 */
#[tokio::main]
async fn main() {
    // oauth::to_oauth_on_browser();

    // oauth::run_redirect_server();

    cmd::wait_for_command();
}
