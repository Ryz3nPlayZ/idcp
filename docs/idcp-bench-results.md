# IDCP Bench Results

`idcp-bench` is the cross-engine benchmark for the current IDCP prototype.

It compares a naive same-machine system against an IDCP-managed plan using:

- `idcp-flow`
- `idcp-memory`
- `idcp-placement`
- `idcp-pressure`
- live flow-path measurement from `idcp-system`

## Current benchmark output

```text
IDCP cross-engine benchmark
profile              mem%     flow%     copy%   live_ns      score_x
agent-mesh          56.5%     77.6%     43.8%     11463        2.40
plugin-host         75.4%    -83.8%     43.8%     36872        3.28
embedding-farm      53.6%     94.0%     65.6%      1272        3.61
terminal-graph      53.8%     42.7%     43.8%     16278        2.27
```

## Interpretation

- memory representation still drives a large part of the score
- flow improvement is highly profile-dependent
- `embedding-farm` benefits most because batching and shared-memory transport align with the workload shape
- `plugin-host` gains heavily from memory savings even when one measured flow run is worse than the naive path
- the benchmark is more honest than the earlier planner-only versions because flow latency is now measured live
- repeated runs matter because single measured samples are noisy

## Live runtime sample

Example controller run for `embedding-farm`:

```text
idcpd daemon complete profile=embedding-farm ticks=3 state_dir=/tmp/idcpd-final-2
summary avg_mem=53.6% avg_flow=94.8% avg_copy=65.6% avg_score=3.63x avg_runtime_latency=35.1% avg_runtime_throughput=60.8%
```

Example report excerpt:

```text
| tick | action | mem% | flow% | copy% | score_x | naive_ns | idcp_ns | naive_tput | idcp_tput |
| 1 | `FavorZeroCopy` | 53.6 | 94.2 | 65.6 | 3.38 | 61043 | 66673 | 16381 | 14998 |
| 2 | `FavorZeroCopy` | 53.6 | 94.8 | 65.6 | 3.42 | 76174 | 66653 | 13127 | 15002 |
| 3 | `FavorZeroCopy` | 53.6 | 95.4 | 65.6 | 4.09 | 103034 | 66650 | 9705 | 15003 |
```

## Important caveat

This is still a prototype benchmark.

It is not proof that IDCP beats the default stack for every modern device or every application. It is evidence that for the built-in same-machine profiles, coordinating flow, memory, placement, and pressure can materially outperform a naive local design.
