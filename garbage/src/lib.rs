//! Create iCalendar file for Innherred Renovasjon garbage pickup dates.
use core::{convert::Infallible, error::Error};
use std::{
    io::{self, Write},
    path::{Path, PathBuf},
};

use chrono::Utc;
use clap::{Parser as ClapParser, Subcommand, ValueEnum};
use url::Url;
use uuid::Uuid;

use crate::{
    calendar::Calendar,
    ir_client::{
        DisposalAddress,
        schedule::{ApiResponse, DisposalDaysApi},
    },
};

pub(crate) mod calendar;
pub(crate) mod ir_client;

#[inline]
#[must_use]
pub(crate) fn io_error_to_string(err: &io::Error, path: &Path) -> String {
    format!("{err}: {}", path.display())
}

#[allow(clippy::unnecessary_wraps)]
fn address_parser(value: &str) -> Result<DisposalAddress, Infallible> {
    Ok(value.into())
}

#[derive(Debug, Clone, ValueEnum)]
enum OutputFormat {
    Ical,
    Json,
}

#[derive(ClapParser, Debug)]
pub struct CalendarArgs {
    #[arg(long, value_parser = address_parser)]
    /// Address
    address: DisposalAddress,
    #[arg(long)]
    /// File path, print to stdout if omitted
    output: Option<PathBuf>,
    /// Output format
    #[arg(value_enum, long, default_value_t = OutputFormat::Ical)]
    format: OutputFormat,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Get delivery dates from Innherred Renovasjon
    Api {
        #[clap(flatten)]
        args: CalendarArgs,
    },
    /// Get delivery dates from JSON file
    File {
        #[clap(flatten)]
        args: CalendarArgs,
        /// File path, read from stdin of omitted
        input: Option<PathBuf>,
    },
}

impl Commands {
    #[allow(clippy::missing_panics_doc)]
    #[allow(clippy::missing_errors_doc)]
    pub fn run(self) -> Result<(), Box<dyn Error>> {
        const NAMESPACE: Uuid = uuid::uuid!("769d988a-38ee-48b1-908c-5d58c0982349");
        let (endpoint, args) = match self {
            Self::Api { args } => (DisposalDaysApi::api(), args),
            Self::File { input, args } => (DisposalDaysApi::file(input), args),
        };

        let output = match args.format {
            OutputFormat::Ical => {
                let response: ApiResponse = endpoint.get(&args.address)?;
                tracing::debug!("Got: {response:?}");
                let created = Utc::now();
                let fractions = response.into_values().collect();
                let url = Url::parse("https://innherredrenovasjon.no/tommeplan/")
                    .expect("Should never happen");
                let cal: ::calendar::Calendar =
                    Calendar::new(NAMESPACE, fractions, args.address, created, url).into();
                tracing::info!("Exported {} calendar events", cal.events.len());
                match args.output {
                    Some(path) => {
                        let file = std::fs::File::create(&path)
                            .map_err(|err| io_error_to_string(&err, &path))?;
                        cal.write(file)
                            .map_err(|err| io_error_to_string(&err, &path))?;
                    }

                    None => {
                        cal.write(std::io::stdout())?;
                    }
                }
                return Ok(());
            }
            OutputFormat::Json => {
                let response: serde_json::Value = endpoint.get(&args.address)?;
                tracing::debug!("Got: {response:?}");
                serde_json::to_string(&response)?
            }
        };
        match args.output {
            Some(path) => {
                // Try to create file before we do any network requests
                let mut file =
                    std::fs::File::create(&path).map_err(|err| io_error_to_string(&err, &path))?;
                write!(file, "{output}").map_err(|err| io_error_to_string(&err, &path))?;
            }
            None => std::io::stdout().write_fmt(format_args!("{output}"))?,
        }
        Ok(())
    }
}
