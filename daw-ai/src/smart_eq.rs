#[derive(Debug, Clone)]
pub struct EqBand {
    pub frequency: f32,
    pub gain_db: f32,
    pub q: f32,
}

#[derive(Debug, Clone)]
pub struct EqSuggestion {
    pub bands: Vec<EqBand>,
    pub description: String,
}

pub struct SmartEq;

impl SmartEq {
    pub fn new() -> Self { SmartEq }

    pub fn analyze(&self, audio: &[f32], sample_rate: u32) -> EqSuggestion {
        let bass_energy = band_energy(audio, 20.0, 200.0, sample_rate);
        let mud_energy = band_energy(audio, 200.0, 500.0, sample_rate);
        let mid_energy = band_energy(audio, 500.0, 2000.0, sample_rate);
        let harsh_energy = band_energy(audio, 2000.0, 5000.0, sample_rate);
        let air_energy = band_energy(audio, 5000.0, 20000.0, sample_rate);

        let total = bass_energy + mud_energy + mid_energy + harsh_energy + air_energy;
        let mut bands = Vec::new();
        let mut desc = Vec::new();

        if total > 0.0 {
            let mud_ratio = mud_energy / total;
            let harsh_ratio = harsh_energy / total;

            if mud_ratio > 0.35 {
                bands.push(EqBand { frequency: 350.0, gain_db: -3.0, q: 0.7 });
                desc.push("Muddy frequencies detected at 350Hz");
            }
            if harsh_ratio > 0.35 {
                bands.push(EqBand { frequency: 3500.0, gain_db: -2.0, q: 1.0 });
                desc.push("Harsh frequencies detected at 3.5kHz");
            }
        }

        if bands.is_empty() {
            desc.push("No significant EQ adjustments needed");
        }

        EqSuggestion {
            bands,
            description: desc.join("; "),
        }
    }
}

impl Default for SmartEq {
    fn default() -> Self { Self::new() }
}

fn band_energy(audio: &[f32], low_hz: f32, high_hz: f32, sample_rate: u32) -> f32 {
    let hp = highpass_filter(audio, low_hz, sample_rate);
    let bp = lowpass_filter(&hp, high_hz, sample_rate);
    bp.iter().map(|x| x * x).sum::<f32>() / bp.len().max(1) as f32
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
