use std::{
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::Duration,
};

use chrono::Timelike;
use crossterm::event::{self, Event, KeyCode};
use ui::UI;

use crate::repository::{
    self,
    models::{self, EventFindMany},
};

mod ui;

const UI_REFRESH_INTERVAL_SEC: u64 = 3;

pub fn show_tui() {
    let (tx, rx): (Sender<()>, Receiver<()>) = mpsc::channel();
    let mut terminal = ratatui::init();
    let events = fetch_today_event();
    let mut ui = UI {
        events,
        ..Default::default()
    };

    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(UI_REFRESH_INTERVAL_SEC));
        tx.send(()).unwrap();
    });

    loop {
        if ui.exit {
            break;
        }

        if event::poll(Duration::from_millis(100)).unwrap() {
            if let Event::Key(key) = event::read().unwrap() {
                match key.code {
                    KeyCode::Char('q') => ui.exit = true,
                    // 他のキー入力の処理をここに追加
                    _ => {}
                }
            }
        }

        if rx.try_recv().is_ok() {
            let events = fetch_today_event();
            ui = UI {
                events,
                ..Default::default()
            };
        }

        terminal.draw(|frame| ui.draw(frame)).unwrap();
    }

    ratatui::restore();
}

fn fetch_today_event() -> Vec<models::Event> {
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
    .map(|(event, _)| event)
    .collect::<Vec<models::Event>>();

    events
}
