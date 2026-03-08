use std::process::ExitCode;

use clap::{error::ErrorKind, Parser};

mod cli;

#[cfg(not(any(feature = "mf289f", feature = "gt5s")))]
compile_error!("the CLI requires at least one router model feature: mf289f or gt5s");

#[tokio::main]
async fn main() -> ExitCode {
    let cli = match cli::args::Cli::try_parse() {
        Ok(cli) => cli,
        Err(err) => {
            let code = if matches!(err.kind(), ErrorKind::DisplayHelp | ErrorKind::DisplayVersion) {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            };
            let _ = err.print();
            return code;
        }
    };

    match cli::run(cli).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err:#}");
            ExitCode::FAILURE
        }
    }
}
