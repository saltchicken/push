use push2::{Push2, Push2Event, Push2State};

use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_10X20},
    pixelcolor::Bgr565,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::Text,
};
use std::{error, thread, time};

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

            state.update_from_event(&event, &mut push2.midi_out, &push2.button_map)?;

            match event {
                Push2Event::PadPressed { coord, .. } => {
                    println!("--- Pad ({}, {}) PRESSED ---", coord.x, coord.y);
                }
                Push2Event::PadReleased { coord } => {
                    println!("--- Pad ({}, {}) RELEASED ---", coord.x, coord.y);
                }
                Push2Event::ButtonPressed { name, .. } => {
                    println!("--- Button {:?} PRESSED ---", name);
                }
                Push2Event::ButtonReleased { name } => {
                    println!("--- Button {:?} RELEASED ---", name);
                }
                Push2Event::EncoderTwisted { name, value } => {
                    println!("--- Encoder {:?} TWISTED, value {} ---", name, value);
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
