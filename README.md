# IDCP

IDCP is a same-machine control fabric for modular software.

The project thesis is simple: modern computers waste capability because local communication, memory representation, placement, and pressure response are usually handled as separate generic subsystems. IDCP explores whether coordinating them as one runtime can make the same hardware do more work with less waste.

This repo is now past the pure research-only stage. It includes:

- low-level local transports
- memory page-family analysis
- placement and pressure planning
- scenario evaluation
- a persistent `idcpd` controller surface
- status snapshots and generated markdown reports

## Why it is cool

IDCP does not try to be another agent framework or another benchmark toy. It is trying to answer a lower-level question:

> Can one same-machine controller improve modular software by coordinating message flow, memory shape, placement, and pressure response together?

The interesting part is the combination:

- local communication path selection instead of one-size-fits-all sockets
- page-family memory reduction instead of treating every page as unrelated raw bytes
- locality-aware placement instead of moving data around blindly
- pressure planning that activates compression, page families, and rebalance behavior together

That gives the repo one coherent systems story instead of isolated optimizations.

## Current product surface

The primary entrypoint is `idcpd`.

It currently supports:

- `plan`: print the chosen IDCP strategy for a profile
- `measure`: print the chosen strategy with a live-measured flow path
- `simulate`: compare naive vs IDCP using the measured path and the cross-engine model
- `run`: execute a real multi-stage runtime pipeline
- `bench`: print the benchmark matrix across all profiles
- `daemon`: run a persistent controller loop and write status/report artifacts
- `status`: read the latest persisted daemon snapshot
- `report`: generate a markdown controller report

This is still user-space prototype software. It is not yet a kernel module, not a drop-in OS replacement, and not a transparent accelerator for arbitrary apps.

## Workspace layout

- `crates/fabric-core`
  - low-level local transports: sync channel, Unix stream, SPSC ring, shared memory + `eventfd`
- `crates/fabric-bench`
  - original transport benchmark harness
- `crates/idcp-flow`
  - flow hints and transport selection
- `crates/idcp-memory`
  - page-family discovery and memory representation analysis
- `crates/idcp-placement`
  - locality and copy-penalty decisions
- `crates/idcp-pressure`
  - pressure classification and response planning
- `crates/idcp-system`
  - scenarios, evaluation, runtime execution, controller loop, report generation
- `crates/idcp-bench`
  - cross-engine benchmark matrix
- `crates/idcpd`
  - product-facing CLI/controller

## Profiles

Built-in profiles:

- `agent-mesh`
- `plugin-host`
- `embedding-farm`
- `terminal-graph`

These represent classes of same-machine modular workloads with different message shape, hot data size, sharing ratio, and pressure behavior.

## Commands

From the repo root:

```bash
cargo run -p idcpd --release -- profiles
cargo run -p idcpd --release -- plan agent-mesh
cargo run -p idcpd --release -- measure embedding-farm
cargo run -p idcpd --release -- simulate terminal-graph
cargo run -p idcpd --release -- run embedding-farm
cargo run -p idcpd --release -- bench
```

Persistent controller flow:

```bash
cargo run -p idcpd --release -- daemon embedding-farm 5 250 /tmp/idcpd
cargo run -p idcpd --release -- status /tmp/idcpd
cargo run -p idcpd --release -- report embedding-farm 5 /tmp/idcpd/report-explicit.md
```

## What the daemon does

`idcpd daemon` runs a repeated controller loop for a chosen profile.

On each tick it:

- evaluates naive and IDCP plans
- measures the chosen flow path
- executes the live runtime pipeline for naive and IDCP
- chooses a control action
- records the results

It writes:

- `latest.txt`
  - last controller snapshot
- `history.log`
  - one-line summaries per daemon run
- `report.md`
  - markdown report for the daemon run

Default state dir:

- `/tmp/idcpd`

## Example daemon output

```text
idcpd daemon complete profile=embedding-farm ticks=3 state_dir=/tmp/idcpd-final-2
summary avg_mem=53.6% avg_flow=94.8% avg_copy=65.6% avg_score=3.63x avg_runtime_latency=35.1% avg_runtime_throughput=60.8%
```

Example status snapshot:

```text
timestamp=1775224630
profile=embedding-farm
tick=3
action=FavorZeroCopy
mem_percent=53.6
flow_percent=95.4
copy_percent=65.6
score_multiplier=4.09
runtime_latency_ns=66650
runtime_throughput=15003
```

## Current benchmark signal

Representative current cross-engine benchmark output:

```text
IDCP cross-engine benchmark
profile              mem%     flow%     copy%   live_ns      score_x
agent-mesh          56.5%     77.6%     43.8%     11463        2.40
plugin-host         75.4%    -83.8%     43.8%     36872        3.28
embedding-farm      53.6%     94.0%     65.6%      1272        3.61
terminal-graph      53.8%     42.7%     43.8%     16278        2.27
```

Representative live runtime report for `embedding-farm`:

- average modeled memory reduction: `53.6%`
- average measured flow reduction: `94.8%`
- average copy-penalty reduction: `65.6%`
- average aggregate score multiplier: `3.63x`
- average runtime latency reduction: `35.1%`
- average runtime throughput increase: `60.8%`

Interpretation:

- IDCP is strongest when batching, zero-copy preference, and page-family savings all align
- gains are profile-dependent
- live runtime gains are smaller than the best modeled gains, which is expected
- single-run measured results vary, so repeated runs matter more than one cherry-picked sample

## Finish line reached in this repo

The finish line for this phase was:

- persistent controller surface
- adaptive control action per tick
- status artifacts
- generated reports
- repeatable benchmark path
- real runtime execution, not just planner output

That finish line is now met.

## Validation

Current validation commands:

```bash
cargo test
cargo fmt --check
cargo run -p idcp-bench --release
cargo run -p idcpd --release -- daemon embedding-farm 3 0 /tmp/idcpd-final-2
cargo run -p idcpd --release -- status /tmp/idcpd-final-2
```

## Limitations

Important boundaries:

- this does not optimize arbitrary software on your machine automatically
- most gains are still profile-driven, not system-wide
- the memory engine is analytical, not hooked into real process memory
- the placement and pressure engines still choose plans, not kernel-enforced actions
- portability beyond Linux-like local transports is not proven

So the repo is product-shaped and runnable, but it is still a prototype controller rather than production systems infrastructure.

## Where to read next

- [docs/v1-v5.md](/home/zemul/idcp/docs/v1-v5.md)
- [docs/architecture.md](/home/zemul/idcp/docs/architecture.md)
- [docs/roadmap.md](/home/zemul/idcp/docs/roadmap.md)
- [docs/handoff.md](/home/zemul/idcp/docs/handoff.md)
- [docs/operations.md](/home/zemul/idcp/docs/operations.md)
- [docs/contributing.md](/home/zemul/idcp/docs/contributing.md)
- [docs/idcp-bench-results.md](/home/zemul/idcp/docs/idcp-bench-results.md)
- [docs/memory-discovery.md](/home/zemul/idcp/docs/memory-discovery.md)
- [docs/finish-line.md](/home/zemul/idcp/docs/finish-line.md)
