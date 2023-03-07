use std::path::PathBuf;

use anyhow::Context;
use serde::Deserialize;

use crate::storage::get_config_file_path;

#[derive(Deserialize)]
pub struct Config {
    pub data_dir: Option<PathBuf>,
}

pub fn read_config() -> anyhow::Result<Config> {
    let config_path = get_config_file_path().context("getting config file path")?;
    let config_str = std::fs::read_to_string(config_path).context("reading config file")?;
    let config: Config = toml::from_str(&config_str).context("parsing config TOML")?;

    Ok(config)
}
