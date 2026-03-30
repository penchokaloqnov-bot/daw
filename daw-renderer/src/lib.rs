pub mod waveform;
pub mod spectrum;
pub mod meter;

#[cfg(feature = "gpu")]
pub mod gpu;

/// Configuration for rendering operations.
#[derive(Debug, Clone)]
pub struct RenderConfig {
    pub width: u32,
    pub height: u32,
    pub sample_rate: u32,
    pub pixels_per_second: f32,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 200,
            sample_rate: 44100,
            pixels_per_second: 100.0,
        }
    }
}

/// Selects the rendering backend to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RendererBackend {
    /// Software (CPU) rendering.
    Cpu,
    /// GPU-accelerated rendering via wgpu.
    Gpu,
}

/// A software render target backed by a pixel buffer (ARGB u32 values).
#[derive(Debug)]
pub struct RenderTarget {
    pub pixels: Vec<u32>,
    pub width: u32,
    pub height: u32,
}

impl RenderTarget {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            pixels: vec![0xFF000000; (width * height) as usize],
            width,
            height,
        }
    }

    pub fn clear(&mut self, color: u32) {
        self.pixels.fill(color);
    }

    #[inline]
    pub fn set_pixel(&mut self, x: u32, y: u32, color: u32) {
        if x < self.width && y < self.height {
            self.pixels[(y * self.width + x) as usize] = color;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::waveform::WaveformRenderer;
    use crate::spectrum::SpectrumAnalyzer;
    use crate::meter::LevelMeter;

    #[test]
    fn test_render_target_bounds() {
        let mut target = RenderTarget::new(100, 50);
        target.set_pixel(99, 49, 0xFFFFFFFF);
        target.set_pixel(100, 50, 0xDEADBEEF); // out of bounds — no panic
        assert_eq!(target.pixels[49 * 100 + 99], 0xFFFFFFFF); // row * width + column
    }

    #[test]
    fn test_waveform_renderer_empty() {
        let renderer = WaveformRenderer::new(RenderConfig::default());
        let mut target = RenderTarget::new(800, 100);
        let written = renderer.render_waveform(&[], &mut target, 0xFF4488FF);
        assert_eq!(written, 0);
    }

    #[test]
    fn test_waveform_renderer_sine() {
        let renderer = WaveformRenderer::new(RenderConfig::default());
        let mut target = RenderTarget::new(800, 100);
        let audio: Vec<f32> = (0..4410)
            .map(|i| (i as f32 * 0.01).sin() * 0.8)
            .collect();
        let written = renderer.render_waveform(&audio, &mut target, 0xFF4488FF);
        assert!(written > 0);
    }

    #[test]
    fn test_stereo_waveform() {
        let renderer = WaveformRenderer::new(RenderConfig::default());
        let mut target = RenderTarget::new(400, 100);
        let left: Vec<f32> = (0..1000).map(|i| (i as f32 * 0.01).sin()).collect();
        let right: Vec<f32> = (0..1000).map(|i| (i as f32 * 0.02).cos()).collect();
        renderer.render_stereo_waveform(&left, &right, &mut target);
    }

    #[test]
    fn test_spectrum_compute_empty() {
        let spectrum = SpectrumAnalyzer::compute_spectrum(&[], 8);
        assert_eq!(spectrum.len(), 8);
        assert!(spectrum.iter().all(|&v| v == 0.0));
    }

    #[test]
    fn test_spectrum_render() {
        let spectrum = vec![0.0, 0.1, 0.5, 0.3, 0.8, 0.2, 0.4, 0.1];
        let mut target = RenderTarget::new(320, 100);
        SpectrumAnalyzer::render_spectrum(&spectrum, &mut target, 0xFF00FF88);
    }

    #[test]
    fn test_level_meter_silence() {
        let mut meter = LevelMeter::new(2, 44100);
        let state = meter.update(&[0.0f32; 512]);
        assert!(state.rms.iter().all(|&v| v < 1e-6));
        assert!(state.peak.iter().all(|&v| v < 1e-6));
    }

    #[test]
    fn test_level_meter_full_scale() {
        let mut meter = LevelMeter::new(1, 100);
        let state = meter.update(&vec![1.0f32; 256]);
        assert!((state.rms[0] - 1.0).abs() < 1e-4);
        assert!((state.peak[0] - 1.0).abs() < 1e-4);
        let mut target = RenderTarget::new(100, 200);
        LevelMeter::render_meter(&state, &mut target);
    }
}
