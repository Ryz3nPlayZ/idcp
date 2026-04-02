# IDCP Bench Results

`idcp-bench` is the first cross-engine benchmark in the repo. It compares a naive same-machine system against an IDCP-managed plan using:

- `idcp-flow`
- `idcp-memory`
- `idcp-placement`
- `idcp-pressure`

## Current output

```text
IDCP cross-engine benchmark
raw_mib=3.34 smart_mib=1.53 page_family_savings=54.3%

mode              mem_mib      flow_ns   copy_penalty        score
naive                3.34        24699            320          637
idcp                 1.53         8578            180         1165

improvement: mem=54.3% flow=65.3% copy=43.8% total_score=1.8x
```

## Interpretation

- memory representation dominates the current win
- flow planning cuts same-host coordination cost materially versus naive loopback
- placement decisions reduce estimated copy penalties
- pressure planning decides when those techniques should actually activate

This is still a model-driven benchmark, not a full production runtime measurement, but it shows the intended system shape:

> IDCP is not one optimization. It is a coordinated same-machine control fabric.

