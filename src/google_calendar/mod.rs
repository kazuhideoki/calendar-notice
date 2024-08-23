#![allow(unused_variables)]
use chrono::Days;
use reqwest::header::{HeaderMap, HeaderValue, InvalidHeaderValue};

use serde::{Deserialize, Serialize};

use crate::oauth::OAuthResponse;

#[derive(Debug, Serialize, Deserialize)]
pub struct CalendarEvents {
    pub kind: String,
    pub etag: String,
    pub summary: String,
    pub description: Option<String>,
    pub updated: String,
    pub time_zone: Option<String>,
    pub access_role: Option<String>,
    pub default_reminders: Option<Vec<Reminder>>,
    pub next_page_token: Option<String>,
    pub items: Vec<Event>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Reminder {
    method: String,
    minutes: i32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EventStatus {
    Confirmed,
    Tentative,
    Cancelled,
    #[serde(other)]
    Unknown,
}

/**
 * TODO model と融合？
 * ※基本的に利用したい値(DB)とGoogle Calendar API の値が一致しているはず。
 * その場合、レスポンスからの変換方法を別途定義する必要があるのか？
 *
 * TODO 不要な値を削る
 */
#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
    pub kind: String,
    pub etag: String,
    pub id: String,
    pub status: Option<EventStatus>,
    pub html_link: Option<String>,
    pub created: String,
    pub updated: String,
    pub summary: String,
    pub description: Option<String>,
    pub creator: EventPerson,
    pub organizer: EventPerson,
    pub start: EventDateTime,
    pub end: EventDateTime,
    pub recurring_event_id: Option<String>,
    pub original_start_time: Option<EventDateTime>,
    pub transparency: Option<String>,
    pub visibility: Option<String>,
    pub i_cal_uid: Option<String>,
    pub sequence: i32,
    pub attendees: Option<Vec<Attendee>>,
    pub reminders: Option<Reminders>,
    pub event_type: Option<String>,
    pub hangout_link: Option<String>,
    pub conference_data: Option<ConferenceData>,
}
impl Default for Event {
    fn default() -> Self {
        Self {
            kind: String::new(),
            etag: String::new(),
            id: String::new(),
            status: Some(EventStatus::Tentative),
            html_link: None,
            created: String::new(),
            updated: String::new(),
            summary: String::new(),
            description: None,
            creator: EventPerson::default(),
            organizer: EventPerson::default(),
            start: EventDateTime::default(),
            end: EventDateTime::default(),
            recurring_event_id: None,
            original_start_time: None,
            transparency: None,
            visibility: None,
            i_cal_uid: None,
            sequence: 0,
            attendees: None,
            reminders: None,
            event_type: None,
            hangout_link: None,
            conference_data: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventPerson {
    email: String,
    display_name: Option<String>,
    self_: Option<bool>,
}
impl Default for EventPerson {
    fn default() -> Self {
        Self {
            email: String::new(),
            display_name: None,
            self_: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventDateTime {
    #[serde(rename = "dateTime", default)]
    pub date_time: Option<String>,
    #[serde(default)]
    pub date: Option<String>,
    #[serde(rename = "timeZone")]
    pub time_zone: Option<String>,
}
impl Default for EventDateTime {
    fn default() -> Self {
        Self {
            date_time: None,
            date: None,
            time_zone: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Attendee {
    email: String,
    display_name: Option<String>,
    organizer: Option<bool>,
    self_: Option<bool>,
    response_status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Reminders {
    use_default: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConferenceData {
    entry_points: Vec<EntryPoint>,
    conference_solution: ConferenceSolution,
    conference_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EntryPoint {
    entry_point_type: String,
    uri: String,
    label: Option<String>,
    pin: Option<String>,
    region_code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConferenceSolution {
    key: ConferenceSolutionKey,
    name: String,
    icon_uri: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConferenceSolutionKey {
    #[serde(rename = "type")]
    type_: String,
}

#[derive(Debug)]
pub enum Error {
    Reqwest(reqwest::Error),
    Unauthorized,
    Parse(String),
}
impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        Error::Reqwest(e)
    }
}
impl From<InvalidHeaderValue> for Error {
    fn from(e: InvalidHeaderValue) -> Self {
        Error::Parse(e.to_string())
    }
}

pub async fn list_events(access_token: String) -> Result<CalendarEvents, Error> {
    let url = format!(
        "https://www.googleapis.com/calendar/v3/calendars/{}/events",
        "primary"
    );

    let mut headers = HeaderMap::new();
    headers.insert(
        "AUTHORIZATION",
        HeaderValue::from_str(&format!("Bearer {}", access_token))?,
    );

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .headers(headers)
        .query(&[
            ("maxResults", "10"),
            ("orderBy", "startTime"),
            ("singleEvents", "true"),
            // ("timeMin", "2024-01-01T00:00:00Z"), // 開始日時を指定
            // 今日から 3日間
            ("timeMin", &chrono::Utc::now().to_rfc3339()),
            (
                "timeMax",
                &chrono::Utc::now()
                    .checked_add_days(Days::new(3))
                    .expect("Failed to add days")
                    .to_rfc3339(),
            ),
        ])
        .send()
        .await?;
    if response.status() == reqwest::StatusCode::UNAUTHORIZED {
        println!("Unauthorized when requesting list events");
        return Err(Error::Unauthorized);
    }

    let text = response.text().await?;
    let events: CalendarEvents =
        serde_json::from_str(&text).map_err(|e| Error::Parse(e.to_string()))?;
    Ok(events)
}
