# Current Results

## Status

Baseline benchmark harness is in place and has been run on this machine.

## Current results

| Transport | Median latency per round trip | Best latency per round trip | Median round trips / sec |
|---|---:|---:|---:|
| `sync_channel(0)` | 20.953 us | 17.529 us | 47,726 |
| `unix_stream` | 18.657 us | 17.806 us | 53,598 |
| `unix_stream_batch32` | 0.586 us | 0.561 us | 1,707,448 |
| `tcp_loopback` | 39.014 us | 34.833 us | 25,632 |
| `tcp_loopback_batch32` | 1.325 us | 1.189 us | 754,574 |
| `spsc_ring` | 0.376 us | 0.359 us | 2,659,088 |

## Initial conclusions

- local loopback TCP is the slowest path in this harness
- Unix sockets are materially cheaper than local TCP
- batching changes the economics of both Unix sockets and TCP
- the pure in-process ring is the fastest path measured so far
- the `fabric-core` abstraction preserves the main performance story after moving beyond handwritten fast paths
