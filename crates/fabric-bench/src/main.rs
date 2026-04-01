use fabric_core::{LocalEndpoint, TransportKind};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::os::unix::net::UnixStream;
use std::thread;
use std::time::{Duration, Instant};

const ITERATIONS: usize = 200_000;
const BATCH_SIZE: usize = 32;
const REPEATS: usize = 5;

struct BenchResult {
    name: &'static str,
    total: Duration,
}

#[derive(Clone)]
struct BenchSample {
    total: Duration,
}

impl BenchSample {
    fn ns_per_round_trip(&self) -> f64 {
        self.total.as_nanos() as f64 / ITERATIONS as f64
    }

    fn round_trips_per_sec(&self) -> f64 {
        ITERATIONS as f64 / self.total.as_secs_f64()
    }
}

struct BenchSummary {
    name: &'static str,
    median_ns: f64,
    best_ns: f64,
    median_rps: f64,
    best_rps: f64,
}

fn main() {
    let benches: [(&str, fn() -> BenchResult); 7] = [
        (TransportKind::SyncChannel.label(), || {
            bench_local_fabric(TransportKind::SyncChannel)
        }),
        (TransportKind::UnixStream.label(), || {
            bench_local_fabric(TransportKind::UnixStream)
        }),
        (TransportKind::SharedMemoryEvent.label(), || {
            bench_local_fabric(TransportKind::SharedMemoryEvent)
        }),
        ("unix_stream_batch32", bench_unix_stream_batched),
        ("tcp_loopback", bench_tcp_loopback),
        ("tcp_loopback_batch32", bench_tcp_loopback_batched),
        (TransportKind::SpscRing.label(), || {
            bench_local_fabric(TransportKind::SpscRing)
        }),
    ];

    println!(
        "{:<22} {:>12} {:>12} {:>12} {:>12}",
        "transport", "median_us", "best_us", "median_rps", "best_rps"
    );
    for (name, bench_fn) in benches {
        let summary = summarize(name, bench_fn);
        println!(
            "{:<22} {:>12.3} {:>12.3} {:>12.0} {:>12.0}",
            summary.name,
            summary.median_ns / 1_000.0,
            summary.best_ns / 1_000.0,
            summary.median_rps,
            summary.best_rps,
        );
    }
}

fn summarize(name: &'static str, bench_fn: fn() -> BenchResult) -> BenchSummary {
    let mut samples = Vec::with_capacity(REPEATS);
    for _ in 0..REPEATS {
        let result = bench_fn();
        debug_assert_eq!(result.name, name);
        samples.push(BenchSample {
            total: result.total,
        });
    }

    samples.sort_by_key(|sample| sample.total);
    let median = &samples[samples.len() / 2];
    let best = &samples[0];

    BenchSummary {
        name,
        median_ns: median.ns_per_round_trip(),
        best_ns: best.ns_per_round_trip(),
        median_rps: median.round_trips_per_sec(),
        best_rps: best.round_trips_per_sec(),
    }
}

fn bench_local_fabric(kind: TransportKind) -> BenchResult {
    let (mut client, mut server) = LocalEndpoint::pair(kind).unwrap();
    let worker = thread::spawn(move || {
        for _ in 0..ITERATIONS {
            let value = server.recv().unwrap();
            server.send(value).unwrap();
        }
    });

    let start = Instant::now();
    for i in 0..ITERATIONS as u64 {
        let ack = client.request(i).unwrap();
        debug_assert_eq!(ack, i);
    }
    let total = start.elapsed();
    worker.join().unwrap();

    BenchResult {
        name: kind.label(),
        total,
    }
}

fn bench_unix_stream_batched() -> BenchResult {
    let (mut client, mut server) = UnixStream::pair().unwrap();

    let worker = thread::spawn(move || {
        let mut buf = [0_u8; BATCH_SIZE * 8];
        for _ in 0..ITERATIONS / BATCH_SIZE {
            server.read_exact(&mut buf).unwrap();
            server.write_all(&buf).unwrap();
        }
    });

    let total = socket_ping_pong_batched(&mut client);
    worker.join().unwrap();

    BenchResult {
        name: "unix_stream_batch32",
        total,
    }
}

fn bench_tcp_loopback() -> BenchResult {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();

    let worker = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream.set_nodelay(true).unwrap();
        let mut buf = [0_u8; 8];
        for _ in 0..ITERATIONS {
            stream.read_exact(&mut buf).unwrap();
            stream.write_all(&buf).unwrap();
        }
    });

    let mut client = TcpStream::connect(addr).unwrap();
    client.set_nodelay(true).unwrap();
    let total = socket_ping_pong(&mut client);
    worker.join().unwrap();

    BenchResult {
        name: "tcp_loopback",
        total,
    }
}

fn bench_tcp_loopback_batched() -> BenchResult {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();

    let worker = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream.set_nodelay(true).unwrap();
        let mut buf = [0_u8; BATCH_SIZE * 8];
        for _ in 0..ITERATIONS / BATCH_SIZE {
            stream.read_exact(&mut buf).unwrap();
            stream.write_all(&buf).unwrap();
        }
    });

    let mut client = TcpStream::connect(addr).unwrap();
    client.set_nodelay(true).unwrap();
    let total = socket_ping_pong_batched(&mut client);
    worker.join().unwrap();

    BenchResult {
        name: "tcp_loopback_batch32",
        total,
    }
}

fn socket_ping_pong<T: Read + Write>(stream: &mut T) -> Duration {
    let mut send = [0_u8; 8];
    let mut recv = [0_u8; 8];
    let start = Instant::now();

    for i in 0..ITERATIONS as u64 {
        send.copy_from_slice(&i.to_le_bytes());
        stream.write_all(&send).unwrap();
        stream.read_exact(&mut recv).unwrap();
        let ack = u64::from_le_bytes(recv);
        debug_assert_eq!(ack, i);
    }

    start.elapsed()
}

fn socket_ping_pong_batched<T: Read + Write>(stream: &mut T) -> Duration {
    let mut send = [0_u8; BATCH_SIZE * 8];
    let mut recv = [0_u8; BATCH_SIZE * 8];
    let start = Instant::now();

    for batch in 0..ITERATIONS / BATCH_SIZE {
        for offset in 0..BATCH_SIZE {
            let value = (batch * BATCH_SIZE + offset) as u64;
            let bytes = value.to_le_bytes();
            let start_idx = offset * 8;
            send[start_idx..start_idx + 8].copy_from_slice(&bytes);
        }

        stream.write_all(&send).unwrap();
        stream.read_exact(&mut recv).unwrap();

        for offset in 0..BATCH_SIZE {
            let start_idx = offset * 8;
            let ack = u64::from_le_bytes(recv[start_idx..start_idx + 8].try_into().unwrap());
            let expected = (batch * BATCH_SIZE + offset) as u64;
            debug_assert_eq!(ack, expected);
        }
    }

    start.elapsed()
}
