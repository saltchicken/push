use crate::display::{Push2Display, Push2DisplayError};
use embedded_graphics::{
    image::Image,
    pixelcolor::Bgr565,
    prelude::*,
    primitives::{Line, Primitive, PrimitiveStyle, Rectangle},
};
use tinybmp::Bmp;

#[cfg(feature = "waveform")]
use hound::{SampleFormat, WavReader};
#[cfg(feature = "waveform")]
use std::error::Error;
#[cfg(feature = "waveform")]
use std::path::Path;
#[cfg(feature = "waveform")]
use thiserror::Error;

#[cfg(feature = "waveform")]
#[derive(Error, Debug)]
pub enum WaveformError {
    #[error("Hound (WAV) error: {0}")]
    HoundError(#[from] hound::Error),
    #[error("Audio file has no samples")]
    NoSamples,
    #[error("Audio file is too short to visualize at this width")]
    TooShort,
    #[error("Unsupported WAV format: {format:?}, {bits}-bit")]
    UnsupportedFormat { format: SampleFormat, bits: u16 },
    #[error("Generic I/O or other error: {0}")]
    Other(#[from] Box<dyn Error + Send + Sync>),
}

pub const ENCODER_REGION_WIDTH: u32 = 960 / 8; // 120
/// The height of the bar drawn for an encoder.
pub const ENCODER_BAR_HEIGHT: u32 = 8;
/// The Y-position (from top) of the encoder bar.
pub const ENCODER_BAR_Y_POS: i32 = 0;
/// Horizontal padding *inside* the 120px region for the bar.
pub const ENCODER_BAR_PADDING_X: u32 = 10;

/// A trait for high-level GUI drawing operations on the Push 2 display.
/// By implementing this as a trait, we separate the core display driver
/// logic (in Push2Display) from the high-level drawing API.
pub trait GuiApi {
    /// Draws a BMP image to the display's frame buffer.
    fn draw_bmp(&mut self, bmp_data: &[u8], position: Point) -> Result<(), Push2DisplayError>;

    /// Draws pre-calculated waveform peaks to the display's frame buffer.
    fn draw_waveform_peaks(
        &mut self,
        peaks: &[(f32, f32)],
        color: Bgr565,
    ) -> Result<(), Push2DisplayError>;

    fn draw_encoder_bar(
        &mut self,
        index: u8,
        value: f32,
        color: Bgr565,
    ) -> Result<(), Push2DisplayError>;

    /// Draws an outline rectangle for one of the 8 top encoders.
    ///
    /// * `index` - The encoder index (0-7).
    /// * `color` - The stroke color of the outline.
    fn draw_encoder_outline(&mut self, index: u8, color: Bgr565) -> Result<(), Push2DisplayError>;
}

impl GuiApi for Push2Display {
    fn draw_bmp(&mut self, bmp_data: &[u8], position: Point) -> Result<(), Push2DisplayError> {
        // Parse the BMP data
        // Map the unit error type `()` to our custom `BmpParseError`
        let bmp: Bmp<Bgr565> =
            Bmp::from_slice(bmp_data).map_err(|_| Push2DisplayError::BmpParseError)?;
        // Create an embedded-graphics Image
        let image = Image::new(&bmp, position);
        // Draw the image to the frame buffer
        // Our DrawTarget error is Infallible, so this .unwrap() is safe.
        image.draw(self).unwrap();
        Ok(())
    }

    fn draw_waveform_peaks(
        &mut self,
        peaks: &[(f32, f32)],
        color: Bgr565,
    ) -> Result<(), Push2DisplayError> {
        let size = self.size();
        let image_height = size.height;
        let mid_y = image_height as f32 / 2.0;
        let line_style = PrimitiveStyle::with_stroke(color, 1);
        for (x, (min, max)) in peaks.iter().enumerate() {
            // Stop drawing if the peak data is wider than the display
            if x as u32 >= size.width {
                break;
            }
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
            // We can draw to `self` because `Push2Display` implements `DrawTarget`
            // The DrawTarget<Error = Infallible>, so .unwrap() is safe.
            Line::new(Point::new(x_i, y_start_i), Point::new(x_i, y_end_i))
                .into_styled(line_style)
                .draw(self)
                .unwrap();
        }
        Ok(())
    }

    fn draw_encoder_bar(
        &mut self,
        index: u8,
        value: f32,
        color: Bgr565,
    ) -> Result<(), Push2DisplayError> {
        if index > 7 {
            return Ok(()); // Invalid index
        }

        // 1. Calculate the *full* bar width (with padding)
        let bar_width_total = ENCODER_REGION_WIDTH - (ENCODER_BAR_PADDING_X * 2);

        // 2. Calculate the *fill* width
        let fill_value = value.clamp(0.0, 1.0);
        let fill_width = (bar_width_total as f32 * fill_value) as u32;

        if fill_width == 0 {
            return Ok(()); // Nothing to draw
        }

        // 3. Calculate position
        let bar_top_left = Point::new(
            (index as u32 * ENCODER_REGION_WIDTH) as i32 + ENCODER_BAR_PADDING_X as i32,
            ENCODER_BAR_Y_POS,
        );

        let fill_size = Size::new(fill_width, ENCODER_BAR_HEIGHT);
        let fill_style = PrimitiveStyle::with_fill(color);

        Rectangle::new(bar_top_left, fill_size)
            .into_styled(fill_style)
            .draw(self)
            .unwrap(); // Infallible

        Ok(())
    }

    // ‼️ Implement the new outline function
    fn draw_encoder_outline(&mut self, index: u8, color: Bgr565) -> Result<(), Push2DisplayError> {
        if index > 7 {
            return Ok(()); // Invalid index
        }

        // 1. Calculate the *full* bar width (with padding)
        let bar_width_total = ENCODER_REGION_WIDTH - (ENCODER_BAR_PADDING_X * 2);

        // 2. Calculate position
        let bar_top_left = Point::new(
            (index as u32 * ENCODER_REGION_WIDTH) as i32 + ENCODER_BAR_PADDING_X as i32,
            ENCODER_BAR_Y_POS,
        );
        let bar_size_total = Size::new(bar_width_total, ENCODER_BAR_HEIGHT);

        // 3. Draw the outline
        let outline_style = PrimitiveStyle::with_stroke(color, 1);
        Rectangle::new(bar_top_left, bar_size_total)
            .into_styled(outline_style)
            .draw(self)
            .unwrap(); // Infallible

        Ok(())
    }
}

#[cfg(feature = "waveform")]
/// Helper function to read a WAV file and normalize all samples to f32
/// This is copied directly from `create_waveform.rs`
fn read_and_normalize_samples(
    mut reader: WavReader<std::io::BufReader<std::fs::File>>,
) -> Result<Vec<f32>, WaveformError> {
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
            return Err(WaveformError::UnsupportedFormat {
                format: spec.sample_format,
                bits: spec.bits_per_sample,
            });
        }
    };
    Ok(samples_f32)
}

