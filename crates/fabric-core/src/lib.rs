use core::ffi::c_void;
use std::cell::UnsafeCell;
use std::hint::spin_loop;
use std::io::{self, Read, Write};
use std::mem::size_of;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::os::unix::net::UnixStream;
use std::ptr::{self, NonNull};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};

const DEFAULT_RING_CAPACITY: usize = 1024;
const PROT_READ: i32 = 0x1;
const PROT_WRITE: i32 = 0x2;
const MAP_SHARED: i32 = 0x01;
const MAP_ANONYMOUS: i32 = 0x20;
const EFD_CLOEXEC: i32 = 0x80000;

unsafe extern "C" {
    fn mmap(
        addr: *mut c_void,
        len: usize,
        prot: i32,
        flags: i32,
        fd: i32,
        offset: isize,
    ) -> *mut c_void;
    fn munmap(addr: *mut c_void, len: usize) -> i32;
    fn eventfd(initval: u32, flags: i32) -> i32;
    fn dup(fd: i32) -> i32;
    fn read(fd: i32, buf: *mut c_void, count: usize) -> isize;
    fn write(fd: i32, buf: *const c_void, count: usize) -> isize;
}

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
    SharedMemoryEvent,
}

impl TransportKind {
    pub const fn label(self) -> &'static str {
        match self {
            Self::SyncChannel => "sync_channel(0)",
            Self::UnixStream => "unix_stream",
            Self::SpscRing => "spsc_ring",
            Self::SharedMemoryEvent => "shm_eventfd",
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
            TransportKind::SharedMemoryEvent => {
                let a_to_b = Arc::new(SharedRing::new()?);
                let b_to_a = Arc::new(SharedRing::new()?);
                let a_to_b_signal = EventFd::new()?;
                let b_to_a_signal = EventFd::new()?;
                Ok((
                    Self {
                        inner: EndpointInner::SharedMemoryEvent {
                            tx_ring: Arc::clone(&a_to_b),
                            rx_ring: Arc::clone(&b_to_a),
                            tx_signal: a_to_b_signal.try_clone()?,
                            rx_signal: b_to_a_signal.try_clone()?,
                        },
                    },
                    Self {
                        inner: EndpointInner::SharedMemoryEvent {
                            tx_ring: b_to_a,
                            rx_ring: a_to_b,
                            tx_signal: b_to_a_signal,
                            rx_signal: a_to_b_signal,
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
            EndpointInner::SharedMemoryEvent {
                tx_ring, tx_signal, ..
            } => {
                tx_ring.push(value);
                tx_signal.notify()?;
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
            EndpointInner::SharedMemoryEvent {
                rx_ring, rx_signal, ..
            } => loop {
                if let Some(value) = rx_ring.try_pop() {
                    return Ok(value);
                }
                rx_signal.wait()?;
            },
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
    SharedMemoryEvent {
        tx_ring: Arc<SharedRing>,
        rx_ring: Arc<SharedRing>,
        tx_signal: EventFd,
        rx_signal: EventFd,
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
            if let Some(value) = self.try_pop() {
                return value;
            }
            spin_loop();
        }
    }

    fn try_pop(&self) -> Option<u64> {
        let tail = self.tail.0.load(Ordering::Relaxed);
        let head = self.head.0.load(Ordering::Acquire);
        if tail == head {
            return None;
        }
        let index = tail & self.mask;
        let value = unsafe { *self.slots.slots[index].get() };
        self.tail.0.store(tail.wrapping_add(1), Ordering::Release);
        Some(value)
    }
}

#[repr(C)]
struct SharedRingLayout {
    head: Aligned<AtomicUsize>,
    tail: Aligned<AtomicUsize>,
    slots: [UnsafeCell<u64>; DEFAULT_RING_CAPACITY],
}

unsafe impl Sync for SharedRingLayout {}

struct SharedRegion {
    ptr: NonNull<SharedRingLayout>,
    len: usize,
}

unsafe impl Send for SharedRegion {}
unsafe impl Sync for SharedRegion {}

impl SharedRegion {
    fn new() -> FabricResult<Self> {
        let len = size_of::<SharedRingLayout>();
        let raw = unsafe {
            mmap(
                ptr::null_mut(),
                len,
                PROT_READ | PROT_WRITE,
                MAP_SHARED | MAP_ANONYMOUS,
                -1,
                0,
            )
        };
        if raw == (-1_isize as *mut c_void) {
            return Err(FabricError::Io(io::Error::last_os_error()));
        }
        let ptr = NonNull::new(raw.cast::<SharedRingLayout>())
            .ok_or_else(|| FabricError::Io(io::Error::other("mmap returned null")))?;
        unsafe {
            ptr.as_ptr().write(SharedRingLayout {
                head: Aligned(AtomicUsize::new(0)),
                tail: Aligned(AtomicUsize::new(0)),
                slots: std::array::from_fn(|_| UnsafeCell::new(0_u64)),
            });
        }
        Ok(Self { ptr, len })
    }
}

impl Drop for SharedRegion {
    fn drop(&mut self) {
        unsafe {
            ptr::drop_in_place(self.ptr.as_ptr());
            let _ = munmap(self.ptr.as_ptr().cast::<c_void>(), self.len);
        }
    }
}

struct SharedRing {
    region: SharedRegion,
}

unsafe impl Send for SharedRing {}
unsafe impl Sync for SharedRing {}

impl SharedRing {
    fn new() -> FabricResult<Self> {
        Ok(Self {
            region: SharedRegion::new()?,
        })
    }

    fn push(&self, value: u64) {
        let layout = unsafe { self.region.ptr.as_ref() };
        loop {
            let head = layout.head.0.load(Ordering::Relaxed);
            let tail = layout.tail.0.load(Ordering::Acquire);
            if head.wrapping_sub(tail) < DEFAULT_RING_CAPACITY {
                let index = head & (DEFAULT_RING_CAPACITY - 1);
                unsafe {
                    *layout.slots[index].get() = value;
                }
                layout.head.0.store(head.wrapping_add(1), Ordering::Release);
                return;
            }
            spin_loop();
        }
    }

    fn try_pop(&self) -> Option<u64> {
        let layout = unsafe { self.region.ptr.as_ref() };
        let tail = layout.tail.0.load(Ordering::Relaxed);
        let head = layout.head.0.load(Ordering::Acquire);
        if tail == head {
            return None;
        }
        let index = tail & (DEFAULT_RING_CAPACITY - 1);
        let value = unsafe { *layout.slots[index].get() };
        layout.tail.0.store(tail.wrapping_add(1), Ordering::Release);
        Some(value)
    }
}

struct EventFd {
    fd: OwnedFd,
}

impl EventFd {
    fn new() -> FabricResult<Self> {
        let fd = unsafe { eventfd(0, EFD_CLOEXEC) };
        if fd < 0 {
            return Err(FabricError::Io(io::Error::last_os_error()));
        }
        Ok(Self {
            fd: unsafe { OwnedFd::from_raw_fd(fd) },
        })
    }

    fn try_clone(&self) -> FabricResult<Self> {
        let fd = unsafe { dup(self.fd.as_raw_fd()) };
        if fd < 0 {
            return Err(FabricError::Io(io::Error::last_os_error()));
        }
        Ok(Self {
            fd: unsafe { OwnedFd::from_raw_fd(fd) },
        })
    }

    fn notify(&self) -> FabricResult<()> {
        let value = 1_u64.to_ne_bytes();
        let written = unsafe {
            write(
                self.fd.as_raw_fd(),
                value.as_ptr().cast::<c_void>(),
                value.len(),
            )
        };
        if written as usize != value.len() {
            return Err(FabricError::Io(io::Error::last_os_error()));
        }
        Ok(())
    }

    fn wait(&self) -> FabricResult<()> {
        let mut buf = [0_u8; 8];
        let read_bytes = unsafe {
            read(
                self.fd.as_raw_fd(),
                buf.as_mut_ptr().cast::<c_void>(),
                buf.len(),
            )
        };
        if read_bytes as usize != buf.len() {
            return Err(FabricError::Io(io::Error::last_os_error()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{LocalEndpoint, TransportKind};
    use std::thread;

    #[test]
    fn request_reply_works_for_all_local_transports() {
        for kind in [
            TransportKind::SyncChannel,
            TransportKind::SpscRing,
            TransportKind::SharedMemoryEvent,
        ] {
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
