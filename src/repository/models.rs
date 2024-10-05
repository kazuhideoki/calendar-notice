use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::{
    google_calendar::{extract_teams_link, extract_zoom_link, EventStatus, GoogleCalendarEvent},
    schema::{events, oauth_tokens},
};

#[derive(
    Debug,
    Queryable,
    Selectable,
    Identifiable,
    Insertable,
    AsChangeset,
    Serialize,
    Deserialize,
    Clone,
    Default,
)]
#[diesel(table_name = oauth_tokens)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[diesel(primary_key(id))]
pub struct OAuthToken {
    pub id: String,
    pub access_token: String,
    pub expires_in: Option<String>,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
    pub token_type: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Queryable, AsChangeset)]
#[diesel(table_name = oauth_tokens)]
pub struct OAuthTokenUpdate {
    pub access_token: Option<String>,
    pub expires_in: Option<String>,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
    pub token_type: Option<String>,
    pub updated_at: String,
}

#[derive(
    Debug,
    Queryable,
    Selectable,
    Identifiable,
    Insertable,
    AsChangeset,
    Serialize,
    Deserialize,
    Clone,
    Default,
)]
#[diesel(table_name = events)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
// TODO createdAt 追加
pub struct Event {
    pub id: String,
    pub summary: Option<String>,
    pub description: Option<String>,
    // TODO enum にできるか？
    pub status: Option<String>,
    pub hangout_link: Option<String>,
    pub zoom_link: Option<String>,
    pub teams_link: Option<String>,
    pub start_datetime: String,
    pub end_datetime: String,
    pub notification_enabled: bool,
    pub notification_sec_from_start: i32,
}

impl From<&GoogleCalendarEvent> for Event {
    fn from(google_calendar_event: &GoogleCalendarEvent) -> Self {
        Event {
            id: google_calendar_event.id.clone(),
            summary: Some(google_calendar_event.summary.clone()),
            description: google_calendar_event.description.clone(),
            status: Some(
                google_calendar_event
                    .status
                    .as_ref()
                    .unwrap_or(&EventStatus::Unknown)
                    .to_string(),
            ),
            hangout_link: google_calendar_event.hangout_link.clone(),
            zoom_link: match google_calendar_event.description {
                Some(ref description) => extract_zoom_link(description),
                None => None,
            },
            teams_link: match google_calendar_event.description {
                Some(ref description) => extract_teams_link(description),
                None => None,
            },
            start_datetime: google_calendar_event
                .start
                .date_time
                .clone()
                .expect("start_datetime must exist"),
            end_datetime: google_calendar_event
                .end
                .date_time
                .clone()
                .expect("end_datetime must exist"),
            notification_enabled: true,
            notification_sec_from_start: 60 * 10,
        }
    }
}

#[derive(Default)]
pub struct EventFindMany {
    pub from: Option<String>,
    pub to: Option<String>,
    pub ids_in: Option<Vec<String>>,
}

#[derive(Queryable, AsChangeset, Default)]
#[diesel(table_name = events)]
pub struct EventUpdate {
    pub summary: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub hangout_link: Option<String>,
    pub zoom_link: Option<String>,
    pub teams_link: Option<String>,
    pub start_datetime: Option<String>,
    pub end_datetime: Option<String>,
    pub notification_enabled: Option<bool>,
    pub notification_sec_from_start: Option<i32>,
}

impl From<&GoogleCalendarEvent> for EventUpdate {
    fn from(google_calendar_event: &GoogleCalendarEvent) -> Self {
        EventUpdate {
            summary: Some(google_calendar_event.summary.clone()),
            description: google_calendar_event.description.clone(),
            status: Some(
                google_calendar_event
                    .status
                    .as_ref()
                    .unwrap_or(&EventStatus::Unknown)
                    .to_string(),
            ),
            hangout_link: google_calendar_event.hangout_link.clone(),
            zoom_link: match google_calendar_event.description {
                Some(ref description) => extract_zoom_link(description),
                None => None,
            },
            teams_link: match google_calendar_event.description {
                Some(ref description) => extract_teams_link(description),
                None => None,
            },
            start_datetime: Some(google_calendar_event.start.date_time.clone().unwrap()),
            end_datetime: Some(google_calendar_event.end.date_time.clone().unwrap()),
            ..Default::default()
        }
    }
}
