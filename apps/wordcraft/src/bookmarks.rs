//! Write in the Mac App Store's sandbox: the documents the user chose stay reachable.
//!
//! Each document Write opens or saves gets a security-scoped bookmark (`lexicon-sandbox`), kept in
//! `bookmarks.json` in the settings folder. At start every window resolves them and keeps access
//! for its run, so recent documents open after a restart and a document handed to a new window
//! (another process) can be read there. Outside the sandbox and off macOS this does nothing.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Mutex;

use wordcraft_engine::cmd::insert::{base64_decode, base64_encode};

/// Most documents remembered; the oldest go first.
const MAX_BOOKMARKS: usize = 64;

/// Path → bookmark (base64), and the order they were remembered in (oldest first).
#[derive(Default)]
struct Store {
    marks: BTreeMap<String, String>,
    order: Vec<String>,
}

static STORE: Mutex<Option<Store>> = Mutex::new(None);

fn file() -> Option<PathBuf> {
    wordcraft_control_key::settings_dir().map(|d| d.join("bookmarks.json"))
}

fn read() -> Store {
    let Some(bytes) = file().and_then(|f| std::fs::read(f).ok()) else { return Store::default() };
    let Ok(list) = serde_json::from_slice::<Vec<(String, String)>>(&bytes) else { return Store::default() };
    let mut store = Store::default();
    for (path, mark) in list.into_iter().take(MAX_BOOKMARKS) {
        if !path.is_empty() && !store.marks.contains_key(&path) {
            store.order.push(path.clone());
            store.marks.insert(path, mark);
        }
    }
    store
}

fn write(store: &Store) {
    let list: Vec<(&String, &String)> = store.order.iter().filter_map(|p| store.marks.get(p).map(|m| (p, m))).collect();
    if let (Some(f), Ok(bytes)) = (file(), serde_json::to_vec(&list)) {
        if let Some(dir) = f.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Err(e) = std::fs::write(&f, bytes) {
            log::warn!("bookmarks: cannot save {}: {e}", f.display());
        }
    }
}

/// At start: regain access to every remembered document; stale bookmarks are made afresh.
pub fn restore_all() {
    if !cfg!(target_os = "macos") {
        return;
    }
    let mut store = read();
    let mut changed = false;
    let paths: Vec<String> = store.order.clone();
    for path in paths {
        let Some(bytes) = store.marks.get(&path).and_then(|m| base64_decode(m)) else { continue };
        if let Some((now, stale)) = lexicon_sandbox::access(&bytes)
            && (stale || now != path)
            && let Some(fresh) = lexicon_sandbox::bookmark(&now)
        {
            store.marks.remove(&path);
            store.order.retain(|p| p != &path);
            store.order.push(now.clone());
            store.marks.insert(now, base64_encode(&fresh));
            changed = true;
        }
    }
    if changed {
        write(&store);
    }
    *STORE.lock().unwrap_or_else(|e| e.into_inner()) = Some(store);
}

/// Remember `path` (Write has access to it now: just opened or saved).
pub fn remember(path: &str) {
    if !cfg!(target_os = "macos") || path.is_empty() {
        return;
    }
    let Some(bytes) = lexicon_sandbox::bookmark(path) else { return };
    let mut guard = STORE.lock().unwrap_or_else(|e| e.into_inner());
    let store = guard.get_or_insert_with(read);
    store.order.retain(|p| p != path);
    store.order.push(path.to_string());
    store.marks.insert(path.to_string(), base64_encode(&bytes));
    while store.order.len() > MAX_BOOKMARKS {
        let old = store.order.remove(0);
        store.marks.remove(&old);
    }
    write(store);
}
