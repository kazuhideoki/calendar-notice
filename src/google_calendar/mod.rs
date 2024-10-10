use std::fmt;
use std::thread;
use std::time::Duration;

use reqwest::header::InvalidHeaderValue;

use crate::env::Env;
use crate::oauth::is_token_expired::is_token_expired;
use crate::oauth::refresh_and_save_token;
use crate::oauth::to_oauth_on_browser;
use crate::{
    google_calendar::{self},
    repository::{
        self,
        models::{Event, EventFindMany, EventUpdate, OAuthToken},
    },
};
use serde::{Deserialize, Serialize};

// pub mod extract_zoom_link;
mod extract_zoom_link;
pub use self::extract_zoom_link::extract_zoom_link;
mod extract_teams_link;
pub use self::extract_teams_link::extract_teams_link;
mod google_calendar_api;

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

impl fmt::Display for EventStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventStatus::Confirmed => write!(f, "confirmed"),
            EventStatus::Tentative => write!(f, "tentative"),
            EventStatus::Cancelled => write!(f, "cancelled"),
            EventStatus::Unknown => write!(f, "unknown"),
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
#[serde(rename_all = "camelCase")]
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

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EventPerson {
    email: String,
    display_name: Option<String>,
    self_: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EventDateTime {
    #[serde(rename = "dateTime", default)]
    pub date_time: Option<String>,
    #[serde(default)]
    pub date: Option<String>,
    #[serde(rename = "timeZone")]
    pub time_zone: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attendee {
    email: String,
    display_name: Option<String>,
    organizer: Option<bool>,
    self_: Option<bool>,
    response_status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Reminders {
    use_default: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConferenceData {
    entry_points: Vec<EntryPoint>,
    conference_solution: ConferenceSolution,
    conference_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntryPoint {
    entry_point_type: String,
    uri: String,
    label: Option<String>,
    pin: Option<String>,
    region_code: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConferenceSolution {
    key: ConferenceSolutionKey,
    name: String,
    icon_uri: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
// const SYNC_CALENDAR_INTERVAL_SEC: u16 = 5;
// TODO 扱う期間を const or env 化
const SYNC_CALENDAR_FROM_SUB_SEC: u16 = 60 * 10;

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

                    let _ =
                        repository::oauth_token::find_latest().expect("new token must be found");
                    sync_events(oauth_token).await.unwrap_or_else(|_| {
                        // println!(
                        //     "Failed to sync events in run_sync_calendar_cron_thread with new token: {:?}",
                        //     e
                        // )
                    });
                }
                Some(oauth_token) => {
                    sync_events(oauth_token).await.unwrap_or_else(|e| {
                        println!(
                            "Failed to sync events in run_sync_calendar_cron_thread: {:?}",
                            e
                        )
                    });
                }
                None => {
                    // println!("OAuth token is not found. Please authenticate ");
                    // TODO 認証完了まで、次のループで再度認証催促が発生するのを防ぐ？
                    to_oauth_on_browser();
                }
            }

            thread::sleep(Duration::from_secs(SYNC_CALENDAR_INTERVAL_SEC.into()));
        }
    });
}

pub async fn sync_events(oauth_token: OAuthToken) -> Result<(), Error> {
    let env = Env::new();
    let now = chrono::Local::now();
    let from = now - chrono::Duration::minutes(SYNC_CALENDAR_FROM_SUB_SEC.into());
    let to = now + chrono::Duration::days(env.event_period.into());

    let google_calendar_result =
        google_calendar_api::list_events(oauth_token.access_token.clone(), from, to).await;
    let google_calendar_parent =
        handle_google_calendar_event_result(google_calendar_result, oauth_token.clone()).await?;
    let _ = update_events(google_calendar_parent, from, to);

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

pub fn update_events(
    google_calendar_parent: GoogleCalendarParent,
    from: chrono::DateTime<chrono::Local>,
    to: chrono::DateTime<chrono::Local>,
) -> Result<(), String> {
    // println!(
    //     "fetched google calendar events: {:?}",
    //     google_calendar_parent
    //         .items
    //         .iter()
    //         .map(|item| &item.summary)
    //         .collect::<Vec<&String>>()
    // );

    let events = repository::event::find_many(EventFindMany {
        from: Some(from.to_rfc3339()),
        to: Some(to.to_rfc3339()),
        ..Default::default()
    })
    .unwrap_or_else(|e| {
        println!(
            "Failed to get upcoming events in handle_sync_events: {:?}",
            e
        );
        vec![]
    });

    let mut existing_events: Vec<&GoogleCalendarEvent> = vec![];
    let mut deleting_events: Vec<&Event> = vec![];
    let mut adding_events: Vec<&GoogleCalendarEvent> = vec![];
    for event in &events {
        let existing_event = google_calendar_parent
            .items
            .iter()
            .find(|e| e.id == event.id);
        if let Some(google_calendar_event) = existing_event {
            existing_events.push(google_calendar_event);
        }

        if existing_event.is_none() {
            deleting_events.push(event);
        }
    }
    for google_calendar_event in &google_calendar_parent.items {
        let existing_event = events.iter().find(|e| e.id == google_calendar_event.id);
        if existing_event.is_none() {
            adding_events.push(google_calendar_event);
        }
    }

    // 更新
    for event in existing_events {
        let event_update: EventUpdate = EventUpdate::from(event);
        let _ = repository::event::update(event.id.clone(), event_update);
    }
    // 削除
    for event in deleting_events {
        let _ = repository::event::delete(event.id.clone());
    }
    // 作成
    let event_result =
        repository::event::create_many(adding_events.iter().map(|e| Event::from(*e)).collect());

    if let Err(e) = event_result {
        return Err(format!("Failed to create events: {:?}", e).to_string());
    }

    Ok(())
}
