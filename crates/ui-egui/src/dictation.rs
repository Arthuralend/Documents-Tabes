//! Lexicon: Home › Dictate with the operating system's speech tools.
//!
//! The host starts recognition (`Services::dictate`) and hands back what was heard
//! (`Services::heard`). Документы types the words into the document as they are heard: each new
//! guess of the phrase replaces the last one in place, the final words settle it (one undo step
//! per phrase), and the words just added glow for a moment ([`trail`]) so it is plain where the
//! text goes. Listening goes on until the Dictate button is pressed again. Where the system types
//! by itself (Windows voice typing, Android's speech screen) no words come back here.

use serde_json::json;
use wordcraft_doc::Pos;

use crate::WordApp;

/// How the host started dictation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Started {
    /// The host listens and reports [`Heard`] events.
    Listening,
    /// The system's own voice typing types into the window.
    System,
    /// The user is asked to allow speech recognition; [`Heard::Authorized`] follows.
    AskingPermission,
}

/// What the host's recogniser reported.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Heard {
    Partial(String),
    Final(String),
    Error(String),
    Authorized(bool),
}

/// A pause this long (ms) after the last words ends the phrase.
const PAUSE_MS: f64 = 1800.0;

#[derive(Default)]
pub struct DictationState {
    /// Listening (the Dictate button is on).
    pub active: bool,
    /// The phrase heard so far, and when it last changed.
    partial: String,
    partial_at: f64,
    /// The phrase is being finished (waiting for its final words).
    finishing: bool,
    /// The phrase as typed into the document so far: from `start` to `end`.
    live: Option<(Pos, Pos)>,
    /// The text of the phrase as typed (what `live` holds).
    partial_typed: String,
    /// The words just added (from, to) and when: they glow for [`TRAIL_MS`].
    trail: Option<(Pos, Pos, f64)>,
}

/// How long (ms) words just dictated glow.
pub const TRAIL_MS: f64 = 1000.0;

/// The words just dictated and how strongly they glow (1 → 0 over [`TRAIL_MS`]), if any.
pub(crate) fn trail(app: &WordApp, now: f64) -> Option<(Pos, Pos, f32)> {
    let (a, b, at) = app.dictation.trail.clone()?;
    let age = now - at;
    (age.is_finite() && (0.0..TRAIL_MS).contains(&age)).then(|| (a, b, (1.0 - age / TRAIL_MS) as f32))
}

/// Bytes `old` and `new` share at their start, at a character boundary.
fn common_prefix(old: &str, new: &str) -> usize {
    old.char_indices().zip(new.chars()).take_while(|((_, a), b)| a == b).map(|((i, a), _)| i + a.len_utf8()).last().unwrap_or(0)
}

/// Put `text` in the document as the phrase being dictated: in place of the phrase's earlier
/// guess while the caret is still at its end, else at the caret as a new phrase.
fn type_phrase(app: &mut WordApp, text: &str, now: f64) {
    let caret_at_end = app.dictation.live.as_ref().is_some_and(|(_, end)| app.session.sel.is_collapsed() && app.session.sel.focus == *end);
    let (start, old) = match app.dictation.live.clone() {
        Some((start, end)) if caret_at_end => {
            app.session.sel = wordcraft_engine::Selection { anchor: start.clone(), focus: end };
            // One undo step for the whole phrase.
            app.session.join_next_undo();
            (start, app.dictation.partial_typed.clone())
        }
        _ => (app.session.sel.ordered().0, String::new()),
    };
    if app.run("text.insert", json!({"text": text})).is_err() {
        app.dictation.live = None;
        return;
    }
    let end = app.session.sel.focus.clone();
    let same_para = start.story == end.story && start.path == end.path;
    let from = Pos { off: (start.off + common_prefix(&old, text)).min(end.off), ..start.clone() };
    app.dictation.trail = same_para.then(|| (from, end.clone(), now));
    app.dictation.live = same_para.then_some((start, end));
    app.dictation.partial_typed = text.to_string();
}

/// Dictate pressed: start listening, or stop.
pub(crate) fn toggle(app: &mut WordApp) {
    if app.dictation.active {
        stop(app);
        app.status(tl!("Dictation stopped."));
        return;
    }
    start(app);
}

/// The languages Dictate offers (code, name in that language).
pub const LANGUAGES: [(&str, &str); 3] = [("ru", "Русский"), ("en", "English"), ("uk", "Українська")];

/// The language to listen for: the one chosen, else the interface's when Dictate offers it, else
/// Russian.
pub(crate) fn language(app: &WordApp) -> String {
    let chosen = app.ui.dictation_language.as_str();
    if LANGUAGES.iter().any(|(c, _)| *c == chosen) {
        return chosen.to_string();
    }
    let ui = crate::i18n::Lang::from_pref(&app.ui.language).code().to_string();
    if LANGUAGES.iter().any(|(c, _)| *c == ui) { ui } else { "ru".to_string() }
}

fn start(app: &mut WordApp) {
    let lang = language(app);
    let started = app.services.dictate.as_ref().map(|f| f(&lang));
    match started {
        Some(Ok(Started::Listening)) => {
            app.dictation.active = true;
            app.dictation.partial.clear();
            app.dictation.finishing = false;
            app.dictation.live = None;
            app.status(tl!("Listening… Speak now; your words appear at the caret."));
        }
        Some(Ok(Started::System)) => app.status(tl!("The system's voice typing is open: speak, and the words are typed at the caret.")),
        Some(Ok(Started::AskingPermission)) => app.status(tl!("Allow speech recognition and the microphone when macOS asks.")),
        Some(Err(e)) if !e.is_empty() => app.status(e),
        _ if cfg!(windows) => app.status(tl!("Press the Windows logo key + H to dictate with Windows voice typing.")),
        _ => app.status(tl!("Voice typing isn't available on this system.")),
    }
}

