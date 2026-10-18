# Structural code implementation

Spec: [Structural code and boilerplate](../specs/structural-code.md).

1. Add public typed contracts, explicit grammars and query fixtures under
   `crates/tinytools-code`; test parser/cache and error boundaries first.
2. Add lexical analysis and folded bodies, then revision-checked edits with
   regressions for ambiguity, imports, comments and UTF-8 boundaries.
3. Add typed manifests, twelve patterns and one-pass rendering; test injection,
   generated-node and expansion-budget failures.
4. Add the optional `crates/tinytools-std/src/code/` adapter with host gates,
   bounded I/O, previews and revision-guarded atomic publication tests. Pin
   outline levels 0/1 and the five-result search default with API regressions.
5. Update public/module docs and benchmark. Checkpoint focused validated source.
6. Run formatting, strict all-target/all-feature Clippy, build and tests; then
   default-feature tests, strict rustdoc and the unchanged per-file coverage gate.
7. Publish owner source, then move consumer pointers in separate commits.

## Completion

- [x] Pure implementation and regression tests
- [x] Native adapter and regression tests
- [x] Documentation and benchmark
- [x] Focused source checkpoint
- [ ] Four contract gates, default tests and strict rustdoc
- [ ] 90% per-file source coverage
- [ ] Owner publication and separate consumer pointer updates

## Focused validation checkpoint

On 2026-10-18, `cargo test -p tinytools-code --all-targets --locked --offline -j1`
passed all 41 integration tests and ran the fixed-fixture benchmark.
`cargo clippy -p tinytools-code --all-targets --locked --offline -j1 -- -D warnings`,
scoped formatting and `git diff --check` also passed. Regressions cover Rust
comma/comment/closing-indent preservation, inner attributes before imports,
control-character string escaping, Python future-import placement, and Go
member contexts.

The native adapter passed 32 focused tests with `cargo test -p tinytools-std
--features code --lib code:: --locked --offline`, strict all-target/all-feature
Clippy, scoped formatting and diff checks. Its regressions include exact JSON
byte budgets, late authority revocation, the last available action, stale
revisions, custom template containment, cancellation and atomic publication.
This is a 73-test source checkpoint. The complete workspace gates, default
tests, strict rustdoc, declared MSRV and per-file coverage remain pending.

## Compatibility follow-up

After the initial source checkpoint, align the deferred tool contracts with the
consumer roadmap: default `ast_outline` to level 1, omit `skeleton` at level 0,
and reject other levels before file reads. Default `chunk_search` to
`min(5, max_page_items)` while retaining explicit bounded page limits. Update
the schemas and native tool documentation alongside focused regressions before
running the complete owner gates on the resulting source.

The follow-up passed all 35 native tests on Rust 1.96.1 and workspace formatting
on Rust 1.99.0. It also replaces three assertions flagged by the initial
upstream Rust 1.99 Clippy run. The initial published source passed CI's strict
documentation, supply-chain checks and Rust 1.88 build; complete workspace
tests and per-file coverage still need to pass on the final source.
