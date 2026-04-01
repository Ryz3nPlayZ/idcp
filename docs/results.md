# Current Results

## Status

Baseline benchmark harness is in place and has been run on this machine.

## Current results

| Transport | Median latency per round trip | Best latency per round trip | Median round trips / sec |
|---|---:|---:|---:|
| `sync_channel(0)` | 7.422 us | 6.944 us | 134,727 |
| `unix_stream` | 9.418 us | 8.971 us | 106,180 |
| `shm_eventfd` | 8.578 us | 8.263 us | 116,577 |
| `unix_stream_batch32` | 0.300 us | 0.282 us | 3,330,839 |
| `tcp_loopback` | 24.699 us | 23.388 us | 40,488 |
| `tcp_loopback_batch32` | 0.743 us | 0.673 us | 1,346,550 |
| `spsc_ring` | 0.245 us | 0.241 us | 4,079,592 |

## Initial conclusions

- local loopback TCP is the slowest path in this harness
- Unix sockets are materially cheaper than local TCP
- batching changes the economics of both Unix sockets and TCP
- the pure in-process ring is the fastest path measured so far
- the `fabric-core` abstraction preserves the main performance story after moving beyond handwritten fast paths
- `shm_eventfd` is a legitimate same-host improvement over Unix sockets, but not a miracle path
- the runtime should eventually choose transports by locality and workload instead of assuming one path is universally best
