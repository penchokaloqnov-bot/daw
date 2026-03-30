# DAW — Rust Digital Audio Workstation

A production-quality DAW engine built entirely in Rust, structured as a Cargo workspace across five engineering phases: lock-free architecture, DSP, AI/ML, GPU rendering, and continuous testing.

---

## Architecture Overview

```
┌──────────────────────────────────────────────────────────────┐
│                        UI Thread                             │
│  (daw-tauri / React frontend via Tauri IPC)                  │
└────────────┬─────────────────────────────────┬───────────────┘
             │ Command Queue (rtrb SPSC)        │ Telemetry Queue
             ▼                                 ▲
┌──────────────────────────────────────────────────────────────┐
│                      Audio Thread                            │
│                                                              │
│  ┌────────────────┐   ┌──────────────────┐                   │
│  │  Memory Pool   │   │  Triple Buffer   │ ← state snapshot  │
│  │ (pooled bufs)  │   │  (lock-free)     │                   │
│  └────────┬───────┘   └──────────────────┘                   │
│           │                                                   │
│  ┌────────▼─────────────────────────────────────┐            │
│  │         Audio Graph (petgraph DAG)            │            │
│  │                                               │            │
│  │  SourceNode → GainNode → MixerNode → Master  │            │
│  │  AutomationEngine (per-parameter curves)      │            │
│  └───────────────────────────────────────────────┘            │
└──────────────────────────────────────────────────────────────┘
             │ Rendered pixels (daw-renderer)
             ▼
┌──────────────────────────────────────────────────────────────┐
│  GPU / CPU Renderer (daw-renderer)                           │
│  Waveform · Spectrum · Level Meter · wgpu compute shaders    │
└──────────────────────────────────────────────────────────────┘
```

---

## Crate Reference

| Crate | Purpose |
|-------|---------|
| `daw-engine` | Core audio engine: lock-free queues, triple buffer, memory pool, audio graph, DSP, automation |
| `daw-ai` | AI/ML: stem separator, smart parametric EQ analyser |
| `daw-collab` | Real-time collaboration: CRDT project state via automerge |
| `daw-renderer` | GPU/CPU rendering: waveform, spectrum analyser, level meters (wgpu optional) |
| `daw-tauri` | Tauri IPC bridge scaffold: command handlers, frontend integration |
| `daw-app` | Demo application wiring all crates together |

---

## Getting Started

```bash
# Build the entire workspace
cargo build --workspace

# Run all tests
cargo test --workspace

# Run the demo application
cargo run -p daw-app

# Enable GPU rendering features
cargo build -p daw-renderer --features gpu
```

**Requirements:** Rust stable ≥ 1.75 (no nightly needed for the workspace).

---

## Phase 1 — Memory & Lock-Free Architecture

*Crate:* `daw-engine`

- **Triple Buffer** (`triple_buffer.rs`) — wait-free state sharing between audio and UI threads; no mutex on the hot path.
- **Memory Pool** (`memory_pool.rs`) — pre-allocated `AudioBufferPool` with `PooledBuffer` RAII wrappers; eliminates per-block heap allocation.
- **Lock-Free Queues** (`commands.rs`) — `rtrb` SPSC ring buffers for `AudioCommand` (UI → engine) and `TelemetryEvent` (engine → UI).

```rust
let (mut writer, mut reader) = triple_buffer(EngineState::default());
let (cmd_prod, cmd_cons) = create_command_queue(256);
let (tel_prod, tel_cons) = create_telemetry_queue(1024);
```

---

## Phase 2 — DSP & Audio Graph

*Crate:* `daw-engine`

- **Audio Graph** (`audio_graph.rs`) — `petgraph` directed acyclic graph; cycle detection on `connect()`; topological-order processing.
- **SIMD DSP** (`dsp.rs`) — `wide` crate for auto-vectorised gain, mixing, panning, peak/RMS.
- **Automation** (`automation.rs`) — per-parameter `AutomationCurve` with sample-accurate interpolation; `fill_buffer` fills an entire block in one call.

```rust
let source = graph.add_node(Box::new(SourceNode::new(440.0, 44100.0)));
let gain   = graph.add_node(Box::new(GainNode::new(0.8)));
let master = graph.add_node(Box::new(MasterNode));
graph.connect(source, gain)?;
graph.connect(gain, master)?;
let output = graph.process_graph(buffer_size, &params)?;
```

---

## Phase 3 — AI/ML & Collaboration

*Crates:* `daw-ai`, `daw-collab`

- **Stem Separator** (`daw-ai/src/stem_separator.rs`) — frequency-domain stem separation (bass, drums, vocals, other) without external ML runtime.
- **Smart EQ** (`daw-ai/src/smart_eq.rs`) — analyses a buffer and returns parametric EQ band suggestions.
- **CRDT Project State** (`daw-collab/src/project.rs`) — `CollaborativeProject` backed by `automerge`; `save()` / `load()` for merge-friendly serialisation; concurrent track edits resolve automatically.

```rust
let mut project = CollaborativeProject::new();
project.add_track(1, "Drums")?;
project.set_track_volume(1, 0.8)?;
let bytes = project.save();
let loaded = CollaborativeProject::load(&bytes)?;
```

---

## Phase 4 — GPU-Accelerated Rendering

*Crate:* `daw-renderer`

- **WaveformRenderer** — CPU waveform peak rendering; stereo split layout.
- **SpectrumAnalyzer** — octave-band DFT energy spectrum with temporal smoothing.
- **LevelMeter** — per-channel RMS + peak + peak-hold with ARGB pixel output.
- **GPU backend** (feature `gpu`) — `wgpu` compute shader pipeline for waveform rendering; `GpuContext` wraps device/queue/surface as `Option` for headless CI.
- **Tauri bridge** (`daw-tauri`) — async command handlers (`cmd_start_playback`, `cmd_stop_playback`, `cmd_set_volume`, `cmd_get_telemetry`) ready for `#[tauri::command]` decoration.

```bash
cargo build -p daw-renderer --features gpu
```

---

## Phase 5 — Testing & Profiling

### Integration Tests

`daw-engine` ships with six integration tests covering the full pipeline:

```bash
cargo test -p daw-engine --verbose
```

### Fuzz Testing

Located in `fuzz/` (managed by `cargo-fuzz`; requires nightly to *run*):

```bash
# Install cargo-fuzz
cargo install cargo-fuzz

# Run fuzz targets (nightly required at runtime)
cargo +nightly fuzz run fuzz_audio_graph
cargo +nightly fuzz run fuzz_automation
```

Targets:
- `fuzz_audio_graph` — builds random DAGs and processes them; ensures no panics on adversarial graph topologies.
- `fuzz_automation` — feeds random automation point sequences; verifies `get_value_at` and `fill_buffer` never panic.

### CI/CD

GitHub Actions workflow (`.github/workflows/ci.yml`):

| Job | What it does |
|-----|-------------|
| `build-and-test` | `cargo build`, `cargo test`, `cargo clippy -D warnings` on ubuntu-latest |
| `headless-audio-test` | Runs `daw-engine`, `daw-ai`, `daw-collab` tests in isolation |

Triggers on pushes to `main` / `copilot/*` branches and all pull requests.
