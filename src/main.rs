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
use oauth::OAuthResponse;

use diesel::prelude::*;
use repository::models::{Event, Notification};
use schema::{events, notifications};

/**
  TODO
  - db module 整理
    - models, repository の単位でまとめる✅
    - google accessToken/refresh の取得 を repository に移動?
    - 従来の他モジュールからの責務分離

  - カレンダー
    - db に保存
    - 表示は DB 参照
*/
#[tokio::main]
async fn main() {
    // let results: Vec<(Event, Notification)> = events::table
    //     .inner_join(notifications::table)
    //     .select((Event::as_select(), Notification::as_select()))
    //     .load(&mut conn)
    //     .expect("Error loading events");
    // println! {"{:?}", results};

    if OAuthResponse::from_db().is_none() {
        oauth::to_oauth_on_browser();
    }

    oauth::run_redirect_server();

    run_notification_cron_thread();

    cmd::wait_for_command().await;
}
