// ‼️ This is a new file: src/state.rs
use crate::{ControlName, EncoderName, PadCoord};
use std::collections::HashMap;

/// Holds the state of a single 8x8 grid pad
#[derive(Debug, Clone, Copy, Default)]
pub struct PadState {
    /// The last recorded velocity (0 = released)
    pub velocity: u8,
    /// The currently set color (0 = off)
    pub color: u8,
}

/// Holds the state of a single control button
#[derive(Debug, Clone, Copy, Default)]
pub struct ButtonState {
    /// The last recorded velocity (0 = released)
    pub velocity: u8,
    /// The currently set brightness/color (0 = off)
    pub light: u8,
}

/// Holds the state of a single encoder
#[derive(Debug, Clone, Copy, Default)]
pub struct EncoderState {
    /// The integrated absolute value.
    pub value: i32,
    // TODO: Add is_touched when touch events are parsed
    // pub is_touched: bool,
}

/// The central struct holding the complete state of the Push 2.
#[derive(Debug)]
pub struct Push2State {
    /// 8x8 grid of pads
    pub pads: [[PadState; 8]; 8],
    /// All other control buttons
    pub buttons: HashMap<ControlName, ButtonState>,
    /// All encoders
    pub encoders: HashMap<EncoderName, EncoderState>,
    /// The touch strip value (0-16383)
    pub slider: u16,
}

impl Push2State {
    /// Creates a new, default state.
    pub fn new() -> Self {
        Self {
            pads: [[PadState::default(); 8]; 8],
            buttons: HashMap::new(),
            encoders: HashMap::new(),
            slider: 0,
        }
    }

    /// Updates the state based on an incoming event.
    /// This only updates the *input* state (velocity, pressed, etc.).
    /// It does not update the *output* state (color, light).
    pub fn update_from_event(&mut self, event: &crate::Push2Event) {
        match event {
            crate::Push2Event::PadPressed { coord, velocity } => {
                self.pads[coord.y as usize][coord.x as usize].velocity = *velocity;
            }
            crate::Push2Event::PadReleased { coord } => {
                self.pads[coord.y as usize][coord.x as usize].velocity = 0;
            }
            crate::Push2Event::ButtonPressed { name, velocity } => {
                self.buttons.entry(*name).or_default().velocity = *velocity;
            }
            crate::Push2Event::ButtonReleased { name } => {
                self.buttons.entry(*name).or_default().velocity = 0;
            }
            crate::Push2Event::EncoderTwisted { name, value } => {
                let state = self.encoders.entry(*name).or_default();
                // Convert Push 2's relative value (1-63 = CW, 65-127 = CCW)
                let delta = if *value > 64 {
                    (128 - *value) as i32 * -1
                } else {
                    *value as i32
                };
                state.value = state.value.wrapping_add(delta);
            }
            crate::Push2Event::SliderMoved { value } => {
                self.slider = *value;
            }
        }
    }

    // --- Methods to update output state (lights) ---

    /// Sets the internal state for a pad's color.
    /// This DOES NOT send the MIDI message.
    pub fn set_pad_color(&mut self, coord: PadCoord, color: u8) {
        self.pads[coord.y as usize][coord.x as usize].color = color;
    }

    /// Sets the internal state for a button's light.
    /// This DOES NOT send the MIDI message.
    pub fn set_button_light(&mut self, name: ControlName, light: u8) {
        self.buttons.entry(name).or_default().light = light;
    }
}
