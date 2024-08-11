#![allow(unused_variables)]
use std::{process::Command, thread, time::Duration};

use crate::google_calendar;

const NOTIFICATION_INTERVAL: u16 = 60 * 10;

pub fn run_notification_cron_thread() {
    tokio::spawn(async {
        loop {
            // 実行
            let upcoming_events = get_upcoming_events().await;
            if upcoming_events.len() > 0 {
                let event_names = upcoming_events
                    .iter()
                    .map(|event| event.summary.clone())
                    .collect::<Vec<String>>()
                    .join(", ");
                println!("Upcoming events: {}", event_names);

                show_native_calendar_app();
            }

            thread::sleep(Duration::from_secs(NOTIFICATION_INTERVAL.into()));
        }
    });
}

async fn get_upcoming_events() -> Vec<google_calendar::Event> {
    let events = google_calendar::list_events()
        .await
        .expect("Failed to get events from Google Calendar. Please check your network connection.");

    let now = chrono::Local::now();
    let upcoming_events: Vec<google_calendar::Event> = events
        .items
        .into_iter()
        .filter(|item| {
            let start_time = item.start.date_time.as_ref().unwrap();
            let start_time = chrono::DateTime::parse_from_rfc3339(start_time).unwrap();
            now.signed_duration_since(start_time).num_seconds() < NOTIFICATION_INTERVAL.into()
        })
        .collect();

    upcoming_events
}

fn show_native_calendar_app() {
    let apple_script = format!(
        r#"
    tell application "Calendar"
        activate
    end tell
"#
    );

    let output = Command::new("osascript")
        .arg("-e")
        .arg(apple_script)
        .output()
        .expect("Failed to execute AppleScript");
}
