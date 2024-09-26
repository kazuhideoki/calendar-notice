use std::{io, process::Command};

use filter_upcoming_events::filter_upcoming_events;

use crate::repository::{
    self,
    models::{Event, EventFindMany, NotificationUpdate},
};
mod filter_upcoming_events;

const NOTIFICATION_INTERVAL_SEC: u16 = 60;
pub const NOTIFICATION_PERIOD_DAYS: i64 = 7;

pub fn spawn_notification_cron() {
    tokio::spawn(async {
        loop {
            let now = chrono::Local::now();
            let events = repository::event::find_many(EventFindMany {
                from: Some(now.to_rfc3339()),
                to: Some((now + chrono::Duration::days(NOTIFICATION_PERIOD_DAYS)).to_rfc3339()),
                ..Default::default()
            });

            match events {
                Ok(events) => {
                    let upcoming_events = filter_upcoming_events(events);
                    for event in upcoming_events {
                        notify(event.clone()).unwrap_or_else(|e| {
                            println!("Failed to notify event {}: {}", event.id, e)
                        });

                        repository::notification::update(
                            event.id.clone(),
                            NotificationUpdate {
                                enabled: Some(false),
                                ..Default::default()
                            },
                        )
                        .unwrap_or_else(|e| println!("Failed to update notification: {}", e));
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

fn notify(event: Event) -> Result<(), io::Error> {
    // ビープ音を鳴らす
    Command::new("osascript").arg("-e").arg("beep").output()?;

    // イベントの内容をダイアログで表示
    let join = "会議に参加";
    let cancel = "キャンセル";
    let dialog_script = format!(
        r#"
                tell application "System Events"
                    set theResponse to display dialog "{}" with title "{}" buttons {{"{}","{}"}} default button "{}"
                    set theButton to button returned of theResponse
                    return theButton
                end tell
                "#,
        "ほげ~", "タイトル", cancel, join, join
    );
    let button_result = Command::new("osascript")
        .arg("-e")
        .arg(dialog_script)
        .output()?;

    // キャンセルされた場合は何もしない
    if button_result.stdout == b"" {
        println!("Canceled joining the meeting");
        return Ok(());
    }

    // Zoom を開く なければ Meet を開く
    match event {
        Event {
            zoom_link: Some(link),
            ..
        } => {
            Command::new("open")
                .arg("-a")
                .arg("zoom.us")
                .arg(link)
                .output()?;
        }
        Event {
            hangout_link: Some(link),
            ..
        } => {
            let script = format!(
                r#"
            tell application "Brave Browser"
                activate
                open location "{}"
            end tell
            "#,
                link
            );
            Command::new("osascript").arg("-e").arg(script).output()?;
        }
        _ => {
            println!("No link for meeting found")
        }
    }

    Ok(())
}
