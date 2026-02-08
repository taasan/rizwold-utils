-- 1. Tabell for ulike kalendere
CREATE TABLE calendars (
    id TEXT PRIMARY KEY, -- UUIDv7
    name TEXT NOT NULL CHECK (LENGTH(name) > 0),
    description TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_modified TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
) WITHOUT ROWID;

-- 2. Tabell for eventer (Hovedregler)
CREATE TABLE events (
    id TEXT PRIMARY KEY, -- UUIDv7
    calendar_id TEXT NOT NULL, -- UUIDv7
    summary TEXT NOT NULL CHECK (LENGTH(summary) > 0),
    description TEXT NOT NULL DEFAULT '',
    url TEXT,
    dtstart_initial TEXT NOT NULL,
    duration_days INTEGER NOT NULL DEFAULT 1 CHECK (duration_days > 0),
    rrule TEXT,
    sequence INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_modified TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (calendar_id) REFERENCES calendars (id) ON DELETE CASCADE
) WITHOUT ROWID;

-- 3. Tabell for unntak (Flytting eller avlysning av enkeltår)
CREATE TABLE event_exceptions (
    id TEXT PRIMARY KEY, -- UUIDv7
    event_id TEXT NOT NULL, -- UUIDv7
    original_date TEXT NOT NULL,
    new_date TEXT,
    new_summary TEXT,
    new_description TEXT,
    CHECK (new_date <> original_date),
    UNIQUE (event_id, original_date),
    FOREIGN KEY (event_id) REFERENCES events (id) ON DELETE CASCADE
) WITHOUT ROWID;


-- 4. Trigger: Oppdaterer sequence og last_modified ved endring i
-- event-tabellen
CREATE TRIGGER trg_events_update
AFTER UPDATE ON events
FOR EACH ROW
WHEN (new.last_modified IS old.last_modified)
BEGIN
    UPDATE events
    SET
        sequence = old.sequence + 1,
        last_modified = CURRENT_TIMESTAMP
    WHERE
        id = old.id;

    -- Oppdater også kalenderens last_modified
    UPDATE calendars
    SET
        last_modified = CURRENT_TIMESTAMP
    WHERE
        id = old.calendar_id;
END;


CREATE TRIGGER trg_exception_delete
AFTER DELETE ON event_exceptions
FOR EACH ROW
BEGIN
    UPDATE events
    SET
        sequence = sequence + 1,
        last_modified = CURRENT_TIMESTAMP
    WHERE id = old.event_id;
END;


CREATE TRIGGER trg_calendars_update AFTER
UPDATE ON calendars FOR EACH ROW BEGIN
    UPDATE calendars
    SET
        last_modified = CURRENT_TIMESTAMP
    WHERE
        id = new.id;

END;

-- 5. Håndtering av unntak (Insert/Update/Delete)
-- Disse trenger ikke WHEN fordi de endrer en annen tabell (events)
CREATE TRIGGER trg_exception_insert
AFTER INSERT ON event_exceptions
FOR EACH ROW
BEGIN
    UPDATE events
    SET
        sequence = sequence + 1,
        last_modified = CURRENT_TIMESTAMP
    WHERE
        id = new.event_id;
END;

CREATE TRIGGER trg_exception_update
AFTER UPDATE ON event_exceptions
FOR EACH ROW
BEGIN
    UPDATE events
    SET
        sequence = sequence + 1,
        last_modified = CURRENT_TIMESTAMP
    WHERE
        id = new.event_id;
END;

CREATE INDEX idx_events_calendar ON events (calendar_id);

CREATE INDEX idx_exceptions_event ON event_exceptions (event_id);

CREATE UNIQUE INDEX idx_event_exceptions_ordering
ON event_exceptions (event_id, original_date ASC);
