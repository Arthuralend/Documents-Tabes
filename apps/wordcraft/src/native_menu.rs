//! macOS: Write's menu bar beside the Apple menu — Write (About, Settings, Hide, Quit), File and
//! Window — as Mac apps have it. The File menu takes the place of the ribbon's File tab on the
//! Mac; its items run the same commands. Labels follow the interface language.

use std::str::FromStr;
use std::sync::mpsc::{Receiver, channel};

use muda::accelerator::Accelerator;
use muda::{Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu};
use serde_json::{Value, json};
use wordcraft_ui_egui::WordApp;

/// One File menu item: English label (translated when shown), what it runs, its shortcut. A label
/// of `-` is a separator.
type Entry = (&'static str, &'static str, Option<&'static str>);

/// The File menu. `backstage:<page>` opens that page of File (Info, Recover, Export); `dialog:`
/// opens a dialog; anything else is a command.
const FILE: [Entry; 15] = [
    ("New", "file.new", Some("CMD+N")),
    ("Open…", "file.open", Some("CMD+O")),
    ("Recent", "backstage:open", None),
    ("-", "", None),
    ("Close", "file.close", Some("CMD+W")),
    ("Save", "file.save", Some("CMD+S")),
    ("Save As…", "file.saveAs", Some("CMD+SHIFT+S")),
    ("Save as PDF", "file.exportPdf", None),
    ("Export", "backstage:export", None),
    ("-", "", None),
    ("Page Setup…", "layout.pageSetup", None),
    ("Print…", "file.print", Some("CMD+P")),
    ("-", "", None),
    ("Info", "backstage:info", None),
    ("Recover", "backstage:recover", None),
];

/// The Write menu's own items.
const ABOUT: &str = "dialog:about";
const SETTINGS: &str = "backstage:options";

pub struct NativeMenu {
    menu: Menu,
    events: Receiver<MenuEvent>,
    /// The interface language the labels were made in: a change rebuilds the menus.
    language: String,
}

/// `s` in the interface language; "Open…" is "Open" translated, with the ellipsis.
fn t(s: &str) -> String {
    match s.strip_suffix('…') {
        Some(base) => format!("{}…", wordcraft_ui_egui::i18n::t(base)),
        None => wordcraft_ui_egui::i18n::t(s).to_string(),
    }
}

/// The menu bar in the current interface language.
fn build() -> Menu {
    let menu = Menu::new();
    let app_menu = Submenu::new("Write", true);
    let _ = app_menu.append_items(&[
        &MenuItem::with_id(ABOUT, t("About Write"), true, None),
        &PredefinedMenuItem::separator(),
        &MenuItem::with_id(SETTINGS, t("Options"), true, Accelerator::from_str("CMD+Comma").ok()),
        &PredefinedMenuItem::separator(),
        &PredefinedMenuItem::services(None),
        &PredefinedMenuItem::separator(),
        &PredefinedMenuItem::hide(None),
        &PredefinedMenuItem::hide_others(None),
        &PredefinedMenuItem::show_all(None),
        &PredefinedMenuItem::separator(),
        // AppKit's terminate:, as before: Write asks about unsaved changes first.
        &PredefinedMenuItem::quit(None),
    ]);
    let _ = menu.append(&app_menu);
    let file = Submenu::new(t("File"), true);
    for (label, id, acc) in FILE {
        if label == "-" {
            let _ = file.append(&PredefinedMenuItem::separator());
        } else {
            let accel = acc.and_then(|a| Accelerator::from_str(a).ok());
            let _ = file.append(&MenuItem::with_id(id, t(label), true, accel));
        }
    }
    let _ = menu.append(&file);
    let window = Submenu::new(t("Window"), true);
    let _ = window.append_items(&[&PredefinedMenuItem::minimize(None), &PredefinedMenuItem::maximize(None), &PredefinedMenuItem::fullscreen(None)]);
    let _ = menu.append(&window);
    menu.init_for_nsapp();
    // Where macOS adds Move & Resize and the tiling shortcuts.
    window.set_as_windows_menu_for_nsapp();
    menu
}

impl NativeMenu {
    pub fn install(ctx: &egui::Context) -> NativeMenu {
        let (tx, rx) = channel();
        let ctx = ctx.clone();
        MenuEvent::set_event_handler(Some(move |e: MenuEvent| {
            let _ = tx.send(e);
            ctx.request_repaint();
        }));
        NativeMenu { menu: build(), events: rx, language: wordcraft_ui_egui::i18n::current().code().to_string() }
    }

    /// Runs the chosen items, and remakes the menus in a new interface language.
    pub fn poll(&mut self, app: &mut WordApp) {
        let language = wordcraft_ui_egui::i18n::current().code().to_string();
        if language != self.language {
            self.menu = build();
            self.language = language;
        }
        while let Ok(ev) = self.events.try_recv() {
            let id = ev.id.as_ref().to_string();
            let (command, params): (&str, Value) = if let Some(page) = id.strip_prefix("backstage:") {
                ("ui.backstage", json!({"value": true, "page": page}))
            } else if let Some(name) = id.strip_prefix("dialog:") {
                ("ui.dialog", json!({"name": name}))
            } else {
                (id.as_str(), json!({}))
            };
            if let Err(e) = app.run(command, params) {
                app.status(e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    /// Every File menu item runs something Write knows.
    #[test]
    fn file_menu_items_run_known_commands() {
        let session = wordcraft_engine::Session::new(wordcraft_doc::Document::new());
        let app = wordcraft_ui_egui::WordApp::new(session, Default::default());
        for (label, id, _) in super::FILE {
            if label == "-" || id.starts_with("backstage:") || id.starts_with("dialog:") {
                continue;
            }
            assert!(app.session.registry.get(id).is_some(), "{label}: {id}");
        }
    }
}
