use std::collections::HashMap;
use tracing::{info, debug};
use crate::{
    audio_graph::{AudioGraph, NodeParams},
    automation::AutomationEngine,
    memory_pool::AudioBufferPool,
    commands::{AudioCommand, TelemetryEvent, CommandConsumer, TelemetryProducer, CommandProducer, TelemetryConsumer},
    dsp,
};

pub struct AudioEngineConfig {
    pub sample_rate: u32,
    pub buffer_size: usize,
    pub num_tracks: usize,
}

pub struct EngineState {
    pub is_playing: bool,
    pub playhead_sample: u64,
    pub tempo_bpm: f32,
    pub master_volume: f32,
}

impl Default for EngineState {
    fn default() -> Self {
        EngineState {
            is_playing: false,
            playhead_sample: 0,
            tempo_bpm: 120.0,
            master_volume: 1.0,
        }
    }
}

pub struct AudioEngine {
    pub config: AudioEngineConfig,
    pub graph: AudioGraph,
    pub automation: AutomationEngine,
    pub buffer_pool: AudioBufferPool,
    command_consumer: CommandConsumer,
    telemetry_producer: TelemetryProducer,
    pub state: EngineState,
    pub track_volumes: HashMap<usize, f32>,
    pub track_pans: HashMap<usize, f32>,
}

impl AudioEngine {
    pub fn new(config: AudioEngineConfig, command_consumer: CommandConsumer, telemetry_producer: TelemetryProducer) -> Self {
        let graph = AudioGraph::new(config.buffer_size);
        let automation = AutomationEngine::new();
        let buffer_pool = AudioBufferPool::new(32, config.buffer_size);
        AudioEngine {
            config,
            graph,
            automation,
            buffer_pool,
            command_consumer,
            telemetry_producer,
            state: EngineState::default(),
            track_volumes: HashMap::new(),
            track_pans: HashMap::new(),
        }
    }

    pub fn process_commands(&mut self) {
        while let Some(cmd) = self.command_consumer.try_recv() {
            debug!("Processing command: {:?}", cmd);
            match cmd {
                AudioCommand::NoteOn { pitch, velocity } => {
                    info!("NoteOn: pitch={}, velocity={}", pitch, velocity);
                }
                AudioCommand::NoteOff { pitch } => {
                    info!("NoteOff: pitch={}", pitch);
                }
                AudioCommand::SetVolume { track_id, value } => {
                    self.track_volumes.insert(track_id, value);
                }
                AudioCommand::SetPan { track_id, value } => {
                    self.track_pans.insert(track_id, value);
                }
                AudioCommand::AddTrack { track_id } => {
                    info!("AddTrack: {}", track_id);
                    self.track_volumes.entry(track_id).or_insert(1.0);
                    self.track_pans.entry(track_id).or_insert(0.0);
                }
                AudioCommand::RemoveTrack { track_id } => {
                    self.track_volumes.remove(&track_id);
                    self.track_pans.remove(&track_id);
                }
                AudioCommand::StartPlayback => {
                    self.state.is_playing = true;
                    info!("Playback started");
                }
                AudioCommand::StopPlayback => {
                    self.state.is_playing = false;
                    info!("Playback stopped");
                }
                AudioCommand::SetTempo { bpm } => {
                    self.state.tempo_bpm = bpm;
                    info!("Tempo set to {} BPM", bpm);
                }
            }
        }
    }

    pub fn process_block(&mut self) -> Vec<f32> {
        self.process_commands();

        if self.state.is_playing {
            self.state.playhead_sample += self.config.buffer_size as u64;
        }

        let params = NodeParams {
            volume: self.state.master_volume,
            pan: 0.0,
            tempo: self.state.tempo_bpm,
        };

        let mut output = self.graph.process_graph(self.config.buffer_size, &params)
            .unwrap_or_else(|_| vec![0.0f32; self.config.buffer_size]);
        dsp::apply_gain_simd(&mut output, self.state.master_volume);

        let peak = dsp::compute_peak(&output);
        let _ = self.telemetry_producer.send(TelemetryEvent::PeakMeter { track_id: 0, peak });
        let _ = self.telemetry_producer.send(TelemetryEvent::PlayheadPosition { sample: self.state.playhead_sample });
        let _ = self.telemetry_producer.send(TelemetryEvent::CpuLoad { percent: 0.5 });

        output
    }
}

pub struct AudioEngineHandle {
    pub command_producer: CommandProducer,
    pub telemetry_consumer: TelemetryConsumer,
}

impl AudioEngineHandle {
    pub fn send_command(&mut self, cmd: AudioCommand) -> bool {
        self.command_producer.send(cmd).is_ok()
    }

    pub fn drain_telemetry(&mut self) -> Vec<TelemetryEvent> {
        let mut events = Vec::new();
        while let Some(e) = self.telemetry_consumer.try_recv() {
            events.push(e);
        }
        events
    }
}
