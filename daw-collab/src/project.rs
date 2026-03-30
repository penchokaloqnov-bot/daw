use automerge::{AutoCommit, ObjType, ScalarValue, ReadDoc};
use automerge::transaction::Transactable;

#[derive(Debug, Clone)]
pub struct TrackState {
    pub id: u64,
    pub name: String,
    pub volume: f32,
    pub pan: f32,
    pub muted: bool,
    pub solo: bool,
}

pub struct CollaborativeProject {
    doc: AutoCommit,
}

impl CollaborativeProject {
    pub fn new() -> Self {
        CollaborativeProject { doc: AutoCommit::new() }
    }

    fn tracks_key(id: u64) -> String { format!("track_{}", id) }

    pub fn add_track(&mut self, id: u64, name: &str) -> Result<(), automerge::AutomergeError> {
        let key = Self::tracks_key(id);
        let track_obj = self.doc.put_object(automerge::ROOT, &key, ObjType::Map)?;
        self.doc.put(&track_obj, "id", ScalarValue::Uint(id))?;
        self.doc.put(&track_obj, "name", ScalarValue::Str(name.into()))?;
        self.doc.put(&track_obj, "volume", ScalarValue::F64(1.0))?;
        self.doc.put(&track_obj, "pan", ScalarValue::F64(0.0))?;
        self.doc.put(&track_obj, "muted", ScalarValue::Boolean(false))?;
        self.doc.put(&track_obj, "solo", ScalarValue::Boolean(false))?;
        Ok(())
    }

    pub fn set_track_volume(&mut self, id: u64, volume: f32) -> Result<(), automerge::AutomergeError> {
        let key = Self::tracks_key(id);
        if let Some((automerge::Value::Object(_), track_oid)) = self.doc.get(automerge::ROOT, &key)? {
            self.doc.put(&track_oid, "volume", ScalarValue::F64(volume as f64))?;
        }
        Ok(())
    }

    pub fn set_track_pan(&mut self, id: u64, pan: f32) -> Result<(), automerge::AutomergeError> {
        let key = Self::tracks_key(id);
        if let Some((automerge::Value::Object(_), track_oid)) = self.doc.get(automerge::ROOT, &key)? {
            self.doc.put(&track_oid, "pan", ScalarValue::F64(pan as f64))?;
        }
        Ok(())
    }

    pub fn mute_track(&mut self, id: u64, muted: bool) -> Result<(), automerge::AutomergeError> {
        let key = Self::tracks_key(id);
        if let Some((automerge::Value::Object(_), track_oid)) = self.doc.get(automerge::ROOT, &key)? {
            self.doc.put(&track_oid, "muted", ScalarValue::Boolean(muted))?;
        }
        Ok(())
    }

    pub fn save(&mut self) -> Vec<u8> {
        self.doc.save()
    }

    pub fn load(bytes: &[u8]) -> Result<Self, automerge::AutomergeError> {
        let doc = AutoCommit::load(bytes)?;
        Ok(CollaborativeProject { doc })
    }

    pub fn merge(&mut self, other_bytes: &[u8]) -> Result<(), automerge::AutomergeError> {
        let mut other = AutoCommit::load(other_bytes)?;
        self.doc.merge(&mut other)?;
        Ok(())
    }

    pub fn get_tracks(&self) -> Vec<TrackState> {
        let mut tracks = Vec::new();
        for key in self.doc.keys(automerge::ROOT) {
            if !key.starts_with("track_") { continue; }
            if let Ok(Some((automerge::Value::Object(_), track_oid))) = self.doc.get(automerge::ROOT, &key) {
                let id = match self.doc.get(&track_oid, "id") {
                    Ok(Some((automerge::Value::Scalar(s), _))) => match s.as_ref() {
                        ScalarValue::Uint(v) => *v,
                        ScalarValue::Int(v) => *v as u64,
                        _ => 0,
                    },
                    _ => 0,
                };
                let name = match self.doc.get(&track_oid, "name") {
                    Ok(Some((automerge::Value::Scalar(s), _))) => match s.as_ref() {
                        ScalarValue::Str(v) => v.to_string(),
                        _ => String::new(),
                    },
                    _ => String::new(),
                };
                let volume = match self.doc.get(&track_oid, "volume") {
                    Ok(Some((automerge::Value::Scalar(s), _))) => match s.as_ref() {
                        ScalarValue::F64(v) => *v as f32,
                        _ => 1.0,
                    },
                    _ => 1.0,
                };
                let pan = match self.doc.get(&track_oid, "pan") {
                    Ok(Some((automerge::Value::Scalar(s), _))) => match s.as_ref() {
                        ScalarValue::F64(v) => *v as f32,
                        _ => 0.0,
                    },
                    _ => 0.0,
                };
                let muted = match self.doc.get(&track_oid, "muted") {
                    Ok(Some((automerge::Value::Scalar(s), _))) => match s.as_ref() {
                        ScalarValue::Boolean(v) => *v,
                        _ => false,
                    },
                    _ => false,
                };
                let solo = match self.doc.get(&track_oid, "solo") {
                    Ok(Some((automerge::Value::Scalar(s), _))) => match s.as_ref() {
                        ScalarValue::Boolean(v) => *v,
                        _ => false,
                    },
                    _ => false,
                };
                tracks.push(TrackState { id, name, volume, pan, muted, solo });
            }
        }
        tracks
    }
}

impl Default for CollaborativeProject {
    fn default() -> Self { Self::new() }
}
