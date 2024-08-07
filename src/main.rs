use dotenvy::dotenv;
use std::{env, process::Command};

mod calendar;

struct Env {
    calendar_name: String,
}
impl Env {
    fn new() -> Self {
        dotenv().ok();

        Env {
            calendar_name: env::var("CALENDAR_NAME").expect("CALENDAR_NAME is not set"),
        }
    }
}

fn main() {
    let calendar_name = Env::new().calendar_name;

    let apple_script = format!(
        r#"
set tomorrow to current date + 60 * 60 * 24

tell application "Calendar"
    tell calendar "{}"
        set curr to every event whose start date is greater than or equal to current date ¬
            and start date is less than or equal to tomorrow

        set eventDetails to {{}}
        repeat with anEvent in curr
            set end of eventDetails to {{id:(id of anEvent) as string, title:(summary of anEvent) as string, start_time:(start date of anEvent) as string}}
        end repeat
    end tell
end tell

return eventDetails
        "#,
        calendar_name
    );

    let output = Command::new("osascript")
        .arg("-e")
        .arg(apple_script)
        .output()
        .expect("Failed to execute AppleScript");
    println!("{:?}", output);
    if output.status.success() {
        println!("Success");
        let result = String::from_utf8_lossy(&output.stdout);
        println!("Today's event: {}", result.trim());
    } else {
        println!("Failed");
        let error = String::from_utf8_lossy(&output.stderr);
        eprintln!("Error: {}", error);
    }
}
