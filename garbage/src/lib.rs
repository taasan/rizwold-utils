//! Create iCalendar file for Innherred Renovasjon garbage pickup dates.
use std::io;
use std::path::Path;

pub mod calendar;
pub mod ir_client;

#[inline]
#[must_use]
pub fn io_error_to_string(err: &io::Error, path: &Path) -> String {
    format!("{err}: {}", path.display())
}
