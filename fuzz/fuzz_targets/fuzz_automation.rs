#![no_main]
use libfuzzer_sys::fuzz_target;
use daw_engine::{AutomationCurve, AutomationPoint};

fuzz_target!(|data: &[u8]| {
    if data.len() < 2 { return; }
    let mut curve = AutomationCurve::new(0.5);
    let chunks = data.chunks(8);
    let mut sample: u64 = 0;
    for chunk in chunks {
        if chunk.len() >= 2 {
            let value = (chunk[0] as f32) / 255.0;
            sample += (chunk[1] as u64) * 100 + 1;
            curve.add_point(AutomationPoint { sample, value });
        }
    }
    let _ = curve.get_value_at(0);
    let _ = curve.get_value_at(sample / 2);
    let _ = curve.get_value_at(sample + 1000);
    let mut buf = vec![0.0f32; 512];
    curve.fill_buffer(0, &mut buf);
});
