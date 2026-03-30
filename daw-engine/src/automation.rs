use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct AutomationPoint {
    pub sample: u64,
    pub value: f32,
}

#[derive(Debug, Clone)]
pub struct AutomationCurve {
    pub points: Vec<AutomationPoint>,
    pub default_value: f32,
}

impl AutomationCurve {
    pub fn new(default_value: f32) -> Self {
        AutomationCurve { points: Vec::new(), default_value }
    }

    pub fn add_point(&mut self, point: AutomationPoint) {
        let pos = self.points.partition_point(|p| p.sample < point.sample);
        self.points.insert(pos, point);
    }

    pub fn get_value_at(&self, sample: u64) -> f32 {
        if self.points.is_empty() {
            return self.default_value;
        }
        if sample <= self.points[0].sample {
            return self.points[0].value;
        }
        if sample >= self.points[self.points.len() - 1].sample {
            return self.points[self.points.len() - 1].value;
        }
        let idx = self.points.partition_point(|p| p.sample <= sample);
        let before = &self.points[idx - 1];
        let after = &self.points[idx];
        let t = (sample - before.sample) as f32 / (after.sample - before.sample) as f32;
        before.value + t * (after.value - before.value)
    }

    pub fn fill_buffer(&self, start_sample: u64, buffer: &mut [f32]) {
        for (i, slot) in buffer.iter_mut().enumerate() {
            *slot = self.get_value_at(start_sample + i as u64);
        }
    }
}

pub struct AutomationEngine {
    pub curves: HashMap<String, AutomationCurve>,
}

impl AutomationEngine {
    pub fn new() -> Self {
        AutomationEngine { curves: HashMap::new() }
    }

    pub fn register_curve(&mut self, name: &str, default: f32) {
        self.curves.entry(name.to_string()).or_insert_with(|| AutomationCurve::new(default));
    }

    pub fn add_automation_point(&mut self, name: &str, sample: u64, value: f32) {
        if let Some(curve) = self.curves.get_mut(name) {
            curve.add_point(AutomationPoint { sample, value });
        }
    }

    pub fn get_value(&self, name: &str, sample: u64) -> f32 {
        self.curves.get(name).map(|c| c.get_value_at(sample)).unwrap_or(0.0)
    }

    pub fn fill_parameter_buffer(&self, name: &str, start_sample: u64, buffer: &mut [f32]) {
        if let Some(curve) = self.curves.get(name) {
            curve.fill_buffer(start_sample, buffer);
        } else {
            for slot in buffer.iter_mut() { *slot = 0.0; }
        }
    }
}

impl Default for AutomationEngine {
    fn default() -> Self { Self::new() }
}
