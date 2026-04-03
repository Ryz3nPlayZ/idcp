# Developer Handoff

This document is for the next developer who inherits the repo.

It explains:

- what the project is
- what is stable enough to trust
- what is still experimental
- what to run first
- where the most important next work is

## Project summary

IDCP is a same-machine controller prototype.

It explores whether a single runtime can improve modular local workloads by coordinating:

- flow
- memory
- placement
- pressure

The codebase is structured and usable, but it is still a prototype controller rather than production infrastructure.

## Start here

Read these in order:

1. [README.md](/home/zemul/idcp/README.md)
2. [architecture.md](/home/zemul/idcp/docs/architecture.md)
3. [finish-line.md](/home/zemul/idcp/docs/finish-line.md)
4. [roadmap.md](/home/zemul/idcp/docs/roadmap.md)
5. [idcp-bench-results.md](/home/zemul/idcp/docs/idcp-bench-results.md)

Then inspect:

- [idcp-system lib](/home/zemul/idcp/crates/idcp-system/src/lib.rs)
- [idcpd main](/home/zemul/idcp/crates/idcpd/src/main.rs)

## Current stable surfaces

These surfaces are stable enough to build on without redesigning everything:

- `ScenarioProfile`
- `ScenarioSpec`
- `evaluate` / `evaluate_measured`
- `execute_runtime`
- `run_controller`
- `render_report_markdown`
- `idcpd` command surface

These are the main integration points for the current phase.

## Current unstable surfaces

These are still easy to change:

- aggregate scoring model
- exact benchmark numbers
- control action policy
- profile definitions
- state file format

Do not assume those are final contracts.

## Commands to run first

From the repo root:

```bash
cargo test
cargo fmt --check
cargo run -p idcp-bench --release
cargo run -p idcpd --release -- bench
cargo run -p idcpd --release -- daemon embedding-farm 3 0 /tmp/idcpd-handoff
cargo run -p idcpd --release -- status /tmp/idcpd-handoff
```

That gives you the current health and product surface quickly.

## What is measured vs inferred

Measured:

- low-level transport latency
- live runtime pipeline latency and throughput

Inferred or modeled:

- memory-family savings
- copy-penalty estimates
- pressure strategy benefit
- aggregate score meaning

This matters when you change docs or benchmarks. Do not blur the line.

## Code map

### Runtime and control

- [idcp-system lib](/home/zemul/idcp/crates/idcp-system/src/lib.rs)
- [idcpd main](/home/zemul/idcp/crates/idcpd/src/main.rs)

### Transport substrate

- [fabric-core lib](/home/zemul/idcp/crates/fabric-core/src/lib.rs)
- [fabric-bench main](/home/zemul/idcp/crates/fabric-bench/src/main.rs)

### Engines

- [idcp-flow lib](/home/zemul/idcp/crates/idcp-flow/src/lib.rs)
- [idcp-memory lib](/home/zemul/idcp/crates/idcp-memory/src/lib.rs)
- [idcp-placement lib](/home/zemul/idcp/crates/idcp-placement/src/lib.rs)
- [idcp-pressure lib](/home/zemul/idcp/crates/idcp-pressure/src/lib.rs)

## First good tasks for a new developer

If you are taking over, these are strong first tasks:

- add repeated-run aggregation to the benchmark surface
- write structured JSON state alongside the text artifacts
- add a new realistic workload profile
- make report generation include repeated-run summaries
- improve benchmark stability and variance reporting

## Things to avoid on takeover

- do not turn this into a generic networking framework
- do not market modeled behavior as measured fact
- do not add distributed systems scope yet
- do not rewrite all crates at once

The codebase is still small enough that focused changes beat big redesigns.

## Handoff definition of success

A successful next contributor should be able to:

- run the repo successfully
- understand the architecture quickly
- identify what is experimental
- make one meaningful improvement without changing the whole thesis

If future changes make that harder, the repo is getting worse, not better.
