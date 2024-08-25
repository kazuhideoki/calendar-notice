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
    let start_time = chrono::DateTime::parse_from_rfc3339(start_time).unwrap();
    start_time.signed_duration_since(now).num_seconds() < NOTIFICATION_INTERVAL_SEC.into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, TimeZone};
    use google_calendar::EventDateTime;

    // テスト用の簡略化されたEvent構造体
    struct TestEvent {
        start: EventDateTime,
    }

    impl TestEvent {
        fn new(time: chrono::DateTime<chrono::Local>) -> Self {
            TestEvent {
                start: EventDateTime {
                    date_time: Some(time.to_rfc3339()),
                    ..Default::default()
                },
            }
        }
    }

    impl From<TestEvent> for google_calendar::CalendarEvent {
        fn from(test_event: TestEvent) -> Self {
            google_calendar::CalendarEvent {
                start: test_event.start,
                ..Default::default()
            }
        }
    }

    #[test]
    fn test_filter_by_start_time() {
        let now = chrono::Local
            .with_ymd_and_hms(2023, 8, 1, 12, 0, 0)
            .unwrap();

        let event1 = TestEvent::new(now + Duration::seconds(NOTIFICATION_INTERVAL_SEC.into()));
        let result1 = filter_by_start_time(&event1.into(), now);
        assert_eq!(result1, false);

        let event2 = TestEvent::new(
            now + Duration::seconds(NOTIFICATION_INTERVAL_SEC.into()) - Duration::microseconds(1),
        );
        let result2 = filter_by_start_time(&event2.into(), now);
        assert_eq!(result2, true);
    }
}
