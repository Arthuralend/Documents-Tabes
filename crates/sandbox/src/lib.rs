//! Write: security-scoped bookmarks for the Mac App Store's sandbox.
//!
//! A sandboxed app may read and write only the files the user chose (an Open or Save panel, a
//! drop, a Finder double-click), and only while it runs. A bookmark made while the app has that
//! access lets it — and another window of the same app, which is another process — open the file
//! again later: recent documents after a restart, a document opened in a new window. Outside the
//! sandbox (the direct download) the bookmarks are harmless, and elsewhere than macOS there are
//! none.

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

#[cfg(target_os = "macos")]
mod mac;

/// A bookmark for `path` (the app must have access to it now), or `None` where there are no
/// bookmarks or macOS refused one.
pub fn bookmark(path: &str) -> Option<Vec<u8>> {
    #[cfg(target_os = "macos")]
    {
        mac::bookmark(path)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = path;
        None
    }
}

/// Resolve `bookmark` and start accessing its file for the rest of the run. Returns the file's
/// path now and whether the bookmark is stale (the file moved: make a new one).
pub fn access(bookmark: &[u8]) -> Option<(String, bool)> {
    #[cfg(target_os = "macos")]
    {
        mac::access(bookmark)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = bookmark;
        None
    }
}

#[cfg(test)]
mod tests {
    #[cfg(not(target_os = "macos"))]
    #[test]
    fn elsewhere_there_are_no_bookmarks() {
        assert!(super::bookmark("/tmp/a.docx").is_none());
        assert!(super::access(b"junk").is_none());
    }
}
