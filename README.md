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

---

## Phase 6 — MIDI Engine & Polyphonic Synthesis

*Crate:* `daw-midi`

- **MIDI Types** (`types.rs`) — `MidiMessage`, `MidiEvent`, `MidiTrack` with sort and duration helpers; `midi_pitch_to_hz` and `ticks_to_samples` utilities.
- **MidiSequencer** (`sequencer.rs`) — tick-based playback with sample-accurate scheduling; converts MIDI events to `AudioCommand::NoteOn/NoteOff`; supports `reset()`, `set_bpm()`, and `advance(num_samples)`.
- **PolySynth** (`synth.rs`) — polyphonic synthesiser with configurable voice count; ADSR envelope per voice; four waveforms: `Sine`, `Sawtooth`, `Square`, `Triangle`; voice stealing when at capacity; implements `AudioNode` for graph integration.

```rust
use daw_midi::{MidiTrack, MidiMessage, MidiSequencer, PolySynth, WaveShape};

let mut track = MidiTrack::new("Arpeggio");
track.add_event(0,   MidiMessage::NoteOn  { channel: 0, pitch: 60, velocity: 100 });
track.add_event(480, MidiMessage::NoteOff { channel: 0, pitch: 60 });
track.sort();

let mut seq   = MidiSequencer::new(track, 480, 44100, 120.0);
let mut synth = PolySynth::new(8, 44100, WaveShape::Sine);

let cmds = seq.advance(512); // returns AudioCommands for this block
for cmd in cmds { /* route to synth.note_on / note_off */ }

let mut block = vec![0.0f32; 512];
synth.generate_block(&mut block);
```

---

## Phase 7 — Built-in DSP Plugins

*Crate:* `daw-engine` (`plugins.rs`)

All plugins implement `AudioNode` and integrate directly into the audio graph.

| Plugin | Description |
|--------|-------------|
| `BiquadFilter` | 7-type biquad (LP/HP/BP/Notch/PeakEQ/LowShelf/HighShelf) — Audio EQ Cookbook coefficients, Direct Form II Transposed |
| `Compressor` | Feed-forward RMS compressor with configurable threshold, ratio, attack/release, makeup gain |
| `SimpleReverb` | Schroeder reverb: 4 parallel comb filters + 2 series allpass filters; `room_size` and `wet` controls |
| `DelayLine` | Feedback delay with configurable delay time (ms), feedback (capped at 0.95), and wet/dry mix |

```rust
use daw_engine::{BiquadFilter, BiquadType, Compressor, SimpleReverb, DelayLine};

let lp  = BiquadFilter::new(BiquadType::LowPass, 4000.0, 0.707, 0.0, 44100.0);
let cmp = Compressor::new(-12.0, 4.0, 5.0, 100.0, 2.0, 44100.0);
let rev = SimpleReverb::new(0.8, 0.3, 44100);
let dly = DelayLine::new(250.0, 0.4, 0.5, 44100);
```

---

## Phase 8 — Stereo Processing

*Crate:* `daw-engine` (`dsp.rs`, `audio_graph.rs`)

Stereo buffers are interleaved: `[L0, R0, L1, R1, ...]`.

| Function / Node | Description |
|-----------------|-------------|
| `apply_gain_stereo(buf, gain_l, gain_r)` | Independent L/R gain on interleaved buffer |
| `apply_pan_stereo(buf, pan)` | Constant-power panning (−1.0 hard-left … +1.0 hard-right) |
| `mono_to_stereo(mono)` | Duplicate each sample → interleaved stereo |
| `stereo_to_mono(stereo)` | Average L+R pairs → mono |
| `compute_stereo_peak(buf)` | Returns `(left_peak, right_peak)` |
| `compute_stereo_rms(buf)` | Returns `(left_rms, right_rms)` |
| `StereoGainNode { gain_l, gain_r }` | `AudioNode` applying per-channel gain in the graph |

---

## Phase 9 — Project File Format

*Crate:* `daw-engine` (`project_file.rs`)

JSON-serialisable project format with validation.

```rust
use daw_engine::{ProjectFile, TrackConfig, AutomationData};

let mut pf = ProjectFile::new("My Project", 44100, 512);
pf.tempo_bpm = 128.0;

let mut t = TrackConfig::new(0, "Lead Synth");
t.node_type = "polysynth".to_string();
t.node_params.insert("max_voices".to_string(), 8.0);
pf.add_track(t);

pf.add_automation(AutomationData {
    parameter: "master_volume".to_string(),
    points: vec![(0, 0.0), (44100, 1.0)],
});

pf.validate()?;
let json  = pf.to_json()?;
let bytes = pf.to_bytes()?;
let back  = ProjectFile::from_json(&json)?;
```

Validation checks: `sample_rate` ∈ [8000, 192000], `buffer_size` power-of-two ∈ [64, 16384], `tempo_bpm` ∈ [20, 400], volumes ∈ [0, 2], pans ∈ [−1, 1].

---

## Phase 10 — Profiling

*Crate:* `daw-engine` (`profiling.rs`)

Optional [Tracy](https://github.com/wolfpld/tracy) integration via the `tracy` feature flag.

```bash
# Build with Tracy profiling enabled
cargo build --workspace --features daw-engine/tracy
```

- `init_profiling()` — sets up the `tracing-tracy` subscriber when the `tracy` feature is active; no-op otherwise.
- `profile_span(name)` — creates a `tracing::Span` at `TRACE` level for custom instrumentation.
- `process_block()` in `AudioEngine` is automatically wrapped in a `"process_block"` span.

When built without `--features tracy` (the default), the entire profiling stack compiles away to zero overhead.
