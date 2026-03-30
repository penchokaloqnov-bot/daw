//! GPU rendering infrastructure using wgpu.
//!
//! `GpuContext::new()` requires a window handle and will return an error if no
//! compatible GPU adapter is available. At test time this module compiles
//! without needing real GPU hardware — all GPU types are `Option`-wrapped.

use tracing::info;

/// WGSL compute shader for waveform rendering.
pub const WAVEFORM_SHADER: &str = r#"
@group(0) @binding(0) var<storage, read> audio_samples: array<f32>;
@group(0) @binding(1) var output_texture: texture_storage_2d<rgba8unorm, write>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let x = id.x;
    let tex_dim = textureDimensions(output_texture);
    if x >= tex_dim.x { return; }

    let num_samples = arrayLength(&audio_samples);
    let samples_per_pixel = f32(num_samples) / f32(tex_dim.x);
    let start = u32(f32(x) * samples_per_pixel);
    let end = u32(f32(x + 1u) * samples_per_pixel);

    var peak_pos: f32 = 0.0;
    var peak_neg: f32 = 0.0;
    for (var i = start; i < end && i < num_samples; i++) {
        let s = audio_samples[i];
        if s > peak_pos { peak_pos = s; }
        if s < peak_neg { peak_neg = s; }
    }

    let half_h = f32(tex_dim.y) / 2.0;
    let y_top = u32(half_h - peak_pos * half_h);
    let y_bot = u32(half_h - peak_neg * half_h);

    for (var y = y_top; y <= y_bot; y++) {
        textureStore(output_texture, vec2<u32>(x, y), vec4<f32>(0.27, 0.53, 1.0, 1.0));
    }
}
"#;

/// GPU context. Fields are `Option`-wrapped so the struct can be constructed
/// without an actual GPU adapter (useful in CI / headless environments).
pub struct GpuContext {
    pub device: Option<wgpu::Device>,
    pub queue: Option<wgpu::Queue>,
    /// Surface is optional — compute-only workloads do not need one.
    pub surface: Option<wgpu::Surface<'static>>,
}

impl GpuContext {
    /// Creates a headless (no surface) GPU context.
    ///
    /// Returns `Err` if no suitable adapter can be found.
    pub async fn new_headless() -> Result<Self, String> {
        let instance = wgpu::Instance::default();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| "No wgpu adapter found".to_string())?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .map_err(|e| e.to_string())?;

        info!("GPU context created: {:?}", adapter.get_info().name);

        Ok(Self {
            device: Some(device),
            queue: Some(queue),
            surface: None,
        })
    }

    /// Creates an empty stub context (no GPU device).
    pub fn new_stub() -> Self {
        Self {
            device: None,
            queue: None,
            surface: None,
        }
    }
}

/// Renders waveforms using a wgpu compute pipeline.
pub struct WgpuWaveformRenderer {
    pub context: GpuContext,
}

impl WgpuWaveformRenderer {
    pub fn new(context: GpuContext) -> Self {
        Self { context }
    }

    /// Creates the waveform compute pipeline on `device`.
    pub fn create_waveform_pipeline(device: &wgpu::Device) -> wgpu::ComputePipeline {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("waveform_shader"),
            source: wgpu::ShaderSource::Wgsl(WAVEFORM_SHADER.into()),
        });

        device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("waveform_pipeline"),
            layout: None,
            module: &shader,
            entry_point: Some("main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        })
    }
}
