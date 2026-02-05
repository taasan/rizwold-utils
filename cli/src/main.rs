use core::error::Error;
use std::{env, ffi::OsString, process::ExitCode};

use clap::{Parser as ClapParser, Subcommand, ValueEnum};
use git_version::git_version;

const VERSION: &str = git_version!(
    prefix = "git:",
    cargo_prefix = "cargo:",
    fallback = "unknown"
);

#[derive(Debug, Clone, ValueEnum)]
enum OutputFormat {
    Ical,
    Json,
}

#[derive(ClapParser, Debug)]
#[clap(version = VERSION)]
#[command(name = "rizwold", multicall = true, about = "rizwold tools")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(external_subcommand)]
    Main(Vec<OsString>),
    Install,
    Garbage {
        #[command(subcommand)]
        command: garbage::Commands,
    },
}

fn handle_cli(cli: Cli) -> Result<(), Box<dyn Error>> {
    match cli.command {
        Commands::Install => todo!(),
        Commands::Garbage { command } => Ok(command.run()?),
        Commands::Main(args) => handle_cli(Cli::parse_from(args.iter().skip(1))),
    }
}

fn try_main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    tracing::debug!("Got CLI args: {cli:?}");
    handle_cli(cli)
    // match cli.command {
    //     Commands::Install => todo!(),
    //     Commands::Garbage { command } => Ok(command.run()?),
    //     Commands::Main(args) => {
    //         eprintln!("{args:#?}");
    //         todo!()
    //     },
    // }
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

    if let Some(dir) = &env::var_os("RIZWOLD_LOG_DIR") {
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
    let (non_blocking, guard) = tracing_appender::non_blocking(std::io::stderr());
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(fmt::layer().with_writer(non_blocking).with_ansi(false))
        .init();

    Some(guard)
}
