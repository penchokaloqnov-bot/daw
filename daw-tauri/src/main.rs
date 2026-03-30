//! Tauri IPC bridge scaffold.
//!
//! Architecture:
//!   React/Vue Frontend ↔ Tauri IPC ↔ Rust Backend (daw-engine)
//!
//! In a full Tauri integration each `cmd_*` function below would be annotated
//! with `#[tauri::command]` and registered via `tauri::Builder::invoke_handler`.
//! For now they are plain async functions demonstrating the intended interface.

use daw_engine::{AudioEngineConfig, AudioEngineHandle, AudioEngine,
    create_command_queue, create_telemetry_queue};
use tracing::{info, warn};

// ---------------------------------------------------------------------------
// Tauri command handlers (would be #[tauri::command] in a real integration)
// ---------------------------------------------------------------------------

/// Starts audio engine playback.
pub async fn cmd_start_playback() {
    info!("cmd_start_playback: starting engine");
}

/// Stops audio engine playback.
pub async fn cmd_stop_playback() {
    info!("cmd_stop_playback: stopping engine");
}

/// Sets the volume for the given track.
pub async fn cmd_set_volume(track_id: usize, volume: f32) {
    if !(0.0..=2.0).contains(&volume) {
        warn!("cmd_set_volume: volume {volume} out of range for track {track_id}");
        return;
    }
    info!("cmd_set_volume: track={track_id} volume={volume:.3}");
}

/// Returns a JSON telemetry snapshot (CPU load, buffer underruns, etc.).
pub async fn cmd_get_telemetry() -> serde_json::Value {
    serde_json::json!({
        "cpu_load": 0.0,
        "buffer_underruns": 0,
        "sample_rate": 44100,
        "buffer_size": 512,
    })
}

// ---------------------------------------------------------------------------
// Application entry point
// ---------------------------------------------------------------------------

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    info!("daw-tauri scaffold starting");

    // Demonstrate the engine running headlessly (no audio device required in CI).
    let config = AudioEngineConfig {
        sample_rate: 44100,
        buffer_size: 512,
        num_tracks: 2,
    };

    let (cmd_prod, cmd_cons) = create_command_queue(256);
    let (tel_prod, tel_cons) = create_telemetry_queue(1024);
    let _engine = AudioEngine::new(config, cmd_cons, tel_prod);
    let _handle = AudioEngineHandle {
        command_producer: cmd_prod,
        telemetry_consumer: tel_cons,
    };

    info!("AudioEngine handle created — Tauri IPC bridge ready");
    info!("Available commands: start_playback, stop_playback, set_volume, get_telemetry");
}
