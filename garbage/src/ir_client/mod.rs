//! Client for the Innherred Renovasjon WP API.
use core::fmt::{self, Debug, Display};

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
/// Represents an address.
///
/// ```
/// use garbage::ir_client::DisposalAddress;
/// let address = DisposalAddress::try_from("Blåveislia 12").unwrap();
/// assert_eq!(address.to_string(), "Blåveislia 12");
/// ```
pub struct DisposalAddress(String);

#[derive(Debug)]
/// A possible error when converting a [`DisposalAddress`] from a string.
pub struct InvalidAddress(&'static str);

impl Display for InvalidAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

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
