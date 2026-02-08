SELECT
    id,
    event_id,
    original_date,
    new_date,
    NULLIF(TRIM(new_summary), '') AS new_summary,
    NULLIF(TRIM(new_description), '') AS new_description
FROM event_exceptions
