use crate::google_calendar;

use super::NOTIFICATION_INTERVAL_SEC;

pub async fn filter_upcoming_events(
    events: google_calendar::CalendarEvents,
) -> Vec<google_calendar::Event> {
    let now = chrono::Local::now();
    let upcoming_events: Vec<google_calendar::Event> = events
        .items
        .into_iter()
        .filter(|item| filter_by_start_time(item, now))
        .collect();

    upcoming_events
}

fn filter_by_start_time(
    item: &google_calendar::Event,
    now: chrono::DateTime<chrono::Local>,
) -> bool {
    let start_time = item
        .start
        .date_time
        .as_ref()
        .map(String::as_str)
        .unwrap_or_else(|| "2099-01-01T00:00:00.000Z");
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

    impl From<TestEvent> for google_calendar::Event {
        fn from(test_event: TestEvent) -> Self {
            google_calendar::Event {
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
