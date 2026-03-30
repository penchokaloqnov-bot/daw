use crate::RenderTarget;

/// Spectrum analyzer with configurable band count and smoothing.
pub struct SpectrumAnalyzer {
    pub bands: usize,
    pub sample_rate: f32,
    /// Smoothing coefficient in [0, 1). Higher values = slower decay.
    pub smoothing: f32,
    smoothed: Vec<f32>,
}

impl SpectrumAnalyzer {
    pub fn new(bands: usize, sample_rate: f32, smoothing: f32) -> Self {
        Self {
            bands,
            sample_rate,
            smoothing: smoothing.clamp(0.0, 0.999),
            smoothed: vec![0.0; bands],
        }
    }

    /// Computes octave-band energy spectrum from `audio`. Each band covers one
    /// octave starting from ~20 Hz; no external FFT crate is required.
    pub fn compute_spectrum(audio: &[f32], num_bands: usize) -> Vec<f32> {
        if audio.is_empty() || num_bands == 0 {
            return vec![0.0; num_bands];
        }

        // Simple DFT-based energy per octave band.
        // Frequencies are 20 * 2^(band / bands * log2(20000/20)).
        let n = audio.len().min(2048); // cap for performance
        let samples = &audio[..n];
        let mut spectrum = vec![0.0f32; num_bands];

        let log_range = (20000.0f32 / 20.0).log2(); // ≈ 9.97 octaves

        for (b, out) in spectrum.iter_mut().enumerate() {
            let freq_lo = 20.0f32 * 2.0f32.powf(log_range * b as f32 / num_bands as f32);
            let freq_hi = 20.0f32 * 2.0f32.powf(log_range * (b + 1) as f32 / num_bands as f32);
            let mut energy = 0.0f32;
            let mut count = 0usize;

            // Accumulate DFT bins whose centre frequency falls in [freq_lo, freq_hi).
            // Bin k has frequency k * sample_rate / n.
            let k_lo = (freq_lo * n as f32 / 44100.0).ceil() as usize;
            let k_hi = (freq_hi * n as f32 / 44100.0).ceil() as usize;
            let k_hi = k_hi.min(n / 2);

            for k in k_lo..=k_hi {
                let mut re = 0.0f32;
                let mut im = 0.0f32;
                let angle_step = std::f32::consts::TAU * k as f32 / n as f32;
                for (i, &s) in samples.iter().enumerate() {
                    let angle = angle_step * i as f32;
                    re += s * angle.cos();
                    im += s * angle.sin();
                }
                energy += re * re + im * im;
                count += 1;
            }

            *out = if count > 0 {
                (energy / count as f32).sqrt() / n as f32
            } else {
                0.0
            };
        }

        spectrum
    }

    /// Updates internal smoothed spectrum and returns it.
    pub fn update(&mut self, audio: &[f32]) -> &[f32] {
        let raw = Self::compute_spectrum(audio, self.bands);
        let alpha = self.smoothing;
        for (s, r) in self.smoothed.iter_mut().zip(raw.iter()) {
            *s = alpha * (*s) + (1.0 - alpha) * r;
        }
        &self.smoothed
    }

    /// Renders `spectrum` as vertical bars onto `target`.
    pub fn render_spectrum(spectrum: &[f32], target: &mut RenderTarget, color: u32) {
        if spectrum.is_empty() || target.width == 0 || target.height == 0 {
            return;
        }

        let w = target.width as usize;
        let h = target.height as usize;
        let band_w = (w / spectrum.len()).max(1);
        let max_val = spectrum.iter().cloned().fold(0.0f32, f32::max).max(1e-9);

        for (b, &val) in spectrum.iter().enumerate() {
            let bar_h = ((val / max_val) * h as f32) as usize;
            let x_start = (b * band_w) as u32;
            let x_end = (x_start + band_w as u32).min(target.width);

            for x in x_start..x_end {
                for row in 0..bar_h {
                    let y = (h - 1 - row) as u32;
                    target.set_pixel(x, y, color);
                }
            }
        }
    }
}
