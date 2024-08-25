#![allow(unused_variables)]
use std::process::Command;

use filter_upcoming_events::filter_upcoming_events;

use crate::repository::{self};
mod filter_upcoming_events;

const NOTIFICATION_INTERVAL_SEC: u16 = 60 * 10;

pub fn run_notification_cron_thread() {
    tokio::spawn(async {
        loop {
            let events = repository::event::find_many(repository::models::EventFindMany {
                from: chrono::Local::now().to_rfc3339(),
                to: (chrono::Local::now() + chrono::Duration::days(1)).to_rfc3339(),
            });

            match events {
                Ok(events) => {
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
                Err(e) => println!("Failed to get events: {:?}", e),
            }
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
