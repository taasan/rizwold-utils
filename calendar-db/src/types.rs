use core::{fmt, num::NonZeroU8};

use chrono::{DateTime, NaiveDate, Utc};
use rusqlite::{
    ToSql,
    types::{FromSql, FromSqlError, FromSqlResult, ToSqlOutput, ValueRef},
};
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize)]
pub struct Calendar {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_modified: DateTime<Utc>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Event {
    pub id: Uuid,
    pub calendar_id: Uuid,
    pub summary: String,
    pub description: Option<String>,
    pub url: Option<Url>,
    pub dtstart_initial: NaiveDate,
    pub duration_days: NonZeroU8,
    pub rrule: Option<rrule::RRule>,
    pub sequence: u32,
    pub created_at: DateTime<Utc>,
    pub last_modified: DateTime<Utc>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct EventException {
    pub id: Uuid,
    pub event_id: Uuid,
    pub original_date: NaiveDate,
    pub new_date: Option<NaiveDate>,
    pub new_summary: Option<String>,
    pub new_description: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct InvalidUrl;

impl fmt::Display for InvalidUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Invalid short URL")
    }
}

impl core::error::Error for InvalidUrl {}

impl From<FromSqlError> for InvalidUrl {
    fn from(_: FromSqlError) -> Self {
        Self
    }
}

impl From<url::ParseError> for InvalidUrl {
    fn from(_: url::ParseError) -> Self {
        Self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Url(url::Url);

impl From<Url> for url::Url {
    fn from(value: Url) -> Self {
        value.0
    }
}

impl<'a> From<&'a Url> for &'a url::Url {
    fn from(value: &'a Url) -> Self {
        &value.0
    }
}

impl fmt::Display for Url {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

fn is_http_or_https(url: &url::Url) -> bool {
    matches!(url.scheme(), "http" | "https")
}

fn has_password(url: &url::Url) -> bool {
    url.password().is_some()
}

fn has_username(url: &url::Url) -> bool {
    !url.username().is_empty()
}

impl TryFrom<&str> for Url {
    type Error = InvalidUrl;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        let url = url::Url::parse(s)?;
        if is_http_or_https(&url) && !has_password(&url) && !has_username(&url) {
            Ok(Self(url))
        } else {
            Err(InvalidUrl)
        }
    }
}

impl TryFrom<String> for Url {
    type Error = InvalidUrl;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl FromSql for Url {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let url = value.as_str()?;
        url.try_into().map_err(|_| FromSqlError::InvalidType)
    }
}

impl ToSql for Url {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.0.as_str()))
    }
}

/// Only values at or after unix epoch are valid
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct UnixTimestamp(pub u64);

impl UnixTimestamp {
    #[must_use]
    pub fn iso8601(self) -> Option<String> {
        let secs: i64 = self.0.try_into().ok()?;
        chrono::DateTime::from_timestamp(secs, 0)
            .map(|x| x.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))
    }
}

impl core::fmt::Display for UnixTimestamp {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

impl FromSql for UnixTimestamp {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        let i64_value = value.as_i64_or_null()?.unwrap_or_default();
        Ok(Self(
            i64_value
                .try_into()
                .map_err(|_| FromSqlError::OutOfRange(i64_value))?,
        ))
    }
}

impl ToSql for UnixTimestamp {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.0.to_string()))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_url_try_from_valid() {
        let result = Url::try_from("http://localhost/");
        assert!(result.is_ok());
    }

    #[test]
    fn test_url_try_from_invalid_scheme() {
        let result = Url::try_from("ftp://localhost/");
        assert!(result.is_err());
    }

    #[test]
    fn test_url_with_username() {
        let result = Url::try_from("http://user@localhost/");
        assert!(result.is_err());
    }

    #[test]
    fn test_url_with_password() {
        let result = Url::try_from("http://:pass@localhost/");
        assert!(result.is_err());
    }
}
