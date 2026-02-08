use core::result::Result;
use std::path::Path;

use anyhow::Context as _;
use chrono::{DateTime, NaiveDate};
use rusqlite::{Connection, OpenFlags, OptionalExtension as _, TransactionBehavior};
use tracing::error;
use uuid::Uuid;

use crate::types::{Calendar, Event, EventException};

use super::{Repository, WritableRepository};

#[derive(Debug)]
pub(crate) struct Sqlite3Repo {
    conn: Connection,
}

impl Sqlite3Repo {
    pub(crate) const fn new(conn: rusqlite::Connection) -> Self {
        Self { conn }
    }

    /// # Errors
    ///
    /// Will return `Err` if `path` cannot be converted to a C-compatible
    /// string or if the underlying SQLite open call fails.
    pub(crate) fn open<P: AsRef<Path>>(
        path: P,
        flags: Option<OpenFlags>,
    ) -> Result<Self, anyhow::Error> {
        let conn = Connection::open_with_flags(path, flags.unwrap_or_default())?;
        Ok(Self::new(conn))
    }
}

impl Repository for Sqlite3Repo {
    fn has_latest_migrations(&self) -> Result<bool, anyhow::Error> {
        let migrations = migrations();
        let user_version: u32 =
            self.conn
                .query_row("SELECT user_version FROM pragma_user_version", [], |row| {
                    row.get(0)
                })?;
        Ok(user_version as usize == migrations.len())
    }

    fn get_calendar(&self, id: uuid::Uuid) -> anyhow::Result<Option<Calendar>> {
        let query =
            "SELECT id, name, description, created_at, last_modified FROM calendars WHERE id = ?";
        Ok(self
            .conn
            .query_row(query, rusqlite::params![id.to_string()], |row| {
                Ok(Calendar {
                    id,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    created_at: row.get(3)?,
                    last_modified: row.get(4)?,
                })
            })
            .optional()?)
    }

    fn for_each_calendar<F>(&self, mut callback: F) -> anyhow::Result<()>
    where
        F: FnMut(Calendar) -> anyhow::Result<()>,
    {
        let query = "SELECT id, name, description, created_at, last_modified FROM calendars";
        let mut stmt = self.conn.prepare(query)?;
        let rows = stmt.query_map([], |row| {
            let id_str: String = row.get(0)?;
            let id = uuid::Uuid::parse_str(&id_str)
                .with_context(|| "Kunne ikke hente kolonne 0")
                .map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Text,
                        e.into(),
                    )
                })?;
            Ok(Calendar {
                id,
                name: row.get(1)?,
                description: row.get(2)?,
                created_at: row.get(3)?,
                last_modified: row.get(4)?,
            })
        })?;
        for row in rows {
            match row {
                Ok(row) => callback(row)?,
                Err(err) => {
                    error!("Failed to get calendar: {err}");
                }
            }
        }
        Ok(())
    }

    fn for_each_event<F>(&self, calendar_id: Option<Uuid>, mut callback: F) -> anyhow::Result<()>
    where
        F: FnMut(Event) -> anyhow::Result<()>,
    {
        let mut query = include_str!("queries/sqlite/select_events.sql").to_string();
        #[allow(clippy::option_if_let_else)]
        let params = match calendar_id {
            Some(id) => {
                query += " WHERE calendar_id = ?";
                rusqlite::params![id.to_string()]
            }
            None => rusqlite::params![],
        };
        let mut stmt = self.conn.prepare(&query)?;
        let rows = stmt.query_map(params, |row| {
            let str_val: String = row.get(0)?;
            let id = uuid::Uuid::parse_str(&str_val)
                .with_context(|| "Kunne ikke hente kolonne 0")
                .map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Text,
                        e.into(),
                    )
                })?;
            let str_val: String = row.get(1)?;
            let calendar_id = uuid::Uuid::parse_str(&str_val)
                // .with_context(|| "Kunne ikke hente kolonne 1")
                .map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Text,
                        e.into(),
                    )
                })?;
            let dtstart_initial: NaiveDate = row.get(5)?;
            let naive_datetime = dtstart_initial
                .and_hms_opt(0, 0, 0)
                .expect("This should not happen");
            let rrule_dtstart: DateTime<rrule::Tz> =
                naive_datetime.and_local_timezone(rrule::Tz::LOCAL).unwrap();

            let str_val: Option<String> = row.get(7)?;
            let rrule = str_val.map_or_else(
                || None,
                |str_val| {
                    let str_val = str_val.trim();
                    if str_val.is_empty() {
                        return None;
                    }
                    match str_val.parse::<rrule::RRule<rrule::Unvalidated>>() {
                        Ok(x) => match x.validate(rrule_dtstart) {
                            Ok(x) => Some(x),
                            Err(err) => {
                                error!("Unable to read RRULE {err}");
                                None
                            }
                        },
                        Err(err) => {
                            error!("Unable to read RRULE {err}");
                            None
                        }
                    }
                },
            );
            Ok(Event {
                id,
                calendar_id,
                summary: row.get(2)?,
                description: row.get(3)?,
                url: row.get(4)?,
                dtstart_initial,
                duration_days: row.get(6)?,
                rrule,
                sequence: row.get(8)?,
                created_at: row.get(9)?,
                last_modified: row.get(10)?,
            })
        })?;
        for row in rows {
            match row {
                Ok(row) => callback(row)?,
                Err(err) => {
                    error!("Failed to get calendar: {err}");
                }
            }
        }
        Ok(())
    }

    fn for_each_event_exceptions<F>(
        &self,
        event_id: Option<Uuid>,
        mut callback: F,
    ) -> anyhow::Result<()>
    where
        F: FnMut(crate::types::EventException) -> anyhow::Result<()>,
    {
        let mut query = include_str!("queries/sqlite/select_event_exceptions.sql").to_string();
        #[allow(clippy::option_if_let_else)]
        let params = match event_id {
            Some(id) => {
                query += " WHERE event_id = ?";
                rusqlite::params![id.to_string()]
            }
            None => rusqlite::params![],
        };
        query += " ORDER BY original_date ASC";
        let mut stmt = self.conn.prepare(&query)?;
        let rows = stmt.query_map(params, |row| {
            let str_val: String = row.get(0)?;
            let id = uuid::Uuid::parse_str(&str_val)
                .with_context(|| "Kunne ikke hente kolonne 0")
                .map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Text,
                        e.into(),
                    )
                })?;
            let str_val: String = row.get(1)?;
            let event_id = uuid::Uuid::parse_str(&str_val)
                .with_context(|| "Kunne ikke hente kolonne 1")
                .map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Text,
                        e.into(),
                    )
                })?;
            Ok(EventException {
                id,
                event_id,
                original_date: row.get(2)?,
                new_date: row.get(3)?,
                new_summary: row.get(4)?,
                new_description: row.get(5)?,
            })
        })?;
        for row in rows {
            match row {
                Ok(row) => callback(row)?,
                Err(err) => {
                    error!("Failed to get calendar: {err}");
                }
            }
        }
        Ok(())
    }
}

