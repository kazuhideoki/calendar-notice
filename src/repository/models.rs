use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::schema::{events, notifications, oauth_tokens};

#[derive(
    Debug, Queryable, Selectable, Identifiable, Insertable, AsChangeset, Serialize, Deserialize,
)]
#[diesel(table_name = events)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Event {
    pub id: String,
    pub summary: String,
    pub description: Option<String>,
    pub status: Option<String>,
    pub start_datetime: String,
    pub end_datetime: String,
}

#[derive(
    Debug, Queryable, Selectable, Identifiable, Insertable, AsChangeset, Serialize, Deserialize,
)]
#[diesel(table_name = notifications)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
#[diesel(primary_key(event_id))]
pub struct Notification {
    pub event_id: String,
    pub enabled: bool,
    pub notification_time_from_start: i32,
}

#[derive(
    Debug, Queryable, Selectable, Identifiable, Insertable, AsChangeset, Serialize, Deserialize,
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

impl Default for OAuthToken {
    fn default() -> Self {
        OAuthToken {
            id: uuid::Uuid::new_v4().to_string(),
            access_token: "".to_string(),
            expires_in: None,
            refresh_token: None,
            scope: None,
            token_type: None,
            created_at: "2024-08-01T00:00:00".to_string(),
            updated_at: "2024-08-01T00:00:00".to_string(),
        }
    }
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
