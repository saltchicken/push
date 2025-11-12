// --- Module Declarations ---
pub mod app_config;
pub mod button_map;
pub mod colors;
pub mod display;
pub mod midi_handler;
pub mod state;

// --- Public API Re-exports ---
pub use app_config::{AppConfig, ConfigError};
pub use button_map::{ButtonMap, ButtonMapError, ControlName, EncoderName, PadCoord};
pub use display::{Push2Display, Push2DisplayError};
pub use midi_handler::{MidiHandler, MidiHandlerError};
pub use state::Push2State;

pub use colors as Push2Colors;

use embedded_graphics::prelude::Point;
use midir::{MidiInputConnection, MidiOutputConnection, SendError};
use std::sync::mpsc::{self, Receiver};
use thiserror::Error;
#[derive(Error, Debug)]

pub enum Push2Error {
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    #[error("Button map error: {0}")]
    ButtonMap(#[from] ButtonMapError),

    #[error("Display error: {0}")]
    Display(#[from] Push2DisplayError),

    #[error("MIDI initialization error: {0}")]
    MidiInit(#[from] MidiHandlerError),

    #[error("MIDI send error: {0}")]
    MidiSend(#[from] SendError),
}

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
    EncoderTwisted {
        name: EncoderName,
        raw_delta: u8,
        value: i32,
    },
    /// The touch slider was moved
    SliderMoved { value: u16 },
}

/// Main struct for interfacing with the Ableton Push 2
pub struct Push2 {
    pub display: Push2Display,
    /// The MIDI output connection, for sending light/color data
    pub midi_out: MidiOutputConnection,
    pub button_map: ButtonMap,
    pub state: Push2State,
    event_rx: Receiver<Vec<u8>>,
    _conn_in: MidiInputConnection<()>,
}

impl Push2 {
    /// Connects to the Push 2 display and MIDI ports.
    ///
    /// The user is responsible for loading and providing the `AppConfig`
    pub fn new() -> Result<Self, Push2Error> {
        let app_config = AppConfig::new()?;

        // --- MIDI Setup ---
        let (tx, rx) = mpsc::channel();
        let midi_handler = MidiHandler::new(&app_config, tx)?;

        let button_map = ButtonMap::new()?;

        // --- Display Setup ---
        let display = Push2Display::new()?;
        let MidiHandler { _conn_in, conn_out } = midi_handler;

        let state = Push2State::new();

        let mut push2 = Self {
            display,
            midi_out: conn_out,
            button_map,
            event_rx: rx,
            _conn_in,
            state,
        };

        push2.reset_all_lights()?;

        // ‼️ Return the initialized struct
        Ok(push2)
    }

    fn reset_all_lights(&mut self) -> Result<(), Push2Error> {
        // --- Reset all 64 pads ---
        // The pads are MIDI notes 36 through 99.
        for address in 36..=99 {
            let message = [NOTE_OFF, address, 0];
            self.midi_out.send(&message)?;
        }

        // --- Reset all control buttons ---
        // We can iterate the keys of the control_map to get all button addresses.
        for address in self.button_map.get_control_addresses() {
            let message = [CONTROL_CHANGE, *address, 0];
            self.midi_out.send(&message)?;
        }

        Ok(())
    }

    pub fn set_pad_color(&mut self, coord: PadCoord, color: u8) -> Result<(), Push2Error> {
        // Send MIDI message
        if let Some(address) = self.button_map.get_note_address(coord) {
            let message = if color == 0 {
                [NOTE_OFF, address, 0]
            } else {
                [NOTE_ON, address, color]
            };
            self.midi_out.send(&message)?;

            // Update state
            let pad = &mut self.state.pads[coord.y as usize][coord.x as usize];
            pad.color = color;

            Ok(())
        } else {
            Ok(())
        }
    }

    pub fn set_button_light(&mut self, name: ControlName, light: u8) -> Result<(), Push2Error> {
        // Send MIDI message
        if let Some(address) = self.button_map.get_control_address(name) {
            let message = if light == 0 {
                [CONTROL_CHANGE, address, 0]
            } else {
                [CONTROL_CHANGE, address, light]
            };
            self.midi_out.send(&message)?;

            // Update state
            let button = self.state.buttons.entry(name).or_default();
            button.light = light;

            Ok(())
        } else {
            Ok(())
        }
    }

    pub fn draw_bmp_to_display(
        &mut self,
        bmp_data: &[u8],
        position: Point,
    ) -> Result<(), Push2Error> {
        self.display.draw_bmp(bmp_data, position)?;
        Ok(())
    }

    /// Polls for the next high-level `Push2Event`.
    /// This is non-blocking
    pub fn poll_event(&mut self) -> Option<Push2Event> {
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
                            raw_delta: velocity,
                            value: 0,
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
            if let Some(mut parsed_event) = event {
                self.state.update_from_event(&parsed_event);
                if let Push2Event::EncoderTwisted { name, value, .. } = &mut parsed_event {
                    *value = self.state.encoders.get(name).map_or(0, |s| s.value);
                }
                return Some(parsed_event);
            }
        }
        // No events in the queue
        None
    }
}

