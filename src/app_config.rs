use serde::Deserialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to parse config file: {0}")]
    ParseError(#[from] Box<ron::error::SpannedError>),
}

#[derive(Deserialize)]
pub struct AppConfig {
    pub midi_input_port: String,
    pub midi_output_port: String,
}

impl AppConfig {
    pub fn new() -> Result<Self, ConfigError> {
        let config_string = include_str!("../config/app_config.ron");
        let config: AppConfig = ron::from_str(config_string).map_err(Box::new)?;
        Ok(config)
    }
}
