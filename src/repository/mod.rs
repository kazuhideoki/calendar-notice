#![allow(unused_variables)]
pub mod models;

use std::time::Duration;

use diesel::{
    connection::SimpleConnection,
    r2d2::{ConnectionManager, Pool, PooledConnection},
    SqliteConnection,
};

use crate::env::Env;

// 参考 https://stackoverflow.com/questions/57123453/how-to-use-diesel-with-sqlite-connections-and-avoid-database-is-locked-type-of
#[derive(Debug)]
pub struct ConnectionOptions {
    pub enable_wal: bool,
    pub enable_foreign_keys: bool,
    pub busy_timeout: Option<Duration>,
}

impl diesel::r2d2::CustomizeConnection<SqliteConnection, diesel::r2d2::Error>
    for ConnectionOptions
{
    fn on_acquire(&self, conn: &mut SqliteConnection) -> Result<(), diesel::r2d2::Error> {
        (|| {
            if self.enable_wal {
                conn.batch_execute("PRAGMA journal_mode = WAL; PRAGMA synchronous = NORMAL;")?;
            }
            if self.enable_foreign_keys {
                conn.batch_execute("PRAGMA foreign_keys = ON;")?;
            }
            if let Some(d) = self.busy_timeout {
                conn.batch_execute(&format!("PRAGMA busy_timeout = {};", d.as_millis()))?;
            }
            Ok(())
        })()
        .map_err(diesel::r2d2::Error::QueryError)
    }
}

fn get_connection() -> PooledConnection<ConnectionManager<SqliteConnection>> {
    let database_url = Env::new().database_url;

    let pool = Pool::builder()
        .max_size(16)
        .connection_customizer(Box::new(ConnectionOptions {
            enable_wal: true,
            enable_foreign_keys: true,
            busy_timeout: Some(Duration::from_secs(30)),
        }))
        .build(ConnectionManager::<SqliteConnection>::new(database_url))
        .unwrap();

    pool.get().unwrap()
}

pub mod event {
    use diesel::{
        query_dsl::methods::FilterDsl, result, ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl,
        SelectableHelper,
    };

    use crate::schema::{events, notifications};

    use super::models::{Event, EventFindMany, EventUpdate, Notification};

    pub fn find_many(query: EventFindMany) -> Result<Vec<(Event, Notification)>, result::Error> {
        let mut query_builder = events::table
            .inner_join(notifications::table.on(events::id.eq(notifications::event_id)))
            .select((Event::as_select(), Notification::as_select()))
            .order(events::start_datetime.asc())
            .into_boxed();

        if let Some(from) = query.from {
            query_builder = FilterDsl::filter(query_builder, events::start_datetime.ge(from));
        }

        if let Some(to) = query.to {
            query_builder = FilterDsl::filter(query_builder, events::end_datetime.le(to));
        }

        if let Some(ids_in) = query.ids_in {
            query_builder = FilterDsl::filter(query_builder, events::id.eq_any(ids_in));
        }

        query_builder.load(&mut super::get_connection())
    }

    pub fn create_many(events: Vec<Event>) -> Result<(), std::io::Error> {
        let result = diesel::insert_into(events::table)
            .values(&events)
            .execute(&mut super::get_connection());

        match result {
            Ok(_) => Ok(()),
            // TODO エラー定義
            Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
        }
    }

    pub fn update(id: String, event_update: EventUpdate) -> Result<(), std::io::Error> {
        let result = diesel::update(events::table.find(id))
            .set(&event_update)
            .execute(&mut super::get_connection());

        match result {
            Ok(_) => Ok(()),
            // TODO エラー定義
            Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
        }
    }
}

pub mod notification {
    use diesel::{query_dsl::methods::FilterDsl, result, ExpressionMethods, QueryDsl, RunQueryDsl};

    use crate::schema::notifications;

    use super::models::{Notification, NotificationFindMany, NotificationUpdate};

    pub fn find_many(query: NotificationFindMany) -> Result<Vec<Notification>, result::Error> {
        let mut query_builder = notifications::table.into_boxed();

        if let Some(event_ids_in) = query.event_ids_in {
            query_builder =
                FilterDsl::filter(query_builder, notifications::event_id.eq_any(event_ids_in));
        }

        query_builder.load(&mut super::get_connection())
    }

    pub fn create_many(notifications: Vec<Notification>) -> Result<(), std::io::Error> {
        let result = diesel::insert_into(notifications::table)
            .values(&notifications)
            .execute(&mut super::get_connection());

        match result {
            Ok(_) => Ok(()),
            // TODO エラー定義
            Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
        }
    }

    pub fn update(
        event_id: String,
        notification_update: NotificationUpdate,
    ) -> Result<(), std::io::Error> {
        let result = diesel::update(notifications::table.find(event_id))
            .set(&notification_update)
            .execute(&mut super::get_connection());

        match result {
            Ok(_) => Ok(()),
            // TODO エラー定義
            Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
        }
    }
}

pub mod oauth_token {
    use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};

    use crate::{repository::models::OAuthToken, schema::oauth_tokens};

    use super::{get_connection, models::OAuthTokenUpdate};

    pub fn create(oauth_token: OAuthToken) -> Result<(), std::io::Error> {
        let result = diesel::insert_into(oauth_tokens::table)
            .values(&oauth_token)
            .execute(&mut get_connection());

        match result {
            Ok(_) => Ok(()),
            // TODO エラー定義
            Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
        }
    }

    pub fn update(id: String, oauth_token_update: OAuthTokenUpdate) -> Result<(), std::io::Error> {
        let result = diesel::update(oauth_tokens::table.find(&id))
            .set(&oauth_token_update)
            .execute(&mut get_connection());

        match result {
            Ok(_) => Ok(()),
            // TODO エラー定義
            Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
        }
    }

    pub fn find_latest() -> Result<Option<OAuthToken>, std::io::Error> {
        let result = oauth_tokens::table
            .order(oauth_tokens::created_at.desc())
            .first::<OAuthToken>(&mut get_connection())
            .optional()
            .expect("Error loading oauth token");

        match result {
            Some(result) => Ok(Some(result)),
            None => Ok(None),
        }
    }
}
