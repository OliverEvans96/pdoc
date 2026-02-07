use std::path::PathBuf;

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::{me::Me, storage::get_config_file_path};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StorageConfig {
    pub data_dir: Option<PathBuf>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
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

    pub fn save(&self) -> anyhow::Result<()> {
        let config_path = get_config_file_path().context("getting config file path")?;
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent).context("creating config directory")?;
        }
        let config_str = toml::to_string_pretty(self).context("serializing config TOML")?;
        std::fs::write(config_path, config_str).context("writing config file")?;

        Ok(())
    }
}
