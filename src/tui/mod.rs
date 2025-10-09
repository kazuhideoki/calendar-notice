use chrono::Timelike;
use tokio::sync::watch::Sender;
use ui::UI;

use crate::{
    env::Env,
    repository::{
        self,
        models::{self, EventFindMany},
    },
};

mod ui;

pub fn show_tui(shutdown_tx: Sender<bool>) {
    let mut terminal = ratatui::init();
    let env = Env::new();
    let selected_day: u32 = 1;
    let events = fetch_today_events(selected_day);
    let mut ui = UI {
        events,
        selected_day,
        event_period: env.event_period,
        ..Default::default()
    };

    let _ = ui.run(&mut terminal, fetch_today_events);

    // TUI終了時にシャットダウンシグナルを送信
    let _ = shutdown_tx.send(true);

    ratatui::restore();
}

fn fetch_today_events(_selected_day: u32) -> Vec<models::Event> {
    let env = Env::new();
    let event_period = env.event_period;
    let start_day = chrono::Local::now()
        .with_hour(0)
        .unwrap()
        .with_minute(0)
        .unwrap()
        .with_second(0)
        .unwrap();
    let end_day = start_day + chrono::Duration::days(event_period as i64);

    let events = repository::event::find_many(EventFindMany {
        from: Some(start_day.to_rfc3339()),
        to: Some(end_day.to_rfc3339()),
        ..Default::default()
    })
    .expect("Failed to find events.")
    .into_iter()
    .collect::<Vec<models::Event>>();

    events
}
