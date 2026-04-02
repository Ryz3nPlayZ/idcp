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
