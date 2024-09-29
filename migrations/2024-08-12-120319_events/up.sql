CREATE TABLE events (
  id TEXT PRIMARY KEY NOT NULL,
  summary TEXT NOT NULL,
  description TEXT,
  status TEXT,
  hangout_link TEXT,
  zoom_link TEXT,
  teams_link TEXT,
  start_datetime DATETIME NOT NULL,
  end_datetime DATETIME NOT NULL
);
