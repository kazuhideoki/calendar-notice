#![allow(unused_variables)]
use crate::repository::models::{Event, Notification};

pub fn filter_upcoming_events(events: Vec<(Event, Notification)>) -> Vec<Event> {
    let now = chrono::Local::now();
    let upcoming_events: Vec<Event> = events
        .into_iter()
        .filter(|event| filter_by_start_time(event, now))
        .map(|(event, _)| event)
        .collect();

    upcoming_events
}

fn filter_by_start_time(
    (
        Event { start_datetime, .. },
        Notification {
            notification_sec_from_start,
            enabled,
            ..
        },
    ): &(Event, Notification),
    now: chrono::DateTime<chrono::Local>,
) -> bool {
    let start_time =
        chrono::DateTime::parse_from_rfc3339(start_datetime.as_str()).unwrap_or_else(|e| {
            panic!(
                "Error occurred when parsing start time in filter_by_start_time: {:?}",
                e
            )
        });
    let notification_sec_from_start = *notification_sec_from_start as i64;
    start_time.signed_duration_since(now).num_seconds() < notification_sec_from_start && *enabled
}

#[cfg(test)]
mod tests {

    use super::*;
    use chrono::TimeZone;

    // テスト用の簡略化されたEvent構造体

    #[test]
    fn test_filter_by_start_time() {
        let now = chrono::Local
            .with_ymd_and_hms(2023, 8, 1, 12, 0, 0)
            .unwrap();
        let duration = 60 * 10;

        let event1 = (
            Event {
                start_datetime: "2023-08-01T12:10:00+09:00".to_string(), // + NOTIFICATION_INTERVAL_SEC
                ..Default::default()
            },
            Notification {
                notification_sec_from_start: duration.into(),
                enabled: true,
                ..Default::default()
            },
        );
        let result1 = filter_by_start_time(&event1, now);
        assert_eq!(result1, false);

        let event2 = (
            Event {
                start_datetime: "2023-08-01T12:09:59+09:00".to_string(), // + duration - 1

                ..Default::default()
            },
            Notification {
                notification_sec_from_start: (duration).into(),
                enabled: true,
                ..Default::default()
            },
        );
        let result2 = filter_by_start_time(&event2.into(), now);
        assert_eq!(result2, true);

        let event3 = (
            Event {
                start_datetime: "2023-08-01T12:10:00+09:00".to_string(), // + NOTIFICATION_INTERVAL_SEC
                ..Default::default()
            },
            Notification {
                notification_sec_from_start: duration.into(),
                enabled: false,
                ..Default::default()
            },
        );
        let result3 = filter_by_start_time(&event3, now);
        assert_eq!(result3, false);
    }
}
