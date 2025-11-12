use embedded_graphics::{pixelcolor::Bgr565, prelude::*};
use log::debug;
use push2::{GuiApi, Push2, Push2Event, button_map::EncoderName};
use std::{error::Error, thread, time};
fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    // --- 1. Initialize Push 2 ---
    debug!("Connecting to Ableton Push 2...");
    let mut push2 = Push2::new()?;
    debug!("Connection established.");
    // --- 2. State for our 8 track encoders ---
    let mut track_encoder_values: [i32; 8] = [64; 8];
    // --- 3. Initial Draw ---
    push2.display.clear(Bgr565::BLACK)?;
    for i in 0..8u8 {
        // Draw the empty outline
        push2.display.draw_encoder_outline(i, Bgr565::WHITE)?;
        push2
            .display
            .draw_encoder_bar(i, track_encoder_values[i as usize], Bgr565::GREEN)?; // ‼️ This now passes an i32
    }
    push2.display.flush()?;
    // --- 4. Main Loop ---
    debug!("Starting event loop. Twist any of the 8 track encoders (above the screen).");
    loop {
        let mut needs_redraw = false;
        // --- 4a. Poll for events ---
        while let Some(event) = push2.poll_event() {
            if let Push2Event::EncoderTwisted { name, raw_delta } = event {
                let index = match name {
                    EncoderName::Track1 => Some(0),
                    EncoderName::Track2 => Some(1),
                    EncoderName::Track3 => Some(2),
                    EncoderName::Track4 => Some(3),
                    EncoderName::Track5 => Some(4),
                    EncoderName::Track6 => Some(5),
                    EncoderName::Track7 => Some(6),
                    EncoderName::Track8 => Some(7),
                    _ => None, // Ignore other encoders (Tempo, Swing, Master)
                };
                if let Some(idx) = index {
                    let delta = if raw_delta > 64 {
                        -((128 - raw_delta) as i32)
                    } else {
                        raw_delta as i32
                    };
                    let current_value = track_encoder_values[idx as usize];
                    let new_value = (current_value.saturating_add(delta)).clamp(0, 127);
                    if track_encoder_values[idx as usize] != new_value {
                        track_encoder_values[idx as usize] = new_value;
                        needs_redraw = true;
                        debug!("Encoder {} ({:?}) updated to: {}", idx, name, new_value);
                    }
                }
            }
        }
        // --- 4b. Render if needed ---
        if needs_redraw {
            // Clear the display
            push2.display.clear(Bgr565::BLACK)?;
            // Redraw all 8 bars and outlines
            for i in 0..8u8 {
                // Draw the outline
                push2.display.draw_encoder_outline(i, Bgr565::WHITE)?;
                // Draw the filled bar
                push2.display.draw_encoder_bar(
                    i,
                    track_encoder_values[i as usize],
                    Bgr565::GREEN,
                )?;
            }
            // Flush the frame buffer to the screen
        }
        push2.display.flush()?;
        // --- 4c. Sleep ---
        // Don't spin the CPU
        thread::sleep(time::Duration::from_millis(16));
    }
}
