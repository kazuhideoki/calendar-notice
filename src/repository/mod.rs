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

pub mod event {
    use chrono::TimeZone;
    use diesel::{query_dsl::methods::FilterDsl, result, ExpressionMethods, RunQueryDsl};

    use crate::schema::events;

    use super::models::{Event, EventFindMany};

    pub fn find_many(query: EventFindMany) -> Result<Vec<Event>, result::Error> {
        events::table
            .filter(events::start_datetime.ge(query.from))
            .filter(events::end_datetime.le(query.to))
            .load(&mut super::get_connection())
    }

    pub fn create_many(event: Vec<Event>) -> Result<(), std::io::Error> {
        let result = diesel::insert_into(events::table)
            .values(&event)
            .execute(&mut super::get_connection());

        match result {
            Ok(_) => Ok(()),
            // TODO エラー定義
            Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
        }
    }

    pub fn update() {
        todo!()
    }
    pub fn delete() {
        todo!()
    }
}
