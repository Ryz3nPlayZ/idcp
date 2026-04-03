# Finish Line

This document defines the finish line for the current IDCP phase and the evidence that it was reached.

## Finish line definition

IDCP would count as a product-shaped prototype when it had all of the following:

- a persistent controller command instead of only one-shot planning commands
- measurable runtime behavior, not only modeled scores
- observable state that can be inspected after a run
- generated reports that summarize the system's decisions and effects
- documentation that explains what the project is, what it does, and how to use it

## Reached state

The repo now satisfies that bar.

## What exists now

- `idcpd daemon <profile> [ticks] [interval_ms] [state_dir]`
  - runs a controller loop
- `idcpd status [state_dir]`
  - reads the latest snapshot
- `idcpd report <profile> [ticks] [output_path]`
  - emits a markdown report
- `idcpd run <profile>`
  - executes a live multi-stage runtime pipeline
- `idcpd bench`
  - prints the cross-engine matrix

## Files produced by the daemon

In the chosen state dir:

- `latest.txt`
- `history.log`
- `report.md`

## Evidence

Example daemon summary:

```text
idcpd daemon complete profile=embedding-farm ticks=3 state_dir=/tmp/idcpd-final-2
summary avg_mem=53.6% avg_flow=94.8% avg_copy=65.6% avg_score=3.63x avg_runtime_latency=35.1% avg_runtime_throughput=60.8%
```

Example report content:

```text
# IDCP Controller Report

- profile: `embedding-farm`
- ticks: `3`
- interval_ms: `0`
```

## Validation

Validation used for this finish line:

```bash
cargo test
cargo fmt --check
cargo run -p idcp-bench --release
cargo run -p idcpd --release -- daemon embedding-farm 3 0 /tmp/idcpd-final-2
cargo run -p idcpd --release -- status /tmp/idcpd-final-2
cargo run -p idcpd --release -- report embedding-farm 3 /tmp/idcpd-final-2/report-explicit.md
```

## Boundaries

This finish line does not mean:

- production-grade system daemon
- system-wide OS integration
- universal wins across arbitrary software
- kernel-level memory or scheduler control

It means the repo is now a coherent, runnable, documented controller prototype instead of disconnected experiments.
