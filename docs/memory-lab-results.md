# Memory Lab Results

`memory-lab` is the first discovery-oriented prototype in the repo. It does not tune the kernel. It estimates how much RAM a smarter representation could save by treating pages as:

- exact clones
- near-clones
- unique pages

and storing them as:

- one raw base page
- optional delta pages against that base

## Current synthetic results

| Workload | Pages | Exact dup pages | Near-dup pages | Raw MiB | Smart MiB | Estimated savings |
|---|---:|---:|---:|---:|---:|---:|
| `exact_dups` | 800 | 599 | 0 | 3.12 | 0.79 | 74.9% |
| `near_dups` | 512 | 0 | 353 | 2.00 | 0.66 | 66.8% |
| `mixed` | 856 | 299 | 171 | 3.34 | 1.53 | 54.3% |

## Interpretation

- exact duplicates are the easy win and still huge
- near-duplicate pages are common enough in template-like workloads to justify a deeper runtime
- base-plus-delta representation can plausibly save substantial memory even when pages are not byte-identical

## What this does and does not prove

This proves the *representation idea* is worth exploring.

It does not yet prove:

- kernel-level deployability
- low runtime overhead
- safety under real workloads
- that real applications exhibit the same page-family structure as these synthetic generators

## Next experiment

Run the same analyzer on:

- process snapshots with duplicated config/template heaps
- model-serving or embedding-worker style memory layouts
- browser / editor / plugin-style repeated object graphs
