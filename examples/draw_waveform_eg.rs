use embedded_graphics::{
    pixelcolor::Bgr565,
    prelude::*,
    primitives::{Line, Primitive, PrimitiveStyle},
};
use hound::{SampleFormat, WavReader};
use log::{debug, info};
use push2::{Push2, Push2Colors, Push2Event, button_map::PadCoord};
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

/// Helper function to read a WAV file and normalize all samples to f32
/// This is copied directly from `create_waveform.rs`
fn read_and_normalize_samples(
    mut reader: WavReader<std::io::BufReader<std::fs::File>>,
) -> Result<Vec<f32>, Box<dyn Error>> {
    let spec = reader.spec();
    let channel_count = spec.channels as usize;
    let samples_f32: Vec<f32> = match (spec.sample_format, spec.bits_per_sample) {
        (SampleFormat::Float, 32) => reader
            .samples::<f32>()
            .filter_map(Result::ok)
            .step_by(channel_count)
            .collect(),
        (SampleFormat::Int, 16) => reader
            .samples::<i16>()
            .filter_map(Result::ok)
            .step_by(channel_count)
            .map(|s| s as f32 / i16::MAX as f32)
            .collect(),
        (SampleFormat::Int, 24) => reader
            .samples::<i32>()
            .filter_map(Result::ok)
            .step_by(channel_count)
            .map(|s| (s >> 8) as f32 / 8_388_607.0) // 2^23 - 1
            .collect(),
        (SampleFormat::Int, 32) => reader
            .samples::<i32>()
            .filter_map(Result::ok)
            .step_by(channel_count)
            .map(|s| s as f32 / i32::MAX as f32)
            .collect(),
        _ => {
            return Err(format!(
                "Unsupported WAV format: {:?}, {}-bit",
                spec.sample_format, spec.bits_per_sample
            )
            .into());
        }
    };
    Ok(samples_f32)
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    // --- 1. Initialize Push 2 ---
    info!("Connecting to Ableton Push 2...");
    let mut push2 = Push2::new()?;
    let display_size = push2.display.size();
    let image_width = display_size.width;
    let image_height = display_size.height;

    // --- 2. Load WAV File (from create_waveform.rs) ---
    let audio_storage_path = get_audio_storage_path()?;
    let input_wav_path = audio_storage_path.join("test.wav");
    info!("Reading input file: {}", input_wav_path.display());

    let reader = WavReader::open(&input_wav_path).map_err(|e| {
        format!(
            "Failed to open WAV file at {}: {}. \n‼️ Did you place 'test.wav' in '{}'?",
            input_wav_path.display(),
            e,
            audio_storage_path.display()
        )
    })?;

    let normalized_samples = read_and_normalize_samples(reader)?;
    if normalized_samples.is_empty() {
        return Err("No valid samples found in WAV file.".into());
    }
    info!(
        "Successfully read {} mono samples.",
        normalized_samples.len()
    );

    // --- 3. Process Samples (from create_waveform.rs) ---
    // ‼️ This line is now a comment
    // // Group samples into chunks, one for each horizontal pixel
    let samples_per_pixel = normalized_samples.len() / image_width as usize;
    if samples_per_pixel == 0 {
        return Err("Audio file is too short to visualize at this width.".into());
    }

    // Find the min and max peak for each chunk
    let peaks: Vec<(f32, f32)> = (0..image_width)
        .map(|x| {
            let chunk_start = (x as usize) * samples_per_pixel;
            let chunk_end = (chunk_start + samples_per_pixel).min(normalized_samples.len());
            let chunk = &normalized_samples[chunk_start..chunk_end];

            let min = chunk.iter().fold(f32::INFINITY, |a, &b| a.min(b));
            let max = chunk.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
            (min.min(0.0), max.max(0.0))
        })
        .collect();

    // --- 4. Draw to Display using embedded-graphics ---
    info!("Drawing waveform to Push 2 display...");
    push2.display.clear(BACKGROUND_COLOR)?;

    // This is the new drawing logic using embedded_graphics::Line
    let mid_y = image_height as f32 / 2.0;
    let line_style = PrimitiveStyle::with_stroke(WAVEFORM_COLOR, 1);

    for (x, (min, max)) in peaks.iter().enumerate() {
        // We subtract from `mid_y` because image Y=0 is the top.
        let y_min_f = mid_y - (*min * mid_y); // y for negative peak
        let y_max_f = mid_y - (*max * mid_y); // y for positive peak

        // Get the top-most and bottom-most pixel coordinates
        let y_start_f = y_max_f.min(y_min_f);
        let y_end_f = y_max_f.max(y_min_f);

        // Convert to i32 for `Point`
        let x_i = x as i32;
        let y_start_i = y_start_f.round() as i32;
        // Ensure the line is at least 1 pixel tall
        let y_end_i = (y_end_f.round() as i32).max(y_start_i);

        // Draw the vertical line for this chunk
        Line::new(Point::new(x_i, y_start_i), Point::new(x_i, y_end_i))
            .into_styled(line_style)
            .draw(&mut push2.display)?;
    }

    // --- 5. Flush buffer to hardware ---
    push2.display.flush()?;
    info!("Waveform drawn. Press pads or Ctrl-C to quit.");

    // --- 6. Keep program alive ---
    // (Copied from push_example.rs)
    loop {
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
        // Don't spin the CPU
        thread::sleep(time::Duration::from_millis(16));
    }
}
