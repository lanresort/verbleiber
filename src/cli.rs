/*
 * Copyright 2022-2023 Jochen Kupperschmidt
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

    /// Specify RFID/barcode reader input device (e.g. `/dev/input/event23`)
    #[clap(short = 'r', long = "reader-input-device")]
    pub reader_input_device: String,

    /// Specify button input device (e.g. `/dev/input/event24`)
    #[clap(short = 'b', long = "button-input-device")]
    pub button_input_device: String,
}

pub(crate) fn parse_args() -> Args {
    Args::parse()
}
