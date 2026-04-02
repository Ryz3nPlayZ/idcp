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
agent-mesh          56.5%      8.9%     43.8%        2.18
plugin-host         75.4%      8.9%     43.8%        3.17
embedding-farm      53.6%     94.3%     65.6%        4.86
terminal-graph      53.8%      8.9%     43.8%        2.17
```

## Interpretation

- memory representation dominates most profiles
- flow planning matters most for the `embedding-farm` profile because batching changes the effective path drastically
- placement decisions reduce estimated copy penalties across all profiles
- pressure planning decides when page families and rebalance work should activate

This is still a model-driven benchmark, not a full production runtime measurement, but it shows the intended system shape:

> IDCP is not one optimization. It is a coordinated same-machine control fabric.
