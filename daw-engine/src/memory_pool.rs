use crossbeam_channel::{bounded, Receiver, Sender};

pub struct AudioBufferPool {
    sender: Sender<Vec<f32>>,
    receiver: Receiver<Vec<f32>>,
    buffer_size: usize,
}

pub struct PooledBuffer {
    data: Option<Vec<f32>>,
    sender: Sender<Vec<f32>>,
}

impl AudioBufferPool {
    pub fn new(num_buffers: usize, buffer_size: usize) -> Self {
        let (sender, receiver) = bounded(num_buffers);
        for _ in 0..num_buffers {
            let buf = vec![0.0f32; buffer_size];
            sender.send(buf).unwrap();
        }
        AudioBufferPool { sender, receiver, buffer_size }
    }

    pub fn checkout(&self) -> Option<PooledBuffer> {
        self.receiver.try_recv().ok().map(|data| PooledBuffer {
            data: Some(data),
            sender: self.sender.clone(),
        })
    }

    pub fn buffer_size(&self) -> usize {
        self.buffer_size
    }
}

impl PooledBuffer {
    pub fn as_slice(&self) -> &[f32] {
        self.data.as_ref().unwrap().as_slice()
    }

    pub fn as_mut_slice(&mut self) -> &mut [f32] {
        self.data.as_mut().unwrap().as_mut_slice()
    }
}

impl Drop for PooledBuffer {
    fn drop(&mut self) {
        if let Some(mut data) = self.data.take() {
            for x in data.iter_mut() { *x = 0.0; }
            let _ = self.sender.send(data);
        }
    }
}

impl std::ops::Deref for PooledBuffer {
    type Target = [f32];
    fn deref(&self) -> &[f32] { self.as_slice() }
}

impl std::ops::DerefMut for PooledBuffer {
    fn deref_mut(&mut self) -> &mut [f32] { self.as_mut_slice() }
}
