# ADR 0001: Start As A Measurement-First Workspace

## Status

Accepted

## Decision

Start IDCP as a Rust workspace with:

- `fabric-core` for transport abstractions
- `fabric-bench` for reproducible benchmarks
- `docs/` for thesis, results, and architecture decisions

## Why

The project needs quantitative discipline from the start. A workspace split keeps benchmark code and reusable library code separate without forcing premature complexity.

