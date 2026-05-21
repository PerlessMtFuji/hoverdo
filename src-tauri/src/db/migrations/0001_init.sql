-- Hoverdo schema v1.
--
-- All "domain" tables share sync-ready columns:
--   id                 TEXT     UUIDv7 (time-ordered)
--   created_at         TEXT     RFC3339 UTC
--   updated_at         TEXT     RFC3339 UTC
--   deleted_at         TEXT     RFC3339 UTC (NULL = live, NOT NULL = soft-deleted)
--   hlc_ts             TEXT     Hybrid Logical Clock at last write
--   origin_device_id   TEXT     UUID of the device that produced the write
--
-- MVP is single-device offline. Those columns exist now so that adding a
-- sync transport in the future is a non-breaking change.

PRAGMA foreign_keys = ON;

CREATE TABLE notes (
    id               TEXT PRIMARY KEY NOT NULL,
    title            TEXT NOT NULL DEFAULT '',
    body             TEXT NOT NULL DEFAULT '',
    color            TEXT,
    created_at       TEXT NOT NULL,
    updated_at       TEXT NOT NULL,
    deleted_at       TEXT,
    hlc_ts           TEXT NOT NULL,
    origin_device_id TEXT NOT NULL
);
CREATE INDEX idx_notes_updated_at ON notes(updated_at DESC) WHERE deleted_at IS NULL;
CREATE INDEX idx_notes_hlc ON notes(hlc_ts);

CREATE TABLE lists (
    id               TEXT PRIMARY KEY NOT NULL,
    title            TEXT NOT NULL DEFAULT '',
    color            TEXT,
    created_at       TEXT NOT NULL,
    updated_at       TEXT NOT NULL,
    deleted_at       TEXT,
    hlc_ts           TEXT NOT NULL,
    origin_device_id TEXT NOT NULL
);
CREATE INDEX idx_lists_updated_at ON lists(updated_at DESC) WHERE deleted_at IS NULL;
CREATE INDEX idx_lists_hlc ON lists(hlc_ts);

CREATE TABLE tasks (
    id               TEXT PRIMARY KEY NOT NULL,
    list_id          TEXT NOT NULL REFERENCES lists(id) ON DELETE CASCADE,
    title            TEXT NOT NULL DEFAULT '',
    done             INTEGER NOT NULL DEFAULT 0,
    due_at           TEXT,
    sort_key         TEXT NOT NULL,
    created_at       TEXT NOT NULL,
    updated_at       TEXT NOT NULL,
    deleted_at       TEXT,
    hlc_ts           TEXT NOT NULL,
    origin_device_id TEXT NOT NULL
);
CREATE INDEX idx_tasks_list ON tasks(list_id, sort_key) WHERE deleted_at IS NULL;
CREATE INDEX idx_tasks_due ON tasks(due_at)
    WHERE due_at IS NOT NULL AND done = 0 AND deleted_at IS NULL;
CREATE INDEX idx_tasks_hlc ON tasks(hlc_ts);

CREATE TABLE tags (
    id               TEXT PRIMARY KEY NOT NULL,
    name             TEXT NOT NULL,
    color            TEXT,
    created_at       TEXT NOT NULL,
    updated_at       TEXT NOT NULL,
    deleted_at       TEXT,
    hlc_ts           TEXT NOT NULL,
    origin_device_id TEXT NOT NULL,
    UNIQUE (name)
);

CREATE TABLE note_tags (
    note_id TEXT NOT NULL REFERENCES notes(id) ON DELETE CASCADE,
    tag_id  TEXT NOT NULL REFERENCES tags(id)  ON DELETE CASCADE,
    PRIMARY KEY (note_id, tag_id)
);

CREATE TABLE list_tags (
    list_id TEXT NOT NULL REFERENCES lists(id) ON DELETE CASCADE,
    tag_id  TEXT NOT NULL REFERENCES tags(id)  ON DELETE CASCADE,
    PRIMARY KEY (list_id, tag_id)
);

CREATE TABLE reminders (
    id               TEXT PRIMARY KEY NOT NULL,
    target_type      TEXT NOT NULL CHECK (target_type IN ('note','task')),
    target_id        TEXT NOT NULL,
    due_at           TEXT NOT NULL,
    fired_at         TEXT,
    created_at       TEXT NOT NULL,
    updated_at       TEXT NOT NULL,
    deleted_at       TEXT,
    hlc_ts           TEXT NOT NULL,
    origin_device_id TEXT NOT NULL
);
CREATE INDEX idx_reminders_due ON reminders(due_at)
    WHERE deleted_at IS NULL AND fired_at IS NULL;

