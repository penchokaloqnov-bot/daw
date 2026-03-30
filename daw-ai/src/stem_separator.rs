pub struct StemSeparatorConfig {
    pub sample_rate: u32,
}

pub struct StemOutput {
    pub drums: Vec<f32>,
    pub bass: Vec<f32>,
    pub vocals: Vec<f32>,
    pub other: Vec<f32>,
}

pub struct StemSeparator {
    pub config: StemSeparatorConfig,
}

impl StemSeparator {
    pub fn new(config: StemSeparatorConfig) -> Self {
        StemSeparator { config }
    }

    pub fn separate(&self, audio: &[f32], sample_rate: u32) -> StemOutput {
        let bass = lowpass_filter(audio, 200.0, sample_rate);
        let hp_200 = highpass_filter(audio, 200.0, sample_rate);
        let drums = lowpass_filter(&hp_200, 2000.0, sample_rate);
        let hp_2k = highpass_filter(&hp_200, 2000.0, sample_rate);
        let vocals = lowpass_filter(&hp_2k, 8000.0, sample_rate);
        let other = highpass_filter(&hp_2k, 8000.0, sample_rate);
        StemOutput { drums, bass, vocals, other }
    }
}

fn lowpass_filter(input: &[f32], cutoff_hz: f32, sample_rate: u32) -> Vec<f32> {
    let rc = 1.0 / (2.0 * std::f32::consts::PI * cutoff_hz);
    let dt = 1.0 / sample_rate as f32;
    let alpha = dt / (rc + dt);
    let mut output = vec![0.0f32; input.len()];
    let mut prev = 0.0f32;
    for (i, &x) in input.iter().enumerate() {
        prev = prev + alpha * (x - prev);
        output[i] = prev;
    }
    output
}

fn highpass_filter(input: &[f32], cutoff_hz: f32, sample_rate: u32) -> Vec<f32> {
    let low = lowpass_filter(input, cutoff_hz, sample_rate);
    input.iter().zip(low.iter()).map(|(&x, &l)| x - l).collect()
}
