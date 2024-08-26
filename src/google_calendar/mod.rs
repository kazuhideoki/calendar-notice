#![allow(unused_variables)]
use std::time::Duration;
use std::{ops::Not, thread};

use chrono::{DateTime, Days, Utc};
use diesel::result;
use reqwest::header::{HeaderMap, HeaderValue, InvalidHeaderValue};

use crate::{
    google_calendar::{self},
    oauth::{self, is_token_expired::is_token_expired, refresh_and_save_token},
    repository::{
        self,
        models::{Event, EventFindMany, EventUpdate, Notification, OAuthToken},
        oauth_token,
    },
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GoogleCalendarParent {
    pub kind: String,
    pub etag: String,
    pub summary: String,
    pub description: Option<String>,
    pub updated: String,
    pub time_zone: Option<String>,
    pub access_role: Option<String>,
    pub default_reminders: Option<Vec<Reminder>>,
    pub next_page_token: Option<String>,
    pub items: Vec<GoogleCalendarEvent>,
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
impl EventStatus {
    pub fn to_string(&self) -> String {
        match self {
            EventStatus::Confirmed => "confirmed".to_string(),
            EventStatus::Tentative => "tentative".to_string(),
            EventStatus::Cancelled => "cancelled".to_string(),
            EventStatus::Unknown => "unknown".to_string(),
        }
    }
}

/**
 * TODO model と融合？
 * ※基本的に利用したい値(DB)とGoogle Calendar API の値が一致しているはず。
 * その場合、レスポンスからの変換方法を別途定義する必要があるのか？
 *
 * TODO 不要な値を削る
 */
#[derive(Debug, Serialize, Deserialize)]
pub struct GoogleCalendarEvent {
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
impl GoogleCalendarEvent {}
impl Default for GoogleCalendarEvent {
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

const SYNC_CALENDAR_INTERVAL_SEC: u16 = 60 * 10;
// TODO 扱う期間を const or env 化
const FROM_SUB_SEC: u16 = 60 * 10;
const TO_ADD_DAYS: u8 = 3;

pub fn spawn_sync_calendar_cron() {
    tokio::spawn(async {
        loop {
            let latest_token = repository::oauth_token::find_latest().unwrap_or_else(|e| {
                panic!(
                    "Failed to get latest token in run_sync_calendar_cron_thread: {:?}",
                    e
                )
            });

            match latest_token {
                Some(oauth_token) if is_token_expired(&oauth_token, chrono::Local::now()) => {
                    refresh_and_save_token(
                        oauth_token.id.clone(),
                        oauth_token.refresh_token.clone().unwrap(),
                    )
                    .await;

                    let new_token =
                        repository::oauth_token::find_latest().expect("new token must be found");
                    sync_events(oauth_token).await.unwrap_or_else(|e| {
                        println!(
                            "Failed to sync events in run_sync_calendar_cron_thread with new token: {:?}",
                            e
                        )
                    });
                }
                Some(oauth_token) => {
                    let _ = sync_events(oauth_token).await.unwrap_or_else(|e| {
                        println!(
                            "Failed to sync events in run_sync_calendar_cron_thread: {:?}",
                            e
                        )
                    });
                }
                None => {
                    println!("OAuth token is not found. Please authenticate ");
                    // TODO 認証完了まで、次のループで再度認証催促が発生するのを防ぐ？
                    oauth::to_oauth_on_browser();
                }
            }

            thread::sleep(Duration::from_secs(SYNC_CALENDAR_INTERVAL_SEC.into()));
        }
    });
}

pub async fn sync_events(oauth_token: OAuthToken) -> Result<(), Error> {
    let google_calendar_result =
        google_calendar::list_events(oauth_token.access_token.clone()).await;
    let google_calendar_parent =
        handle_google_calendar_event_result(google_calendar_result, oauth_token.clone()).await?;
    let _ = update_events(google_calendar_parent);

    Ok(())
}

pub async fn handle_google_calendar_event_result(
    google_calendar_result: Result<GoogleCalendarParent, Error>,
    oauth_token: OAuthToken,
) -> Result<GoogleCalendarParent, Error> {
    match google_calendar_result {
        Ok(google_calendar_parent) => Ok(google_calendar_parent),
        Err(google_calendar::Error::Unauthorized) => {
            refresh_and_save_token(
                oauth_token.id.clone(),
                oauth_token.refresh_token.clone().unwrap(),
            )
            .await;
            Err(Error::Unauthorized)
        }
        Err(e) => {
            println!(
                "Failed to get events from Google Calendar in handle_sync_events: {:?}",
                e
            );
            Err(e)
        }
    }
}

pub fn update_events(google_calendar_parent: GoogleCalendarParent) -> Result<(), String> {
    println!(
        "fetched google calendar events: {:?}",
        google_calendar_parent
            .items
            .iter()
            .map(|item| &item.summary)
            .collect::<Vec<&String>>()
    );

    let duplicated_events = repository::event::find_many(EventFindMany {
        ids_in: Some(
            google_calendar_parent
                .items
                .iter()
                .map(|event| event.id.clone())
                .collect(),
        ),
        ..Default::default()
    })
    .unwrap_or_else(|e| {
        println!(
            "Failed to get upcoming events in handle_sync_events: {:?}",
            e
        );
        vec![]
    });

    // すでに存在するイベントは、events を更新する
    for (event, _) in &duplicated_events {
        let event_update: EventUpdate = google_calendar_parent
            .items
            .iter()
            .find(|e| e.id == event.id)
            .map(|e| EventUpdate {
                summary: Some(e.summary.clone()),
                description: e.description.clone(),
                status: Some(
                    e.status
                        .as_ref()
                        .unwrap_or(&EventStatus::Unknown)
                        .to_string(),
                ),
                start_datetime: Some(e.start.date_time.clone().unwrap()),
                end_datetime: Some(e.end.date_time.clone().unwrap()),
            })
            .expect("EventUpdate must be created");
        let _ = repository::event::update(event.id.clone(), event_update);
    }

    // 新規イベントは、events と notifications を作成する
    let new_google_calendar_events = google_calendar_parent.items.iter().filter(|event| {
        !duplicated_events
            .iter()
            .any(|duplicated_event| duplicated_event.0.id == event.id)
    });

    let event_creates: Vec<Event> = new_google_calendar_events
        .clone()
        .map(|event| Event {
            id: event.id.clone(),
            summary: event.summary.clone(),
            description: event.description.clone(),
            status: Some(
                event
                    .status
                    .as_ref()
                    .unwrap_or(&EventStatus::Unknown)
                    .to_string(),
            ),
            start_datetime: event
                .start
                .date_time
                .clone()
                .expect("start_datetime must exist"),
            end_datetime: event
                .end
                .date_time
                .clone()
                .expect("end_datetime must exist"),
        })
        .collect();
    let event_result = repository::event::create_many(event_creates);
    if let Err(e) = event_result {
        return Err(format!("Failed to create events: {:?}", e).to_string());
    }

    let notification_creates: Vec<Notification> = new_google_calendar_events
        .map(|event| Notification {
            event_id: event.id.clone(),
            enabled: true,
            notification_sec_from_start: 60 * 10,
        })
        .collect();
    let notification_result = repository::notification::create_many(notification_creates);
    if let Err(e) = notification_result {
        return Err(format!("Failed to create notifications: {:?}", e).to_string());
    }

    Ok(())
}

// TODO 期間をクエリパラメータで指定できるようにする
// TODO item だけ返却でも良いのでは？
pub async fn list_events(access_token: String) -> Result<GoogleCalendarParent, Error> {
    let url = format!(
        "https://www.googleapis.com/calendar/v3/calendars/{}/events",
        "primary"
    );

    let mut headers = HeaderMap::new();
    headers.insert(
        "AUTHORIZATION",
        HeaderValue::from_str(&format!("Bearer {}", access_token))?,
    );

    let now = chrono::Utc::now();
    let response = reqwest::Client::new()
        .get(&url)
        .headers(headers)
        .query(&[
            ("maxResults", "10"),
            ("orderBy", "startTime"),
            ("singleEvents", "true"),
            (
                "timeMin",
                &(now - chrono::Duration::minutes(FROM_SUB_SEC.into())).to_rfc3339(),
            ),
            (
                "timeMax",
                &(now + chrono::Duration::days(TO_ADD_DAYS.into())).to_rfc3339(),
            ),
        ])
        .send()
        .await?;
    if response.status() == reqwest::StatusCode::UNAUTHORIZED {
        println!("Unauthorized when requesting list events");
        return Err(Error::Unauthorized);
    }

    let text = response.text().await?;
    let google_calendar_parent: GoogleCalendarParent =
        serde_json::from_str(&text).map_err(|e| Error::Parse(e.to_string()))?;
    Ok(google_calendar_parent)
}
