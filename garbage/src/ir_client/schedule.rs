//! Garbage disposal dates API.

use core::fmt::Debug;
use std::path::PathBuf;
extern crate alloc;
use alloc::collections::BTreeMap;

use chrono::NaiveDateTime;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use ureq::Agent;

use crate::{io_error_to_string, ir_client::DisposalAddress};

pub type ApiResponse = BTreeMap<String, GarbageFraction>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GarbageFraction {
    pub fraction_id: String,
    pub fraction_name: String,
    // weeks
    pub frequency: u8,
    // pub frequency_human: Option<String>,
    pub dates: Vec<NaiveDateTime>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WasteFraction {
    FoodWaste,               // 1111
    PlasticPackaging,        // 4
    GlassMetal,              // 5
    PaperCardboard,          // 1222
    ResidualWaste,           // 9992
    Unknown(String, String), // (ID, Navn)
}

impl From<GarbageFraction> for WasteFraction {
    fn from(value: GarbageFraction) -> Self {
        Self::from_api(&value.fraction_id, &value.fraction_name)
    }
}

impl From<&GarbageFraction> for WasteFraction {
    fn from(value: &GarbageFraction) -> Self {
        Self::from_api(&value.fraction_id, &value.fraction_name)
    }
}

impl WasteFraction {
    #[must_use]
    pub fn from_api(id: &str, name: &str) -> Self {
        match id {
            "1111" => Self::FoodWaste,
            "4" => Self::PlasticPackaging,
            "5" => Self::GlassMetal,
            "1222" => Self::PaperCardboard,
            "9992" => Self::ResidualWaste,
            _ => Self::Unknown(id.to_string(), name.to_string()),
        }
    }

    #[must_use]
    pub fn get_id(&self) -> String {
        match self {
            Self::FoodWaste => "1111".to_string(),
            Self::PlasticPackaging => "4".to_string(),
            Self::GlassMetal => "5".to_string(),
            Self::PaperCardboard => "1222".to_string(),
            Self::ResidualWaste => "9992".to_string(),
            Self::Unknown(id, _) => id.clone(),
        }
    }

    #[must_use]
    pub fn name(&self) -> String {
        match self {
            Self::FoodWaste => "Matavfall".to_string(),
            Self::PlasticPackaging => "Plastemballasje".to_string(),
            Self::GlassMetal => "Glass- og metallemballasje".to_string(),
            Self::PaperCardboard => "Papp/papir".to_string(),
            Self::ResidualWaste => "Restavfall".to_string(),
            Self::Unknown(name, _) => name.clone(),
        }
    }
    #[must_use]
    pub const fn icon(&self) -> &'static str {
        match self {
            Self::FoodWaste => "🍌",
            Self::PlasticPackaging => "♻️",
            Self::GlassMetal => "🥫",
            Self::PaperCardboard => "🧃",
            Self::ResidualWaste => "🗑️",
            Self::Unknown(_, _) => "❓",
        }
    }
}

/// Disposal day provider.
pub enum DisposalDaysApi {
    /// Fetches JSON from IR WP API.
    Api(Agent),

    /// Reads JSON from a file.
    File(Option<PathBuf>),
}

impl DisposalDaysApi {
    /// Read dates from REST API.
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn api() -> Self {
        let config = Agent::config_builder()
            .https_only(true)
            .accept("application/json")
            .build();
        tracing::debug!("Constructing HTTP agent with config: {config:?}");
        Self::Api(config.into())
    }

    #[must_use]
    /// Read dates from file.
    pub const fn file(path: Option<PathBuf>) -> Self {
        Self::File(path)
    }

    /// Get a list of delivery dates.
    #[allow(clippy::missing_errors_doc)]
    pub fn get<T: DeserializeOwned>(
        &self,
        address: &DisposalAddress,
    ) -> Result<T, Box<dyn core::error::Error>> {
        let response: T = match self {
            Self::Api(client) => {
                let url = "https://innherredrenovasjon.no/wp-json/ir/v1/garbage-disposal-dates-by-address";
                tracing::debug!("Reading from url: {url}");
                client
                    .get(url)
                    .query("address", &address.0)
                    .call()?
                    .body_mut()
                    .read_json()?
            }
            Self::File(Some(path)) => {
                tracing::debug!("Reading from file: {}", path.display());
                serde_json::from_reader(
                    std::fs::File::open(path).map_err(|err| io_error_to_string(&err, path))?,
                )?
            }
            Self::File(None) => {
                tracing::debug!("Reading from stdin");
                serde_json::from_reader(std::io::stdin())?
            }
        };
        Ok(response)
    }
}
