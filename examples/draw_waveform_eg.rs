use embedded_graphics::{pixelcolor::Bgr565, prelude::*};
use log::{debug, info};
use push2::{GuiApi, Push2, Push2Colors, Push2Event, gui};
use std::{error::Error, path::PathBuf, thread, time};

// --- Color Configuration ---
const BACKGROUND_COLOR: Bgr565 = Bgr565::BLACK;
const WAVEFORM_COLOR: Bgr565 = Bgr565::GREEN;

/// Helper function to find the user's audio directory
pub fn get_audio_storage_path() -> std::io::Result<PathBuf> {
    match dirs::audio_dir() {
        Some(mut path) => {
            path.push("soundboard-recordings");
            std::fs::create_dir_all(&path)?;
            Ok(path)
        }
        None => Err(std::io::Error::other("Could not find audio directory")),
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    // --- 1. Initialize Push 2 ---
    info!("Connecting to Ableton Push 2...");
    let mut push2 = Push2::new()?;
    let display_size = push2.display.size();
    let image_width = display_size.width;

    // --- 2. Load WAV File (from create_waveform.rs) ---
    let audio_storage_path = get_audio_storage_path()?;
    let input_wav_path = audio_storage_path.join("test.wav");
    info!("Reading input file: {}", input_wav_path.display());

    let peaks = gui::load_waveform_peaks(&input_wav_path, image_width)?;
    info!("Successfully calculated {} waveform peaks.", peaks.len());

    push2.display.clear(BACKGROUND_COLOR)?;
    push2.display.draw_waveform_peaks(&peaks, WAVEFORM_COLOR)?;
    push2.display.flush()?;

    // --- 6. Main Loop (Now much lighter) ---
    info!("Render complete. Starting event loop...");
    loop {
        // --- 6a. Poll for events ---
        while let Some(event) = push2.poll_event() {
            match event {
                Push2Event::PadPressed { coord, .. } => {
                    debug!("Pad ({}, {}) PRESSED", coord.x, coord.y);
                    push2.set_pad_color(coord, Push2Colors::GREEN_PALE)?;
                }
                Push2Event::PadReleased { coord } => {
                    debug!("Pad ({}, {}) RELEASED", coord.x, coord.y);
                    push2.set_pad_color(coord, 0)?;
                }
                _ => {} // Ignore other events
            }
        }

        push2.display.flush()?;
        thread::sleep(time::Duration::from_millis(16));
    }
}
