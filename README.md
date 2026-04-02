# IDCP

IDCP is a research repo for an intra-device communication and control fabric.

The thesis is broader now: commodity computers waste capability because communication, memory, placement, and pressure response are handled as separate generic subsystems. IDCP explores whether coordinating them as one same-machine fabric can make the same hardware do materially more with less.

## Initial scope

- benchmark local communication paths honestly
- compare transport shape and batching strategies
- explore discovery-grade memory representations
- coordinate flow, memory, placement, and pressure under one model
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

## Current shape

- `fabric-core`: low-level transport substrate
- `idcp-flow`: flow planning across same-machine localities
- `idcp-memory`: page-family discovery and memory representation analysis
- `idcp-placement`: locality and placement decisions
- `idcp-pressure`: pressure response planning
- `idcp-bench`: cross-engine benchmark
- `idcpd`: daemon-facing planner prototype
