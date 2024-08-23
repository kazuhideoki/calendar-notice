#![allow(unused_variables)]
use std::{process::Command, thread, time::Duration};

use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use filter_upcoming_events::filter_upcoming_events;

use crate::{
    db::establish_connection,
    google_calendar,
    models::OAuthToken,
    schema::oauth_tokens::{self},
};
mod filter_upcoming_events;

const NOTIFICATION_INTERVAL_SEC: u16 = 60 * 10;

pub fn run_notification_cron_thread() {
    tokio::spawn(async {
        loop {
            let mut conn = establish_connection();
            let latest_token = oauth_tokens::table
                .order(oauth_tokens::created_at.desc())
                .first::<OAuthToken>(&mut conn)
                .optional()
                .expect("Error loading oauth token");
            match latest_token {
                Some(oauth_token) => {
                    let events = google_calendar::list_events(oauth_token.access_token).await.expect(
                    "Failed to get events from Google Calendar. Please check your network connection.",
                );

                    let upcoming_events = filter_upcoming_events(events).await;
                    if upcoming_events.len() > 0 {
                        let event_names = upcoming_events
                            .iter()
                            .map(|event| event.summary.clone())
                            .collect::<Vec<String>>()
                            .join(", ");
                        println!("Upcoming events: {}", event_names);

                        notify();
                    }
                }
                None => {
                    // TODO token 切れチェック
                    // TODO Unauthorized が返ってきたら再認証する
                    println!("OAuth token is not found. Please authenticate again.");
                    return;
                }
            }

            thread::sleep(Duration::from_secs(NOTIFICATION_INTERVAL_SEC.into()));
        }
    });
}

fn notify() {
    // カレンダーAPPを開く
    let apple_script = format!(
        r#"
    tell application "Calendar"
        activate
    end tell
"#
    );
    Command::new("osascript")
        .arg("-e")
        .arg(apple_script)
        .output()
        .expect("Failed to execute AppleScript");

    // ビープ音を鳴らす
    Command::new("osascript")
        .arg("-e")
        .arg("beep")
        .output()
        .expect("Failed to execute AppleScript");
}
