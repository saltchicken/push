use hound::{SampleFormat, WavReader};
use image::{ImageBuffer, Rgb};
use imageproc::drawing::draw_line_segment_mut;
use std::error::Error;
use std::path::PathBuf;

// --- Image Configuration ---
const IMAGE_WIDTH: u32 = 960;
const IMAGE_HEIGHT: u32 = 160;
const BACKGROUND_COLOR: Rgb<u8> = Rgb([20, 20, 20]);
const WAVEFORM_COLOR: Rgb<u8> = Rgb([100, 255, 150]);

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
            .map(|s| (s >> 8) as f32 / 8_388_607.0)
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
    println!("Generating waveform image...");

    let audio_storage_path = get_audio_storage_path()?;
    println!("Using audio directory: {}", audio_storage_path.display());

    let input_wav_path = audio_storage_path.join("test.wav");
    let output_bmp_path = audio_storage_path.join("waveform.bmp");

    println!("Reading input file: {}", input_wav_path.display());
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

    //    let samples: Vec<i16> = reader.samples::<i16>().filter_map(Result::ok).collect();
    //

    //    let left_channel_samples: Vec<i16> = samples
    //        .iter()
    //        .step_by(channel_count)
    //        .copied()
    //        .collect();
    //
    //    if left_channel_samples.is_empty() {
    //        return Err("No valid samples found in WAV file.".into());
    //    }

    // 2. --- Process Samples for Drawing ---
    // [Image of a sound waveform]
    // A WAV file has thousands of samples per second. We can't draw all of them.
    // We'll group the samples into "chunks", where each chunk corresponds to
    // one vertical column of pixels in our final image.
    let samples_per_pixel = normalized_samples.len() / IMAGE_WIDTH as usize;
    if samples_per_pixel == 0 {
        return Err("Audio file is too short to visualize at this width.".into());
    }

    // This gives us the "peak" of the waveform for that slice of time.

    let peaks: Vec<(f32, f32)> = (0..IMAGE_WIDTH)
        .map(|x| {
            let chunk_start = (x as usize) * samples_per_pixel;
            let chunk_end = (chunk_start + samples_per_pixel).min(normalized_samples.len());
            let chunk = &normalized_samples[chunk_start..chunk_end];

            // ‼️ Find min and max in a single pass
            let (min, max) = chunk.iter().fold(
                (f32::INFINITY, f32::NEG_INFINITY), // Start with (min, max)
                |(current_min, current_max), &sample| {
                    (current_min.min(sample), current_max.max(sample))
                },
            );

            (min.min(0.0), max.max(0.0))
        })
        .collect(); // 3. --- Create and Draw on the Image ---
    let mut img = ImageBuffer::from_pixel(IMAGE_WIDTH, IMAGE_HEIGHT, BACKGROUND_COLOR);
    let mid_y = IMAGE_HEIGHT as f32 / 2.0;

    for (x, (min, max)) in peaks.iter().enumerate() {
        // We subtract from `mid_y` because image Y=0 is the top.
        let y_min_f = mid_y - (*min * mid_y);
        let y_max_f = mid_y - (*max * mid_y);

        let y_start = y_max_f.min(y_min_f);
        let y_end = y_max_f.max(y_min_f).max(y_start + 1.0);

        draw_line_segment_mut(
            &mut img,
            (x as f32, y_start),
            (x as f32, y_end),
            WAVEFORM_COLOR,
        );
    }

    println!("Saving output file: {}", output_bmp_path.display());
    img.save(&output_bmp_path)?;

    println!(
        "Successfully saved 'waveform.bmp' to {}",
        audio_storage_path.display()
    );

    Ok(())
}
