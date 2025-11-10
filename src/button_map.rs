use serde::Deserialize;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ButtonMapError {
    #[error("Failed to parse embedded button_map.ron: {0}")]
    ParseError(#[from] Box<ron::error::SpannedError>),
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PadCoord {
    pub x: u8,
    pub y: u8,
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ControlName {
    TapTempo,
    Metronome,
    Delete,
    Undo,
    Mute,
    Solo,
    Stop,
    Convert,
    DoubleLoop,
    Quantize,
    Duplicate,
    New,
    FixedLength,
    Automate,
    Record,
    Play,
    UpperRow1,
    UpperRow2,
    UpperRow3,
    UpperRow4,
    UpperRow5,
    UpperRow6,
    UpperRow7,
    UpperRow8,
    LowerRow1,
    LowerRow2,
    LowerRow3,
    LowerRow4,
    LowerRow5,
    LowerRow6,
    LowerRow7,
    LowerRow8,
    Beat1_32t,
    Beat1_32,
    Beat1_16t,
    Beat1_16,
    Beat1_8t,
    Beat1_8,
    Beat1_4t,
    Beat1_4,
    Setup,
    User,
    AddDevice,
    AddTrack,
    Device,
    Mix,
    Browse,
    Clip,
    Master,
    Up,
    Down,
    Left,
    Right,
    Repeat,
    Accent,
    Scale,
    Layout,
    Note,
    Session,
    OctaveUp,
    OctaveDown,
    PageLeft,
    PageRight,
    Shift,
    Select,
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EncoderName {
    Tempo,
    Swing,
    Track1,
    Track2,
    Track3,
    Track4,
    Track5,
    Track6,
    Track7,
    Track8,
    Master,
}

#[derive(Deserialize)]
pub struct ButtonMap {
    note_map: HashMap<u8, PadCoord>,
    control_map: HashMap<u8, ControlName>,
    encoder_map: HashMap<u8, EncoderName>,
}

impl ButtonMap {
    pub fn new() -> Result<Self, ButtonMapError> {
        let map_string = include_str!("../config/button_map.ron");
        let map: ButtonMap = ron::from_str(map_string).map_err(Box::new)?;
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
