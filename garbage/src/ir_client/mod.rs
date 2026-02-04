//! Client for the Innherred Renovasjon WP API.
use core::fmt::{self, Debug, Display};

use serde::Serialize;

/// Represents an address.
#[derive(Debug, Clone, Serialize)]
pub struct DisposalAddress(String);

impl<'a> From<&'a str> for DisposalAddress {
    fn from(value: &'a str) -> Self {
        Self(value.to_string())
    }
}

impl Display for DisposalAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

pub mod schedule;
