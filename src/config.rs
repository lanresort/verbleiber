/*
 * Copyright 2022 Jochen Kupperschmidt
 * License: MIT
 */

use anyhow::Result;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
pub(crate) struct Config {
    pub sounds_path: PathBuf,
    pub user_sounds: HashMap<String, String>,
    pub whereabouts_sounds: HashMap<String, Vec<String>>,
}

pub(crate) fn load_config(path: &Path) -> Result<Config> {
    let text = read_to_string(path)?;
    let config: Config = toml::from_str(&text)?;
    Ok(config)
}
