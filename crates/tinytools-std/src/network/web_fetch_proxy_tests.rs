//! web_fetch builds its client through the host's `prepare_client`, so the
//! host's proxy settings apply to it as they do to http_request and curl.

#![allow(clippy::unwrap_used)]

use super::*;
use crate::network::test_support::{DEFAULT_LIMITS, TestHtml, TestNetGate, pinned_to_local};

#[tokio::test]
async fn web_fetch_builds_its_client_through_prepare_client() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        let _ = socket.read(&mut [0u8; 8192]).await;
        let _ = socket
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok")
            .await;
    });
    let gate = TestNetGate::supervised();
    let tool = WebFetchTool::new(
        gate.clone(),
        vec![],
        None,
        None,
        DEFAULT_LIMITS,
        Arc::new(TestHtml),
    );
    let result = tool
        .fetch_validated(&pinned_to_local(&format!("http://{addr}/p")), 1_000, false)
        .await
        .unwrap();
    assert!(!result.is_error, "{}", result.output());
    assert_eq!(gate.prepared(), vec!["tool.web_fetch".to_string()]);
}
