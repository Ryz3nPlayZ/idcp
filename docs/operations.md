# Operations

This document covers how to run the current IDCP controller surface and inspect its outputs.

## Prerequisites

- Rust toolchain with Cargo
- Linux-like environment for the current local transport implementations

The prototype currently relies on Unix-domain sockets and `eventfd`-style behavior, so Linux is the intended environment.

## Quick start

From the repo root:

```bash
cargo test
cargo run -p idcpd --release -- profiles
cargo run -p idcpd --release -- bench
```

## Running a daemon session

Example:

```bash
cargo run -p idcpd --release -- daemon embedding-farm 5 100 /tmp/idcpd
```

Arguments:

- profile
- ticks
- interval in milliseconds
- optional state directory

If no state directory is supplied, the default is:

- `/tmp/idcpd`

## Inspecting state

```bash
cargo run -p idcpd --release -- status /tmp/idcpd
```

This prints the contents of `latest.txt`.

## Generating a standalone report

```bash
cargo run -p idcpd --release -- report embedding-farm 5 /tmp/idcpd/report-explicit.md
```

This reruns the controller for the chosen profile and writes a markdown report.

## State directory layout

After a daemon run, expect:

- `latest.txt`
- `history.log`
- `report.md`

### `latest.txt`

Contains:

- timestamp
- profile
- last tick
- last chosen action
- memory / flow / copy improvements
- score multiplier
- latest runtime latency and throughput

### `history.log`

Contains:

- one summary line per daemon run

### `report.md`

Contains:

- summary metrics
- per-tick controller entries

## Common commands

Plan only:

```bash
cargo run -p idcpd --release -- plan agent-mesh
```

Plan with measured flow path:

```bash
cargo run -p idcpd --release -- measure agent-mesh
```

Naive vs IDCP comparison:

```bash
cargo run -p idcpd --release -- simulate agent-mesh
```

Live runtime execution:

```bash
cargo run -p idcpd --release -- run embedding-farm
```

Benchmark matrix:

```bash
cargo run -p idcp-bench --release
```

## Reading results

When looking at output, separate:

- modeled improvements
- measured transport improvements
- measured runtime improvements

Do not collapse them into one claim.

Useful fields:

- `mem%`
- `flow%`
- `copy%`
- `score_x`
- `runtime_latency_ns`
- `runtime_throughput`

## Current operational limitations

- no long-lived background supervision
- no IPC control socket or HTTP API
- no structured JSON state yet
- no config files yet
- no automated benchmark aggregation yet

The daemon command is persistent only for the duration of a controller run. It is a controller loop runner, not a resident service manager.
