#![allow(unused_variables)]
use std::{process::Command, thread, time::Duration};

const NOTIFICATION_CRON_INTERVAL: u16 = 60 * 10;

pub fn run_notification_cron_thread() {
    thread::spawn(|| {
        loop {
            // 実行
            // show_native_calendar_app();

            thread::sleep(Duration::from_secs(NOTIFICATION_CRON_INTERVAL.into()));
        }
    });
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
