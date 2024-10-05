use chrono::prelude::*;
use std::{
    io,
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::Duration,
};
// ANCHOR: imports

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Rect},
    style::{Color, Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{
        block::{Position, Title},
        Block, Cell, Row, Table, Widget,
    },
    DefaultTerminal, Frame,
};

use crate::repository::{self, models};

// const UI_REFRESH_INTERVAL_SEC: u64 = 60; // TODO
const UI_REFRESH_INTERVAL_SEC: u64 = 5;

#[derive(Default)]
pub struct UI {
    pub events: Vec<repository::models::Event>,
    pub selected_event_id: Option<String>,
    pub exit: bool,
}
impl UI {
    pub fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
        fetch_events: fn() -> Vec<models::Event>,
    ) -> io::Result<()> {
        // イベント更新用のチャンネルを設定
        let (tx, rx): (Sender<()>, Receiver<()>) = mpsc::channel();

        // 定期的なイベント更新を行うバックグラウンドスレッドを起動
        thread::spawn(move || loop {
            thread::sleep(Duration::from_secs(UI_REFRESH_INTERVAL_SEC));
            if tx.send(()).is_err() {
                break; // メインスレッドが終了した場合にループを抜ける
            }
        });

        loop {
            // 終了フラグが立っていたらループを抜ける
            if self.exit {
                break;
            }

            // キー入力イベントをノンブロッキングでチェック
            if event::poll(Duration::from_millis(100)).unwrap() {
                self.handle_key_events()?;
            }

            // 定期的なイベント更新をチェック
            if rx.try_recv().is_ok() {
                let events = fetch_events();
                self.events = events;
            }

            // 現在の UI 状態に基づいて画面を描画
            terminal.draw(|frame| self.draw(frame))?;
        }

        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_key_events(&mut self) -> io::Result<bool> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                match key_event.code {
                    KeyCode::Char(c) if c.is_ascii_digit() => {
                        let num = c.to_digit(10).unwrap_or(0);
                        if num > 0 && num <= self.events.len() as u32 {
                            self.selected_event_id = Some(self.events[num as usize - 1].id.clone());
                        }
                    }
                    KeyCode::Char('q') => {
                        self.selected_event_id = None;
                        self.exit()
                    }
                    _ => {}
                }
            }
            _ => {}
        };
        Ok(true)
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &UI {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Title::from(" 本日の予定 ".bold());
        let instructions = Title::from(Line::from(vec![
            // TODO ショートカットキーの説明を追加
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]));
        let block = Block::bordered()
            .title(title.alignment(Alignment::Center))
            .title(
                instructions
                    .alignment(Alignment::Center)
                    .position(Position::Bottom),
            )
            .border_set(border::THICK);

        let header_cells = ["No.", "日付", "説明"]
            .iter()
            .map(|h| Cell::from(*h).style(Style::default().fg(Color::Yellow)));
        let header = Row::new(header_cells).style(Style::default().bg(Color::DarkGray));

        let rows = self.events.iter().enumerate().map(|(index, event)| {
            let datetime = DateTime::parse_from_rfc3339(&event.start_datetime)
                .expect("Invalid datetime format");
            let cells = [
                Cell::from((index + 1).to_string()),
                Cell::from(datetime.format("%m-%d %H-%M").to_string()),
                Cell::from(
                    event
                        .summary
                        .clone()
                        .unwrap_or("[タイトル未設定]".to_string()),
                ),
            ];

            Row::new(cells)
        });

        let table = Table::new(
            rows,
            &[
                Constraint::Length(5),
                Constraint::Length(15),
                Constraint::Percentage(80),
            ],
        )
        .header(header)
        .block(block);

        table.render(area, buf);
    }
}
