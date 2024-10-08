CREATE TABLE events (
  id TEXT PRIMARY KEY NOT NULL,
  summary TEXT NULL,
  description TEXT NULL,
  status TEXT,
  hangout_link TEXT,
  zoom_link TEXT,
  teams_link TEXT,
  start_datetime DATETIME NOT NULL,
  end_datetime DATETIME NOT NULL,
  notification_enabled BOOLEAN NOT NULL DEFAULT TRUE,
  notification_sec_from_start INTEGER NOT NULL
);
