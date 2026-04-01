# Thesis

Modern software frequently routes local communication through abstractions designed for remote systems. That simplicity is convenient, but it introduces avoidable cost:

- too many kernel crossings
- too many copies
- too much serialization
- too little batching
- too little awareness of locality

IDCP explores whether a local-first communication plane can do better while preserving a clean API.

## Research questions

1. How much overhead do naive local TCP and socket-heavy designs add?
2. Which mechanisms matter most: batching, shared memory, wakeup strategy, or scheduling?
3. Is the right output a library, a daemon, or both?
4. Is there a concrete product use case where this matters enough to adopt?

## Candidate product wedges

- AI tool bus
- plugin runtime
- terminal backend
- local event pipeline

