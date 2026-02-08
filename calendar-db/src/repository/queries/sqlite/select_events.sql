SELECT
    id,
    calendar_id,
    NULLIF(TRIM(summary), '') AS summary,
    NULLIF(TRIM(description), '') AS description,
    NULLIF(TRIM(url), '') AS url,
    dtstart_initial,
    duration_days,
    rrule,
    sequence,
    created_at,
    last_modified
FROM events