CREATE TABLE widget_instances (
    id               TEXT PRIMARY KEY NOT NULL,
    kind             TEXT NOT NULL CHECK (kind IN ('sticky','todo')),
    target_id        TEXT NOT NULL,
    win_x            INTEGER,
    win_y            INTEGER,
    win_w            INTEGER NOT NULL DEFAULT 280,
    win_h            INTEGER NOT NULL DEFAULT 320,
    opacity          REAL    NOT NULL DEFAULT 1.0,
    always_on_top    INTEGER NOT NULL DEFAULT 0,
    theme_override   TEXT,
    last_opened_at   TEXT,
    created_at       TEXT NOT NULL,
    updated_at       TEXT NOT NULL,
    deleted_at       TEXT,
    hlc_ts           TEXT NOT NULL,
    origin_device_id TEXT NOT NULL
);
CREATE INDEX idx_widget_instances_target
    ON widget_instances(kind, target_id) WHERE deleted_at IS NULL;

-- Single-row settings table (KV). Built-in keys reserved with `app.` prefix:
--   app.device_id          UUID of this device (stable, generated once)
--   app.theme.preference   'light'|'dark'|null (null = follow OS)
CREATE TABLE settings (
    key   TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);

-- Append-only journal of every mutation. MVP only writes to it (undo
-- could read it); a future sync transport will use it as the source of
-- truth for push/pull.
CREATE TABLE change_log (
    seq        INTEGER PRIMARY KEY AUTOINCREMENT,
    table_name TEXT NOT NULL,
    row_id     TEXT NOT NULL,
    op         TEXT NOT NULL CHECK (op IN ('insert','update','delete')),
    payload    TEXT,
    hlc_ts     TEXT NOT NULL,
    applied_at TEXT NOT NULL
);
CREATE INDEX idx_change_log_table_seq ON change_log(table_name, seq);

-- Full-text search. We index only live rows; triggers below keep these in
-- sync with the source tables. `content_rowid='rowid'` lets FTS5 piggyback
-- on the implicit rowid that SQLite assigns to every non-WITHOUT-ROWID table.
CREATE VIRTUAL TABLE notes_fts USING fts5(
    title, body,
    content='notes', content_rowid='rowid',
    tokenize='unicode61 remove_diacritics 2'
);
CREATE VIRTUAL TABLE lists_fts USING fts5(
    title,
    content='lists', content_rowid='rowid',
    tokenize='unicode61 remove_diacritics 2'
);
CREATE VIRTUAL TABLE tasks_fts USING fts5(
    title,
    content='tasks', content_rowid='rowid',
    tokenize='unicode61 remove_diacritics 2'
);

CREATE TRIGGER notes_ai AFTER INSERT ON notes WHEN new.deleted_at IS NULL BEGIN
    INSERT INTO notes_fts(rowid, title, body) VALUES (new.rowid, new.title, new.body);
END;
CREATE TRIGGER notes_ad AFTER DELETE ON notes BEGIN
    INSERT INTO notes_fts(notes_fts, rowid, title, body)
        VALUES('delete', old.rowid, old.title, old.body);
END;
CREATE TRIGGER notes_au AFTER UPDATE ON notes BEGIN
    INSERT INTO notes_fts(notes_fts, rowid, title, body)
        VALUES('delete', old.rowid, old.title, old.body);
    INSERT INTO notes_fts(rowid, title, body)
        SELECT new.rowid, new.title, new.body WHERE new.deleted_at IS NULL;
END;

CREATE TRIGGER lists_ai AFTER INSERT ON lists WHEN new.deleted_at IS NULL BEGIN
    INSERT INTO lists_fts(rowid, title) VALUES (new.rowid, new.title);
END;
CREATE TRIGGER lists_ad AFTER DELETE ON lists BEGIN
    INSERT INTO lists_fts(lists_fts, rowid, title)
        VALUES('delete', old.rowid, old.title);
END;
CREATE TRIGGER lists_au AFTER UPDATE ON lists BEGIN
    INSERT INTO lists_fts(lists_fts, rowid, title)
        VALUES('delete', old.rowid, old.title);
    INSERT INTO lists_fts(rowid, title)
        SELECT new.rowid, new.title WHERE new.deleted_at IS NULL;
END;

CREATE TRIGGER tasks_ai AFTER INSERT ON tasks WHEN new.deleted_at IS NULL BEGIN
    INSERT INTO tasks_fts(rowid, title) VALUES (new.rowid, new.title);
END;
CREATE TRIGGER tasks_ad AFTER DELETE ON tasks BEGIN
    INSERT INTO tasks_fts(tasks_fts, rowid, title)
        VALUES('delete', old.rowid, old.title);
END;
CREATE TRIGGER tasks_au AFTER UPDATE ON tasks BEGIN
    INSERT INTO tasks_fts(tasks_fts, rowid, title)
        VALUES('delete', old.rowid, old.title);
    INSERT INTO tasks_fts(rowid, title)
        SELECT new.rowid, new.title WHERE new.deleted_at IS NULL;
END;
