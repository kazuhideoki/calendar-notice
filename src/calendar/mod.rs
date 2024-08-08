#![allow(unused_variables)]
use regex::Regex;
use std::{num::ParseIntError, process::Command};

use time::{Date, Month, PrimitiveDateTime, Time};

#[derive(Debug)]
pub struct Event {
    pub id: String,
    pub title: String,
    pub start_time: PrimitiveDateTime,
}

#[derive(Debug)]
pub enum Error {
    ExecutionError(String),
    ParseError(String),
}

impl From<time::error::ComponentRange> for Error {
    fn from(err: time::error::ComponentRange) -> Self {
        Error::ParseError(err.to_string())
    }
}
impl From<regex::Error> for Error {
    fn from(err: regex::Error) -> Self {
        Error::ParseError(err.to_string())
    }
}
impl From<ParseIntError> for Error {
    fn from(err: ParseIntError) -> Self {
        Error::ParseError(err.to_string())
    }
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
            let result = String::from_utf8_lossy(&output.stdout);

            let events: Vec<Event> = result
                .trim()
                .split(EVENT_SEPARATOR)
                .filter(|s| !s.is_empty()) // 最後の要素の空の文字列をフィルタリング。
                .map(|e| Event::parse_event(e).unwrap())
                .collect();

            Ok(events)
        } else {
            let error = String::from_utf8_lossy(&output.stderr);

            Err(Error::ExecutionError(error.to_string()))
        }
    }

    fn parse_event(event_str: &str) -> Result<Self, Error> {
        let parts: Vec<&str> = event_str.split(PARTS_SEPARATOR).collect();
        // TODO 取得項目とともに型安全にする
        if parts.len() != 3 {
            return Err(Error::ParseError(format!(
                "Expected 3 parts, found {}",
                parts.len()
            )));
        }

        Ok(Event {
            id: parts[0].to_string(),
            title: parts[1].to_string(),
            start_time: Self::parse_japanese_date(&parts[2].to_string())?,
        })
    }

    fn parse_japanese_date(date_string: &str) -> Result<PrimitiveDateTime, Error> {
        let re = Regex::new(
            r"(\d{4})年(\d{1,2})月(\d{1,2})日 (?:月|火|水|木|金|土|日)曜日 (\d{2}):(\d{2}):(\d{2})",
        )?;

        if let Some(caps) = re.captures(date_string) {
            let year: i32 = caps[1].parse()?;
            let month: u8 = caps[2].parse()?;
            let day: u8 = caps[3].parse()?;
            let hour: u8 = caps[4].parse()?;
            let minute: u8 = caps[5].parse()?;
            let second: u8 = caps[6].parse()?;

            // TODO エラーハンドリング
            let date = Date::from_calendar_date(year, Month::try_from(month)?, day)?;
            let time = Time::from_hms(hour, minute, second)?;

            Ok(PrimitiveDateTime::new(date, time))
        } else {
            Err(Error::ParseError("Date format is invalid".to_string()))
        }
    }
}
