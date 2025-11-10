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
    pub fn load_from_path(path: &str) -> Result<Self, ConfigError> {
        // ‼️ Use the 'path' argument here
        let config_string = fs::read_to_string(path)
            .map_err(|e| ConfigError::ReadError(path.into(), Box::new(e)))?;
        let config: AppConfig = ron::from_str(&config_string)
            .map_err(|e| ConfigError::ParseError(path.into(), Box::new(e)))?;
        Ok(config)
    }
}
