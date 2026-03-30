use daw_engine::plugins::{BiquadFilter, BiquadType, Compressor, SimpleReverb, DelayLine};
use daw_engine::dsp::generate_sine_wave;

#[test]
fn test_biquad_lowpass_attenuates_above_cutoff() {
    // Low-pass at 1kHz; feed a 10kHz sine — output should be attenuated
    let mut filter = BiquadFilter::new(BiquadType::LowPass, 1000.0, 0.707, 0.0, 44100.0);

    let mut phase = 0.0f32;
    let mut input = vec![0.0f32; 4096];
    generate_sine_wave(10000.0, 44100.0, &mut phase, &mut input);

    let input_peak: f32 = input.iter().map(|x| x.abs()).fold(0.0f32, f32::max);

    filter.process_buffer(&mut input);
    let output_peak: f32 = input.iter().map(|x| x.abs()).fold(0.0f32, f32::max);

    assert!(output_peak < input_peak * 0.5, "Expected attenuation: input_peak={}, output_peak={}", input_peak, output_peak);
}

#[test]
fn test_biquad_highpass_attenuates_dc() {
    // High-pass at 1kHz; feed DC (constant value) — output should settle near zero
    let mut filter = BiquadFilter::new(BiquadType::HighPass, 1000.0, 0.707, 0.0, 44100.0);

    let mut input = vec![1.0f32; 4096];
    filter.process_buffer(&mut input);

    // After settling, the last few hundred samples should be near zero
    let tail: &[f32] = &input[3800..];
    let peak: f32 = tail.iter().map(|x| x.abs()).fold(0.0f32, f32::max);
    assert!(peak < 0.01, "DC should be attenuated to near zero, got {}", peak);
}

#[test]
fn test_compressor_reduces_loud_signal() {
    let mut comp = Compressor::new(-6.0, 4.0, 1.0, 100.0, 0.0, 44100.0);

    // Feed a loud signal at 0.9 amplitude
    let input: Vec<f32> = (0..4096).map(|i| if i % 2 == 0 { 0.9 } else { -0.9 }).collect();
    let output: Vec<f32> = input.iter().map(|&x| comp.process_sample(x)).collect();

    // After the compressor settles (skip first 1000 samples), output should be smaller than input
    let input_peak: f32 = input[1000..].iter().map(|x| x.abs()).fold(0.0f32, f32::max);
    let output_peak: f32 = output[1000..].iter().map(|x| x.abs()).fold(0.0f32, f32::max);

    assert!(output_peak < input_peak, "Compressor should reduce peak: input={}, output={}", input_peak, output_peak);
}

#[test]
fn test_reverb_produces_tail() {
    let mut reverb = SimpleReverb::new(0.8, 0.5, 44100);

    // Feed an impulse followed by silence
    // The shortest comb delay is 1116 samples at 44100Hz,
    // so the reverb tail starts after that.
    let total = 3000usize;
    let mut output = vec![0.0f32; total];
    output[0] = reverb.process_sample(1.0);
    for i in 1..total {
        output[i] = reverb.process_sample(0.0);
    }

    // After the comb delay (> 1116 samples), there should be non-zero energy
    let tail_energy: f32 = output[1116..2500].iter().map(|x| x * x).sum();
    assert!(tail_energy > 0.0, "Reverb should produce a tail after the comb delay");
}

#[test]
fn test_delay_line_output_appears_after_delay() {
    // Delay of ~10ms at 44100 Hz = ~441 samples
    let delay_ms = 10.0;
    let sample_rate = 44100u32;
    let expected_delay = (delay_ms / 1000.0 * sample_rate as f32) as usize;

    let mut delay = DelayLine::new(delay_ms, 0.0, 1.0, sample_rate);

    let mut output = vec![0.0f32; expected_delay + 100];
    // Feed impulse at sample 0
    output[0] = delay.process_sample(1.0);
    for i in 1..output.len() {
        output[i] = delay.process_sample(0.0);
    }

    // The delayed signal should appear around expected_delay samples later
    let delayed_peak_idx = output.iter().enumerate()
        .max_by(|a, b| a.1.abs().partial_cmp(&b.1.abs()).unwrap())
        .map(|(i, _)| i)
        .unwrap();

    assert!(
        delayed_peak_idx >= expected_delay.saturating_sub(2),
        "Delay peak should be at ~{} samples, got {}", expected_delay, delayed_peak_idx
    );
}
