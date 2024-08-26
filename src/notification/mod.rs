#![allow(unused_variables)]
use std::process::Command;

use filter_upcoming_events::filter_upcoming_events;

use crate::repository::{self, models::EventFindMany};
mod filter_upcoming_events;

const NOTIFICATION_INTERVAL_SEC: u16 = 60;

pub fn spawn_notification_cron() {
    tokio::spawn(async {
        loop {
            let now = chrono::Local::now();
            let events = repository::event::find_many(EventFindMany {
                from: Some(now.to_rfc3339()),
                to: Some((now + chrono::Duration::days(2)).to_rfc3339()),
                ..Default::default()
            });

            match events {
                Ok(events) => {
                    let upcoming_events = filter_upcoming_events(events);
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
                Err(e) => println!("Failed to get events: {:?}", e),
            }

            tokio::time::sleep(tokio::time::Duration::from_secs(
                NOTIFICATION_INTERVAL_SEC.into(),
            ))
            .await;
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
