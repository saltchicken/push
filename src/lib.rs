// --- Module Declarations ---
pub mod app_config;
pub mod button_map;
pub mod display;
pub mod midi_handler;

// --- Public API Re-exports ---
pub use app_config::{AppConfig, ConfigError};
pub use button_map::{ButtonMap, ButtonMapError, ControlName, EncoderName, PadCoord};
pub use display::{Push2Display, Push2DisplayError};
pub use midi_handler::MidiHandler;

use midir::{MidiInputConnection, MidiOutputConnection};
use std::error::Error;
use std::sync::mpsc::{self, Receiver};

// --- MIDI Message Constants ---
pub const NOTE_ON: u8 = 144;
pub const NOTE_OFF: u8 = 128;
pub const CONTROL_CHANGE: u8 = 176;
pub const PITCH_BEND: u8 = 224;

/// High-level events from the Ableton Push 2
#[derive(Debug, Clone, Copy)]
pub enum Push2Event {
    /// A grid pad was pressed
    PadPressed { coord: PadCoord, velocity: u8 },
    /// A grid pad was released
    PadReleased { coord: PadCoord },
    /// A control button was pressed
    ButtonPressed { name: ControlName, velocity: u8 },
    /// A control button was released
    ButtonReleased { name: ControlName },
    /// An encoder was twisted
    EncoderTwisted { name: EncoderName, value: u8 },
    /// The touch slider was moved
    SliderMoved { value: u16 },
}

/// Main struct for interfacing with the Ableton Push 2
pub struct Push2 {
    pub display: Push2Display,
    /// The MIDI output connection, for sending light/color data
    pub midi_out: MidiOutputConnection,
    button_map: ButtonMap,
    event_rx: Receiver<Vec<u8>>,
    _conn_in: MidiInputConnection<()>,
}

impl Push2 {
    /// Connects to the Push 2 display and MIDI ports.
    ///
    /// The user is responsible for loading and providing the `AppConfig`
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let app_config = AppConfig::new()?;

        // --- MIDI Setup ---
        let (tx, rx) = mpsc::channel();
        let midi_handler = MidiHandler::new(&app_config, tx)?;

        let button_map = ButtonMap::new()?;

        // --- Display Setup ---
        let display = Push2Display::new()?;
        let MidiHandler { _conn_in, conn_out } = midi_handler;

        Ok(Self {
            display,
            midi_out: conn_out,
            button_map,
            event_rx: rx,
            _conn_in,
        })
    }

    /// Polls for the next high-level `Push2Event`.
    /// This is non-blocking
    pub fn poll_event(&self) -> Option<Push2Event> {
        while let Ok(message) = self.event_rx.try_recv() {
            if message.is_empty() {
                continue;
            }

            let status = message[0];

            // Try to parse the raw MIDI message into a high-level event
            let event = match status {
                // --- NOTE ON / NOTE OFF (144 or 128) ---
                NOTE_ON | NOTE_OFF => {
                    if message.len() < 3 {
                        continue;
                    }
                    let address = message[1];
                    let velocity = message[2];

                    if let Some(pad_coord) = self.button_map.get_note(address) {
                        if status == NOTE_ON && velocity > 0 {
                            Some(Push2Event::PadPressed {
                                coord: pad_coord,
                                velocity,
                            })
                        } else {
                            Some(Push2Event::PadReleased { coord: pad_coord })
                        }
                    } else {
                        None // Unknown note
                    }
                }

                // --- CONTROL CHANGE (176) ---
                CONTROL_CHANGE => {
                    if message.len() < 3 {
                        continue;
                    }
                    let address = message[1];
                    let velocity = message[2];

                    if let Some(control_name) = self.button_map.get_control(address) {
                        if velocity > 0 {
                            Some(Push2Event::ButtonPressed {
                                name: control_name,
                                velocity,
                            })
                        } else {
                            Some(Push2Event::ButtonReleased { name: control_name })
                        }
                    } else if let Some(encoder_name) = self.button_map.get_encoder(address) {
                        Some(Push2Event::EncoderTwisted {
                            name: encoder_name,
                            value: velocity,
                        })
                    } else {
                        None // Unknown CC
                    }
                }

                // --- PITCH BEND (224) ---
                PITCH_BEND => {
                    if message.len() < 3 {
                        continue;
                    }
                    let lsb = message[1]; // 7 bits of data
                    let msb = message[2]; // 7 bits of data

                    // Combine LSB and MSB into a 14-bit value (0-16383)
                    let value = ((msb as u16) << 7) | (lsb as u16);

                    Some(Push2Event::SliderMoved { value })
                }

                _ => None, // Ignore other messages
            };

            // If we parsed a valid event, return it
            if event.is_some() {
                return event;
            }
        }
        // No events in the queue
        None
    }
}
