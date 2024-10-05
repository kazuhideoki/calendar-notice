use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::schema::{events, oauth_tokens};

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
