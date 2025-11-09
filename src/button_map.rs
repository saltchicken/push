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
pub enum NoteName {
    Pad0x7,
    Pad1x7,
    Pad2x7,
    Pad3x7,
    Pad4x7,
    Pad5x7,
    Pad6x7,
    Pad7x7,
    Pad0x6,
    Pad1x6,
    Pad2x6,
    Pad3x6,
    Pad4x6,
    Pad5x6,
    Pad6x6,
    Pad7x6,
    Pad0x5,
    Pad1x5,
    Pad2x5,
    Pad3x5,
    Pad4x5,
    Pad5x5,
    Pad6x5,
    Pad7x5,
    Pad0x4,
    Pad1x4,
    Pad2x4,
    Pad3x4,
    Pad4x4,
    Pad5x4,
    Pad6x4,
    Pad7x4,
    Pad0x3,
    Pad1x3,
    Pad2x3,
    Pad3x3,
    Pad4x3,
    Pad5x3,
    Pad6x3,
    Pad7x3,
    Pad0x2,
    Pad1x2,
    Pad2x2,
    Pad3x2,
    Pad4x2,
    Pad5x2,
    Pad6x2,
    Pad7x2,
    Pad0x1,
    Pad1x1,
    Pad2x1,
    Pad3x1,
    Pad4x1,
    Pad5x1,
    Pad6x1,
    Pad7x1,
    Pad0x0,
    Pad1x0,
    Pad2x0,
    Pad3x0,
    Pad4x0,
    Pad5x0,
    Pad6x0,
    Pad7x0,
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ControlName {
    Control29,
    Control20,
    Control21,
    Control24,
    Control25,
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
    note_map: HashMap<u8, NoteName>,
    control_map: HashMap<u8, ControlName>,
    encoder_map: HashMap<u8, EncoderName>,
}

impl ButtonMap {
    pub fn new() -> Result<Self, ButtonMapError> {
        let config_path = "config/button_map.ron";
        println!("Loading button map from: {}", config_path);

        let config_string = fs::read_to_string(config_path)
            .map_err(|e| ButtonMapError::ReadError(config_path.into(), Box::new(e)))?;

        let map: ButtonMap = ron::from_str(&config_string)
            .map_err(|e| ButtonMapError::ParseError(config_path.into(), Box::new(e)))?;

        println!(
            "Successfully loaded {} note, {} control, and {} encoder mappings.",
            map.note_map.len(),
            map.control_map.len(),
            map.encoder_map.len()
        );

        Ok(map)
    }

    pub fn get_note(&self, address: u8) -> Option<NoteName> {
        self.note_map.get(&address).copied()
    }

    pub fn get_control(&self, address: u8) -> Option<ControlName> {
        self.control_map.get(&address).copied()
    }

    pub fn get_encoder(&self, address: u8) -> Option<EncoderName> {
        self.encoder_map.get(&address).copied()
    }
}

