#![allow(unused_variables)]

mod command_line;
mod env;
mod google_calendar;
mod notification;
mod oauth;
mod repository;
mod schema;

use command_line as cmd;
use notification::run_notification_cron_thread;
use oauth::is_token_expired;

/**
  TODO
  - db module 整理
    - (google accessToken/refresh の取得 を repository に移動?)
    - is_token_expired のテスト
    - 従来の他モジュールからの責務分離

  - カレンダー
    - db に保存
    - 表示は DB 参照
*/
#[tokio::main]
async fn main() {
    // join 時のサンプル
    // let results: Vec<(Event, Notification)> = events::table
    //     .inner_join(notifications::table)
    //     .select((Event::as_select(), Notification::as_select()))
    //     .load(&mut conn)
    //     .expect("Error loading events");
    // println! {"{:?}", results};n

    let latest_token = repository::oauth_token::find_latest().unwrap();
    if latest_token
        .as_ref()
        .map_or(true, |token| is_token_expired(token))
    {
        oauth::to_oauth_on_browser();
    }

    oauth::run_redirect_server();

    run_notification_cron_thread();

    cmd::wait_for_command().await;
}
