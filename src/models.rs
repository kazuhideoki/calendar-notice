use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::schema::{events, notifications};

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
