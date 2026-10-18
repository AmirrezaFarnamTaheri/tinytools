# Structural code and boilerplate

Status: Accepted. Owner: TinyTools.

## Problem and boundary

Agents need bounded structural inspection and revision-checked edits without
shipping whole files or rewriting unrelated declarations. A pure engine owns
syntax and rendering; optional native tools own filesystem mechanics behind the
host's `FsGate`. The dependency-light `tinytools` vocabulary remains unchanged.

## Behavior

`tinytools-code` accepts explicit Rust, TypeScript, TSX, JavaScript, Python, Go,
JSON, and TOML languages. It returns lexical symbols/references, syntax
diagnostics and folded source. A bounded cache reuses unchanged trees and applies
UTF-8-safe incremental edits. Cache byte accounting measures retained source,
not Tree-sitter's native allocations. Results disclose record truncation.

Patches require an exact source revision and an unambiguous symbol selector.
They replace bodies or declarations, insert imports, append members, or wrap one
node. Original and final source must parse. Source outside the selected edit is
preserved. A truncated outline only permits revision-addressed IDs. These are
syntax guarantees, not typechecking or semantic reference resolution.

A typed chunk catalog offers twelve starter patterns. Search and inspect return
metadata. Rendering expands typed parameters and standalone comment slots in
one pass; inserted text is never interpreted as another template directive.
Manifests may constrain generated node kinds. Custom manifests use portable
relative template paths. Dependency declarations are informational and never
install anything. Template syntax, expansions and output are bounded.

`tinytools-std`'s optional `code` feature exports deferred `ast_outline`,
`ast_symbol`, `ast_patch`, `chunk_search`, `chunk_inspect`, and `chunk_apply` tools.
Every filesystem read/write passes the host gate, including custom manifests and
templates. Tools bound source, manifest, template and response bytes and expose
paging for inspection. Receipts are JSON. Mutations require caller-provided
expected revisions. Only the literal JSON boolean `dry_run: true` grants a
read-only preview; static minimum permission is ReadOnly. Preview has no external
effect. Responses are budgeted before publication; files are staged, revision
guards rechecked, and atomically published with no-clobber creation or
permission-preserving replacement. Cancellation cannot expose a partial file.

## Acceptance

All grammars, cache paths, ambiguity, revision failures, import prologues,
container edits, slot/type restrictions, budget failures and native filesystem
races have deterministic regressions. The four workspace gates, default feature
tests, strict rustdoc and unchanged 90% per-file coverage gate must pass. Engine
benchmarks report source/folded bytes and timings without speed or token claims.
Owner source is committed before consumers move gitlinks in separate commits.

No blocking open questions. See the [implementation plan](../plans/structural-code.md).
