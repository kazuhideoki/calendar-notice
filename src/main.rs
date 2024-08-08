#![allow(unused_variables)]
use dotenvy::dotenv;
use std::env;

mod calendar;

struct Env {
    calendar_name: String,
}
impl Env {
    fn new() -> Self {
        dotenv().ok();

        Env {
            calendar_name: env::var("CALENDAR_NAME").expect("CALENDAR_NAME is not set"),
        }
    }
}

fn main() {
    let calendar_name = Env::new().calendar_name;

    let events = calendar::Event::get_events(&calendar_name).expect("Failed to get events");

    events.iter().for_each(|event| {
        println!(
            "🔵 {:?} 時間:{:?}",
            event.title,
            event.start_time.to_string()
        );
    });
}
