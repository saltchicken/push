use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ButtonMapError {
    #[error("Failed to read config file {0}: {1}")]
    ReadError(String, #[source] Box<std::io::Error>),
    #[error("Failed to parse config file {0}: {1}")]
    ParseError(String, #[source] Box<ron::error::SpannedError>),
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PadCoord {
    pub x: u8,
    pub y: u8,
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ControlName {
    Control29,
    Control20,
    Control21,
    Control24,
    Control25,
    Control48,
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EncoderName {
    Control14,
    Control15,
    Control71,
    Control72,
    Control73,
    Control74,
    Control75,
    Control76,
    Control77,
    Control78,
    Control79,
}

#[derive(Deserialize)]
pub struct ButtonMap {
    note_map: HashMap<u8, PadCoord>,
    control_map: HashMap<u8, ControlName>,
    encoder_map: HashMap<u8, EncoderName>,
}

impl ButtonMap {
    pub fn load_from_path(path: &str) -> Result<Self, ButtonMapError> {
        let config_string = fs::read_to_string(path)
            .map_err(|e| ButtonMapError::ReadError(path.into(), Box::new(e)))?;
        let map: ButtonMap = ron::from_str(&config_string)
            .map_err(|e| ButtonMapError::ParseError(path.into(), Box::new(e)))?;
        Ok(map)
    }
    pub fn get_note(&self, address: u8) -> Option<PadCoord> {
        self.note_map.get(&address).copied()
    }

    pub fn get_control(&self, address: u8) -> Option<ControlName> {
        self.control_map.get(&address).copied()
    }

    pub fn get_encoder(&self, address: u8) -> Option<EncoderName> {
        self.encoder_map.get(&address).copied()
    }
}

