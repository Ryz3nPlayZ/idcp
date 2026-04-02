# Memory Discovery Direction

The lower-level bet is no longer "faster local messaging." It is:

> computers waste RAM because identical, near-identical, and compressible pages are treated as unrelated raw memory.

The first discovery primitive under test is **page families**:

- exact clones
- near-clones
- unique pages

If page families can be detected cheaply, a runtime can represent memory as:

- one base page
- zero or more deltas
- optional compressed cold form

That is a stronger claim than normal KSM-style same-page merging because it targets **near-duplicate** memory, not just byte-identical pages.

## Why this matters

- worker processes often load the same templates, configs, embeddings, and model-adjacent structures
- many pages differ by small edits, timestamps, ids, or metadata
- storing every such page as a full raw 4 KiB page is often wasteful

## First prototype

`memory-lab` estimates potential gains from:

- exact-page deduplication
- near-duplicate clustering
- delta encoding against a chosen base page

This is not yet a runtime. It is a discovery tool for determining whether a smarter memory representation is plausible.
