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


// --- Stereo processing ---

pub fn apply_gain_stereo(buffer: &mut [f32], gain_l: f32, gain_r: f32) {
    for chunk in buffer.chunks_mut(2) {
        if chunk.len() == 2 {
            chunk[0] *= gain_l;
            chunk[1] *= gain_r;
        }
    }
}

pub fn apply_pan_stereo(buffer: &mut [f32], pan: f32) {
    use std::f32::consts::PI;
    let gain_l = (PI / 4.0 * (1.0 - pan)).cos();
    let gain_r = (PI / 4.0 * (1.0 + pan)).cos();
    apply_gain_stereo(buffer, gain_l, gain_r);
}

pub fn mono_to_stereo(mono: &[f32]) -> Vec<f32> {
    let mut stereo = Vec::with_capacity(mono.len() * 2);
    for &s in mono {
        stereo.push(s);
        stereo.push(s);
    }
    stereo
}

pub fn stereo_to_mono(stereo: &[f32]) -> Vec<f32> {
    stereo.chunks(2)
        .map(|c| if c.len() == 2 { (c[0] + c[1]) * 0.5 } else { c[0] })
        .collect()
}

pub fn compute_stereo_peak(stereo: &[f32]) -> (f32, f32) {
    let mut left_peak = 0.0f32;
    let mut right_peak = 0.0f32;
    for (i, &s) in stereo.iter().enumerate() {
        if i % 2 == 0 {
            left_peak = left_peak.max(s.abs());
        } else {
            right_peak = right_peak.max(s.abs());
        }
    }
    (left_peak, right_peak)
}

pub fn compute_stereo_rms(stereo: &[f32]) -> (f32, f32) {
    let mut left_sum_sq = 0.0f32;
    let mut right_sum_sq = 0.0f32;
    let mut left_count = 0usize;
    let mut right_count = 0usize;
    for (i, &s) in stereo.iter().enumerate() {
        if i % 2 == 0 {
            left_sum_sq += s * s;
            left_count += 1;
        } else {
            right_sum_sq += s * s;
            right_count += 1;
        }
    }
    let left_rms = if left_count > 0 { (left_sum_sq / left_count as f32).sqrt() } else { 0.0 };
    let right_rms = if right_count > 0 { (right_sum_sq / right_count as f32).sqrt() } else { 0.0 };
    (left_rms, right_rms)
}
