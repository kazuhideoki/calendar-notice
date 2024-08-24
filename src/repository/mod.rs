pub mod models;

use diesel::{connection::SimpleConnection, Connection, SqliteConnection};

use crate::env::Env;

fn get_connection() -> SqliteConnection {
    let database_url = Env::new().database_url;
    let mut conn = SqliteConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url));

    conn.batch_execute("PRAGMA foreign_keys = ON")
        .expect("Failed to enable foreign keys");

    conn
}

pub mod oauth_token {
    use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};

    use crate::{repository::models::OAuthToken, schema::oauth_tokens};

    use super::get_connection;

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
