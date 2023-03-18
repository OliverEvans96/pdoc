use std::path::PathBuf;

use anyhow::Context;
use serde::Deserialize;

use crate::{me::Me, storage::get_config_file_path};

#[derive(Clone, Debug, Deserialize)]
pub struct StorageConfig {
    pub data_dir: Option<PathBuf>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub me: Me,
    pub storage: StorageConfig,
}

impl Config {
    pub fn load() -> anyhow::Result<Config> {
        let config_path = get_config_file_path().context("getting config file path")?;
        let config_str = std::fs::read_to_string(config_path).context("reading config file")?;
        let config: Config = toml::from_str(&config_str).context("parsing config TOML")?;

        Ok(config)
    }
}
