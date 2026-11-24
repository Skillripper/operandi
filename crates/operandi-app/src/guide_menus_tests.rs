//! The User Guide's Menus pages (docs/guide/src/menus/) are written from the
//! app's own menu model (`menu_model`, the forms in assets/forms/menu.*),
//! never by hand. This test fails when a page and the menus differ; run it
//! with OPERANDI_BLESS_GUIDE=1 to write the pages again:
//!
//!   OPERANDI_BLESS_GUIDE=1 scripts/check-fast.sh guide_menus
use super::*;
use crate::app_menu::{Action, Entry, Menu, action_line};
use crate::guide_shortcuts_tests::guide_chord;

const DIR: &str = "../../docs/guide/src/menus";

/// A menu's file name: its title in lower case, spaces as dashes.
fn slug(title: &str) -> String {
    title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_owned()
}

/// An entry's title as the guide writes it. The guide never names another
/// application (guide RULES.md), so a format named after one is written by
/// its file type alone.
fn guide_title(title: &str) -> String {
    let title = title.replace("Modo Scene (LXO)", "LXO Scene");
    // Undo and Redo name the change they'd take back; the guide doesn't.
    for word in ["Undo", "Redo"] {
        if title.starts_with(&format!("{word} ")) {
            return word.to_owned();
        }
    }
    title
}

/// A title with its key in brackets ("Redo  [Shift+Cmd+Z] ") split into
/// the title and the key.
fn split_key(title: &str) -> (String, Option<String>) {
    match title
        .trim_end()
        .strip_suffix(']')
        .and_then(|t| t.rsplit_once("  ["))
    {
        Some((name, key)) => (name.trim().to_owned(), Some(key.to_owned())),
        None => (title.to_owned(), None),
    }
}

/// A native key equivalent ("s", "S") as a chord: Cmd, plus Shift for a
/// capital letter.
fn key_equivalent(key: &str) -> String {
    match key.chars().next() {
        Some(c) if key.chars().count() == 1 && c.is_ascii_uppercase() => format!("Shift+Cmd+{c}"),
        Some(c) if key.chars().count() == 1 => format!("Cmd+{}", c.to_ascii_uppercase()),
        _ => key.to_owned(),
    }
}

/// A key shown in a menu title ("Ctrl+Z") as the key map saves it
/// ("Cmd+Z"). On Linux and Windows a title shows Cmd as Ctrl
/// (`input::display_chord`), which would make the guide call it Control and
/// write different pages there than on the Mac. The entry's own binding wins;
/// otherwise any binding that shows as this key.
fn saved_chord(
    shown: &str,
    line: &str,
    title: &str,
    keys: &std::collections::BTreeMap<String, String>,
) -> String {
    let shows_as = |k: &&String| input::display_chord(k) == shown;
    keys.get(line)
        .filter(shows_as)
        .or_else(|| keys.get(title).filter(shows_as))
        .or_else(|| keys.values().find(shows_as))
        .cloned()
        .unwrap_or_else(|| shown.to_owned())
}

/// The menus as a user with one mesh in Polygon mode sees them.
fn menus() -> Vec<Menu> {
    let mut a = Operandi::empty();
    a.run(Command::NewMesh);
    a.run(Command::Mode(Mode::Polygon));
    a.menu_model(&Context::default())
}

