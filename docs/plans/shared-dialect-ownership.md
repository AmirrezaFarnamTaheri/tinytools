# Shared Dialect Ownership

Accepted behavior: [Agent Tool Protocols](../specs/agent-tool-protocols.md).
Consumer migration: https://github.com/tinyhumansai/openhuman/issues/7400.

1. Capture the existing host description-catalogue output before moving it,
   preserving UTF-8, embedded newlines and trailing whitespace.
2. Move unexecuted response detection and stripping to tinytools-agent/dialect
   with the existing regressions and explicit marker-free compatibility.
3. Add ToolCallFormat dispatcher-name selection and code-style access; test
   explicit modes and both automatic capability outcomes before implementation.
4. Expose the stable description-only XML protocol and pin its exact output.
5. In TinyAgents, render description catalogues using that protocol, compare the
   entire captured output, and bridge typed dispatch policy to shared selection.
6. Land the owning TinyTools change before the dependent TinyAgents gitlink.
   Consumers then collapse duplicate format/dispatcher vocabulary, preserve
   their explicit JSON default, and use the catalogue renderer.
7. Verify consumer dumped-prompt bytes and multi-turn capture cache keys after
   consuming available upstream commits. Execution policy remains with the host.

TinyTools validation: formatting, all-target/all-feature Clippy and build,
all-feature tests, and the CI per-file coverage gate. TinyAgents validation:
catalogue golden, selection tests, full workspace checks and its CI matrix.
