#![allow(unused_variables)]

mod command_line;
mod env;
mod google_calendar;
mod models;
mod notification;
mod oauth;
pub mod schema;

use command_line as cmd;
use diesel::connection::SimpleConnection;
use env::Env;
use models::{Event, Notification};
use notification::run_notification_cron_thread;
use oauth::OAuthResponse;

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use schema::{events, notifications};

pub fn establish_connection() -> SqliteConnection {
    // let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let database_url = Env::new().database_url;
    let mut conn = SqliteConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url));

    // conn.execute("PRAGMA foreign_keys = ON")
    conn.batch_execute("PRAGMA foreign_keys = ON")
        .expect("Failed to enable foreign keys");

    conn
}

/**
  TODO
  - ORM diesel で DB にアクセスする
  - DB スキーマ
    - OAuthResponse
    - イベント (summary, description, status, id, start, end )
      - 通知設定とのリレーション
    - 通知設定 (on/off);
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

    if OAuthResponse::from_file().is_none() {
        oauth::to_oauth_on_browser();
    }

    oauth::run_redirect_server();

    run_notification_cron_thread();

    cmd::wait_for_command().await;
}
