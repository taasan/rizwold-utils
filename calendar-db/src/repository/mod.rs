use uuid::Uuid;

use crate::types::{Calendar, Event, EventException};

pub mod sqlite;

pub trait Repository {
    /// # Errors
    ///
    /// May return an error if the query fails.
    fn get_calendar(&self, id: Uuid) -> anyhow::Result<Option<Calendar>>;

    /// # Errors
    ///
    /// May return an error if the query fails.
    fn for_each_calendar<F>(&self, callback: F) -> anyhow::Result<()>
    where
        F: FnMut(Calendar) -> anyhow::Result<()>;

    /// # Errors
    ///
    /// May return an error if the query fails.
    fn for_each_event<F>(&self, calendar_id: Option<Uuid>, callback: F) -> anyhow::Result<()>
    where
        F: FnMut(Event) -> anyhow::Result<()>;
    /// # Errors
    ///
    /// May return an error if the query fails.
    fn for_each_event_exceptions<F>(
        &self,
        event_id: Option<Uuid>,
        callback: F,
    ) -> anyhow::Result<()>
    where
        F: FnMut(EventException) -> anyhow::Result<()>;

    /// # Errors
    /// May return a `RepositoryError` if database communication fails.
    fn has_latest_migrations(&self) -> Result<bool, anyhow::Error>;
}

#[allow(clippy::module_name_repetitions)]
pub trait WritableRepository: Repository {
    /// # Errors
    ///
    /// May return a `RepositoryError` if the migration fails.
    fn migrate(&mut self) -> Result<(), anyhow::Error>;
}
