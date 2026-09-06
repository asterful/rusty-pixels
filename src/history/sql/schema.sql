CREATE TABLE IF NOT EXISTS event_types (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);

INSERT OR IGNORE INTO event_types (id, name) VALUES (0, 'PAINT'), (1, 'RESIZE'), (2, 'ROLLBACK'), (3, 'INIT');


CREATE TABLE IF NOT EXISTS events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_type_id INTEGER NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP NOT NULL,
    FOREIGN KEY(event_type_id) REFERENCES event_types(id)
);

CREATE TABLE IF NOT EXISTS paint_event (
    event_id INTEGER PRIMARY KEY,
    x INTEGER NOT NULL,
    y INTEGER NOT NULL,
    color_hex TEXT NOT NULL,
    FOREIGN KEY(event_id) REFERENCES events(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS resize_event (
    event_id INTEGER PRIMARY KEY,
    width INTEGER NOT NULL,
    height INTEGER NOT NULL,
    anchor_type INTEGER NOT NULL,
    FOREIGN KEY(event_id) REFERENCES events(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS rollback_event (
    event_id INTEGER PRIMARY KEY,
    target_event_id INTEGER NOT NULL,
    CONSTRAINT no_self_rollback CHECK (event_id <> target_event_id),
    FOREIGN KEY(event_id) REFERENCES events(id) ON DELETE CASCADE,
    FOREIGN KEY(target_event_id) REFERENCES events(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    last_event_id INTEGER NOT NULL,
    width INTEGER NOT NULL,
    height INTEGER NOT NULL,
    canvas_blob BLOB NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(last_event_id) REFERENCES events(id) ON DELETE CASCADE
);

-- 5. PERFORMANCE: Critical indexing for fast reconstruction.
CREATE INDEX IF NOT EXISTS idx_events_type ON events(event_type_id);
CREATE INDEX IF NOT EXISTS idx_snapshots_event ON snapshots(last_event_id);

-- 6. SECURITY: Locks the lookup table to prevent runtime modifications.
CREATE TRIGGER IF NOT EXISTS lock_event_types_ins BEFORE INSERT ON event_types BEGIN SELECT RAISE(FAIL, 'event_types is read-only'); END;
CREATE TRIGGER IF NOT EXISTS lock_event_types_upd BEFORE UPDATE ON event_types BEGIN SELECT RAISE(FAIL, 'event_types is read-only'); END;
CREATE TRIGGER IF NOT EXISTS lock_event_types_del BEFORE DELETE ON event_types BEGIN SELECT RAISE(FAIL, 'event_types is read-only'); END;