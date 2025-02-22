/*
 * Copyright 2022-2025 Jochen Kupperschmidt
 * License: MIT
 */

use clap::Parser;
use std::path::PathBuf;

/// Command-line arguments
#[derive(Parser, Debug)]
#[clap(about, author, version)]
pub(crate) struct Args {
    /// Specify configuration filename (e.g. `config.toml`)
    #[clap(short = 'c', long = "config")]
    pub config_filename: PathBuf,
}

pub(crate) fn parse_args() -> Args {
    Args::parse()
}
