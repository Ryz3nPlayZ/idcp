# Research Log

## Day 1

- repo initialized as a Rust workspace
- thesis and benchmark bar documented before implementation
- baseline benchmark harness added for local TCP, Unix sockets, sync channels, batching, and a pure in-process ring
- first baseline run recorded:
  - `tcp_loopback`: `38.456 us`
  - `unix_stream`: `14.805 us`
  - `spsc_ring`: `0.336 us`
- `fabric-core` now owns a first reusable local endpoint API
- benchmark rerun through the abstraction:
  - `tcp_loopback`: `39.014 us`
  - `unix_stream`: `18.657 us`
  - `spsc_ring`: `0.376 us`
- `shm_eventfd` transport added for shared memory plus kernel wakeups
- benchmark result for `shm_eventfd`:
  - `unix_stream`: `9.418 us`
  - `shm_eventfd`: `8.578 us`
  - `spsc_ring`: `0.245 us`
- conclusion so far: shared memory plus wakeups helps, but the pure ring remains the dominant hot path
- project direction widened: a deeper memory breakthrough matters more than transport tuning alone
- first memory-discovery prototype chosen: detect exact and near-duplicate page families and estimate base-plus-delta savings
- first `memory-lab` run:
  - `exact_dups`: `74.9%` estimated savings
  - `near_dups`: `66.8%` estimated savings
  - `mixed`: `54.3%` estimated savings
- conclusion so far: near-duplicate pages are promising enough to justify a deeper page-family runtime
