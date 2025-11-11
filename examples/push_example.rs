use push2::{Push2, Push2Event, Push2State};

use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_10X20},
    pixelcolor::Bgr565,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::Text,
};
use std::{error, thread, time};

const PAD_COLOR_ON: u8 = 122;
const BUTTON_LIGHT_ON: u8 = 2;

fn main() -> Result<(), Box<dyn error::Error>> {
    // --- Config Loading ---

    let mut push2 = Push2::new()?;
    let mut state = Push2State::new();

    // --- Display Setup (Application Logic) ---
    let text_style = MonoTextStyle::new(&FONT_10X20, Bgr565::WHITE);
    let mut position = Point::new(0, 70);
    let mut step = 4;

    println!("\nConnection open. Press any pad...");

    // --- Main Loop ---
    loop {
        while let Some(event) = push2.poll_event() {
            println!("Received event: {:?}", event);

            state.update_from_event(&event);

            match event {
                Push2Event::PadPressed { coord, .. } => {
                    println!("--- Pad ({}, {}) PRESSED ---", coord.x, coord.y);
                    push2.set_pad_color(coord, PAD_COLOR_ON)?;
                }
                Push2Event::PadReleased { coord } => {
                    println!("--- Pad ({}, {}) RELEASED ---", coord.x, coord.y);
                    push2.set_pad_color(coord, 0)?;
                }
                Push2Event::ButtonPressed { name, .. } => {
                    println!("--- Button {:?} PRESSED ---", name);
                    push2.set_button_light(name, BUTTON_LIGHT_ON)?;
                }
                Push2Event::ButtonReleased { name } => {
                    println!("--- Button {:?} RELEASED ---", name);
                    push2.set_button_light(name, 0)?;
                }
                Push2Event::EncoderTwisted { name, .. } => {
                    // println!("--- Encoder {:?} TWISTED, raw value {} ---", name, value);

                    let current_value = state.encoders.get(&name).map_or(0, |s| s.value);
                    println!("    New tracked value for {:?}: {}", name, current_value);
                }
                Push2Event::SliderMoved { value } => {
                    println!("--- Slider MOVED, value {} ---", value);
                }
            }
        }

        // --- Original Display Logic (Application-specific) ---

        push2.display.clear(Bgr565::BLACK)?;
        Rectangle::new(Point::zero(), push2.display.size())
            .into_styled(PrimitiveStyle::with_stroke(Bgr565::WHITE, 1))
            .draw(&mut push2.display)?;

        position.x += step;
        if position.x >= push2.display.size().width as i32 || position.x <= 0 {
            step *= -1;
        }

        Text::new("Hello!", position, text_style).draw(&mut push2.display)?;
        push2.display.flush()?;

        thread::sleep(time::Duration::from_millis(1000 / 60));
    }
}
