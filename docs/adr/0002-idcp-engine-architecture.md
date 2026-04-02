# ADR 0002: IDCP Engine Architecture

## Status

Accepted

## Decision

IDCP is now organized around four cooperating engines:

- `idcp-flow`: intra-device communication planning
- `idcp-memory`: page-family discovery and memory representation analysis
- `idcp-placement`: placement and locality planning
- `idcp-pressure`: pressure response and memory-relief planning

Integration points:

- `idcp-bench`: cross-engine benchmark
- `idcpd`: daemon-facing planner prototype

## Why

The project target is no longer "faster local messaging" alone. The larger bet is that the machine becomes more capable when flow, memory, placement, and pressure are coordinated instead of optimized in isolation.

