#![allow(unused_variables)]
use std::process::Command;

#[derive(Debug)]
pub struct Event {
    pub id: String,
    pub title: String,
    pub start_time: String,
}

#[derive(Debug)]
pub enum Error {
    ExecutionError,
    ParseError(String),
}

const EVENT_SEPARATOR: &str = "###";
const PARTS_SEPARATOR: &str = "|||";

impl Event {
    pub fn get_events(calendar_name: &str) -> Result<Vec<Event>, Error> {
        let apple_script = format!(
            r#"
set tomorrow to current date + 60 * 60 * 24
set EVENT_SEPARATOR to "{}"
set PARTS_SEPARATOR to "{}"

tell application "Calendar"
    tell calendar "{}"
        set curr to every event whose start date is greater than or equal to current date ¬
            and start date is less than or equal to tomorrow

        set eventDetails to ""
        repeat with anEvent in curr
            set eventDetails to eventDetails & (id of anEvent) & PARTS_SEPARATOR & ¬
                                               (summary of anEvent) & PARTS_SEPARATOR & ¬
                                               ((start date of anEvent) as string)
            if anEvent is not last item of curr then
                set eventDetails to eventDetails & EVENT_SEPARATOR
            end if
        end repeat
    end tell
end tell

return eventDetails
            "#,
            EVENT_SEPARATOR, PARTS_SEPARATOR, calendar_name
        );

        let output = Command::new("osascript")
            .arg("-e")
            .arg(apple_script)
            .output()
            .expect("Failed to execute AppleScript");

        if output.status.success() {
            println!("Success");
            let result = String::from_utf8_lossy(&output.stdout);
            println!("Today's events: {}", result.trim());

            let events: Vec<Event> = result
                .trim()
                .split(EVENT_SEPARATOR)
                .filter(|s| !s.is_empty()) // 最後の要素の空の文字列をフィルタリング。
                .map(|e| Event::parse_event(e).unwrap())
                .collect();

            events.iter().for_each(|event| {
                println!("{:?}", event);
            });

            Ok(events)
        } else {
            println!("Failed");
            let error = String::from_utf8_lossy(&output.stderr);
            eprintln!("Error: {}", error);

            Err(Error::ExecutionError)
        }
    }

    fn parse_event(event_str: &str) -> Result<Self, Error> {
        let parts: Vec<&str> = event_str.split(PARTS_SEPARATOR).collect();
        if parts.len() != 3 {
            return Err(Error::ParseError(format!(
                "Expected 3 parts, found {}",
                parts.len()
            )));
        }

        Ok(Event {
            id: parts[0].to_string(),
            title: parts[1].to_string(),
            start_time: parts[2].to_string(),
        })
    }
}
