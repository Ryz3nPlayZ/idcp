# Research Log

## Day 1

- repo initialized as a Rust workspace
- thesis and benchmark bar documented before implementation
- baseline benchmark harness added for local TCP, Unix sockets, sync channels, batching, and a pure in-process ring
- first baseline run recorded:
  - `tcp_loopback`: `38.456 us`
  - `unix_stream`: `14.805 us`
  - `spsc_ring`: `0.336 us`
- next step is to wrap multiple transport choices behind a stable API
