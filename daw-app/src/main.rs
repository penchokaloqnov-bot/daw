use daw_engine::*;
use daw_ai::{SmartEq, StemSeparator, StemSeparatorConfig};
use daw_collab::CollaborativeProject;
use daw_midi::{MidiMessage, MidiTrack, MidiSequencer, PolySynth, WaveShape};
use tracing::info;

fn main() {
    tracing_subscriber::fmt::init();

    info!("Starting DAW demo");

    let config = AudioEngineConfig {
        sample_rate: 44100,
        buffer_size: 512,
        num_tracks: 4,
    };

    let (cmd_prod, cmd_cons) = create_command_queue(256);
    let (tel_prod, tel_cons) = create_telemetry_queue(1024);

    let mut engine = AudioEngine::new(config, cmd_cons, tel_prod);

    let mut handle = AudioEngineHandle {
        command_producer: cmd_prod,
        telemetry_consumer: tel_cons,
    };

    let source = engine.graph.add_node(Box::new(SourceNode::new(440.0, 44100.0)));
    let gain = engine.graph.add_node(Box::new(GainNode::new(0.8)));
    let master = engine.graph.add_node(Box::new(MasterNode));
    engine.graph.connect(source, gain).unwrap();
    engine.graph.connect(gain, master).unwrap();

    assert!(handle.send_command(AudioCommand::SetVolume { track_id: 0, value: 0.9 }), "Command queue full");
    assert!(handle.send_command(AudioCommand::SetTempo { bpm: 128.0 }), "Command queue full");
    assert!(handle.send_command(AudioCommand::StartPlayback), "Command queue full");

    let mut all_audio = Vec::new();
    for i in 0..10 {
        let block = engine.process_block();
        println!("Block {}: {} samples, peak={:.4}", i, block.len(), compute_peak(&block));
        all_audio.extend_from_slice(&block);
    }

    let telemetry = handle.drain_telemetry();
    println!("Received {} telemetry events", telemetry.len());
    for event in &telemetry {
        match event {
            TelemetryEvent::PeakMeter { track_id, peak } => {
                println!("  Peak meter track={}: {:.4}", track_id, peak);
            }
            TelemetryEvent::PlayheadPosition { sample } => {
                println!("  Playhead: sample={}", sample);
            }
            TelemetryEvent::CpuLoad { percent } => {
                println!("  CPU load: {:.1}%", percent);
            }
        }
    }

    // --- MIDI + Synth Demo ---
    println!("\n--- MIDI & PolySynth Demo ---");

    // C4-E4-G4 triad arpeggiated at 120 BPM, 480 ticks/beat
    let mut midi_track = MidiTrack::new("Arpeggio");
    // C4=60, E4=64, G4=67; each note 240 ticks = half beat
    let notes = [(60u8, 0u64), (64, 240), (67, 480)];
    for (pitch, start_tick) in notes {
        midi_track.add_event(start_tick, MidiMessage::NoteOn { channel: 0, pitch, velocity: 100 });
        midi_track.add_event(start_tick + 200, MidiMessage::NoteOff { channel: 0, pitch });
    }
    midi_track.sort();

    let mut synth = PolySynth::new(8, 44100, WaveShape::Sine);
    let mut seq = MidiSequencer::new(midi_track, 480, 44100, 120.0);

    // Add synth + filter + reverb chain in a new graph
    let mut synth_graph = AudioGraph::new(512);
    let synth_node = synth_graph.add_node(Box::new(PolySynth::new(8, 44100, WaveShape::Sine)));
    let filter_node = synth_graph.add_node(Box::new(BiquadFilter::new(
        BiquadType::LowPass, 4000.0, 0.707, 0.0, 44100.0,
    )));
    let reverb_node = synth_graph.add_node(Box::new(SimpleReverb::new(0.7, 0.3, 44100)));
    synth_graph.connect(synth_node, filter_node).unwrap();
    synth_graph.connect(filter_node, reverb_node).unwrap();

    // Drive the standalone synth with the sequencer for 20 blocks
    let mut synth_audio = Vec::new();
    for block_idx in 0..20 {
        let cmds = seq.advance(512);
        for cmd in cmds {
            match cmd {
                AudioCommand::NoteOn { pitch, velocity } => synth.note_on(pitch, velocity),
                AudioCommand::NoteOff { pitch } => synth.note_off(pitch),
                _ => {}
            }
        }
        let mut block = vec![0.0f32; 512];
        synth.generate_block(&mut block);
        let peak = compute_peak(&block);
        if block_idx < 5 {
            println!("Synth block {}: peak={:.4}, active_voices={}", block_idx, peak, synth.active_voice_count());
        }
        synth_audio.extend_from_slice(&block);
    }
    println!("Synth produced {} samples total", synth_audio.len());

    // --- Collaborative Project Demo ---
    println!("\n--- Collaborative Project Demo ---");
    let mut project = CollaborativeProject::new();
    project.add_track(1, "Drums").unwrap();
    project.add_track(2, "Bass").unwrap();
    project.set_track_volume(1, 0.8).unwrap();
    project.mute_track(2, true).unwrap();

    let saved = project.save();
    println!("Saved project: {} bytes", saved.len());

    let loaded = CollaborativeProject::load(&saved).unwrap();
    let tracks = loaded.get_tracks();
    println!("Loaded {} tracks", tracks.len());
    for track in &tracks {
        println!("  Track {}: '{}' volume={:.2} muted={}", track.id, track.name, track.volume, track.muted);
    }

    // --- Project File Save/Load Demo ---
    println!("\n--- Project File Format Demo ---");
    let mut pf = ProjectFile::new("My DAW Project", 44100, 512);
    pf.tempo_bpm = 120.0;
    pf.master_volume = 0.9;

    let mut t1 = TrackConfig::new(0, "Synth");
    t1.node_type = "polysynth".to_string();
    t1.node_params.insert("max_voices".to_string(), 8.0);
    pf.add_track(t1);

    let mut t2 = TrackConfig::new(1, "Drums");
    t2.volume = 0.8;
    pf.add_track(t2);

    pf.add_automation(AutomationData {
        parameter: "master_volume".to_string(),
        points: vec![(0, 0.0), (44100, 1.0)],
    });

    pf.validate().expect("Project should be valid");
    let json = pf.to_json().expect("Serialize to JSON");
    println!("Project JSON ({} bytes):\n{}", json.len(), &json[..json.len().min(300)]);

    let loaded_pf = ProjectFile::from_json(&json).expect("Deserialize from JSON");
    assert_eq!(loaded_pf.name, pf.name);
    assert_eq!(loaded_pf.tracks.len(), 2);
    println!("Project loaded back successfully: '{}' with {} tracks", loaded_pf.name, loaded_pf.tracks.len());

    // --- AI EQ Analysis Demo ---
    println!("\n--- AI EQ Analysis Demo ---");
    let eq = SmartEq::new();
    let suggestion = eq.analyze(&all_audio[..512.min(all_audio.len())], 44100);
    println!("EQ suggestion: {}", suggestion.description);
    for band in &suggestion.bands {
        println!("  Band: {}Hz, {:+.1}dB, Q={:.1}", band.frequency, band.gain_db, band.q);
    }

    let separator = StemSeparator::new(StemSeparatorConfig { sample_rate: 44100 });
    let stems = separator.separate(&all_audio[..512.min(all_audio.len())], 44100);
    println!("\nStem separation on {} samples:", 512.min(all_audio.len()));
    println!("  Bass peak: {:.4}", stems.bass.iter().map(|x| x.abs()).fold(0.0f32, f32::max));
    println!("  Vocals peak: {:.4}", stems.vocals.iter().map(|x| x.abs()).fold(0.0f32, f32::max));

    info!("DAW demo complete");
}
