mod app_config;
mod button_map;
mod display;
mod midi_handler;

use app_config::AppConfig;
use button_map::ButtonMap;
use display::Push2Display;

use midi_handler::MidiHandler;

use embedded_graphics::{
    mono_font::{MonoTextStyle, ascii::FONT_10X20},
    pixelcolor::Bgr565,
    prelude::*,
    primitives::{PrimitiveStyle, Rectangle},
    text::Text,
};

use std::sync::mpsc::channel;
use std::{error, thread, time};

const NOTE_ON: u8 = 144;
const NOTE_OFF: u8 = 128;
const CONTROL_CHANGE: u8 = 176;

fn main() -> Result<(), Box<dyn error::Error>> {
    let app_config = AppConfig::new().map_err(|e| {
        println!("Failed to load 'config/app_config.ron': {}", e);
        println!("Please create one with 'midi_input_port' and 'midi_output_port' fields.");
        e
    })?;
    // --- MIDI Setup  ---
    let (tx, rx) = channel();

    let mut midi_handler = MidiHandler::new(&app_config, tx)?;

    // --- Display Setup ---
    let mut display = Push2Display::new()?;
    let text_style = MonoTextStyle::new(&FONT_10X20, Bgr565::WHITE);
    let mut position = Point::new(0, 70);
    let mut step = 4;

    // --- Create ButtonMap ---
    let button_map = ButtonMap::new()?;
    println!("\nConnection open. Press any pad...");

    // --- Main Loop ---
    loop {
        while let Ok(message) = rx.try_recv() {
            if message.len() < 3 {
                continue;
            }

            let status = message[0];
            let address = message[1];
            let velocity = message[2];

            println!(
                "Received message: status: {}, address: {}, velocity: {}",
                status, address, velocity
            );

            // We match on the STATUS byte first!
            match status {
                // --- NOTE ON / NOTE OFF (144 or 128) ---
                NOTE_ON | NOTE_OFF => {
                    // This is a pad, so we check the note_map
                    if let Some(pad_coord) = button_map.get_note(address) {
                        if status == NOTE_ON && velocity > 0 {
                            // Note On
                            println!("--- Pad ({}, {}) PRESSED ---", pad_coord.x, pad_coord.y);
                            midi_handler.conn_out.send(&[NOTE_ON, address, 122])?; // 122 = White
                        } else {
                            // Note Off (128 or 144 w/ vel 0)
                            println!("--- Pad ({}, {}) RELEASED ---", pad_coord.x, pad_coord.y);
                            midi_handler.conn_out.send(&[NOTE_OFF, address, 0])?; // 0 = Off
                        }
                    }
                }

                // --- CONTROL CHANGE (176) ---
                CONTROL_CHANGE => {
                    // This could be a button OR an encoder, so we check both maps
                    if let Some(control_name) = button_map.get_control(address) {
                        if velocity > 0 {
                            // Button down
                            println!("--- Button {:?} PRESSED ---", control_name);
                            midi_handler
                                .conn_out
                                .send(&[CONTROL_CHANGE, address, 127])?; // 127 = Bright White
                        } else {
                            // Button up
                            println!("--- Button {:?} RELEASED ---", control_name);
                            midi_handler.conn_out.send(&[CONTROL_CHANGE, address, 0])?; // 0 = Off
                        }
                    } else if let Some(encoder_name) = button_map.get_encoder(address) {
                        println!(
                            "--- Encoder {:?} TWISTED, value {} ---",
                            encoder_name, velocity
                        );
                    }
                }
                _ => {} // Ignore other messages
            }
        }

        // --- Original Display Logic ---
        // TODO: Replace this with something that is not a test case
        display.clear(Bgr565::BLACK)?;
        Rectangle::new(Point::zero(), display.size())
            .into_styled(PrimitiveStyle::with_stroke(Bgr565::WHITE, 1))
            .draw(&mut display)?;
        position.x += step;
        if position.x >= display.size().width as i32 || position.x <= 0 {
            step *= -1;
        }
        Text::new("Hello!", position, text_style).draw(&mut display)?;
        display.flush()?;
        thread::sleep(time::Duration::from_millis(1000 / 60));
    }
}

