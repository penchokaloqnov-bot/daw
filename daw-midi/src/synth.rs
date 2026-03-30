use std::f32::consts::PI;
use daw_engine::{AudioNode, NodeParams};
use crate::types::midi_pitch_to_hz;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WaveShape {
    Sine,
    Sawtooth,
    Square,
    Triangle,
}

pub struct SynthVoice {
    pub pitch: u8,
    pub velocity: u8,
    pub frequency: f32,
    pub phase: f32,
    pub amplitude: f32,
    pub wave_shape: WaveShape,
    pub is_active: bool,
    pub attack_samples: u32,
    pub decay_samples: u32,
    pub sustain_level: f32,
    pub release_samples: u32,
    pub envelope_sample: u32,
    pub releasing: bool,
    pub release_start_amplitude: f32,
    sample_rate: u32,
}

impl SynthVoice {
    pub fn new(pitch: u8, velocity: u8, sample_rate: u32, wave_shape: WaveShape) -> Self {
        let frequency = midi_pitch_to_hz(pitch);
        let amplitude = velocity as f32 / 127.0;
        SynthVoice {
            pitch,
            velocity,
            frequency,
            phase: 0.0,
            amplitude,
            wave_shape,
            is_active: true,
            attack_samples: (sample_rate as f32 * 0.01) as u32,
            decay_samples: (sample_rate as f32 * 0.05) as u32,
            sustain_level: 0.7,
            release_samples: (sample_rate as f32 * 0.1) as u32,
            envelope_sample: 0,
            releasing: false,
            release_start_amplitude: 0.0,
            sample_rate,
        }
    }

    pub fn trigger_release(&mut self) {
        self.release_start_amplitude = self.current_envelope_value();
        self.releasing = true;
        self.envelope_sample = 0;
    }

    pub fn current_envelope_value(&self) -> f32 {
        if self.releasing {
            if self.release_samples == 0 {
                return 0.0;
            }
            let t = (self.envelope_sample as f32 / self.release_samples as f32).min(1.0);
            self.release_start_amplitude * (1.0 - t)
        } else if self.envelope_sample < self.attack_samples {
            if self.attack_samples == 0 {
                return self.amplitude;
            }
            let t = self.envelope_sample as f32 / self.attack_samples as f32;
            t * self.amplitude
        } else if self.envelope_sample < self.attack_samples + self.decay_samples {
            if self.decay_samples == 0 {
                return self.sustain_level * self.amplitude;
            }
            let t = (self.envelope_sample - self.attack_samples) as f32 / self.decay_samples as f32;
            self.amplitude * (1.0 - t * (1.0 - self.sustain_level))
        } else {
            self.sustain_level * self.amplitude
        }
    }

    pub fn generate_sample(&mut self) -> f32 {
        let envelope = self.current_envelope_value();

        let waveform = match self.wave_shape {
            WaveShape::Sine => self.phase.sin(),
            WaveShape::Sawtooth => (self.phase / PI) - 1.0,
            WaveShape::Square => if self.phase < PI { 1.0 } else { -1.0 },
            WaveShape::Triangle => {
                let phase_norm = self.phase / (2.0 * PI);
                if phase_norm < 0.25 {
                    4.0 * phase_norm
                } else if phase_norm < 0.75 {
                    2.0 - 4.0 * phase_norm
                } else {
                    4.0 * phase_norm - 4.0
                }
            }
        };

        let result = waveform * envelope;

        self.phase += 2.0 * PI * self.frequency / self.sample_rate as f32;
        if self.phase >= 2.0 * PI {
            self.phase -= 2.0 * PI;
        }

        self.envelope_sample += 1;

        if self.releasing && self.envelope_sample >= self.release_samples {
            self.is_active = false;
        }

        result
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }
}

pub struct PolySynth {
    pub voices: Vec<SynthVoice>,
    pub max_voices: usize,
    pub sample_rate: u32,
    pub wave_shape: WaveShape,
}

impl PolySynth {
    pub fn new(max_voices: usize, sample_rate: u32, wave_shape: WaveShape) -> Self {
        PolySynth {
            voices: Vec::new(),
            max_voices,
            sample_rate,
            wave_shape,
        }
    }

    pub fn note_on(&mut self, pitch: u8, velocity: u8) {
        // Steal voice with same pitch if any
        if let Some(v) = self.voices.iter_mut().find(|v| v.pitch == pitch && v.is_active) {
            v.trigger_release();
        }

        // If at max voices, steal the oldest (first) active voice
        if self.voices.len() >= self.max_voices {
            self.voices.remove(0);
        }

        self.voices.push(SynthVoice::new(pitch, velocity, self.sample_rate, self.wave_shape));
    }

    pub fn note_off(&mut self, pitch: u8) {
        if let Some(v) = self.voices.iter_mut().find(|v| v.pitch == pitch && v.is_active && !v.releasing) {
            v.trigger_release();
        }
    }

    pub fn note_off_all(&mut self) {
        for v in self.voices.iter_mut() {
            if v.is_active && !v.releasing {
                v.trigger_release();
            }
        }
    }

    pub fn generate_block(&mut self, output: &mut [f32]) {
        let scale = 1.0 / self.max_voices as f32;
        for sample in output.iter_mut() {
            let sum: f32 = self.voices.iter_mut()
                .filter(|v| v.is_active)
                .map(|v| v.generate_sample())
                .sum();
            *sample = sum * scale;
        }
        self.voices.retain(|v| v.is_active);
    }

    pub fn active_voice_count(&self) -> usize {
        self.voices.iter().filter(|v| v.is_active).count()
    }
}

impl AudioNode for PolySynth {
    fn process(&mut self, _inputs: &[&[f32]], output: &mut [f32], _params: &NodeParams) {
        self.generate_block(output);
    }

    fn name(&self) -> &str {
        "PolySynth"
    }
}
