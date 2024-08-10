use chrono::Days;
use reqwest::header::{HeaderMap, HeaderValue, InvalidHeaderValue};

use crate::repository::oauth_state;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CalendarEvents {
    kind: String,
    etag: String,
    summary: String,
    description: Option<String>,
    updated: String,
    time_zone: Option<String>,
    access_role: Option<String>,
    default_reminders: Option<Vec<Reminder>>,
    pub next_page_token: Option<String>,
    pub items: Vec<Event>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Reminder {
    method: String,
    minutes: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
    kind: String,
    etag: String,
    id: String,
    status: String,
    html_link: Option<String>,
    created: String,
    updated: String,
    pub summary: String,
    description: Option<String>,
    creator: EventPerson,
    organizer: EventPerson,
    start: EventDateTime,
    end: EventDateTime,
    recurring_event_id: Option<String>,
    original_start_time: Option<EventDateTime>,
    transparency: Option<String>,
    visibility: Option<String>,
    i_cal_uid: Option<String>,
    sequence: i32,
    attendees: Option<Vec<Attendee>>,
    reminders: Option<Reminders>,
    event_type: Option<String>,
    hangout_link: Option<String>,
    conference_data: Option<ConferenceData>,
}

#[derive(Debug, Serialize, Deserialize)]
struct EventPerson {
    email: String,
    display_name: Option<String>,
    self_: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct EventDateTime {
    #[serde(rename = "dateTime", default)]
    date_time: Option<String>,
    #[serde(default)]
    date: Option<String>,
    #[serde(rename = "timeZone")]
    time_zone: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Attendee {
    email: String,
    display_name: Option<String>,
    organizer: Option<bool>,
    self_: Option<bool>,
    response_status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Reminders {
    use_default: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ConferenceData {
    entry_points: Vec<EntryPoint>,
    conference_solution: ConferenceSolution,
    conference_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct EntryPoint {
    entry_point_type: String,
    uri: String,
    label: Option<String>,
    pin: Option<String>,
    region_code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ConferenceSolution {
    key: ConferenceSolutionKey,
    name: String,
    icon_uri: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ConferenceSolutionKey {
    #[serde(rename = "type")]
    type_: String,
}

#[derive(Debug)]
pub enum Error {
    Reqwest(reqwest::Error),
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

pub async fn list_events() -> Result<CalendarEvents, Error> {
    let url = format!(
        "https://www.googleapis.com/calendar/v3/calendars/{}/events",
        "primary"
    );

    let mut headers = HeaderMap::new();
    headers.insert(
        "AUTHORIZATION",
        HeaderValue::from_str(&format!(
            "Bearer {}",
            oauth_state().lock().unwrap().as_ref().unwrap().access_token
        ))?,
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

    let text = response.text().await?;
    let events: CalendarEvents =
        serde_json::from_str(&text).map_err(|e| Error::Parse(e.to_string()))?;
    Ok(events)
}
