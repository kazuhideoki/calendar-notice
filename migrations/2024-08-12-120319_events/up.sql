CREATE TABLE events (
  id TEXT PRIMARY KEY NOT NULL,
  summary TEXT NOT NULL,
  description TEXT,
  status TEXT,
  start_datetime DATETIME NOT NULL,
  end_datetime DATETIME NOT NULL,
  CHECK (
    start_datetime IS datetime(start_datetime) AND
    end_datetime IS datetime(end_datetime) AND
    start_datetime <= end_datetime
  )
);
