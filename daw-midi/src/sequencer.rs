use crate::types::{MidiTrack, MidiMessage, ticks_to_samples};
use daw_engine::AudioCommand;

pub struct MidiSequencer {
    pub track: MidiTrack,
    pub ticks_per_beat: u32,
    pub sample_rate: u32,
    pub current_sample: u64,
    pub next_event_idx: usize,
    pub bpm: f32,
}

impl MidiSequencer {
    pub fn new(track: MidiTrack, ticks_per_beat: u32, sample_rate: u32, bpm: f32) -> Self {
        MidiSequencer {
            track,
            ticks_per_beat,
            sample_rate,
            current_sample: 0,
            next_event_idx: 0,
            bpm,
        }
    }

    pub fn reset(&mut self) {
        self.current_sample = 0;
        self.next_event_idx = 0;
    }

    pub fn set_bpm(&mut self, bpm: f32) {
        self.bpm = bpm;
    }

    pub fn advance(&mut self, num_samples: u64) -> Vec<AudioCommand> {
        let mut commands = Vec::new();
        let window_end = self.current_sample + num_samples;

        while self.next_event_idx < self.track.events.len() {
            let event = &self.track.events[self.next_event_idx];
            let event_sample = ticks_to_samples(
                event.tick,
                self.ticks_per_beat,
                self.bpm,
                self.sample_rate,
            );

            if event_sample >= window_end {
                break;
            }

            match &event.message {
                MidiMessage::NoteOn { pitch, velocity, .. } => {
                    commands.push(AudioCommand::NoteOn { pitch: *pitch, velocity: *velocity });
                }
                MidiMessage::NoteOff { pitch, .. } => {
                    commands.push(AudioCommand::NoteOff { pitch: *pitch });
                }
                other => {
                    tracing::debug!("Ignoring MIDI message: {:?}", other);
                }
            }

            self.next_event_idx += 1;
        }

        self.current_sample += num_samples;
        commands
    }

    pub fn is_finished(&self) -> bool {
        self.next_event_idx >= self.track.events.len()
    }
}
