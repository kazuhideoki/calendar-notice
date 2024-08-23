use std::sync::{Mutex, OnceLock};

use crate::env::Env;
use diesel::connection::SimpleConnection;
use diesel::prelude::*;

pub fn establish_connection() -> SqliteConnection {
    let database_url = Env::new().database_url;
    let mut conn = SqliteConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url));

    conn.batch_execute("PRAGMA foreign_keys = ON")
        .expect("Failed to enable foreign keys");

    conn
}
