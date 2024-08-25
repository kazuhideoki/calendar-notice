-- notifications テーブルの作成
CREATE TABLE notifications (
  event_id TEXT PRIMARY KEY NOT NULL,
  enabled BOOLEAN NOT NULL DEFAULT TRUE,
  notification_sec_from_start INTEGER NOT NULL,
  FOREIGN KEY (event_id) REFERENCES events(id) ON DELETE CASCADE
);

-- events テーブルにインデックスを追加（既存のテーブルにインデックスを追加）
CREATE INDEX idx_events_id ON events(id);
