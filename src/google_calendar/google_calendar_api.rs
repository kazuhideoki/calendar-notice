use chrono::{DateTime, Local};
use reqwest::header::{HeaderMap, HeaderValue};

use super::{Error, GoogleCalendarParent};

// TODO 期間をクエリパラメータで指定できるようにする
// TODO item だけ返却でも良いのでは？
pub async fn list_events(
    access_token: String,
    from: DateTime<Local>,
    to: DateTime<Local>,
) -> Result<GoogleCalendarParent, Error> {
    let url = format!(
        "https://www.googleapis.com/calendar/v3/calendars/{}/events",
        "primary"
    );

    let mut headers = HeaderMap::new();
    headers.insert(
        "AUTHORIZATION",
        HeaderValue::from_str(&format!("Bearer {}", access_token))?,
    );

    let response = reqwest::Client::new()
        .get(&url)
        .headers(headers)
        .query(&[
            ("maxResults", "10"),
            ("orderBy", "startTime"),
            ("singleEvents", "true"),
            ("timeMin", &from.to_rfc3339()),
            ("timeMax", &to.to_rfc3339()),
        ])
        .send()
        .await?;
    if response.status() == reqwest::StatusCode::UNAUTHORIZED {
        // println!("Unauthorized when requesting list events");
        return Err(Error::Unauthorized);
    }

    let text = response.text().await?;
    // println!("🔶 text: {:?}", text);
    let google_calendar_parent: GoogleCalendarParent =
        serde_json::from_str(&text).map_err(|e| Error::Parse(e.to_string()))?;
    // println!("🔵 google_calendar_parent: {:?}", google_calendar_parent);
    Ok(google_calendar_parent)
}

// TODO
pub async fn update_join() {}
