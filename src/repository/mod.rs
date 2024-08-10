use std::sync::{Mutex, OnceLock};

use crate::oauth::OAuthResponse;

pub fn oauth_state() -> &'static std::sync::Mutex<Option<OAuthResponse>> {
    static AUTHENTICATION_CODE: OnceLock<Mutex<Option<OAuthResponse>>> = OnceLock::new();
    AUTHENTICATION_CODE.get_or_init(|| Mutex::new(None))
}
