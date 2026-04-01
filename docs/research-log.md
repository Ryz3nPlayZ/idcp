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
- next step is to add a shared-memory-plus-signal transport for a more realistic same-host path
