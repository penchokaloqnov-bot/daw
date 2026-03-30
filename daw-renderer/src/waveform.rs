use crate::{RenderConfig, RenderTarget};

/// Renders waveform visualizations onto a [`RenderTarget`].
pub struct WaveformRenderer {
    pub config: RenderConfig,
}

impl WaveformRenderer {
    pub fn new(config: RenderConfig) -> Self {
        Self { config }
    }

    /// Renders a mono waveform. Returns the number of pixels written.
    pub fn render_waveform(&self, audio: &[f32], target: &mut RenderTarget, color: u32) -> usize {
        if audio.is_empty() || target.width == 0 || target.height == 0 {
            return 0;
        }

        let width = target.width as usize;
        let height = target.height as usize;
        let half_h = (height / 2) as i32;
        let samples_per_pixel = (audio.len() as f32 / width as f32).max(1.0);
        let mut pixels_written = 0;

        for x in 0..width {
            let start = ((x as f32) * samples_per_pixel) as usize;
            let end = (((x + 1) as f32) * samples_per_pixel) as usize;
            let end = end.min(audio.len());

            if start >= audio.len() {
                break;
            }

            let slice = &audio[start..end];
            let peak_pos = slice.iter().cloned().fold(0.0f32, f32::max).clamp(-1.0, 1.0);
            let peak_neg = slice.iter().cloned().fold(0.0f32, f32::min).clamp(-1.0, 1.0);

            let y_top = (half_h - (peak_pos * half_h as f32) as i32).max(0) as u32;
            let y_bot = (half_h - (peak_neg * half_h as f32) as i32).min(height as i32 - 1) as u32;

            for y in y_top..=y_bot {
                target.set_pixel(x as u32, y, color);
                pixels_written += 1;
            }
        }

        pixels_written
    }

    /// Renders stereo waveform: left channel in upper half, right in lower half.
    pub fn render_stereo_waveform(
        &self,
        left: &[f32],
        right: &[f32],
        target: &mut RenderTarget,
    ) {
        let full_height = target.height;
        let half_height = full_height / 2;

        // Render left channel into upper half via a sub-target
        let mut left_target = RenderTarget::new(target.width, half_height);
        self.render_waveform(left, &mut left_target, 0xFF4488FF);

        // Render right channel into lower half via a sub-target
        let mut right_target = RenderTarget::new(target.width, half_height);
        self.render_waveform(right, &mut right_target, 0xFF44FF88);

        // Blit both sub-targets into the main target
        for y in 0..half_height {
            for x in 0..target.width {
                let src_idx = (y * target.width + x) as usize;
                target.set_pixel(x, y, left_target.pixels[src_idx]);
                target.set_pixel(x, y + half_height, right_target.pixels[src_idx]);
            }
        }
    }
}
