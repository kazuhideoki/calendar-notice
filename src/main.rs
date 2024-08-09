#![allow(unused_variables)]

mod oauth;

/**
 * TODO
 * - コマンドラインを受け付けるようにする
 * - token をグローバルで扱えるようにする -> コマンドライン
 * - カレンダーを取得する
 */
#[tokio::main]
async fn main() {
    oauth::to_oauth_on_browser();

    oauth::run_redirect_server();

    // メインタスクを終了させないようにする
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for ctrl+c event");
    println!("Shutting down");
}
