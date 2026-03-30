use crate::RenderTarget;

/// RMS/peak level meter for one or more channels.
pub struct LevelMeter {
    pub channels: usize,
    /// Number of audio samples to hold peak before decay.
    pub peak_hold_samples: u32,
    hold_counter: Vec<u32>,
}

/// Current metering state for all channels.
#[derive(Debug, Clone)]
pub struct MeterState {
    pub rms: Vec<f32>,
    pub peak: Vec<f32>,
    pub peak_hold: Vec<f32>,
}

impl LevelMeter {
    pub fn new(channels: usize, peak_hold_samples: u32) -> Self {
        Self {
            channels,
            peak_hold_samples,
            hold_counter: vec![0; channels],
        }
    }

    /// Updates meter state given interleaved `audio` samples.
    /// Channels are interleaved: [L0, R0, L1, R1, ...].
    pub fn update(&mut self, audio: &[f32]) -> MeterState {
        let ch = self.channels.max(1);
        let mut rms = vec![0.0f32; ch];
        let mut peak = vec![0.0f32; ch];
        let mut counts = vec![0usize; ch];

        for (i, &s) in audio.iter().enumerate() {
            let c = i % ch;
            let abs_s = s.abs();
            rms[c] += s * s;
            counts[c] += 1;
            if abs_s > peak[c] {
                peak[c] = abs_s;
            }
        }

        for c in 0..ch {
            if counts[c] > 0 {
                rms[c] = (rms[c] / counts[c] as f32).sqrt();
            }
        }

        // Update peak hold
        let mut peak_hold = vec![0.0f32; ch];
        for c in 0..ch {
            if peak[c] >= peak_hold[c] {
                peak_hold[c] = peak[c];
                self.hold_counter[c] = self.peak_hold_samples;
            } else if self.hold_counter[c] > 0 {
                self.hold_counter[c] -= 1;
                peak_hold[c] = peak[c]; // simplified: decay immediately after hold
            }
        }

        MeterState { rms, peak, peak_hold }
    }

    /// Renders vertical level meter bars for each channel.
    pub fn render_meter(state: &MeterState, target: &mut RenderTarget) {
        let ch = state.rms.len();
        if ch == 0 || target.width == 0 || target.height == 0 {
            return;
        }

        let bar_w = (target.width as usize / ch).max(1) as u32;
        let h = target.height;

        for c in 0..ch {
            let x_start = (c as u32) * bar_w;
            let x_end = (x_start + bar_w - 1).min(target.width - 1);

            // RMS bar (green)
            let rms_h = (state.rms[c].clamp(0.0, 1.0) * h as f32) as u32;
            for y in (h - rms_h.min(h))..h {
                for x in x_start..=x_end {
                    target.set_pixel(x, y, 0xFF00CC44);
                }
            }

            // Peak indicator line (yellow)
            let peak_h = (state.peak[c].clamp(0.0, 1.0) * h as f32) as u32;
            if peak_h > 0 {
                let peak_y = h.saturating_sub(peak_h);
                for x in x_start..=x_end {
                    target.set_pixel(x, peak_y, 0xFFFFDD00);
                }
            }

            // Peak hold indicator (red)
            let hold_h = (state.peak_hold[c].clamp(0.0, 1.0) * h as f32) as u32;
            if hold_h > 0 {
                let hold_y = h.saturating_sub(hold_h);
                for x in x_start..=x_end {
                    target.set_pixel(x, hold_y, 0xFFFF2200);
                }
            }
        }
    }
}
