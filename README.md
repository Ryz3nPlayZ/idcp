# IDCP

IDCP is a research repo for a locality-aware communication plane.

The working thesis is simple: local software often pays unnecessary transport overhead by treating same-process and same-host communication like remote RPC. This repo exists to measure that waste, prototype cheaper transports, and determine whether a reusable runtime is worth building.

## Initial scope

- benchmark local communication paths honestly
- compare transport shape and batching strategies
- prototype a stable local messaging API
- record decisions with hard numbers

## Non-goals for the first phase

- distributed systems at cluster scale
- replacing every RPC framework
- AI agent features before the transport story is proven
- speculative product complexity without measurements

## First benchmark bar

A candidate path only counts as a serious win if it demonstrates:

- at least `5x` lower median latency than local loopback TCP
- at least `2x` lower median latency than Unix sockets
- equivalent correctness on the same workload

