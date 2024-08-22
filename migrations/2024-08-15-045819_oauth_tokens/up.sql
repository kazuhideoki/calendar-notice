CREATE TABLE oauth_tokens (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    access_token TEXT NOT NULL,
    expires_in DATETIME CHECK(expires_in IS datetime(expires_in) AND expires_in > CURRENT_TIMESTAMP),
    refresh_token TEXT,
    scope TEXT,
    token_type VARCHAR(20),
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL
);
