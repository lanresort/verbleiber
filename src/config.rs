/*
 * Copyright 2022-2024 Jochen Kupperschmidt
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
    pub buttons_to_whereabouts: HashMap<String, String>,
    pub whereabouts_sounds: HashMap<String, Vec<String>>,
}

#[derive(Deserialize)]
pub(crate) struct ApiConfig {
    pub base_url: String,
    pub auth_token: String,
    pub party_id: String,
    pub timeout_in_seconds: u64,
}

pub(crate) fn load_config(path: &Path) -> Result<Config> {
    let text = read_to_string(path)?;
    let config: Config = toml::from_str(&text)?;
    Ok(config)
}
