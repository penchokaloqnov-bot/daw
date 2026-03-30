use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrackConfig {
    pub id: usize,
    pub name: String,
    pub volume: f32,
    pub pan: f32,
    pub muted: bool,
    pub solo: bool,
    pub node_type: String,
    pub node_params: HashMap<String, f32>,
}

impl TrackConfig {
    pub fn new(id: usize, name: &str) -> Self {
        TrackConfig {
            id,
            name: name.to_string(),
            volume: 1.0,
            pan: 0.0,
            muted: false,
            solo: false,
            node_type: "source".to_string(),
            node_params: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AutomationData {
    pub parameter: String,
    pub points: Vec<(u64, f32)>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectFile {
    pub version: u32,
    pub name: String,
    pub sample_rate: u32,
    pub buffer_size: usize,
    pub tempo_bpm: f32,
    pub master_volume: f32,
    pub tracks: Vec<TrackConfig>,
    pub automation: Vec<AutomationData>,
}

impl ProjectFile {
    pub fn new(name: &str, sample_rate: u32, buffer_size: usize) -> Self {
        ProjectFile {
            version: 1,
            name: name.to_string(),
            sample_rate,
            buffer_size,
            tempo_bpm: 120.0,
            master_volume: 1.0,
            tracks: Vec::new(),
            automation: Vec::new(),
        }
    }

    pub fn add_track(&mut self, track: TrackConfig) {
        self.tracks.push(track);
    }

    pub fn add_automation(&mut self, data: AutomationData) {
        self.automation.push(data);
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        Ok(self.to_json()?.into_bytes())
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        let json = std::str::from_utf8(bytes)
            .map_err(|e| serde_json::Error::io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;
        Self::from_json(json)
    }

    pub fn validate(&self) -> Result<(), String> {
        if !(8000..=192000).contains(&self.sample_rate) {
            return Err(format!("sample_rate {} out of range [8000, 192000]", self.sample_rate));
        }
        if !self.buffer_size.is_power_of_two() || self.buffer_size < 64 || self.buffer_size > 16384 {
            return Err(format!("buffer_size {} must be a power of 2 in [64, 16384]", self.buffer_size));
        }
        if !(20.0..=400.0).contains(&self.tempo_bpm) {
            return Err(format!("tempo_bpm {} out of range [20.0, 400.0]", self.tempo_bpm));
        }
        if !(0.0..=2.0).contains(&self.master_volume) {
            return Err(format!("master_volume {} out of range [0.0, 2.0]", self.master_volume));
        }
        for track in &self.tracks {
            if !(0.0..=2.0).contains(&track.volume) {
                return Err(format!("track {} volume {} out of range [0.0, 2.0]", track.id, track.volume));
            }
            if !(-1.0..=1.0).contains(&track.pan) {
                return Err(format!("track {} pan {} out of range [-1.0, 1.0]", track.id, track.pan));
            }
        }
        Ok(())
    }
}
