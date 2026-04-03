# Contributing

This repo is still small enough that contribution quality matters more than contribution volume.

## Principles

- keep claims narrower than the evidence
- prefer measured behavior over modeled behavior when possible
- keep the architecture coherent
- avoid scope drift

## Before changing code

Read:

- [README.md](/home/zemul/idcp/README.md)
- [architecture.md](/home/zemul/idcp/docs/architecture.md)
- [roadmap.md](/home/zemul/idcp/docs/roadmap.md)
- [handoff.md](/home/zemul/idcp/docs/handoff.md)

## Good contribution categories

- improve benchmark rigor
- add a realistic workload profile
- improve controller observability
- strengthen report generation
- add structured state output
- replace modeled behavior with measured behavior

## Risky contribution categories

- broad rewrites across all crates
- introducing distributed systems scope
- adding speculative hardware claims
- changing the project thesis without updating docs and evidence

## Validation expectations

Before opening or handing off changes, run:

```bash
cargo fmt --check
cargo test
cargo run -p idcp-bench --release
```

If you touch the controller surface, also run:

```bash
cargo run -p idcpd --release -- daemon embedding-farm 3 0 /tmp/idcpd-contrib
cargo run -p idcpd --release -- status /tmp/idcpd-contrib
```

## Documentation expectations

Any contribution that changes product surface, measurements, or architecture should update the relevant docs:

- `README.md`
- `docs/idcp-bench-results.md`
- `docs/architecture.md`
- `docs/roadmap.md`

## Benchmark honesty

Do not:

- present one lucky run as stable truth
- hide negative or noisy results
- blur measured numbers with estimates

Do:

- note when results are noisy
- prefer repeated runs when possible
- explain tradeoffs clearly

## Style expectations

- keep changes focused
- preserve crate boundaries unless there is a strong reason not to
- add tests for new logic in `idcp-system` when behavior changes
- keep CLI behavior discoverable and documented

## Definition of a good patch

A good patch should do at least one of these:

- make the controller more real
- make the measurements more trustworthy
- make the system easier to understand
- make handoff easier for the next developer
