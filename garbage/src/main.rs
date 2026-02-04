use core::error::Error;
use std::process::ExitCode;

use clap::Parser as ClapParser;

use garbage::Commands;

#[derive(Debug, ClapParser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

fn try_main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    tracing::debug!("Got CLI args: {cli:?}");
    cli.command.run()
}

fn main() -> ExitCode {
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