fn entries(
    list: &[Entry],
    depth: usize,
    keys: &std::collections::BTreeMap<String, String>,
    out: &mut String,
) {
    let pad = "  ".repeat(depth);
    for entry in list {
        match entry {
            Entry::Separator => {}
            Entry::Submenu(menu) => {
                out.push_str(&format!("{pad}- **{} ▸**\n", guide_title(&menu.title)));
                // Lists of open scenes or recent projects depend on what the
                // user has open; describe them instead of listing them.
                if menu.id == "menu.file.current" || menu.id.contains("recent") {
                    out.push_str(&format!(
                        "{pad}  - the open scenes or recent projects, one entry each\n"
                    ));
                } else {
                    entries(&menu.entries, depth + 1, keys, out);
                }
            }
            Entry::Item(item) => {
                let line = match item.action {
                    Action::Line(i) => action_line(i).unwrap_or_default(),
                    _ => String::new(),
                };
                let (title, shown) = split_key(&item.title);
                let shown = shown.map(|k| saved_chord(&k, &line, &title, keys));
                let key = shown
                    .or_else(|| keys.get(line.as_str()).cloned())
                    .or_else(|| keys.get(title.as_str()).cloned())
                    .filter(|k| !k.is_empty())
                    .or_else(|| Some(key_equivalent(&item.command_key)).filter(|k| !k.is_empty()))
                    .map(|k| format!(" (**{}**)", guide_chord(&k)))
                    .unwrap_or_default();
                let command = if line.is_empty() {
                    String::new()
                } else {
                    format!(" · `{}`", line.replace('`', "'"))
                };
                out.push_str(&format!(
                    "{pad}- **{}**{key}{command}\n",
                    guide_title(&title)
                ));
            }
        }
    }
}

fn pages() -> Vec<(String, String)> {
    let keys = input::default_bindings();
    let menus = menus();
    let mut index = String::from(
        "<!-- help-ids: ; template: panel ; since: 0.1.0 -->\n\
         <!-- Written by crates/operandi-app/src/guide_menus_tests.rs from the app's \
         menus. Don't edit by hand: run OPERANDI_BLESS_GUIDE=1 scripts/check-fast.sh guide_menus -->\n\
         # Menus\n\n\
         Every command in Operandi is in a menu. On a Mac the menus are in the \
         macOS menu bar at the top of the screen; on Windows and Linux they are \
         in the menu bar at the top of the window. Each page lists one menu's \
         entries in order, with the key that runs an entry (if it has one) and \
         the command it runs, which you can also type in the **Command entry** \
         or use in scripts.\n\n\
         Menus are forms: **Edit Forms** in Layout mode changes them, so yours \
         may differ.\n\n",
    );
    let mut out = Vec::new();
    for menu in &menus {
        let file = format!("{}.md", slug(&menu.title));
        index.push_str(&format!("- [{}]({file})\n", menu.title));
        let ids = if menu.id.is_empty() {
            String::new()
        } else {
            menu.id.clone()
        };
        let mut page = format!(
            "<!-- help-ids: {ids} ; template: panel ; since: 0.1.0 -->\n\
             <!-- Written by crates/operandi-app/src/guide_menus_tests.rs from the app's \
             menus. Don't edit by hand: run OPERANDI_BLESS_GUIDE=1 scripts/check-fast.sh guide_menus -->\n\
             # {} menu\n\n\
             The entries of the **{}** menu, in order. A **▸** opens a submenu. \
             Keys in brackets run the entry; the `code` is the command it runs.\n\n",
            menu.title, menu.title
        );
        entries(&menu.entries, 0, &keys, &mut page);
        page.push_str("\n## Related\n\n- [Menus](index.md)\n- [Keyboard shortcuts](../reference/keyboard-shortcuts.md)\n");
        out.push((file, page));
    }
    out.push(("index.md".into(), index));
    out
}

#[test]
fn guide_menu_pages_match_the_menus() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(DIR);
    let bless = std::env::var_os("OPERANDI_BLESS_GUIDE").is_some();
    if bless {
        std::fs::create_dir_all(&dir).unwrap();
    }
    for (file, page) in pages() {
        let path = dir.join(&file);
        if bless {
            std::fs::write(&path, &page).unwrap();
            continue;
        }
        let saved = std::fs::read_to_string(&path).unwrap_or_default();
        assert!(
            saved == page,
            "docs/guide/src/menus/{file} differs from the menus; \
             run OPERANDI_BLESS_GUIDE=1 scripts/check-fast.sh guide_menus"
        );
    }
}
