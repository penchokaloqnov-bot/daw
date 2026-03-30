use daw_midi::{
    MidiMessage, MidiTrack, MidiSequencer, PolySynth, WaveShape,
};
use daw_midi::types::{midi_pitch_to_hz, ticks_to_samples};
use daw_engine::AudioCommand;

#[test]
fn test_midi_pitch_to_hz() {
    let a4 = midi_pitch_to_hz(69);
    assert!((a4 - 440.0).abs() < 0.1, "A4 should be 440 Hz, got {}", a4);

    let a5 = midi_pitch_to_hz(81);
    assert!((a5 - 880.0).abs() < 0.5, "A5 should be 880 Hz, got {}", a5);

    let c4 = midi_pitch_to_hz(60);
    assert!((c4 - 261.63).abs() < 0.1, "C4 should be ~261.63 Hz, got {}", c4);
}

#[test]
fn test_ticks_to_samples() {
    // 120 BPM, 480 ticks/beat, 44100 Hz
    // 1 beat = 480 ticks = 44100/2 = 22050 samples
    let samples = ticks_to_samples(480, 480, 120.0, 44100);
    assert_eq!(samples, 22050, "1 beat should be 22050 samples, got {}", samples);

    let zero = ticks_to_samples(0, 480, 120.0, 44100);
    assert_eq!(zero, 0);
}

#[test]
fn test_midi_track_sort_and_duration() {
    let mut track = MidiTrack::new("test");
    track.add_event(480, MidiMessage::NoteOff { channel: 0, pitch: 60 });
    track.add_event(0, MidiMessage::NoteOn { channel: 0, pitch: 60, velocity: 100 });
    track.add_event(240, MidiMessage::NoteOn { channel: 0, pitch: 64, velocity: 90 });

    assert_eq!(track.duration_ticks(), 480);

    track.sort();
    assert_eq!(track.events[0].tick, 0);
    assert_eq!(track.events[1].tick, 240);
    assert_eq!(track.events[2].tick, 480);
}

#[test]
fn test_midi_sequencer_advance() {
    let mut track = MidiTrack::new("test");
    // NoteOn at tick 0, NoteOff at tick 480 (120bpm, 480tpb → 22050 samples each)
    track.add_event(0, MidiMessage::NoteOn { channel: 0, pitch: 60, velocity: 100 });
    track.add_event(480, MidiMessage::NoteOff { channel: 0, pitch: 60 });

    let mut seq = MidiSequencer::new(track, 480, 44100, 120.0);

    // First advance: 22050 samples — should get NoteOn (tick 0 → sample 0)
    let cmds = seq.advance(22050);
    assert_eq!(cmds.len(), 1, "Should get NoteOn");
    assert!(matches!(cmds[0], AudioCommand::NoteOn { pitch: 60, velocity: 100 }));

    // Second advance: another 22050 samples — should get NoteOff (tick 480 → sample 22050)
    let cmds = seq.advance(22050);
    assert_eq!(cmds.len(), 1, "Should get NoteOff");
    assert!(matches!(cmds[0], AudioCommand::NoteOff { pitch: 60 }));

    assert!(seq.is_finished());
}

#[test]
fn test_polysynth_basic_generation() {
    let mut synth = PolySynth::new(8, 44100, WaveShape::Sine);
    synth.note_on(69, 100);

    let mut output = vec![0.0f32; 128];
    synth.generate_block(&mut output);

    let has_nonzero = output.iter().any(|&s| s != 0.0);
    assert!(has_nonzero, "Output should not be all zeros");

    let peak = output.iter().map(|x| x.abs()).fold(0.0f32, f32::max);
    assert!(peak <= 1.0, "Peak should be <= 1.0, got {}", peak);
}

#[test]
fn test_polysynth_voice_stealing() {
    let mut synth = PolySynth::new(2, 44100, WaveShape::Sine);
    synth.note_on(60, 100);
    synth.note_on(64, 100);
    synth.note_on(67, 100); // should steal oldest voice

    assert!(synth.active_voice_count() <= 2, "Should have at most 2 voices");
}

#[test]
fn test_all_wave_shapes_produce_nonzero() {
    for wave in [WaveShape::Sine, WaveShape::Sawtooth, WaveShape::Square, WaveShape::Triangle] {
        let mut synth = PolySynth::new(4, 44100, wave);
        synth.note_on(69, 100);

        let mut output = vec![0.0f32; 256];
        synth.generate_block(&mut output);

        let has_nonzero = output.iter().any(|&s| s != 0.0);
        assert!(has_nonzero, "Wave shape {:?} produced all-zero output", wave);
    }
}
