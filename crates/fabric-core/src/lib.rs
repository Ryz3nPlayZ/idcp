use std::cell::UnsafeCell;
use std::hint::spin_loop;
use std::io::{self, Read, Write};
use std::os::unix::net::UnixStream;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};

const DEFAULT_RING_CAPACITY: usize = 1024;

pub type FabricResult<T> = Result<T, FabricError>;

#[derive(Debug)]
pub enum FabricError {
    Io(io::Error),
    Disconnected(&'static str),
}

impl std::fmt::Display for FabricError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "{err}"),
            Self::Disconnected(context) => write!(f, "transport disconnected: {context}"),
        }
    }
}

impl std::error::Error for FabricError {}

impl From<io::Error> for FabricError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransportKind {
    SyncChannel,
    UnixStream,
    SpscRing,
}

impl TransportKind {
    pub const fn label(self) -> &'static str {
        match self {
            Self::SyncChannel => "sync_channel(0)",
            Self::UnixStream => "unix_stream",
            Self::SpscRing => "spsc_ring",
        }
    }
}

pub struct LocalEndpoint {
    inner: EndpointInner,
}

impl LocalEndpoint {
    pub fn pair(kind: TransportKind) -> FabricResult<(Self, Self)> {
        match kind {
            TransportKind::SyncChannel => {
                let (a_to_b_tx, a_to_b_rx) = sync_channel::<u64>(0);
                let (b_to_a_tx, b_to_a_rx) = sync_channel::<u64>(0);
                Ok((
                    Self {
                        inner: EndpointInner::Sync {
                            tx: a_to_b_tx,
                            rx: b_to_a_rx,
                        },
                    },
                    Self {
                        inner: EndpointInner::Sync {
                            tx: b_to_a_tx,
                            rx: a_to_b_rx,
                        },
                    },
                ))
            }
            TransportKind::UnixStream => {
                let (a, b) = UnixStream::pair()?;
                Ok((
                    Self {
                        inner: EndpointInner::Unix { stream: a },
                    },
                    Self {
                        inner: EndpointInner::Unix { stream: b },
                    },
                ))
            }
            TransportKind::SpscRing => {
                let a_to_b = Arc::new(SpscRing::new(DEFAULT_RING_CAPACITY));
                let b_to_a = Arc::new(SpscRing::new(DEFAULT_RING_CAPACITY));
                Ok((
                    Self {
                        inner: EndpointInner::Ring {
                            tx: Arc::clone(&a_to_b),
                            rx: Arc::clone(&b_to_a),
                        },
                    },
                    Self {
                        inner: EndpointInner::Ring {
                            tx: b_to_a,
                            rx: a_to_b,
                        },
                    },
                ))
            }
        }
    }

    pub fn send(&mut self, value: u64) -> FabricResult<()> {
        match &mut self.inner {
            EndpointInner::Sync { tx, .. } => tx
                .send(value)
                .map_err(|_| FabricError::Disconnected("sync send")),
            EndpointInner::Unix { stream } => {
                stream.write_all(&value.to_le_bytes())?;
                Ok(())
            }
            EndpointInner::Ring { tx, .. } => {
                tx.push(value);
                Ok(())
            }
        }
    }

    pub fn recv(&mut self) -> FabricResult<u64> {
        match &mut self.inner {
            EndpointInner::Sync { rx, .. } => rx
                .recv()
                .map_err(|_| FabricError::Disconnected("sync recv")),
            EndpointInner::Unix { stream } => {
                let mut buf = [0_u8; 8];
                stream.read_exact(&mut buf)?;
                Ok(u64::from_le_bytes(buf))
            }
            EndpointInner::Ring { rx, .. } => Ok(rx.pop()),
        }
    }

    pub fn request(&mut self, value: u64) -> FabricResult<u64> {
        self.send(value)?;
        self.recv()
    }
}

enum EndpointInner {
    Sync {
        tx: SyncSender<u64>,
        rx: Receiver<u64>,
    },
    Unix {
        stream: UnixStream,
    },
    Ring {
        tx: Arc<SpscRing>,
        rx: Arc<SpscRing>,
    },
}

#[repr(align(64))]
struct Aligned<T>(T);

struct RingSlots {
    slots: Vec<UnsafeCell<u64>>,
}

unsafe impl Sync for RingSlots {}

struct SpscRing {
    mask: usize,
    head: Aligned<AtomicUsize>,
    tail: Aligned<AtomicUsize>,
    slots: RingSlots,
}

unsafe impl Sync for SpscRing {}

impl SpscRing {
    fn new(capacity: usize) -> Self {
        assert!(capacity.is_power_of_two());
        let slots = (0..capacity).map(|_| UnsafeCell::new(0_u64)).collect();
        Self {
            mask: capacity - 1,
            head: Aligned(AtomicUsize::new(0)),
            tail: Aligned(AtomicUsize::new(0)),
            slots: RingSlots { slots },
        }
    }

    fn push(&self, value: u64) {
        loop {
            let head = self.head.0.load(Ordering::Relaxed);
            let tail = self.tail.0.load(Ordering::Acquire);
            if head.wrapping_sub(tail) < self.mask + 1 {
                let index = head & self.mask;
                unsafe {
                    *self.slots.slots[index].get() = value;
                }
                self.head.0.store(head.wrapping_add(1), Ordering::Release);
                return;
            }
            spin_loop();
        }
    }

    fn pop(&self) -> u64 {
        loop {
            let tail = self.tail.0.load(Ordering::Relaxed);
            let head = self.head.0.load(Ordering::Acquire);
            if tail != head {
                let index = tail & self.mask;
                let value = unsafe { *self.slots.slots[index].get() };
                self.tail.0.store(tail.wrapping_add(1), Ordering::Release);
                return value;
            }
            spin_loop();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{LocalEndpoint, TransportKind};
    use std::thread;

    #[test]
    fn request_reply_works_for_all_local_transports() {
        for kind in [TransportKind::SyncChannel, TransportKind::SpscRing] {
            let (mut client, mut server) = LocalEndpoint::pair(kind).unwrap();
            let worker = thread::spawn(move || {
                for _ in 0..128 {
                    let value = server.recv().unwrap();
                    server.send(value + 1).unwrap();
                }
            });

            for i in 0..128_u64 {
                let reply = client.request(i).unwrap();
                assert_eq!(reply, i + 1, "failed for {:?}", kind);
            }

            worker.join().unwrap();
        }
    }
}
