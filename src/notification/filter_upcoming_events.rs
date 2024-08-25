use crate::repository::models::Event;

use super::NOTIFICATION_INTERVAL_SEC;

/**
 * TODO 固定時間ではなく、notification の 通知時間を読み込むロジックに変更
 */
pub async fn filter_upcoming_events(events: Vec<Event>) -> Vec<Event> {
    let now = chrono::Local::now();
    let upcoming_events: Vec<Event> = events
        .into_iter()
        .filter(|event| filter_by_start_time(event, now))
        .collect();

    upcoming_events
}

fn filter_by_start_time(event: &Event, now: chrono::DateTime<chrono::Local>) -> bool {
    let start_time = event.start_datetime.as_str();
    let start_time = chrono::DateTime::parse_from_rfc3339(start_time).unwrap_or_else(|e| {
        panic!(
            "Error occurred when parsing start time in filter_by_start_time: {:?}",
            e
        )
    });
    start_time.signed_duration_since(now).num_seconds() < NOTIFICATION_INTERVAL_SEC.into()
}

#[cfg(test)]
mod tests {

    use crate::google_calendar::{self, EventDateTime};

    use super::*;
    use chrono::TimeZone;

    // テスト用の簡略化されたEvent構造体

    #[test]
    fn test_filter_by_start_time() {
        let now = chrono::Local
            .with_ymd_and_hms(2023, 8, 1, 12, 0, 0)
            .unwrap();

        let event1 = Event {
            start_datetime: "2023-08-01T12:10:00+09:00".to_string(), // + NOTIFICATION_INTERVAL_SEC
            ..Default::default()
        };
        let result1 = filter_by_start_time(&event1, now);
        assert_eq!(result1, false);

        let event2 = Event {
            start_datetime: "2023-08-01T12:09:59+09:00".to_string(), // + NOTIFICATION_INTERVAL_SEC - 1
            ..Default::default()
        };
        let result2 = filter_by_start_time(&event2.into(), now);
        assert_eq!(result2, true);
    }
}
