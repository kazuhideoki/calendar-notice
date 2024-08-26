use chrono::{DateTime, TimeZone};

use crate::repository::models::OAuthToken;

pub fn is_token_expired<Tz: TimeZone>(token: &OAuthToken, now: DateTime<Tz>) -> bool {
    let updated_at = chrono::DateTime::parse_from_rfc3339(&token.updated_at)
        .unwrap_or_else(|e| panic!("Failed to parse updated_at: {:?}", e));
    let expires_in = token
        .expires_in
        .as_ref()
        .unwrap_or(&"0".to_string())
        .parse::<i64>()
        .expect("Failed to parse expires_in");
    let expires_at = updated_at + chrono::Duration::seconds(expires_in);

    expires_at < now
}

#[cfg(test)]
mod tests {
    use crate::repository::models::OAuthToken;

    use super::*;

    #[test]
    fn test_is_token_expired() {
        let token = OAuthToken {
            expires_in: Some("3600".to_string()),
            updated_at: "2021-07-01T00:00:00+09:00".to_string(),
            ..Default::default()
        };

        let now = chrono::DateTime::parse_from_rfc3339("2021-07-01T01:00:00+09:00").unwrap();
        assert_eq!(is_token_expired(&token, now), false);

        let now = chrono::DateTime::parse_from_rfc3339("2021-07-01T01:00:01+09:00").unwrap();
        assert_eq!(is_token_expired(&token, now), true);
    }
}
