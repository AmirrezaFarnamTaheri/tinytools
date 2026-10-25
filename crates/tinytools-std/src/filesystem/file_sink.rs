//! The filesystem write the tools go through.
//!
//! Every tool that reports "the write failed" has a branch that only runs when
//! the OS refuses, and that branch is the one worth asserting: it decides
//! whether a failed edit is reported honestly or fabricated as a success.
//!
//! Provoking a real refusal from a test is where this gets awkward. A
//! mode-bit denial is a discretionary check, and a root test runner carries
//! `CAP_DAC_OVERRIDE` and writes through it — so a fixture built on
//! permissions succeeds under CI, and the case either fails on a premise that
//! never held or is skipped and asserts nothing at all. Both read as coverage.
//!
//! So the write is a seam instead. Production installs the OS; a test installs
//! one that refuses, and the reporting branch becomes ordinary to assert on
//! any platform and as any user.

use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use async_trait::async_trait;
use tokio::io::AsyncWriteExt;

/// Where a tool's file writes land.
#[async_trait]
pub(super) trait FileSink: std::fmt::Debug + Send + Sync {
    async fn write(&self, path: &Path, contents: &[u8]) -> std::io::Result<()>;
}

/// The real filesystem.
#[derive(Debug)]
pub(super) struct OsFileSink;

#[async_trait]
impl FileSink for OsFileSink {
    async fn write(&self, path: &Path, contents: &[u8]) -> std::io::Result<()> {
        atomic_write(path, contents).await
    }
}

/// Monotonic counter for temp-file names within a process.
static TEMP_FILE_SEQ: AtomicU64 = AtomicU64::new(0);

/// Atomically replace `path`'s contents with `contents`.
///
/// Writes a fresh (`create_new`) sibling temp file in the same directory, so
/// the rename stays on one filesystem, gives it the target's permissions, and
/// `rename`s it over the target. A crash mid-write leaves the old file or the
/// complete new one, never a truncation, and a symlink at `path` is replaced
/// rather than followed. The rename installs a new inode: other hard links
/// keep the old contents, and the file is owned by the writing user.
pub(super) async fn atomic_write(path: &Path, contents: &[u8]) -> std::io::Result<()> {
    let dir = path.parent().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("{} has no parent directory", path.display()),
        )
    })?;
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    let seq = TEMP_FILE_SEQ.fetch_add(1, Ordering::Relaxed);
    let tmp = dir.join(format!(".{name}.{}.{seq}.tmp", std::process::id()));
    let mut file = tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&tmp)
        .await
        .map_err(|e| std::io::Error::new(e.kind(), format!("stage temp file: {e}")))?;
    let mut result = stage(&mut file, path, contents).await;
    drop(file);
    if result.is_ok() {
        result = tokio::fs::rename(&tmp, path).await;
    }
    if result.is_err() {
        let _ = tokio::fs::remove_file(&tmp).await;
    }
    result
}

/// Fill the temp file, copy the target's permissions onto it (or `+x` is
/// lost on the rename), and flush it to disk before the rename.
async fn stage(file: &mut tokio::fs::File, target: &Path, contents: &[u8]) -> std::io::Result<()> {
    file.write_all(contents).await?;
    if let Ok(meta) = tokio::fs::metadata(target).await {
        file.set_permissions(meta.permissions()).await?;
    }
    file.sync_all().await
}

/// The sink a tool uses unless a caller installs another one.
pub(super) fn os_sink() -> Arc<dyn FileSink> {
    Arc::new(OsFileSink)
}

/// A sink that refuses every write with `kind`, for asserting the branch a
/// tool takes when the filesystem says no.
#[cfg(test)]
#[derive(Debug)]
pub(super) struct RefusingSink(pub std::io::ErrorKind);

#[cfg(test)]
#[async_trait]
impl FileSink for RefusingSink {
    async fn write(&self, path: &Path, _contents: &[u8]) -> std::io::Result<()> {
        Err(std::io::Error::new(
            self.0,
            format!("refused by the test sink: {}", path.display()),
        ))
    }
}

#[cfg(test)]
#[path = "file_sink_tests.rs"]
mod tests;
