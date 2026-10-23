//! The OS sink replaces a file atomically: through a temp file and a rename.

#![allow(clippy::unwrap_used)]

use super::*;

// A symlink swapped in after a tool's symlink check must not redirect the
// write: the rename replaces the link itself, and its target is untouched.
#[cfg(unix)]
#[tokio::test]
async fn a_write_replaces_a_symlink_instead_of_following_it() {
    let dir = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    let secret = outside.path().join("secret.txt");
    std::fs::write(&secret, "secret").unwrap();
    let link = dir.path().join("a.txt");
    std::os::unix::fs::symlink(&secret, &link).unwrap();

    OsFileSink.write(&link, b"new").await.unwrap();

    assert_eq!(std::fs::read_to_string(&secret).unwrap(), "secret");
    assert!(!std::fs::symlink_metadata(&link).unwrap().is_symlink());
    assert_eq!(std::fs::read_to_string(&link).unwrap(), "new");
}

// The rename installs a new inode, so the target's mode bits are copied to
// it first: an executable script stays executable after an edit.
#[cfg(unix)]
#[tokio::test]
async fn a_write_keeps_the_targets_permissions() {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().unwrap();
    let script = dir.path().join("run.sh");
    std::fs::write(&script, "old").unwrap();
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o750)).unwrap();

    OsFileSink.write(&script, b"new").await.unwrap();

    let mode = std::fs::metadata(&script).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o750);
    assert_eq!(std::fs::read_to_string(&script).unwrap(), "new");
    let leftovers = std::fs::read_dir(dir.path()).unwrap().count();
    assert_eq!(leftovers, 1, "a temp file was left behind");
}

#[tokio::test]
async fn failed_writes_report_why_and_leave_no_temp_file() {
    let no_parent = atomic_write(Path::new("/"), b"x").await.unwrap_err();
    assert_eq!(no_parent.kind(), std::io::ErrorKind::InvalidInput);

    let dir = tempfile::tempdir().unwrap();
    let missing_dir = atomic_write(&dir.path().join("nope").join("a.txt"), b"x")
        .await
        .unwrap_err();
    assert!(missing_dir.to_string().contains("stage temp file"));

    // Renaming a file over a directory fails after the temp file was staged.
    let target = dir.path().join("a.txt");
    std::fs::create_dir(&target).unwrap();
    assert!(atomic_write(&target, b"x").await.is_err());
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
}
