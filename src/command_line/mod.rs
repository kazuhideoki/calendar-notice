use chrono::Timelike;
use clap::Parser;
use std::io::{self, BufRead};
use ui::UI;

use crate::{
    google_calendar::sync_events,
    notification::NOTIFICATION_PERIOD_DAYS,
    oauth::{self, is_token_expired::is_token_expired, refresh_and_save_token},
    repository::{
        self,
        models::{self, EventFindMany, NotificationUpdate, OAuthToken},
    },
};

mod ui;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None, disable_help_flag = true)]
struct Args {
    #[arg(short, long)]
    test: Option<String>,
}

#[derive(PartialEq, Debug)]
enum CommandLineState {
    Top,
    UpdateEnable,
    End,
}

pub async fn run_command_loop_async() {
    let mut state = CommandLineState::Top;

    // TODO 整理
    let mut terminal = ratatui::init();
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

    let mut ui = UI {
        events,
        ..Default::default()
    };
    let _ = ui.run(&mut terminal);
    ratatui::restore();

    loop {
        state = command_line_loop(state).await;

        if state == CommandLineState::End {
            break;
        }
    }

    println!("end!");
}

async fn command_line_loop(mut state: CommandLineState) -> CommandLineState {
    let stdin_lines = io::stdin().lock().lines();

    println!("🔵 state is {:?}", state);

    let mut next_state = CommandLineState::Top;

    // ここまでに一覧が表示されている
    // この入力がワンキーコマンドになる
    for line in stdin_lines {
        match line {
            Ok(input) => {
                println!("Input: {}", input);
                if input.trim().is_empty() {
                    println!("Input is empty");
                    next_state = CommandLineState::End;
                    break;
                } else if state == CommandLineState::UpdateEnable {
                    handle_update_enabled(&mut state, input).await;
                    next_state = CommandLineState::Top;
                    break;
                } else {
                    match input.as_str() {
                        "token" => {
                            handle_command_token();
                            next_state = CommandLineState::Top;
                            break;
                        }
                        "refresh" => {
                            handle_command_refresh().await;
                            next_state = CommandLineState::Top;
                            break;
                        }
                        "sync" => {
                            handle_command_sync().await;
                            next_state = CommandLineState::Top;
                            break;
                        }
                        "list" => {
                            handle_list_notification(&mut state).await;
                            next_state = CommandLineState::UpdateEnable;
                            break;
                        }
                        _ => {
                            next_state = CommandLineState::Top;
                            break;
                        }
                    }
                }
            }
            Err(e) => eprintln!("Error occurred when reading line: {:?}", e),
        };
        println!("match ended");
    }

    next_state
}

// TODO きれいに出力
fn handle_command_token() {
    let oauth_token = repository::oauth_token::find_latest();

    match oauth_token {
        Ok(Some(token)) => {
            println!("{:?}", token);
        }
        Ok(None) => {
            println!("OAuth token is not found");
        }
        Err(e) => {
            eprintln!("Error: {:?}", e);
        }
    }
}

async fn handle_command_refresh() {
    let OAuthToken {
        id, refresh_token, ..
    } = repository::oauth_token::find_latest().unwrap().unwrap();

    match refresh_token {
        Some(refresh_token) => {
            refresh_and_save_token(id, refresh_token).await;
        }
        None => {
            println!("Refresh token is not found");
        }
    }
}

async fn handle_command_sync() {
    let token_result = repository::oauth_token::find_latest();
    match token_result {
        Ok(token) => {
            if token
                .as_ref()
                .map_or(true, |token| is_token_expired(token, chrono::Local::now()))
            {
                println!("OAuth token is not found. Please authenticate again.");
                if let Some(OAuthToken {
                    refresh_token: Some(refresh_token),
                    ..
                }) = token.clone()
                {
                    let _ = refresh_and_save_token(token.clone().unwrap().id, refresh_token).await;
                    let token = repository::oauth_token::find_latest().unwrap();
                    sync_events(token.unwrap()).await.unwrap_or_else(|e| {
                        eprintln!(
                            "Failed to sync events in handle_command_sync with new token: {:?}",
                            e
                        )
                    });
                } else {
                    println!("Refresh token is not found");
                    oauth::to_oauth_on_browser();
                }
            } else {
                sync_events(token.unwrap()).await.unwrap_or_else(|e| {
                    eprintln!("Failed to sync events in handle_command_sync: {:?}", e)
                });
            }
        }
        Err(e) => {
            eprintln!("Error occurred when getting latest token: {:?}", e);
        }
    }
}

async fn handle_list_notification(_: &mut CommandLineState) {
    let now = chrono::Local::now();
    let events = repository::event::find_many(EventFindMany {
        from: Some(now.to_rfc3339()),
        to: Some((now + chrono::Duration::days(NOTIFICATION_PERIOD_DAYS)).to_rfc3339()),
        ..Default::default()
    })
    .unwrap();

    let mut count = 0;
    println!();
    println!("通知設定");
    for (event, notification) in events.clone() {
        count += 1;
        let notified = if notification.enabled {
            format!("{}分前通知", notification.notification_sec_from_start / 60)
        } else {
            "通知なし".to_string()
        };
        println!(
            "{}: {}: {}開始 {}",
            count,
            event.summary,
            chrono::DateTime::parse_from_rfc3339(event.start_datetime.as_str())
                .expect("Error occurred when parsing start time, handle_list_notification")
                .format("%m-%d %H:%M"),
            notified
        );
    }
    println!("番号を入力すると、通知のオンオフを切り替えます。");
}

async fn handle_update_enabled(state: &mut CommandLineState, input: String) {
    let maybe_num = input.parse::<i32>();

    match maybe_num {
        Ok(num) => {
            let now = chrono::Local::now();
            let events = repository::event::find_many(EventFindMany {
                from: Some(now.to_rfc3339()),
                to: Some((now + chrono::Duration::days(NOTIFICATION_PERIOD_DAYS)).to_rfc3339()),
                ..Default::default()
            })
            .unwrap();

            let events = events.get((num - 1) as usize);
            if let Some((event, notification)) = events {
                let enabled = !notification.enabled;
                repository::notification::update(
                    notification.event_id.clone(),
                    NotificationUpdate {
                        enabled: Some(enabled),
                        ..Default::default()
                    },
                )
                .unwrap_or_else(|e| println!("Failed to update notification: {}", e));
                let enabled_str = if enabled { "有効" } else { "無効" };
                println!(
                    "{} の通知を {} に更新しました！",
                    event.summary, enabled_str
                );
            } else {
                println!("数字が範囲外です");
            }

            let _ = handle_list_notification(state).await;
        }
        Err(_) => println!("数字を入力してください"),
    }
}