fn stop(app: &mut WordApp) {
    app.dictation.active = false;
    if let Some(f) = app.services.dictate_stop.as_ref() {
        f();
    }
}

/// Each frame: type what was heard, end phrases after a pause, keep listening.
pub(crate) fn poll(app: &mut WordApp, now: f64) {
    let heard = app.services.heard.as_ref().map(|f| f()).unwrap_or_default();
    for h in heard {
        match h {
            Heard::Partial(text) => {
                if text != app.dictation.partial {
                    app.dictation.partial = text.clone();
                    app.dictation.partial_at = now;
                    // Straight into the document, as it is heard.
                    if app.dictation.active && !text.trim().is_empty() {
                        type_phrase(app, text.trim(), now);
                    }
                }
            }
            Heard::Final(text) => {
                let text = text.trim();
                if !text.is_empty() {
                    type_phrase(app, &format!("{text} "), now);
                } else if app.dictation.live.is_some() {
                    // Nothing final: the guess stays as typed, with a space after it.
                    let typed = format!("{} ", app.dictation.partial_typed);
                    type_phrase(app, &typed, now);
                }
                app.dictation.live = None;
                app.dictation.partial_typed.clear();
                app.dictation.partial.clear();
                app.dictation.finishing = false;
                // Listening goes on with the next phrase.
                if app.dictation.active {
                    app.dictation.active = false;
                    start(app);
                }
            }
            Heard::Error(e) => {
                // A phrase ended without words is not an error worth showing.
                let had_words = !app.dictation.partial.is_empty();
                app.dictation.partial.clear();
                // What was typed of the phrase stays.
                app.dictation.live = None;
                app.dictation.partial_typed.clear();
                app.dictation.finishing = false;
                if app.dictation.active && !had_words {
                    app.dictation.active = false;
                    start(app);
                } else if app.dictation.active {
                    app.dictation.active = false;
                    app.status(e);
                }
            }
            Heard::Authorized(true) => start(app),
            Heard::Authorized(false) => {
                app.dictation.active = false;
                app.status(tl!("Speech recognition isn't allowed: turn it on in System Settings › Privacy & Security."));
            }
        }
    }
    if app.dictation.active && !app.dictation.finishing && !app.dictation.partial.is_empty() && now - app.dictation.partial_at > PAUSE_MS {
        app.dictation.finishing = true;
        if let Some(f) = app.services.dictate_stop.as_ref() {
            f();
        }
    }
    if let Some(ctx) = &app.ctx {
        if trail(app, now).is_some() {
            // The glow fades smoothly.
            ctx.request_repaint();
        } else if app.dictation.active {
            ctx.request_repaint_after(std::time::Duration::from_millis(250));
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use super::*;
    use wordcraft_engine::Session;

    #[test]
    fn heard_words_are_typed_and_listening_goes_on() {
        let queue: Rc<RefCell<Vec<Heard>>> = Rc::default();
        let starts = Rc::new(RefCell::new(0));
        let stops = Rc::new(RefCell::new(0));
        let (q, s1, s2) = (queue.clone(), starts.clone(), stops.clone());
        let services = crate::Services {
            dictate: Some(Box::new(move |_| {
                *s1.borrow_mut() += 1;
                Ok(Started::Listening)
            })),
            dictate_stop: Some(Box::new(move || *s2.borrow_mut() += 1)),
            heard: Some(Box::new(move || std::mem::take(&mut *q.borrow_mut()))),
            ..Default::default()
        };
        let mut a = WordApp::new(Session::new(wordcraft_doc::Document::new()), services);
        a.run("tools.dictate", json!({})).unwrap();
        assert!(a.dictation.active);
        queue.borrow_mut().push(Heard::Partial("привет".into()));
        poll(&mut a, 1000.0);
        let text = |a: &WordApp| a.session.doc.plain_text(wordcraft_doc::StoryRef::Body);
        assert_eq!(text(&a), "привет", "the words go straight into the document");
        assert!(trail(&a, 1000.0).is_some_and(|(_, _, k)| k > 0.99), "and glow");
        queue.borrow_mut().push(Heard::Partial("привет ми".into()));
        poll(&mut a, 1100.0);
        assert_eq!(text(&a), "привет ми", "a new guess replaces the last in place");
        let (from, to, _) = trail(&a, 1100.0).unwrap();
        assert_eq!((from.off, to.off), ("привет".len(), "привет ми".len()), "only the new words glow");
        assert!(trail(&a, 1100.0 + TRAIL_MS + 1.0).is_none(), "the glow fades within a second");
        poll(&mut a, 1100.0 + PAUSE_MS + 1.0);
        assert_eq!(*stops.borrow(), 1, "a pause ends the phrase");
        queue.borrow_mut().push(Heard::Final("привет мир".into()));
        poll(&mut a, 5000.0);
        assert_eq!(text(&a), "привет мир ", "the final words settle the phrase");
        assert_eq!(*starts.borrow(), 2, "listening goes on");
        a.run("edit.undo", json!({})).unwrap();
        assert_eq!(text(&a), "", "one undo takes the whole phrase");
        a.run("tools.dictate", json!({})).unwrap();
        assert!(!a.dictation.active, "pressing Dictate again stops");
    }
}
