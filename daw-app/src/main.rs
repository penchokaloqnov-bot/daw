use daw_engine::*;
use daw_ai::{SmartEq, StemSeparator, StemSeparatorConfig};
use daw_collab::CollaborativeProject;
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
