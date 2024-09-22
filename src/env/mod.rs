#![allow(unused_variables, dead_code)]
use dotenvy::dotenv;
use std::env;

#[derive(Default)]
pub struct Env {
    pub database_url: String,
    pub port: u16,
    pub base_url: String,
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
        }
    }
}
