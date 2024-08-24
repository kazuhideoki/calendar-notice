CREATE TABLE oauth_tokens (
    id TEXT PRIMARY KEY NOT NULL,
    access_token TEXT NOT NULL,
    expires_in DATETIME,
    refresh_token TEXT,
    scope TEXT,
    token_type VARCHAR(20),
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL
);
