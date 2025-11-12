use log::{info, warn};
use serde::Deserialize;
use std::fs;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to parse config file: {0}")]
    ParseError(#[from] Box<ron::error::SpannedError>),
    #[error("Could not find or create config directory: {0}")]
    ConfigDirError(std::io::Error),
    #[error("Could not read or write config file: {0}")]
    ConfigFileError(std::io::Error),
}

#[derive(Deserialize)]
pub struct AppConfig {
    pub midi_input_port: String,
    pub midi_output_port: String,
}

impl AppConfig {
    pub fn new() -> Result<Self, ConfigError> {
        if let Some(mut config_path) = dirs::config_dir() {
            config_path.push("push2");
            fs::create_dir_all(&config_path).map_err(ConfigError::ConfigDirError)?;

            config_path.push("app_config.ron");

            match fs::read_to_string(&config_path) {
                Ok(config_string) => {
                    info!("Loading config from: {:?}", config_path);
                    let config: AppConfig = ron::from_str(&config_string).map_err(Box::new)?;
                    return Ok(config);
                }
                Err(_) => {
                    info!("No config file found. Writing default to {:?}", config_path);
                    let default_config_string = include_str!("../config/app_config.ron");
                    fs::write(&config_path, default_config_string)
                        .map_err(ConfigError::ConfigFileError)?;

                    let config: AppConfig =
                        ron::from_str(default_config_string).map_err(Box::new)?;
                    return Ok(config);
                }
            }
        }

        warn!("Could not find config directory. Falling back to embedded config.");
        let config_string = include_str!("../config/app_config.ron");
        let config: AppConfig = ron::from_str(config_string).map_err(Box::new)?;
        Ok(config)
    }
}