/*
 * Copyright 2022-2025 Jochen Kupperschmidt
 * License: MIT
 */

use anyhow::Result;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
pub(crate) struct Config {
    pub reader_input_device: String,
    pub button_input_device: String,
    pub sounds_path: PathBuf,
    pub api: ApiConfig,
    pub party: PartyConfig,
}

#[derive(Deserialize)]
pub(crate) struct ApiConfig {
    pub base_url: String,
    pub client_token: String,
    pub tls_verify: bool,
    pub timeout_in_seconds: u64,
}

#[derive(Deserialize)]
pub(crate) struct PartyConfig {
    pub party_id: String,
    pub buttons_to_whereabouts: HashMap<String, String>,
    pub whereabouts_sounds: HashMap<String, Vec<String>>,
}

pub(crate) fn load_config(path: &Path) -> Result<Config> {
    let text = read_to_string(path)?;
    let config: Config = toml::from_str(&text)?;
    Ok(config)
}
