use chrono::Timelike;
use ui::UI;

use crate::repository::{
    self,
    models::{self, EventFindMany},
};

mod ui;

pub fn show_tui() {
    let mut terminal = ratatui::init();
    let events = fetch_today_events();
    let mut ui = UI {
        events,
        ..Default::default()
    };

    let _ = ui.run(&mut terminal, fetch_today_events);

    ratatui::restore();
}

fn fetch_today_events() -> Vec<models::Event> {
    let start_of_today = chrono::Local::now()
        .with_hour(0)
        .unwrap()
        .with_minute(0)
        .unwrap()
        .with_second(0)
        .unwrap();
    let tomorrow = start_of_today + chrono::Duration::days(1);

    let events = repository::event::find_many(EventFindMany {
        from: Some(start_of_today.to_rfc3339()),
        to: Some(tomorrow.to_rfc3339()),
        ..Default::default()
    })
    .expect("Failed to find events.")
    .into_iter()
    .collect::<Vec<models::Event>>();

    events
}