#[cfg(feature = "waveform")]
/// Loads a .wav file, normalizes its samples, and calculates the min/max
/// peaks for each horizontal pixel.
///
/// * `path` - The path to the .wav file.
/// * `width` - The number of horizontal pixels (e.g., 960).
pub fn load_waveform_peaks(path: &Path, width: u32) -> Result<Vec<(f32, f32)>, WaveformError> {
    let reader = WavReader::open(path)?;
    let normalized_samples = read_and_normalize_samples(reader)?;
    if normalized_samples.is_empty() {
        return Err(WaveformError::NoSamples);
    }
    let samples_per_pixel = normalized_samples.len() / width as usize;
    if samples_per_pixel == 0 {
        return Err(WaveformError::TooShort);
    }
    // Find the min and max peak for each chunk
    let peaks: Vec<(f32, f32)> = (0..width)
        .map(|x| {
            let chunk_start = (x as usize) * samples_per_pixel;
            let chunk_end = (chunk_start + samples_per_pixel).min(normalized_samples.len());
            let chunk = &normalized_samples[chunk_start..chunk_end];
            let min = chunk.iter().fold(f32::INFINITY, |a, &b| a.min(b));
            let max = chunk.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
            (min.min(0.0), max.max(0.0))
        })
        .collect();
    Ok(peaks)
}
