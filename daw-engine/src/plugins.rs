use std::f32::consts::PI;
use crate::audio_graph::{AudioNode, NodeParams};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BiquadType {
    LowPass,
    HighPass,
    BandPass,
    Notch,
    PeakEQ,
    LowShelf,
    HighShelf,
}

pub struct BiquadFilter {
    pub b0: f32,
    pub b1: f32,
    pub b2: f32,
    pub a1: f32,
    pub a2: f32,
    x1: f32,
    x2: f32,
    #[allow(dead_code)]
    y1: f32,
    #[allow(dead_code)]
    y2: f32,
    pub filter_type: BiquadType,
    pub frequency: f32,
    pub q: f32,
    pub gain_db: f32,
    pub sample_rate: f32,
}

impl BiquadFilter {
    pub fn new(filter_type: BiquadType, frequency: f32, q: f32, gain_db: f32, sample_rate: f32) -> Self {
        let mut f = BiquadFilter {
            b0: 0.0, b1: 0.0, b2: 0.0, a1: 0.0, a2: 0.0,
            x1: 0.0, x2: 0.0, y1: 0.0, y2: 0.0,
            filter_type,
            frequency,
            q,
            gain_db,
            sample_rate,
        };
        f.compute_coefficients();
        f
    }

    fn compute_coefficients(&mut self) {
        let w0 = 2.0 * PI * self.frequency / self.sample_rate;
        let cos_w0 = w0.cos();
        let sin_w0 = w0.sin();
        let alpha = sin_w0 / (2.0 * self.q);
        let a = 10.0f32.powf(self.gain_db / 40.0);

        let (b0, b1, b2, a0, a1, a2) = match self.filter_type {
            BiquadType::LowPass => {
                let b0 = (1.0 - cos_w0) / 2.0;
                let b1 = 1.0 - cos_w0;
                let b2 = (1.0 - cos_w0) / 2.0;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_w0;
                let a2 = 1.0 - alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            BiquadType::HighPass => {
                let b0 = (1.0 + cos_w0) / 2.0;
                let b1 = -(1.0 + cos_w0);
                let b2 = (1.0 + cos_w0) / 2.0;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_w0;
                let a2 = 1.0 - alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            BiquadType::BandPass => {
                let b0 = sin_w0 / 2.0;
                let b1 = 0.0;
                let b2 = -sin_w0 / 2.0;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_w0;
                let a2 = 1.0 - alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            BiquadType::Notch => {
                let b0 = 1.0;
                let b1 = -2.0 * cos_w0;
                let b2 = 1.0;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_w0;
                let a2 = 1.0 - alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            BiquadType::PeakEQ => {
                let b0 = 1.0 + alpha * a;
                let b1 = -2.0 * cos_w0;
                let b2 = 1.0 - alpha * a;
                let a0 = 1.0 + alpha / a;
                let a1 = -2.0 * cos_w0;
                let a2 = 1.0 - alpha / a;
                (b0, b1, b2, a0, a1, a2)
            }
            BiquadType::LowShelf => {
                let b0 = a * ((a + 1.0) - (a - 1.0) * cos_w0 + 2.0 * a.sqrt() * alpha);
                let b1 = 2.0 * a * ((a - 1.0) - (a + 1.0) * cos_w0);
                let b2 = a * ((a + 1.0) - (a - 1.0) * cos_w0 - 2.0 * a.sqrt() * alpha);
                let a0 = (a + 1.0) + (a - 1.0) * cos_w0 + 2.0 * a.sqrt() * alpha;
                let a1 = -2.0 * ((a - 1.0) + (a + 1.0) * cos_w0);
                let a2 = (a + 1.0) + (a - 1.0) * cos_w0 - 2.0 * a.sqrt() * alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            BiquadType::HighShelf => {
                let b0 = a * ((a + 1.0) + (a - 1.0) * cos_w0 + 2.0 * a.sqrt() * alpha);
                let b1 = -2.0 * a * ((a - 1.0) + (a + 1.0) * cos_w0);
                let b2 = a * ((a + 1.0) + (a - 1.0) * cos_w0 - 2.0 * a.sqrt() * alpha);
                let a0 = (a + 1.0) - (a - 1.0) * cos_w0 + 2.0 * a.sqrt() * alpha;
                let a1 = 2.0 * ((a - 1.0) - (a + 1.0) * cos_w0);
                let a2 = (a + 1.0) - (a - 1.0) * cos_w0 - 2.0 * a.sqrt() * alpha;
                (b0, b1, b2, a0, a1, a2)
            }
        };

        self.b0 = b0 / a0;
        self.b1 = b1 / a0;
        self.b2 = b2 / a0;
        self.a1 = a1 / a0;
        self.a2 = a2 / a0;
    }

    pub fn process_sample(&mut self, x: f32) -> f32 {
        // Direct Form II Transposed
        let y = self.b0 * x + self.x1;
        self.x1 = self.b1 * x - self.a1 * y + self.x2;
        self.x2 = self.b2 * x - self.a2 * y;
        y
    }

    pub fn process_buffer(&mut self, buffer: &mut [f32]) {
        for sample in buffer.iter_mut() {
            *sample = self.process_sample(*sample);
        }
    }

    pub fn update_params(&mut self, frequency: f32, q: f32, gain_db: f32) {
        self.frequency = frequency;
        self.q = q;
        self.gain_db = gain_db;
        self.compute_coefficients();
    }
}

impl AudioNode for BiquadFilter {
    fn process(&mut self, inputs: &[&[f32]], output: &mut [f32], _params: &NodeParams) {
        if let Some(input) = inputs.first() {
            let len = output.len().min(input.len());
            output[..len].copy_from_slice(&input[..len]);
        } else {
            for s in output.iter_mut() { *s = 0.0; }
        }
        self.process_buffer(output);
    }

    fn name(&self) -> &str { "BiquadFilter" }
}

// ---

pub struct Compressor {
    pub threshold_db: f32,
    pub ratio: f32,
    pub attack_samples: f32,
    pub release_samples: f32,
    pub makeup_gain_db: f32,
    pub envelope: f32,
}

impl Compressor {
    pub fn new(
        threshold_db: f32,
        ratio: f32,
        attack_ms: f32,
        release_ms: f32,
        makeup_gain_db: f32,
        sample_rate: f32,
    ) -> Self {
        Compressor {
            threshold_db,
            ratio,
            attack_samples: attack_ms / 1000.0 * sample_rate,
            release_samples: release_ms / 1000.0 * sample_rate,
            makeup_gain_db,
            envelope: 0.0,
        }
    }

    pub fn process_sample(&mut self, x: f32) -> f32 {
        let level = x.abs();
        if level > self.envelope {
            self.envelope += (level - self.envelope) / self.attack_samples.max(1.0);
        } else {
            self.envelope += (level - self.envelope) / self.release_samples.max(1.0);
        }

        let level_db = if self.envelope > 0.0 {
            20.0 * self.envelope.log10().max(-100.0)
        } else {
            -100.0
        };

        let gain_db = if level_db > self.threshold_db {
            let gain_reduction_db = (level_db - self.threshold_db) * (1.0 - 1.0 / self.ratio);
            -gain_reduction_db + self.makeup_gain_db
        } else {
            self.makeup_gain_db
        };

        x * 10.0f32.powf(gain_db / 20.0)
    }
}

impl AudioNode for Compressor {
    fn process(&mut self, inputs: &[&[f32]], output: &mut [f32], _params: &NodeParams) {
        if let Some(input) = inputs.first() {
            let len = output.len().min(input.len());
            for (o, &i) in output[..len].iter_mut().zip(input[..len].iter()) {
                *o = self.process_sample(i);
            }
            if output.len() > len {
                for s in output[len..].iter_mut() { *s = 0.0; }
            }
        } else {
            for s in output.iter_mut() { *s = 0.0; }
        }
    }

    fn name(&self) -> &str { "Compressor" }
}

// ---

pub struct SimpleReverb {
    comb_delays: [usize; 4],
    comb_feedbacks: [f32; 4],
    comb_buffers: [Vec<f32>; 4],
    comb_indices: [usize; 4],
    allpass_delays: [usize; 2],
    allpass_feedbacks: [f32; 2],
    allpass_buffers: [Vec<f32>; 2],
    allpass_indices: [usize; 2],
    pub room_size: f32,
    pub wet: f32,
    pub sample_rate: u32,
}

impl SimpleReverb {
    pub fn new(room_size: f32, wet: f32, sample_rate: u32) -> Self {
        let scale = sample_rate as f32 / 44100.0;
        let comb_delays = [
            (1116.0 * scale) as usize,
            (1188.0 * scale) as usize,
            (1277.0 * scale) as usize,
            (1356.0 * scale) as usize,
        ];
        let allpass_delays = [
            (556.0 * scale) as usize,
            (441.0 * scale) as usize,
        ];
        let feedback = 0.84 * room_size.clamp(0.1, 1.0);
        let comb_feedbacks = [feedback; 4];
        let allpass_feedbacks = [0.5; 2];

        SimpleReverb {
            comb_buffers: [
                vec![0.0f32; comb_delays[0].max(1)],
                vec![0.0f32; comb_delays[1].max(1)],
                vec![0.0f32; comb_delays[2].max(1)],
                vec![0.0f32; comb_delays[3].max(1)],
            ],
            comb_delays,
            comb_feedbacks,
            comb_indices: [0; 4],
            allpass_buffers: [
                vec![0.0f32; allpass_delays[0].max(1)],
                vec![0.0f32; allpass_delays[1].max(1)],
            ],
            allpass_delays,
            allpass_feedbacks,
            allpass_indices: [0; 2],
            room_size,
            wet,
            sample_rate,
        }
    }

    pub fn process_sample(&mut self, input: f32) -> f32 {
        // 4 comb filters in parallel
        let mut comb_sum = 0.0f32;
        for i in 0..4 {
            let delay = self.comb_delays[i].max(1);
            let idx = self.comb_indices[i];
            let delayed = self.comb_buffers[i][idx];
            self.comb_buffers[i][idx] = input + delayed * self.comb_feedbacks[i];
            self.comb_indices[i] = (idx + 1) % delay;
            comb_sum += delayed;
        }

        // 2 allpass filters in series
        let mut ap_out = comb_sum;
        for i in 0..2 {
            let delay = self.allpass_delays[i].max(1);
            let idx = self.allpass_indices[i];
            let delayed = self.allpass_buffers[i][idx];
            let fb = self.allpass_feedbacks[i];
            let new_val = ap_out + delayed * (-fb);
            self.allpass_buffers[i][idx] = ap_out + delayed * fb;
            self.allpass_indices[i] = (idx + 1) % delay;
            ap_out = new_val;
        }

        let dry = 1.0 - self.wet;
        dry * input + self.wet * ap_out
    }
}

impl AudioNode for SimpleReverb {
    fn process(&mut self, inputs: &[&[f32]], output: &mut [f32], _params: &NodeParams) {
        if let Some(input) = inputs.first() {
            let len = output.len().min(input.len());
            for (o, &i) in output[..len].iter_mut().zip(input[..len].iter()) {
                *o = self.process_sample(i);
            }
            if output.len() > len {
                for s in output[len..].iter_mut() { *s = 0.0; }
            }
        } else {
            for s in output.iter_mut() { *s = 0.0; }
        }
    }

    fn name(&self) -> &str { "SimpleReverb" }
}

// ---

pub struct DelayLine {
    buffer: Vec<f32>,
    write_index: usize,
    pub delay_samples: usize,
    pub feedback: f32,
    pub wet: f32,
}

impl DelayLine {
    pub fn new(delay_ms: f32, feedback: f32, wet: f32, sample_rate: u32) -> Self {
        let delay_samples = (delay_ms / 1000.0 * sample_rate as f32) as usize;
        let buf_size = delay_samples + 1;
        DelayLine {
            buffer: vec![0.0f32; buf_size.max(2)],
            write_index: 0,
            delay_samples,
            feedback: feedback.min(0.95),
            wet,
        }
    }

    pub fn process_sample(&mut self, input: f32) -> f32 {
        let buf_size = self.buffer.len();
        let read_index = (self.write_index + buf_size - self.delay_samples) % buf_size;
        let delayed = self.buffer[read_index];

        self.buffer[self.write_index] = input + self.feedback * delayed;
        self.write_index = (self.write_index + 1) % buf_size;

        let dry = 1.0 - self.wet;
        dry * input + self.wet * delayed
    }

    pub fn set_delay_ms(&mut self, delay_ms: f32, sample_rate: u32) {
        self.delay_samples = (delay_ms / 1000.0 * sample_rate as f32) as usize;
    }
}

impl AudioNode for DelayLine {
    fn process(&mut self, inputs: &[&[f32]], output: &mut [f32], _params: &NodeParams) {
        if let Some(input) = inputs.first() {
            let len = output.len().min(input.len());
            for (o, &i) in output[..len].iter_mut().zip(input[..len].iter()) {
                *o = self.process_sample(i);
            }
            if output.len() > len {
                for s in output[len..].iter_mut() { *s = 0.0; }
            }
        } else {
            for s in output.iter_mut() { *s = 0.0; }
        }
    }

    fn name(&self) -> &str { "DelayLine" }
}
