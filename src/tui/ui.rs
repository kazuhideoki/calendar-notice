use chrono::prelude::*;
use tokio::spawn;
// ANCHOR: imports
use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Rect},
    style::{Color, Style, Stylize},
    symbols::border,
    text::{Line, Text},
    widgets::{
        block::{Position, Title},
        Block, Cell, Paragraph, Row, Table, Widget,
    },
    DefaultTerminal, Frame,
};

use crate::repository;

#[derive(Default)]
pub struct UI {
    pub events: Vec<repository::models::Event>,
    pub selected_event_id: Option<String>,
    pub exit: bool,
}
impl UI {
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?; // ここでキー入力を待つことになる。exitフラグが立つまで。なので別スレッドにしたい
        }
        Ok(())
    }

    pub fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    /// updates the application's state based on user input
    pub fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char(c) if c.is_ascii_digit() => {
                let num = c.to_digit(10).unwrap();
                if num <= self.events.len() as u32 {
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

    pub fn exit(&mut self) {
        self.exit = true;
    }
    pub fn restart(&mut self) {
        self.exit = false;
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
                Cell::from(event.summary.clone()),
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
