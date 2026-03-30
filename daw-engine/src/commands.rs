use rtrb::{RingBuffer, Producer, Consumer, PushError};

#[derive(Debug, Clone)]
pub enum AudioCommand {
    NoteOn { pitch: u8, velocity: u8 },
    NoteOff { pitch: u8 },
    SetVolume { track_id: usize, value: f32 },
    SetPan { track_id: usize, value: f32 },
    AddTrack { track_id: usize },
    RemoveTrack { track_id: usize },
    StartPlayback,
    StopPlayback,
    SetTempo { bpm: f32 },
}

#[derive(Debug, Clone)]
pub enum TelemetryEvent {
    PeakMeter { track_id: usize, peak: f32 },
    PlayheadPosition { sample: u64 },
    CpuLoad { percent: f32 },
}

pub struct CommandProducer(pub(crate) Producer<AudioCommand>);
pub struct CommandConsumer(pub(crate) Consumer<AudioCommand>);
pub struct TelemetryProducer(pub(crate) Producer<TelemetryEvent>);
pub struct TelemetryConsumer(pub(crate) Consumer<TelemetryEvent>);

impl CommandProducer {
    pub fn send(&mut self, cmd: AudioCommand) -> Result<(), AudioCommand> {
        self.0.push(cmd).map_err(|e| match e { PushError::Full(v) => v })
    }
}

impl CommandConsumer {
    pub fn try_recv(&mut self) -> Option<AudioCommand> {
        self.0.pop().ok()
    }
}

impl TelemetryProducer {
    pub fn send(&mut self, event: TelemetryEvent) -> Result<(), TelemetryEvent> {
        self.0.push(event).map_err(|e| match e { PushError::Full(v) => v })
    }
}

impl TelemetryConsumer {
    pub fn try_recv(&mut self) -> Option<TelemetryEvent> {
        self.0.pop().ok()
    }
}

pub fn create_command_queue(capacity: usize) -> (CommandProducer, CommandConsumer) {
    let (prod, cons) = RingBuffer::new(capacity);
    (CommandProducer(prod), CommandConsumer(cons))
}

pub fn create_telemetry_queue(capacity: usize) -> (TelemetryProducer, TelemetryConsumer) {
    let (prod, cons) = RingBuffer::new(capacity);
    (TelemetryProducer(prod), TelemetryConsumer(cons))
}