#[inline]
const fn migrations() -> [&'static str; 1] {
    [include_str!("migrations/sqlite/1.up.sql")]
}

impl WritableRepository for Sqlite3Repo {
    fn migrate(&mut self) -> Result<(), anyhow::Error> {
        // EXCLUSIVE ensures that it starts with an exclusive write lock. No other
        // readers will be allowed. This generally shouldn't be needed if there is
        // a file lock, but might be helpful in cases where cargo's `FileLock`
        // failed.
        let migrations = migrations();
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Exclusive)?;
        let user_version: u32 =
            tx.query_row("SELECT user_version FROM pragma_user_version", [], |row| {
                row.get(0)
            })?;
        if (user_version as usize) < migrations.len() {
            for migration in &migrations[(user_version as usize)..] {
                tx.execute_batch(migration)?;
            }
            tx.pragma_update(None, "user_version", u32::try_from(migrations.len())?)?;
        }
        tx.commit()?;
        Ok(())
    }
}

/// # Errors
///
/// Will return `Err` if `path` cannot be converted to a C-compatible
/// string or if the underlying SQLite open call fails.
pub fn open_readonly_repository<P: AsRef<Path>>(path: P) -> Result<impl Repository, anyhow::Error> {
    Sqlite3Repo::open(path, Some(OpenFlags::SQLITE_OPEN_READ_ONLY))
}

/// # Errors
///
/// Will return `Err` if `path` cannot be converted to a C-compatible
/// string or if the underlying SQLite open call fails.
pub fn open_writable_repository<P: AsRef<Path>>(
    path: P,
) -> Result<impl WritableRepository, anyhow::Error> {
    Sqlite3Repo::open(path, None)
}

/// # Errors
///
/// Will return `Err` if the underlying SQLite open call fails.
#[doc(hidden)]
pub fn open_writable_in_memory_repository() -> Result<impl WritableRepository, anyhow::Error> {
    Ok(Sqlite3Repo::new(rusqlite::Connection::open_in_memory()?))
}

// #[cfg(test)]
// mod test {
//     use rusqlite::Connection;
//
//     use super::Sqlite3Repo;
//     use crate::repository::WritableRepository;
//
//     fn repo() -> Sqlite3Repo {
//         let mut repo = Sqlite3Repo::new(Connection::open_in_memory().unwrap());
//         repo.migrate().unwrap();
//         repo
//     }
// }
