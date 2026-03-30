use wide::f32x4;

pub fn apply_gain_simd(buffer: &mut [f32], gain: f32) {
    let gain_vec = f32x4::splat(gain);
    let chunks = buffer.len() / 4;
    let remainder = buffer.len() % 4;

    for i in 0..chunks {
        let offset = i * 4;
        let v = f32x4::new([buffer[offset], buffer[offset+1], buffer[offset+2], buffer[offset+3]]);
        let result = v * gain_vec;
        let arr: [f32; 4] = result.into();
        buffer[offset..offset+4].copy_from_slice(&arr);
    }

    let offset = chunks * 4;
    for j in 0..remainder {
        buffer[offset + j] *= gain;
    }
}

pub fn mix_buffers_simd(dst: &mut [f32], src: &[f32], gain: f32) {
    let gain_vec = f32x4::splat(gain);
    let len = dst.len().min(src.len());
    let chunks = len / 4;
    let remainder = len % 4;

    for i in 0..chunks {
        let offset = i * 4;
        let d = f32x4::new([dst[offset], dst[offset+1], dst[offset+2], dst[offset+3]]);
        let s = f32x4::new([src[offset], src[offset+1], src[offset+2], src[offset+3]]);
        let result = d + s * gain_vec;
        let arr: [f32; 4] = result.into();
        dst[offset..offset+4].copy_from_slice(&arr);
    }

    let offset = chunks * 4;
    for j in 0..remainder {
        dst[offset + j] += src[offset + j] * gain;
    }
}

pub fn apply_pan_simd(buffer: &mut [f32], pan: f32) {
    let scale = 1.0 - pan.abs() * 0.5;
    apply_gain_simd(buffer, scale);
}

pub fn generate_sine_wave(frequency: f32, sample_rate: f32, phase: &mut f32, output: &mut [f32]) {
    use std::f32::consts::PI;
    let phase_increment = 2.0 * PI * frequency / sample_rate;
    for sample in output.iter_mut() {
        *sample = phase.sin();
        *phase += phase_increment;
        if *phase > 2.0 * PI { *phase -= 2.0 * PI; }
    }
}

pub fn db_to_linear(db: f32) -> f32 {
    10.0f32.powf(db / 20.0)
}

pub fn linear_to_db(linear: f32) -> f32 {
    if linear <= 0.0 {
        return -f32::INFINITY;
    }
    20.0 * linear.log10()
}

pub fn compute_peak(buffer: &[f32]) -> f32 {
    buffer.iter().map(|x| x.abs()).fold(0.0f32, f32::max)
}

pub fn compute_rms(buffer: &[f32]) -> f32 {
    if buffer.is_empty() { return 0.0; }
    let sum_sq: f32 = buffer.iter().map(|x| x * x).sum();
    (sum_sq / buffer.len() as f32).sqrt()
}
