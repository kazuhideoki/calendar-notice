#![allow(unused_variables, dead_code)]
use dotenvy::dotenv;
use std::env;

pub struct Env {
    pub test_token: Option<String>,
    pub database_url: String,
}

impl Env {
    pub fn new() -> Self {
        dotenv().ok();

        Env {
            test_token: env::var("TEST_TOKEN").ok(),
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
        }
    }
}
