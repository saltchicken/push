use crate::{ControlName, EncoderName};
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
#[derive(Debug)]
pub struct Push2State {
    pub pads: [[PadState; 8]; 8],
    pub buttons: HashMap<ControlName, ButtonState>,
    pub slider: u16,
}
impl Push2State {
    /// Creates a new, default state.
    pub fn new() -> Self {
        Self {
            pads: [[PadState::default(); 8]; 8],
            buttons: HashMap::new(),
            slider: 0,
        }
    }
    /// Updates the state based on an incoming event.
    /// This only updates the *input* state (velocity, pressed, etc.).
    pub fn update_from_event(&mut self, event: &crate::Push2Event) {
        match event {
            crate::Push2Event::PadPressed { coord, velocity } => {
                let pad = &mut self.pads[coord.y as usize][coord.x as usize];
                pad.velocity = *velocity;
            }
            crate::Push2Event::PadReleased { coord } => {
                let pad = &mut self.pads[coord.y as usize][coord.x as usize];
                pad.velocity = 0;
            }
            crate::Push2Event::ButtonPressed { name, velocity } => {
                let button = self.buttons.entry(*name).or_default();
                button.velocity = *velocity;
            }
            crate::Push2Event::ButtonReleased { name } => {
                let button = self.buttons.entry(*name).or_default();
                button.velocity = 0;
            }
            crate::Push2Event::SliderMoved { value } => {
                self.slider = *value;
            }
            _ => {}
        }
    }
}
impl Default for Push2State {
    fn default() -> Self {
        Self::new()
    }
}
