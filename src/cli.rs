/*
 * Copyright 2022-2025 Jochen Kupperschmidt
 * License: MIT
 */

use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Command-line arguments
#[derive(Parser, Debug)]
#[clap(about, author, version)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    /// Run the Verbleiber client
    Run {
        /// Specify configuration filename (e.g. `config.toml`)
        #[clap(short = 'c', long = "config")]
        config_filename: PathBuf,
    },
}

pub(crate) fn parse_cli() -> Cli {
    Cli::parse()
}
