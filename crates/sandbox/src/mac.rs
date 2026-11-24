//! macOS: NSURL security-scoped bookmarks.

use objc2::runtime::Bool;
use objc2_foundation::{NSData, NSString, NSURL, NSURLBookmarkCreationOptions, NSURLBookmarkResolutionOptions};

/// Most bytes a bookmark may have: they come from a settings file.
const MAX_BOOKMARK: usize = 64 * 1024;

pub fn bookmark(path: &str) -> Option<Vec<u8>> {
    if path.is_empty() {
        return None;
    }
    let url = NSURL::fileURLWithPath(&NSString::from_str(path));
    let data = url
        .bookmarkDataWithOptions_includingResourceValuesForKeys_relativeToURL_error(NSURLBookmarkCreationOptions::WithSecurityScope, None, None)
        .ok()?;
    let bytes = data.to_vec();
    (!bytes.is_empty() && bytes.len() <= MAX_BOOKMARK).then_some(bytes)
}

pub fn access(bookmark: &[u8]) -> Option<(String, bool)> {
    if bookmark.is_empty() || bookmark.len() > MAX_BOOKMARK {
        return None;
    }
    let data = NSData::with_bytes(bookmark);
    let mut stale = Bool::NO;
    // SAFETY: `stale` is a valid, writable Bool for the duration of the call, as the method asks.
    let url = unsafe {
        NSURL::URLByResolvingBookmarkData_options_relativeToURL_bookmarkDataIsStale_error(
            &data,
            NSURLBookmarkResolutionOptions::WithSecurityScope,
            None,
            &mut stale,
        )
    }
    .ok()?;
    // SAFETY: an NSURL resolved from a security-scoped bookmark; access is kept for the rest of
    // the run on purpose (the document may be saved again at any time), so there is no matching
    // stop call.
    let started = unsafe { url.startAccessingSecurityScopedResource() };
    if !started {
        return None;
    }
    let path = url.path()?.to_string();
    Some((path, stale.as_bool()))
}
