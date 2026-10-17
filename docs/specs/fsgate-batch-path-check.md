# FsGate batch path check

## Status and owner

Implemented. Owner: tinytools-std (`filesystem`).

## Problem

`glob` asked `FsGate::is_path_string_allowed` once per hit. A host whose policy
lives behind a bus (a module call per question) pays one round trip per hit,
which is thousands for a large tree.

## Goals and non-goals

- Let a gate answer many path strings in one call.
- `glob` asks once per search.
- Non-goal: changing single-path callers (`apply_patch` keeps its per-edit
  check, which reports a per-edit error).

## Proposed behavior

`FsGate` gains a provided method:

```rust
fn paths_string_allowed(&self, paths: &[String]) -> Vec<bool>
```

The default maps `is_path_string_allowed` over the slice, so every existing
gate behaves identically. A host that can answer a batch more cheaply
overrides it. `glob` collects every pattern match first, calls the method
exactly once with the exact strings it would return, and keeps only the hits
answered `true`.

## Invariants and constraints

- Answers are in request order and the same length as `paths`.
- A caller treats an answer of the wrong length as every path refused (fail
  closed) and logs it.
- Results of a search are unchanged for gates that keep the default.

## Acceptance criteria

- The default method equals the per-path mapping.
- A many-hit `glob` issues exactly one batch call.
- A wrong-length answer yields zero matches.

## Open questions

None.
