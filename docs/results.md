# Current Results

## Status

Baseline benchmark harness is in place and has been run on this machine.

## Baseline results

| Transport | Median latency per round trip | Best latency per round trip | Median round trips / sec |
|---|---:|---:|---:|
| `sync_channel(0)` | 12.997 us | 12.202 us | 76,942 |
| `unix_stream` | 14.805 us | 14.307 us | 67,544 |
| `unix_stream_batch32` | 0.459 us | 0.432 us | 2,177,590 |
| `tcp_loopback` | 38.456 us | 36.052 us | 26,004 |
| `tcp_loopback_batch32` | 1.232 us | 1.072 us | 811,803 |
| `spsc_ring` | 0.336 us | 0.171 us | 2,978,026 |

## Initial conclusions

- local loopback TCP is the slowest path in this harness
- Unix sockets are materially cheaper than local TCP
- batching changes the economics of both Unix sockets and TCP
- the pure in-process ring is the fastest path measured so far
- there is enough signal here to justify building a unified transport abstraction next
