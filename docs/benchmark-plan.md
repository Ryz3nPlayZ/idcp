# Benchmark Plan

## Goal

Prove or disprove that a locality-aware transport/runtime can materially outperform naive local socket-heavy communication.

## Baselines

- loopback TCP
- Unix sockets
- in-process sync channel

## Candidate methods

- lock-free in-process ring
- shared memory with lightweight signaling
- batched socket communication

## Metrics

- median latency per round trip
- best latency per round trip
- round trips per second
- payload size
- batch size
- machine metadata

Future phases:

- CPU utilization
- context switches
- tail latency
- fairness under contention

