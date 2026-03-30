#[derive(Debug, Clone, PartialEq)]
pub enum MidiMessage {
    NoteOn { channel: u8, pitch: u8, velocity: u8 },
    NoteOff { channel: u8, pitch: u8 },
    ControlChange { channel: u8, controller: u8, value: u8 },
    ProgramChange { channel: u8, program: u8 },
    PitchBend { channel: u8, value: i16 },
    AfterTouch { channel: u8, pressure: u8 },
    AllNotesOff { channel: u8 },
}

#[derive(Debug, Clone)]
pub struct MidiEvent {
    pub tick: u64,
    pub message: MidiMessage,
}

#[derive(Debug, Clone)]
pub struct MidiTrack {
    pub name: String,
    pub events: Vec<MidiEvent>,
}

impl MidiTrack {
    pub fn new(name: &str) -> Self {
        MidiTrack { name: name.to_string(), events: Vec::new() }
    }

    pub fn add_event(&mut self, tick: u64, message: MidiMessage) {
        self.events.push(MidiEvent { tick, message });
    }

    pub fn sort(&mut self) {
        self.events.sort_by_key(|e| e.tick);
    }

    pub fn duration_ticks(&self) -> u64 {
        self.events.iter().map(|e| e.tick).max().unwrap_or(0)
    }
}

pub fn midi_pitch_to_hz(pitch: u8) -> f32 {
    440.0 * 2.0_f32.powf((pitch as f32 - 69.0) / 12.0)
}

pub fn ticks_to_samples(tick: u64, ticks_per_beat: u32, bpm: f32, sample_rate: u32) -> u64 {
    (tick as f64 * 60.0 / (bpm as f64 * ticks_per_beat as f64) * sample_rate as f64) as u64
}
