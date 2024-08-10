use dotenvy::dotenv;
use std::env;

pub struct Env {
    pub test_token: Option<String>,
}

impl Env {
    pub fn new() -> Self {
        dotenv().ok();

        Env {
            test_token: env::var("TEST_TOKEN").ok(),
        }
    }
}
