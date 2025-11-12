// ‼️ Add this new file at examples/color_palette.rs
use push2::{button_map::PadCoord, Push2, Push2Event};
use log::{debug, info};
use std::{error, thread, time};

fn main() -> Result<(), Box<dyn error::Error>> {
    env_logger::init();

    // --- Config Loading ---
    let mut push2 = Push2::new()?;

    info!("Setting all 64 pads to colors 0-63...");
    info!("Pad (0, 0) [Top-Left] = Color 0");
    info!("Pad (7, 0) [Top-Right] = Color 7");
    info!("Pad (0, 1) = Color 8");
    info!("Pad (7, 7) [Bottom-Right] = Color 63");

    // --- Set all pad colors ---
    for y in 0..8 {
        for x in 0..8 {
            // Calculate the color index: 0-63
            // This maps (y * 8 + x) to the color index
            let color_index = (y * 8 + x) as u8;
            let coord = PadCoord { x, y };

            // Set the pad color
            push2.set_pad_color(coord, color_index)?;
        }
    }

    info!("All pads set. The device will remain lit.");
    info!("‼️ Press any pad to log its (x, y) coordinates and color index.");
    info!("Press Ctrl-C to quit.");

    // --- Main Loop (to keep the program alive and handle presses) ---
    loop {
        // Poll for events
        while let Some(event) = push2.poll_event() {
            match event {
                // ‼️ Optional: If you press a pad, log which one it was
                // ‼️ This helps map the physical pad to the color index.
                Push2Event::PadPressed { coord, .. } => {
                    let color_index = (coord.y * 8 + coord.x) as u8;
                    info!(
                        "Pad ({}, {}) PRESSED. Color index: {}",
                        coord.x, coord.y, color_index
                    );
                }
                _ => {
                    // Log other events if you want
                    debug!("Received event: {:?}", event);
                }
            }
        }

        // Don't spin the CPU
        thread::sleep(time::Duration::from_millis(16));
    }
}
