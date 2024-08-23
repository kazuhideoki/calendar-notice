#![allow(unused_variables)]

mod command_line;
mod db;
mod env;
mod google_calendar;
mod models;
mod notification;
mod oauth;
mod schema;

use command_line as cmd;
use db::establish_connection;
use models::{Event, Notification};
use notification::run_notification_cron_thread;
use oauth::OAuthResponse;

use diesel::prelude::*;
use schema::{events, notifications};

/**
  TODO
  - oauth_tokens で insert, select できるように✅
  - db module 整理
    -
  - カレンダー
    - 保存 と OAuth をいい感じに
    - 表示は DB 参照
*/
#[tokio::main]
async fn main() {
    let mut conn = establish_connection();
    let results: Vec<(Event, Notification)> = events::table
        .inner_join(notifications::table)
        .select((Event::as_select(), Notification::as_select()))
        .load(&mut conn)
        .expect("Error loading events");
    println! {"{:?}", results};

    if OAuthResponse::from_db().is_none() {
        oauth::to_oauth_on_browser();
    }

    oauth::run_redirect_server();

    run_notification_cron_thread();

    cmd::wait_for_command().await;
}
