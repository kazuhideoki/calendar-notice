use chrono::{DateTime, TimeZone};

use crate::repository::models::OAuthToken;

const EXPIRED_MARGIN_SEC: i64 = 60;

/**
 * トークンが有効期限切れかどうかを判定する。実際の有効期限よりも少し余裕を持たせる
 */
pub fn is_token_expired<Tz: TimeZone>(token: &OAuthToken, now: DateTime<Tz>) -> bool {
    let updated_at = chrono::DateTime::parse_from_rfc3339(&token.updated_at)
        .unwrap_or_else(|e| panic!("Failed to parse updated_at: {:?}", e));
    let expires_in = token
        .expires_in
        .as_ref()
        .unwrap_or(&"0".to_string())
        .parse::<i64>()
        .expect("Failed to parse expires_in");
    let expired_with_margin_at =
        updated_at + chrono::Duration::seconds(expires_in - EXPIRED_MARGIN_SEC);

    expired_with_margin_at < now
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

        let now = chrono::DateTime::parse_from_rfc3339("2021-07-01T00:59:00+09:00").unwrap();
        assert_eq!(is_token_expired(&token, now), false);

        let now = chrono::DateTime::parse_from_rfc3339("2021-07-01T01:59:01+09:00").unwrap();
        assert_eq!(is_token_expired(&token, now), true);
    }
}
