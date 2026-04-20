# IDCP Bench Results (Publishable Run)

This document captures a full conventional-vs-IDCP benchmark run for the current prototype.

## Methodology

- Command: `cargo run -p idcp-bench --release -- 12`
- Date (UTC): 2026-04-20
- Trials per scenario: 12
- Conventional baseline: `ExecutionMode::Naive`
- IDCP mode: `ExecutionMode::Idcp`
- Scenarios exercised (all 4 subsystems are engaged in each scenario):
  - flow-sensitive mesh (transport + placement)
  - memory-sharing host (memory + pressure)
  - batch-heavy pipeline (flow batching + pressure)
  - interactive graph (low-latency + locality)

## Benchmark Output

```text
# IDCP conventional-vs-idcp benchmark
trials_per_scenario=12
| scenario | purpose | mem% | flow% | copy% | runtime_lat% | runtime_tput% | score_x |
| :-- | :-- | --: | --: | --: | --: | --: | --: |
| agent-mesh | flow-sensitive mesh (transport + placement) | 56.5% | -33.3% | 43.8% | 0.0% | -0.7% | 2.18 |
  - default plan: transport=`unix_stream` batching=1 zero_copy=false mem_mib=2.50 copy_ns=320
  - idcp plan: transport=`shm_eventfd` batching=1 zero_copy=true mem_mib=1.09 copy_ns=180
  - flow_ns mean±sd (median): default=27532±5943 (29452), idcp=36706±3961 (34855) (n=12)
  - runtime latency_ns mean±sd (median): default=93432±12047 (93146) (min=77268 max=118373) idcp=93388±8593 (92302) (min=76070 max=112700)
  - runtime throughput mean±sd: default=10873±1334 idcp=10798±994

| plugin-host | memory-sharing host (memory + pressure) | 75.4% | -22.6% | 43.8% | 3.2% | 5.9% | 3.17 |
  - default plan: transport=`unix_stream` batching=1 zero_copy=false mem_mib=2.35 copy_ns=320
  - idcp plan: transport=`shm_eventfd` batching=1 zero_copy=true mem_mib=0.58 copy_ns=180
  - flow_ns mean±sd (median): default=26884±5375 (25165), idcp=32970±3061 (32186) (n=12)
  - runtime latency_ns mean±sd (median): default=86976±4996 (87212) (min=80122 max=96197) idcp=84149±11137 (87559) (min=48667 max=91340)
  - runtime throughput mean±sd: default=11534±657 idcp=12218±2546

| embedding-farm | batch-heavy pipeline (flow batching + pressure) | 53.6% | 92.4% | 65.6% | -12.1% | -9.7% | 4.86 |
  - default plan: transport=`unix_stream` batching=1 zero_copy=false mem_mib=3.47 copy_ns=320
  - idcp plan: transport=`shm_eventfd` batching=16 zero_copy=true mem_mib=1.61 copy_ns=110
  - flow_ns mean±sd (median): default=27233±4141 (26846), idcp=2059±68 (2073) (n=12)
  - runtime latency_ns mean±sd (median): default=86086±4287 (86298) (min=79188 max=92718) idcp=96529±12464 (93406) (min=81650 max=118134)
  - runtime throughput mean±sd: default=11645±587 idcp=10520±1256

| terminal-graph | interactive graph (low-latency + locality) | 53.8% | -3.8% | 43.8% | 1.3% | 1.1% | 2.17 |
  - default plan: transport=`unix_stream` batching=1 zero_copy=false mem_mib=1.64 copy_ns=320
  - idcp plan: transport=`shm_eventfd` batching=1 zero_copy=true mem_mib=0.76 copy_ns=180
  - flow_ns mean±sd (median): default=45891±2158 (45504), idcp=47617±1576 (47326) (n=12)
  - runtime latency_ns mean±sd (median): default=98910±15469 (96584) (min=80060 max=128441) idcp=97601±15118 (90502) (min=78370 max=131047)
  - runtime throughput mean±sd: default=10351±1555 idcp=10466±1443
```

## Interpretation

- **Memory and copy penalties consistently improve** in IDCP mode across all scenarios.
- **Flow is profile-dependent**: IDCP strongly improves flow for `embedding-farm`, but regresses for `agent-mesh`, `plugin-host`, and slightly for `terminal-graph` in this run.
- **Runtime is mixed**:
  - `plugin-host` and `terminal-graph` show modest runtime wins.
  - `agent-mesh` is near parity.
  - `embedding-farm` shows a runtime regression despite very strong flow improvements, signaling a batching/runtime pipeline trade-off.

## Research Priorities Suggested by Data

1. Add adaptive transport fallback when measured flow degrades vs baseline for small payload cross-process workloads.
2. Perform batching sweeps and choose batch size from measured runtime Pareto front (latency vs throughput), not static profile defaults.
3. Add ablation benchmarks (`flow-only`, `memory-only`, `placement-only`, `pressure-only`) to isolate each subsystem contribution to final runtime.
4. Add multi-baseline comparisons (not just one naive baseline) for stronger publishable claims.
