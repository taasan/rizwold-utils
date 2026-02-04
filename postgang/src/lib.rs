//! Create iCalendar file for norwegian mailbox delivery dates.
use core::error::Error;
use std::{
    fs::File,
    io::{self, Write as _, stdout},
    path::{Path, PathBuf},
};

use clap::{Parser as ClapParser, ValueEnum};

use crate::bring_client::mailbox_delivery_dates::DeliveryDays;
use crate::bring_client::{ApiKey, ApiUid, NorwegianPostalCode};

pub mod bring_client;
pub mod calendar;

#[inline]
#[must_use]
pub fn io_error_to_string(err: &io::Error, path: &Path) -> String {
    format!("{err}: {}", path.display())
}

fn postal_code_parser(value: &str) -> Result<NorwegianPostalCode, String> {
    NorwegianPostalCode::try_from(value).map_err(|err| err.to_string())
}

fn parse_api_key(value: &str) -> Result<ApiKey, String> {
    ApiKey::try_from(value).map_err(|err| format!("{err:?}"))
}

fn parse_api_uid(value: &str) -> Result<ApiUid, String> {
    ApiUid::try_from(value).map_err(|err| format!("{err:?}"))
}

#[derive(ClapParser, Debug)]
pub enum Commands {
    /// Get delivery dates from Bring API
    Api {
        #[clap(flatten)]
        args: CalendarArgs,
        #[arg(long, env = "POSTGANG_API_UID", value_parser = parse_api_uid, hide_env_values = true)]
        api_uid: ApiUid,
        #[arg(long, env = "POSTGANG_API_KEY", value_parser = parse_api_key, hide_env_values = true)]
        api_key: ApiKey,
    },
    /// Get delivery dates from JSON file
    File {
        #[clap(flatten)]
        args: CalendarArgs,
        /// File path, read from stdin of omitted
        input: Option<PathBuf>,
    },
}

#[derive(Debug, Clone, ValueEnum)]
enum OutputFormat {
    Ical,
    Json,
}

#[derive(ClapParser, Debug)]
pub struct CalendarArgs {
    #[arg(long, value_parser = postal_code_parser)]
    /// Postal code
    code: NorwegianPostalCode,
    #[arg(long)]
    /// File path, print to stdout if omitted
    output: Option<PathBuf>,
    /// Output format
    #[arg(value_enum, long, default_value_t = OutputFormat::Ical)]
    format: OutputFormat,
}

#[derive(ClapParser, Debug)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

impl Commands {
    #[allow(clippy::missing_panics_doc)]
    #[allow(clippy::missing_errors_doc)]
    pub fn run(self) -> Result<(), Box<dyn Error>> {
        let (endpoint, args) = match self {
            Self::Api {
                args,
                api_uid,
                api_key,
            } => (DeliveryDays::api(api_key, api_uid), args),
            Self::File { input, args } => (DeliveryDays::file(input), args),
        };

        let output = match args.format {
            OutputFormat::Ical => {
                let cal = endpoint.get_calendar(args.code)?;

                match args.output {
                    Some(path) => {
                        let file =
                            File::create(&path).map_err(|err| io_error_to_string(&err, &path))?;
                        cal.write(file)
                            .map_err(|err| io_error_to_string(&err, &path))?;
                    }

                    None => {
                        cal.write(stdout())?;
                    }
                }
                return Ok(());
            }

            OutputFormat::Json => {
                let response: serde_json::Value = endpoint.get(args.code)?;
                tracing::debug!("Got: {response:?}");
                serde_json::to_string(&response)?
            }
        };

        match args.output {
            Some(path) => {
                // Try to create file before we do any network requests
                let mut file =
                    File::create(&path).map_err(|err| io_error_to_string(&err, &path))?;
                write!(file, "{output}").map_err(|err| io_error_to_string(&err, &path))?;
            }

            None => stdout().write_fmt(format_args!("{output}"))?,
        }

        Ok(())
    }
}
