# Architecture

This document explains the current IDCP architecture, the role of each crate, the runtime data flow, and the boundaries of the current prototype.

## System intent

IDCP is a same-machine control fabric.

It tries to improve modular local software by coordinating four concerns together:

- flow
- memory
- placement
- pressure

The central claim is that local systems are often wasteful because those concerns are handled independently by generic defaults.

## Current architecture

The workspace is split into three layers:

1. substrate
2. engines
3. product surface

## 1. Substrate

### `fabric-core`

Purpose:

- low-level local transport abstraction

Current transports:

- `SyncChannel`
- `UnixStream`
- `SpscRing`
- `SharedMemoryEvent`

Main type:

- `LocalEndpoint`

Responsibilities:

- create endpoint pairs
- send and receive `u64` values across different local transports
- provide the low-level basis for transport benchmarks and runtime demos

Important limitation:

- payloads are still tiny and simplified
- transport semantics are prototype-grade, not production-ready

### `fabric-bench`

Purpose:

- original isolated transport benchmark harness

Use:

- compare naive local communication paths
- validate that local transport choice matters before higher-level orchestration is added

## 2. Engines

### `idcp-flow`

Purpose:

- choose the local communication plan from locality, payload class, and latency sensitivity

Input:

- `FlowHint`

Output:

- `FlowPlan`

Current behavior:

- prefers `SpscRing` for in-thread and in-process hot paths
- prefers `SharedMemoryEvent` for cross-process local hot paths
- uses batching for larger payloads
- falls back toward `UnixStream` for less favorable shapes

This is currently rule-based, not learned.

### `idcp-memory`

Purpose:

- analyze memory as page families instead of unrelated raw pages

Concepts:

- exact duplicates
- near duplicates
- unique pages
- base + delta representation

Input:

- synthetic `PageWorkload`

Output:

- `MemoryReport`

Current role:

- estimate the potential reduction in raw bytes if page-family representations are used

Important limitation:

- it does not inspect or transform live process memory
- it is an analytical engine, not an allocator or pager

### `idcp-placement`

Purpose:

- estimate where execution should live relative to hot/shared data

Concepts:

- `L1Hot`
- `CoreLocal`
- `SharedLocal`
- `CrossProcess`

Output:

- `PlacementDecision`

Current role:

- estimate copy penalty and affinity score

Important limitation:

- it does not actually pin threads, manage NUMA, or enforce placement at the OS level

### `idcp-pressure`

Purpose:

- classify memory pressure and decide which high-level responses should activate

Pressure levels:

- `Healthy`
- `Elevated`
- `Critical`

Output:

- `PressurePlan`

Current role:

- decide whether to enable compression-like behavior, page families, and rebalance work

Important limitation:

- this is a planner, not a live kernel-integrated reclaim controller

### `idcp-system`

Purpose:

- integrate all engine outputs into one scenario evaluation surface

This is the current orchestration core.

It owns:

- built-in profiles
- scenario specs
- naive vs IDCP evaluation
- measured flow-path benchmarking
- live runtime execution
- controller loop
- report generation

Important types:

- `ScenarioProfile`
- `ScenarioSpec`
- `ScenarioEvaluation`
- `RuntimeResult`
- `ControllerConfig`
- `ControllerSnapshot`
- `ControllerSummary`

This is the crate that currently makes IDCP feel like one system rather than multiple experiments.

## 3. Product surface

### `idcp-bench`

Purpose:

- print the cross-engine benchmark matrix across built-in profiles

This is the fastest way to see the current high-level signal.

### `idcpd`

Purpose:

- expose the product-facing controller surface

Current commands:

- `profiles`
- `plan`
- `measure`
- `simulate`
- `run`
- `bench`
- `daemon`
- `status`
- `report`

Current role:

- interactive CLI
- daemon-style controller runner
- report generator

Important limitation:

- despite the name, this is not yet a long-lived background service with process supervision or a control API

## Runtime flow

For a controller run:

1. choose a built-in profile
2. evaluate naive and IDCP plans
3. measure the chosen flow path
4. execute a live runtime pipeline for naive and IDCP
5. compute improvements
6. choose a control action
7. persist status and report artifacts

## Persisted daemon artifacts

The daemon writes:

- `latest.txt`
- `history.log`
- `report.md`

These are the current handoff surface for operators and developers.

## What is real vs modeled

### Real today

- local transport implementations
- live transport measurement
- live multi-stage runtime execution
- controller snapshots
- persisted daemon artifacts

### Modeled today

- memory reduction impact
- placement copy-penalty effect
- pressure actions
- aggregate score function

This distinction matters. The repo is product-shaped, but not yet a production systems controller.

## Current boundaries

IDCP is not yet:

- a kernel module
- a memory manager
- a scheduler replacement
- a system-wide transparent optimization layer
- a distributed runtime

It is currently:

- a well-structured same-machine controller prototype

## Recommended mental model

Think of IDCP as:

- a planner
- a local runtime evaluator
- a control-loop experiment

not as:

- a general-purpose operating system component

That distinction keeps expectations sane for future contributors.
