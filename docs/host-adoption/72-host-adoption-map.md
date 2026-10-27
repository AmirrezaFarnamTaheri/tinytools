# OpenHuman host adoption map: fetched-site status

OpenHuman's TinyAgents middleware can replace its duplicate status parser and
fetched-site disposition policy with the `tinytools::status` API:

- Call `classify_for_tool(tool, error, timed_out)`. For ordinary results it
  returns `ToolFailureClassification::Classified`, preserving the existing
  `classify(error, timed_out)` behavior. For a valid `web_fetch` status it
  returns `FetchedSite(FetchedSiteFailure)`.
- Map `FetchedSiteFailure.recovery` to the host's existing recovery budgets:
  `SiteRefused` → `site_refused` with budget 2, `Transient` → `transient` with
  budget 2, and `Ordinary` → no classified budget so the exact-repeat guard
  remains responsible. The `status` field is available for diagnostics.
- For `Ordinary`, feed `FetchedSiteFailure::headline(error)` to any separate
  text heuristic. It returns only the first producer-owned status line, so a
  response excerpt cannot turn a 404 into an auth, timeout, or outage verdict.
- Keep the general `classify` API for call sites without tool identity. Bare
  `403 Forbidden` remains `BadCredentials`, and a structured 403 from a
  connector still follows that connector's credential policy because the
  tool-aware result recognizes only the `web_fetch` tool.
- Keep explicit executor timeouts and producer-owned policy markers ahead of
  fetched-site classification. The typed status result does not replace the
  host's separate host-per-URL recovery scope; that remains a small URL
  projection (`web_fetch` + `url` argument → parsed URL host).

The status parser is `tinytools::status::web_fetch_status`, which the producer
and host share as one accepted shape.
