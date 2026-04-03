# Roadmap

This roadmap is intended for handoff. It describes where the repo should go next, what "done" means for each phase, and how to prioritize work without losing the original thesis.

## North star

IDCP should become a same-machine runtime controller that can demonstrate measurable wins for modular local workloads by coordinating:

- communication
- memory representation
- placement
- pressure response

The project should not drift into:

- a generic distributed systems framework
- an AI agent product
- a benchmark-only repo
- a bag of unrelated low-level experiments

## Product goal

A future contributor should be able to say:

> IDCP can run a local modular workload, choose a better same-machine strategy, show the decision, persist the state, and prove the gain with repeatable reports.

## Phase 1: completed

Status:

- complete

Delivered:

- transport substrate
- memory discovery primitive
- four-engine architecture
- scenario system
- benchmark matrix
- runtime execution path
- controller loop
- daemon/status/report commands
- handoff-ready base documentation

Reference:

- [v1-v5.md](/home/zemul/idcp/docs/v1-v5.md)

## Phase 2: make the controller more real

Goal:

- replace more modeled behavior with measured behavior

Priority items:

- add repeated-run aggregation to `idcp-bench`
- record variance, median, p95, and p99 where meaningful
- turn runtime execution into multi-flow runs instead of a single fixed pipeline
- add a richer controller state model instead of plain text artifacts only
- persist structured output alongside markdown, likely JSON

Success criteria:

- fewer single-sample benchmark claims
- controller reports show repeated-run summaries
- `idcpd` state is easier to consume programmatically

## Phase 3: add realistic workloads

Goal:

- prove the system on workloads less synthetic than the built-in profiles

Priority items:

- create realistic worker-graph generators
- add duplicate-heavy plugin style workloads
- add embedding/serving style workloads with larger payloads
- add trace-driven scenario input instead of only hardcoded profile specs

Success criteria:

- at least one benchmark path driven by generated traces
- at least one runtime case that stresses the system beyond toy messaging

## Phase 4: strengthen the memory story

Goal:

- move from analytical page-family discovery toward runtime-managed representation choices

Priority items:

- implement a base+delta page-store prototype
- add cost accounting for delta creation and lookup
- compare raw vs family representation over repeated accesses
- test family invalidation behavior under mutation

Success criteria:

- memory engine output is no longer only "estimated savings"
- at least one runtime path uses a family-aware representation decision

## Phase 5: strengthen the placement and pressure story

Goal:

- make placement and pressure decisions more than static heuristics

Priority items:

- track runtime contention and queue depth
- make pressure decisions depend on repeated observations instead of one-shot evaluation
- add pressure-oriented scenarios with explicit oversubscription
- distinguish throughput-optimized vs latency-optimized control modes

Success criteria:

- controller actions differ across ticks for real reasons
- pressure behavior is observable in reports, not only inferred

## Phase 6: operator and developer experience

Goal:

- make the repo much easier for another developer to extend

Priority items:

- add structured config file support
- add a machine-readable report format
- add a proper benchmark command matrix script
- add a release checklist
- add issue templates and contribution workflow docs if the repo becomes more public

Success criteria:

- a new contributor can add a profile or engine rule without reverse-engineering the entire repo

## Technical debt list

These are the most important debts to pay down:

- transport benchmarks are noisy across single runs
- aggregate score is simplistic and should not be overinterpreted
- the daemon is controller-shaped but not a supervised background service
- memory/placement/pressure are still mostly planner logic
- state output is text-first rather than structured-first

## What not to do

Avoid these mistakes:

- do not claim universal system-wide wins
- do not overfit to one lucky benchmark run
- do not add speculative GPU/PCIe/NUMA claims without evidence
- do not dilute the project with unrelated app features
- do not hide modeled behavior behind marketing language

## Recommended order of work

If a new developer starts tomorrow, the order should be:

1. repeated-run benchmark aggregation
2. structured daemon state
3. richer runtime worker graphs
4. trace-driven profiles
5. family-aware runtime representation experiments
6. better pressure adaptation

That path improves credibility faster than chasing more ambitious but ungrounded features.
