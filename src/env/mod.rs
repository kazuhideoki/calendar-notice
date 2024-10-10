#![allow(unused_variables, dead_code)]
use dotenvy::dotenv;
use std::env;

const DEFAULT_EVENT_PERIOD: u32 = 7;

#[derive(Default)]
pub struct Env {
    pub database_url: String,
    pub port: u16,
    pub base_url: String,
    pub event_period: u32,
}

impl Env {
    pub fn new() -> Self {
        dotenv().ok();

        Env {
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            port: env::var("PORT")
                .expect("PORT must be set")
                .parse()
                .expect("PORT must be a number"),
            base_url: env::var("BASE_URL").expect("BASE_URL must be set"),
            event_period: env::var("EVENT_PERIOD")
                .unwrap_or(DEFAULT_EVENT_PERIOD.to_string())
                .parse()
                .expect("EVENT_PERIOD must be a number"),
        }
    }
}
