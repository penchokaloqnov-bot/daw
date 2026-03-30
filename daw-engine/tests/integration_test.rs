use daw_engine::*;

#[test]
fn test_buffer_pool_checkout_and_return() {
    let pool = AudioBufferPool::new(4, 512);
    {
        let buf = pool.checkout().expect("should checkout buffer");
        assert_eq!(buf.len(), 512);
    } // buf dropped here, returned to pool
    let buf2 = pool.checkout().expect("should checkout after return");
    assert_eq!(buf2.len(), 512);
}

#[test]
fn test_command_queue() {
    let (mut prod, mut cons) = create_command_queue(16);
    prod.send(AudioCommand::SetVolume { track_id: 0, value: 0.8 }).unwrap();
    prod.send(AudioCommand::StartPlayback).unwrap();

    let cmd1 = cons.try_recv().expect("should have command");
    match cmd1 {
        AudioCommand::SetVolume { track_id, value } => {
            assert_eq!(track_id, 0);
            assert!((value - 0.8).abs() < 1e-6);
        }
        _ => panic!("unexpected command"),
    }
    let cmd2 = cons.try_recv().expect("should have command");
    assert!(matches!(cmd2, AudioCommand::StartPlayback));
}

#[test]
fn test_audio_graph_topo_sort() {
    let mut graph = AudioGraph::new(64);
    let source = graph.add_node(Box::new(SourceNode::new(440.0, 44100.0)));
    let gain = graph.add_node(Box::new(GainNode::new(0.5)));
    let master = graph.add_node(Box::new(MasterNode));

    graph.connect(source, gain).unwrap();
    graph.connect(gain, master).unwrap();

    let params = NodeParams::default();
    let output = graph.process_graph(64, &params).unwrap();
    assert_eq!(output.len(), 64);
    let peak = compute_peak(&output);
    assert!(peak > 0.0, "Expected non-zero output, got peak={}", peak);
}

#[test]
fn test_automation_curve_interpolation() {
    let mut curve = AutomationCurve::new(0.0);
    curve.add_point(AutomationPoint { sample: 0, value: 0.0 });
    curve.add_point(AutomationPoint { sample: 100, value: 1.0 });

    assert!((curve.get_value_at(0) - 0.0).abs() < 1e-6);
    assert!((curve.get_value_at(50) - 0.5).abs() < 1e-6);
    assert!((curve.get_value_at(100) - 1.0).abs() < 1e-6);
    assert!((curve.get_value_at(0) - 0.0).abs() < 1e-6);
    assert!((curve.get_value_at(200) - 1.0).abs() < 1e-6);
}

#[test]
fn test_simd_gain_matches_scalar() {
    let original: Vec<f32> = (0..64).map(|i| (i as f32) * 0.01).collect();
    let gain = 0.75;

    let mut simd_buf = original.clone();
    apply_gain_simd(&mut simd_buf, gain);

    for (i, (&orig, &simd)) in original.iter().zip(simd_buf.iter()).enumerate() {
        let scalar = orig * gain;
        assert!((simd - scalar).abs() < 1e-6, "Mismatch at {}: simd={}, scalar={}", i, simd, scalar);
    }
}

#[test]
fn test_engine_block_processing() {
    let config = AudioEngineConfig {
        sample_rate: 44100,
        buffer_size: 512,
        num_tracks: 4,
    };
    let (cmd_prod, cmd_cons) = create_command_queue(64);
    let (tel_prod, tel_cons) = create_telemetry_queue(256);

    let mut engine = AudioEngine::new(config, cmd_cons, tel_prod);
    let mut handle = AudioEngineHandle {
        command_producer: cmd_prod,
        telemetry_consumer: tel_cons,
    };

    let source = engine.graph.add_node(Box::new(SourceNode::new(440.0, 44100.0)));
    let master = engine.graph.add_node(Box::new(MasterNode));
    engine.graph.connect(source, master).unwrap();

    handle.send_command(AudioCommand::StartPlayback).unwrap();
    handle.send_command(AudioCommand::SetTempo { bpm: 140.0 }).unwrap();

    for _ in 0..10 {
        let block = engine.process_block();
        assert_eq!(block.len(), 512);
    }

    let telemetry = handle.drain_telemetry();
    assert!(!telemetry.is_empty(), "Should have telemetry events");
}
