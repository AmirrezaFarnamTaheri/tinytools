//! The status shape emitted by the standard `web_fetch` tool.
//!
//! This parser and its disposition live in `tinytools` so a host can use the
//! producer's exact result contract without a separate tool-implementation
//! crate.

/// The safe recovery meaning of a structured HTTP failure from a fetched site.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FetchedSiteRecovery {
    /// The public site refused this request (401 or 403).
    SiteRefused,
    /// The site reported a retryable rate limit or server failure (429 or 5xx).
    Transient,
    /// The fetch failed for another HTTP status; the host's ordinary repeat
    /// guard applies without a classified retry budget.
    Ordinary,
}

/// A structured HTTP failure produced by `web_fetch`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FetchedSiteFailure {
    /// The site's HTTP status code, in the 400–599 range.
    pub status: u16,
    /// The recovery meaning of this status for a fetched public site.
    pub recovery: FetchedSiteRecovery,
}

impl FetchedSiteFailure {
    /// Return only the producer-owned status line for safe text classification.
    ///
    /// The response excerpt after this line is untrusted page content and must
    /// not override the site's status with words such as `forbidden`,
    /// `timeout`, or `service unavailable`.
    #[must_use]
    pub fn headline(self, error: &str) -> &str {
        error.lines().next().unwrap_or(error)
    }
}

/// Parse the status code from the structured `web_fetch` error shape.
///
/// The shape is `HTTP <code> <reason> from <host>; <guidance>`, optionally
/// followed by an untrusted response excerpt. Only a leading, complete shape
/// with a non-empty single-token host and a 4xx/5xx code is accepted. Bare
/// status lines and later quoted occurrences are ignored.
#[must_use]
pub fn web_fetch_status(text: &str) -> Option<u16> {
    let rest = text.trim_start().strip_prefix("HTTP ")?;
    let code = rest
        .get(..3)
        .filter(|c| c.bytes().all(|b| b.is_ascii_digit()))?;
    let rest = rest[3..].strip_prefix(' ')?;
    let status: u16 = code.parse().ok().filter(|s| (400..=599).contains(s))?;
    let (_reason, after_from) = rest.lines().next()?.split_once(" from ")?;
    let (host, _guidance) = after_from.split_once(';')?;
    (!host.is_empty() && !host.contains(char::is_whitespace)).then_some(status)
}

/// Classify a structured fetched-site status, only for the `web_fetch` tool.
///
/// Explicit executor timeouts and producer-owned policy markers keep their
/// normal precedence. Other tools, bare status text, malformed headers, and
/// statuses quoted after the opening line return `None` and retain their
/// ordinary classifier behavior.
#[must_use]
pub fn classify_web_fetch_status(
    tool: &str,
    error: &str,
    timed_out: bool,
) -> Option<FetchedSiteFailure> {
    if tool != "web_fetch" || timed_out || opens_with_policy_marker(error) {
        return None;
    }
    let status = web_fetch_status(error)?;
    let recovery = match status {
        401 | 403 => FetchedSiteRecovery::SiteRefused,
        429 | 500..=599 => FetchedSiteRecovery::Transient,
        _ => FetchedSiteRecovery::Ordinary,
    };
    Some(FetchedSiteFailure { status, recovery })
}

fn opens_with_policy_marker(error: &str) -> bool {
    let text = error.trim_start().to_ascii_lowercase();
    [super::POLICY_BLOCKED_MARKER, super::POLICY_DENIED_MARKER]
        .iter()
        .any(|marker| {
            if text.starts_with(marker) {
                return true;
            }
            let Some(rest) = text.strip_prefix("error executing ") else {
                return false;
            };
            rest.split_once(": ").is_some_and(|(tool, body)| {
                !tool.contains(char::is_whitespace) && body.starts_with(marker)
            })
        })
}
