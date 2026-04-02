# IDCP Bench Results

`idcp-bench` is the first cross-engine benchmark in the repo. It compares a naive same-machine system against an IDCP-managed plan using:

- `idcp-flow`
- `idcp-memory`
- `idcp-placement`
- `idcp-pressure`

## Current output

```text
IDCP cross-engine benchmark
profile              mem%     flow%     copy%      score_x
profile              mem%     flow%     copy%   live_ns      score_x
agent-mesh          56.5%     28.0%     43.8%      7752        2.24
plugin-host         75.4%     28.3%     43.8%      8188        3.25
embedding-farm      53.6%     96.0%     65.6%       659        4.67
terminal-graph      53.8%     40.4%     43.8%      8107        2.25
```

## Interpretation

- memory representation still dominates most profiles
- flow planning is now partly live-measured instead of purely table-driven
- `embedding-farm` still benefits most from flow planning because batching changes the effective path drastically
- placement decisions reduce estimated copy penalties across all profiles
- pressure planning decides when page families and rebalance work should activate

This is still a model-driven benchmark, not a full production runtime measurement, but it shows the intended system shape:

> IDCP is not one optimization. It is a coordinated same-machine control fabric.

## Live runtime samples

`idcpd run embedding-farm`

```text
idcp runtime profile=embedding-farm
mode           messages     latency_ns     throughput
naive             12000          31427          31819
idcp              12000          26847          37247
```

`idcpd run agent-mesh`

```text
idcp runtime profile=agent-mesh
mode           messages     latency_ns     throughput
naive             20000          27607          36222
idcp              20000          27162          36815
```

Interpretation:

- the runtime path now executes a real multi-stage worker pipeline
- the wins are smaller than the planner-only numbers, which is expected and healthier
- `embedding-farm` benefits more than `agent-mesh`, which matches the earlier benchmark signal
