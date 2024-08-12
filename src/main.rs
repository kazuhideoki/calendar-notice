#![allow(unused_variables)]
#[macro_use]
pub extern crate diesel;

mod command_line;
mod env;
mod google_calendar;
mod notification;
mod oauth;

use command_line as cmd;
use diesel::prelude::*;
use notification::run_notification_cron_thread;
use oauth::OAuthResponse;

// infer_schema!("dotenv:DATABASE_URL");

/**
  TODO
  - docker で sqlite を使う
  - ORM diesel で DB にアクセスする
  - DB スキーマ
    - OAuthResponse
    - イベント (summary, description, status, id, start, end )
      - 通知設定とのリレーション
    - 通知設定 (on/off)
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
