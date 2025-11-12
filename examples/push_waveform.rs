use push2::{Push2, Push2Colors, Push2Event};

use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_10X20},
    pixelcolor::Bgr565,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::Text,
};
use log::{debug, info, trace};
use std::{error, fs, thread, time};

mod soundboard_modules;
use soundboard_modules::get_audio_storage_path;

const PAD_COLOR_ON: u8 = Push2Colors::GREEN_PALE;
const BUTTON_LIGHT_ON: u8 = Push2Colors::GREEN_PALE;

fn main() -> Result<(), Box<dyn error::Error>> {
    env_logger::init();
    // --- Config Loading ---

    let mut push2 = Push2::new()?;

    let audio_storage_path = get_audio_storage_path()?;
    let bmp_path = audio_storage_path.join("waveform.bmp");

    info!("Loading waveform from: {}", bmp_path.display());
    let bmp_data = fs::read(&bmp_path).map_err(|e| {
        format!(
            "Failed to read 'waveform.bmp' from {}: {}. \n‼️ Did you run the 'create_waveform' example first?",
            audio_storage_path.display(),
            e
        )
    })?;

    // --- Display Setup (Application Logic) ---
    let text_style = MonoTextStyle::new(&FONT_10X20, Bgr565::WHITE);
    let mut position = Point::new(0, 70);
    let mut step = 4;

    info!("\nConnection open. Press any pad...");

    // --- Main Loop ---
    loop {
        while let Some(event) = push2.poll_event() {
            debug!("Received event: {:?}", event);

            match event {
                Push2Event::PadPressed { coord, .. } => {
                    debug!("--- Pad ({}, {}) PRESSED ---", coord.x, coord.y);
                    push2.set_pad_color(coord, PAD_COLOR_ON)?;
                }
                Push2Event::PadReleased { coord } => {
                    debug!("--- Pad ({}, {}) RELEASED ---", coord.x, coord.y);
                    push2.set_pad_color(coord, 0)?;
                }
                Push2Event::ButtonPressed { name, .. } => {
                    debug!("--- Button {:?} PRESSED ---", name);
                    push2.set_button_light(name, BUTTON_LIGHT_ON)?;
                }
                Push2Event::ButtonReleased { name } => {
                    debug!("--- Button {:?} RELEASED ---", name);
                    push2.set_button_light(name, 0)?;
                }
                Push2Event::EncoderTwisted {
                    name,
                    value,
                    raw_delta,
                } => {
                    trace!(
                        "--- Encoder {:?} TWISTED, raw value {} ---",
                        name, raw_delta
                    );
                    debug!("    New tracked value for {:?}: {}", name, value);
                }
                Push2Event::SliderMoved { value } => {
                    debug!("--- Slider MOVED, value {} ---", value);
                }
            }
        }

        // --- Original Display Logic (Application-specific) ---

        push2.display.clear(Bgr565::BLACK)?;

        push2.draw_bmp_to_display(&bmp_data, Point::zero())?;

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
