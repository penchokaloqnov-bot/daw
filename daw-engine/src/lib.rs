pub mod triple_buffer;
pub mod memory_pool;
pub mod commands;
pub mod audio_graph;
pub mod dsp;
pub mod automation;
pub mod engine;
pub mod plugins;
pub mod project_file;
pub mod profiling;

pub use triple_buffer::{TripleBufferWriter, TripleBufferReader, triple_buffer};
pub use memory_pool::{AudioBufferPool, PooledBuffer};
pub use commands::{
    AudioCommand, TelemetryEvent,
    CommandProducer, CommandConsumer, TelemetryProducer, TelemetryConsumer,
    create_command_queue, create_telemetry_queue,
};
pub use audio_graph::{AudioGraph, AudioNode, NodeId, NodeParams, GraphError, SourceNode, MixerNode, GainNode, MasterNode, StereoGainNode};
pub use dsp::{apply_gain_simd, mix_buffers_simd, apply_pan_simd, generate_sine_wave, db_to_linear, linear_to_db, compute_peak, compute_rms, apply_gain_stereo, apply_pan_stereo, mono_to_stereo, stereo_to_mono, compute_stereo_peak, compute_stereo_rms};
pub use automation::{AutomationPoint, AutomationCurve, AutomationEngine};
pub use engine::{AudioEngine, AudioEngineConfig, AudioEngineHandle, EngineState};
pub use plugins::{BiquadFilter, BiquadType, Compressor, SimpleReverb, DelayLine};
pub use project_file::{ProjectFile, TrackConfig, AutomationData};
pub use profiling::{init_profiling, profile_span};
