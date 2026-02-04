use core::convert::Infallible;
use core::error::Error;
use std::{env, io::Write, path::PathBuf, process::ExitCode};

use chrono::Utc;
use clap::{Parser as ClapParser, Subcommand, ValueEnum};
use git_version::git_version;

use garbage::{
    calendar::Calendar,
    io_error_to_string,
    ir_client::{
        DisposalAddress,
        schedule::{ApiResponse, DisposalDaysApi},
    },
};

const VERSION: &str = git_version!(
    prefix = "git:",
    cargo_prefix = "cargo:",
    fallback = "unknown"
);

#[allow(clippy::unnecessary_wraps)]
fn address_parser(value: &str) -> Result<DisposalAddress, Infallible> {
    Ok(value.into())
}

#[derive(ClapParser, Debug)]
struct CalendarArgs {
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
enum GarbageCommands {
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

#[derive(Debug, Clone, ValueEnum)]
enum OutputFormat {
    Ical,
    Json,
}

#[derive(ClapParser, Debug)]
#[clap(version = VERSION)]
#[command(
    name = "main-binary",
    multicall = true, // Navnet på binæren/linken bestemmer sub-enum
    about = "Multicall-verktøy med nøstede kommandoer"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Install,
    Garbage {
        #[command(subcommand)]
        command: GarbageCommands,
    },
}

fn handle_cmd(endpoint: &DisposalDaysApi, args: CalendarArgs) -> Result<(), Box<dyn Error>> {
    let output = match args.format {
        OutputFormat::Ical => {
            let response: ApiResponse = endpoint.get(&args.address)?;
            tracing::debug!("Got: {response:?}");
            let created = Utc::now();
            let fractions = response.into_values().collect();
            let cal = Calendar::new(fractions, args.address, Some(created));
            tracing::info!("Exported {} calendar events", cal.count());
            format!("{cal}")
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

fn try_main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    tracing::debug!("Got CLI args: {cli:?}");
    let (endpoint, args) = match cli.command {
        Commands::Install => todo!(),
        Commands::Garbage { command } => match command {
            GarbageCommands::Api { args } => (DisposalDaysApi::api(), args),
            GarbageCommands::File { input, args } => (DisposalDaysApi::file(input), args),
        },
    };

    handle_cmd(&endpoint, args)
}

fn main() -> ExitCode {
    let _logger_guard = init_logging();
    tracing::info!("garbage ({VERSION}): Creating garbage disposal calendar");

    match try_main() {
        Ok(()) => {
            tracing::info!("Success");
            ExitCode::SUCCESS
        }
        Err(err) => {
            tracing::error!("{err}");
            ExitCode::FAILURE
        }
    }
}

fn init_logging() -> Option<tracing_appender::non_blocking::WorkerGuard> {
    use std::fs::create_dir_all;
    use tracing_appender::rolling::{RollingFileAppender, Rotation};
    use tracing_subscriber::{EnvFilter, fmt, prelude::*};

    if let Some(dir) = &env::var_os("GARBAGE_LOG_DIR") {
        if let Err(err) = create_dir_all(dir) {
            eprintln!("Unable to initialize logging to file: {err}");
        } else {
            match RollingFileAppender::builder()
                .rotation(Rotation::WEEKLY)
                .max_log_files(8)
                .filename_prefix("garbage.log")
                .build(dir)
            {
                Err(err) => {
                    eprintln!(
                        "Unable to initialize logging in directory: {}",
                        dir.display()
                    );
                    eprintln!("Cause: {err}");
                }
                Ok(file_appender) => {
                    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

                    tracing_subscriber::registry()
                        .with(EnvFilter::from_default_env())
                        .with(fmt::layer().with_writer(non_blocking).with_ansi(false))
                        .init();
                    return Some(guard);
                }
            }
        }
    }
    None
}
