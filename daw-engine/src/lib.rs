pub mod triple_buffer;
pub mod memory_pool;
pub mod commands;
pub mod audio_graph;
pub mod dsp;
pub mod automation;
pub mod engine;

pub use triple_buffer::{TripleBufferWriter, TripleBufferReader, triple_buffer};
pub use memory_pool::{AudioBufferPool, PooledBuffer};
pub use commands::{
    AudioCommand, TelemetryEvent,
    CommandProducer, CommandConsumer, TelemetryProducer, TelemetryConsumer,
    create_command_queue, create_telemetry_queue,
};
pub use audio_graph::{AudioGraph, AudioNode, NodeId, NodeParams, GraphError, SourceNode, MixerNode, GainNode, MasterNode};
pub use dsp::{apply_gain_simd, mix_buffers_simd, apply_pan_simd, generate_sine_wave, db_to_linear, linear_to_db, compute_peak, compute_rms};
pub use automation::{AutomationPoint, AutomationCurve, AutomationEngine};
pub use engine::{AudioEngine, AudioEngineConfig, AudioEngineHandle, EngineState};
