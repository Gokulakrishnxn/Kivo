//! A single-producer, single-consumer ring buffer for microphone samples.
//!
//! The audio callback runs on a real-time thread owned by CoreAudio. It must
//! never allocate, never take a lock and never log — being slow there is not a
//! performance problem, it is a correctness bug that shows up as dropouts in
//! the recording. So the storage is allocated once up front and the two ends
//! coordinate through a pair of atomics.
//!
//! Overrun policy is deliberate: if the consumer falls behind, the producer
//! drops the newest samples rather than blocking or overwriting history.
//! Losing the tail of an utterance is recoverable; stalling CoreAudio is not.

use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

pub struct Ring {
    buf: UnsafeCell<Vec<f32>>,
    cap: usize,
    write: AtomicUsize,
    read: AtomicUsize,
    dropped: AtomicUsize,
}

// Safety: exactly one producer touches `write`, exactly one consumer touches
// `read`, and the data slots each side accesses are disjoint by construction.
unsafe impl Send for Ring {}
unsafe impl Sync for Ring {}

impl Ring {
    /// `seconds` of headroom at `sample_rate`. Anything longer than this is not
    /// dictation, it is recording, which is a non-goal.
    pub fn new(sample_rate: usize, seconds: usize) -> Arc<Self> {
        let cap = sample_rate * seconds + 1;
        Arc::new(Self {
            buf: UnsafeCell::new(vec![0.0; cap]),
            cap,
            write: AtomicUsize::new(0),
            read: AtomicUsize::new(0),
            dropped: AtomicUsize::new(0),
        })
    }

    /// Producer side. Called from the audio callback: no allocation, no locks.
    pub fn push(&self, samples: &[f32]) {
        let w = self.write.load(Ordering::Relaxed);
        let r = self.read.load(Ordering::Acquire);
        let used = w.wrapping_sub(r);
        let free = self.cap - 1 - used;

        let take = samples.len().min(free);
        if take < samples.len() {
            self.dropped
                .fetch_add(samples.len() - take, Ordering::Relaxed);
        }

        let buf = unsafe { &mut *self.buf.get() };
        for (n, s) in samples.iter().take(take).enumerate() {
            buf[(w + n) % self.cap] = *s;
        }
        self.write.store(w.wrapping_add(take), Ordering::Release);
    }

    /// Consumer side. Drains everything currently available.
    pub fn drain(&self) -> Vec<f32> {
        let w = self.write.load(Ordering::Acquire);
        let r = self.read.load(Ordering::Relaxed);
        let used = w.wrapping_sub(r);
        let buf = unsafe { &*self.buf.get() };

        let mut out = Vec::with_capacity(used);
        for n in 0..used {
            out.push(buf[(r + n) % self.cap]);
        }
        self.read.store(w, Ordering::Release);
        out
    }

    pub fn len(&self) -> usize {
        self.write
            .load(Ordering::Acquire)
            .wrapping_sub(self.read.load(Ordering::Acquire))
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Samples lost to overrun since the last reset. Surfaced in settings
    /// rather than hidden: a user who is dropping audio deserves to know.
    pub fn dropped(&self) -> usize {
        self.dropped.load(Ordering::Relaxed)
    }

    /// Zero the storage and rewind. Audio is never kept around after use.
    pub fn reset(&self) {
        let buf = unsafe { &mut *self.buf.get() };
        buf.iter_mut().for_each(|s| *s = 0.0);
        self.write.store(0, Ordering::Release);
        self.read.store(0, Ordering::Release);
        self.dropped.store(0, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_samples_in_order() {
        let r = Ring::new(10, 1);
        r.push(&[1.0, 2.0, 3.0]);
        assert_eq!(r.drain(), vec![1.0, 2.0, 3.0]);
        assert!(r.is_empty());
    }

    #[test]
    fn wraps_around_the_end() {
        let r = Ring::new(4, 1); // capacity 5
        r.push(&[1.0, 2.0, 3.0]);
        assert_eq!(r.drain(), vec![1.0, 2.0, 3.0]);
        r.push(&[4.0, 5.0, 6.0]);
        assert_eq!(r.drain(), vec![4.0, 5.0, 6.0]);
    }

    #[test]
    fn drops_the_newest_rather_than_blocking_on_overrun() {
        let r = Ring::new(4, 1); // holds 4
        r.push(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
        assert_eq!(r.drain(), vec![1.0, 2.0, 3.0, 4.0]);
        assert_eq!(r.dropped(), 2);
    }

    #[test]
    fn reset_zeroes_the_storage() {
        let r = Ring::new(10, 1);
        r.push(&[0.5; 5]);
        r.reset();
        assert!(r.is_empty());
        assert_eq!(r.dropped(), 0);
        assert!(r.drain().is_empty());
    }
}
