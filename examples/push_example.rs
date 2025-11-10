use push2::{AppConfig, ButtonMap, Push2, Push2Event};

use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_10X20},
    pixelcolor::Bgr565,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::Text,
};
use std::{error, thread, time};

const NOTE_ON: u8 = 144;
const NOTE_OFF: u8 = 128;

fn main() -> Result<(), Box<dyn error::Error>> {
    // --- Config Loading ---

    let app_config = AppConfig::load_from_path("config/app_config.ron").map_err(|e| {
        println!("Failed to load 'config/app_config.ron': {}", e);
        println!("Please create one with 'midi_input_port' and 'midi_output_port' fields.");
        e
    })?;

    let button_map = ButtonMap::load_from_path("config/button_map.ron")?;
    println!("Successfully loaded configs.");

    // --- Push2 Library Setup ---
    let mut push2 = Push2::new(app_config, button_map)?;

    // --- Display Setup (Application Logic) ---
    let text_style = MonoTextStyle::new(&FONT_10X20, Bgr565::WHITE);
    let mut position = Point::new(0, 70);
    let mut step = 4;

    println!("\nConnection open. Press any pad...");

    // --- Main Loop ---
    loop {
        while let Some(event) = push2.poll_event() {
            println!("Received event: {:?}", event);

            match event {
                Push2Event::PadPressed { coord, .. } => {
                    println!("--- Pad ({}, {}) PRESSED ---", coord.x, coord.y);

                    let address = 36 + coord.x + (7 - coord.y) * 8;
                    push2.midi_out.send(&[NOTE_ON, address, 122])?; // 122 = White
                }
                Push2Event::PadReleased { coord } => {
                    println!("--- Pad ({}, {}) RELEASED ---", coord.x, coord.y);
                    let address = 36 + coord.x + (7 - coord.y) * 8;
                    push2.midi_out.send(&[NOTE_OFF, address, 0])?; // 0 = Off
                }
                Push2Event::ButtonPressed { name, .. } => {
                    println!("--- Button {:?} PRESSED ---", name);
                    // Note: You'll need a map from ControlName back to u8 if you want to mirror
                    // This example just prints.
                }
                Push2Event::ButtonReleased { name } => {
                    println!("--- Button {:?} RELEASED ---", name);
                }
                Push2Event::EncoderTwisted { name, value } => {
                    println!("--- Encoder {:?} TWISTED, value {} ---", name, value);
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
