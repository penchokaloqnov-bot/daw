use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::cell::UnsafeCell;

struct Inner<T> {
    slots: [UnsafeCell<T>; 3],
    // Packed: bits 0-1 = ready slot index, bit 2 = new_data flag
    ready: AtomicUsize,
}

unsafe impl<T: Send> Send for Inner<T> {}
unsafe impl<T: Send> Sync for Inner<T> {}

pub struct TripleBufferWriter<T> {
    inner: Arc<Inner<T>>,
    write_slot: usize,
}

pub struct TripleBufferReader<T> {
    inner: Arc<Inner<T>>,
    read_slot: usize,
}

pub fn triple_buffer<T: Clone>(initial: T) -> (TripleBufferWriter<T>, TripleBufferReader<T>) {
    let inner = Arc::new(Inner {
        slots: [
            UnsafeCell::new(initial.clone()),
            UnsafeCell::new(initial.clone()),
            UnsafeCell::new(initial),
        ],
        ready: AtomicUsize::new(1), // slot 1 is ready initially, bit 2 = 0 (no new data)
    });
    let writer = TripleBufferWriter { inner: Arc::clone(&inner), write_slot: 0 };
    let reader = TripleBufferReader { inner, read_slot: 2 };
    (writer, reader)
}

impl<T> TripleBufferWriter<T> {
    pub fn write_slot(&mut self) -> &mut T {
        unsafe { &mut *self.inner.slots[self.write_slot].get() }
    }

    pub fn publish(&mut self) {
        let old = self.inner.ready.swap(self.write_slot | 4, Ordering::AcqRel);
        self.write_slot = old & 3;
    }
}

impl<T> TripleBufferReader<T> {
    pub fn has_new_data(&self) -> bool {
        self.inner.ready.load(Ordering::Acquire) & 4 != 0
    }

    pub fn update(&mut self) -> bool {
        let ready = self.inner.ready.load(Ordering::Acquire);
        if ready & 4 == 0 {
            return false;
        }
        let old = self.inner.ready.swap(self.read_slot, Ordering::AcqRel);
        self.read_slot = old & 3;
        true
    }

    pub fn read(&self) -> &T {
        unsafe { &*self.inner.slots[self.read_slot].get() }
    }
}
