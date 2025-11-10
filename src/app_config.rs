use serde::Deserialize;
use std::fs;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read config file {0}: {1}")]
    ReadError(String, #[source] Box<std::io::Error>),
    #[error("Failed to parse config file {0}: {1}")]
    ParseError(String, #[source] Box<ron::error::SpannedError>),
}

#[derive(Deserialize)]
pub struct AppConfig {
    pub midi_input_port: String,
    pub midi_output_port: String,
}

impl AppConfig {
    pub fn new() -> Result<Self, ConfigError> {
        let config_path = "config/app_config.ron";
        println!("Loading app config from: {}", config_path);
        let config_string = fs::read_to_string(config_path)
            .map_err(|e| ConfigError::ReadError(config_path.into(), Box::new(e)))?;

        let config: AppConfig = ron::from_str(&config_string)
            .map_err(|e| ConfigError::ParseError(config_path.into(), Box::new(e)))?;

        println!("Successfully loaded app config.");
        Ok(config)
    }
}
