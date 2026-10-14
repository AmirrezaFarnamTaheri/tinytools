use super::*;

fn candidates() -> Vec<RankCandidate> {
    vec![
        RankCandidate::new(
            "calendar_invite",
            "calendar_invite Send a calendar invite to attendees. title start attendees",
        )
        .with_family("calendar"),
        RankCandidate::new(
            "pdf_read",
            "pdf_read Read the text of a PDF document. path pages",
        )
        .with_family("documents"),
        RankCandidate::new(
            "stock_quote",
            "stock_quote Fetch the latest price for a ticker symbol. symbol",
        )
        .with_family("finance"),
    ]
}

#[test]
fn tokenize_splits_identifiers_and_camel_case() {
    assert_eq!(
        tokenize("memory_hybrid_search readWorkflowResource v2"),
        vec![
            "memory", "hybrid", "search", "read", "workflow", "resource", "v2"
        ]
    );
}

#[test]
fn index_ranks_by_description_and_breaks_ties_by_key() {
    let index = Bm25Index::build([
        ("b_tool", "send a message"),
        ("a_tool", "send a message"),
        ("c_tool", "read a file"),
    ]);
    assert_eq!(index.len(), 3);
    assert_eq!(index.search("send message", 5), vec![1, 0]);
}

#[test]
fn index_on_one_document_corpus_still_finds_it() {
    let index = Bm25Index::build([("only", "fetch the latest price for a ticker symbol")]);
    assert_eq!(index.search("ticker price", 5), vec![0]);
    assert_eq!(index.search("calendar", 5).len(), 0);
}

#[test]
fn index_returns_nothing_for_stopword_only_queries() {
    let index = Bm25Index::build([("a", "send a message"), ("b", "read a file")]);
    assert_eq!(index.search("the a of", 5).len(), 0);
    assert_eq!(Bm25Index::default().search("anything", 5).len(), 0);
    assert!(Bm25Index::default().is_empty());
}

#[test]
fn bm25_ranker_returns_positive_hits_only_best_first() {
    let hits = Bm25Ranker::rank_sync(&candidates(), "read the text of a pdf", 5);
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].key, "pdf_read");
    assert!(hits[0].score > 0.0);
    assert!(hits[0].confidence.is_none());
}

#[test]
fn bm25_ranker_matches_on_family() {
    let hits = Bm25Ranker::rank_sync(&candidates(), "finance", 5);
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].key, "stock_quote");
}

#[test]
fn bm25_ranker_honours_limit() {
    let hits = Bm25Ranker::rank_sync(&candidates(), "send read fetch", 2);
    assert_eq!(hits.len(), 2);
}

#[tokio::test]
async fn bm25_ranker_rejects_empty_intent() {
    let err = Bm25Ranker
        .rank("   ", &RankContext::empty(), &candidates(), 3)
        .await
        .err()
        .map(|e| e.to_string());
    assert_eq!(
        err.as_deref(),
        Some("invalid ranking input: intent is empty")
    );
}

#[tokio::test]
async fn ranker_is_object_safe_behind_an_arc() {
    let ranker: std::sync::Arc<dyn ToolRanker> = std::sync::Arc::new(Bm25Ranker);
    assert_eq!(ranker.kind(), "bm25");
    let hits = ranker
        .rank("calendar invite", &RankContext::empty(), &candidates(), 3)
        .await
        .unwrap_or_default();
    assert_eq!(
        hits.first().map(|hit| hit.key.as_str()),
        Some("calendar_invite")
    );
}

#[test]
fn rank_error_displays_without_credentials() {
    let err = RankError::Backend {
        reason: "status 401".to_owned(),
    };
    assert_eq!(err.to_string(), "ranker backend failed: status 401");
    assert_eq!(RankError::Timeout.to_string(), "ranker timed out");
    assert_eq!(
        RankError::backend("offline").to_string(),
        "ranker backend failed: offline"
    );
    assert_eq!(
        RankError::invalid_input("bad key").to_string(),
        "invalid ranking input: bad key"
    );
}

#[test]
fn empty_context_has_no_chat() {
    assert_eq!(RankContext::empty().chat, None);
    assert_eq!(RankContext::empty().recent_turns, Vec::<String>::new());
}

#[test]
fn context_carries_the_chat_it_was_built_for() {
    let chat = RankChat::new("thread-1", "client-1").with_request_id("req-9");
    let context = RankContext::empty().with_chat(chat.clone());
    assert_eq!(context.chat, Some(chat.clone()));
    assert_eq!(context.clone().chat, Some(chat));
    let carried = context.chat.unwrap_or_else(|| RankChat::new("", ""));
    assert_eq!(carried.thread_id, "thread-1");
    assert_eq!(carried.client_id, "client-1");
    assert_eq!(carried.request_id.as_deref(), Some("req-9"));
}

/// A ranker is handed the context's chat unchanged, so it can route a
/// disclosure to it.
#[tokio::test]
async fn ranker_receives_the_chat_in_its_context() {
    struct Echo(std::sync::Mutex<Option<RankChat>>);
    #[async_trait::async_trait]
    impl ToolRanker for Echo {
        fn kind(&self) -> &'static str {
            "echo"
        }
        async fn rank(
            &self,
            _intent: &str,
            context: &RankContext,
            _candidates: &[RankCandidate],
            _limit: usize,
        ) -> Result<Vec<RankHit>, RankError> {
            if let Ok(mut seen) = self.0.lock() {
                *seen = context.chat.clone();
            }
            Ok(Vec::new())
        }
    }
    let echo = Echo(std::sync::Mutex::new(None));
    let context = RankContext::empty().with_chat(RankChat::new("t", "c"));
    let ranked = echo.rank("anything", &context, &candidates(), 1).await;
    assert!(ranked.is_ok());
    let seen = echo.0.lock().map(|chat| chat.clone()).unwrap_or_default();
    assert_eq!(seen, Some(RankChat::new("t", "c")));
}
