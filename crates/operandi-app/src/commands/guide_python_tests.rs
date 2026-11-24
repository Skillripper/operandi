//! The User Guide's Scripting pages stay true to the code (guide RULES.md):
//!
//! - Scripting > Python reference is written from `operandi.py` (the
//!   `operandi` module's functions and their docstrings), the command
//!   registry's queries and the app's script commands, never by hand. The
//!   test fails when the page and the code differ; run it with
//!   OPERANDI_BLESS_GUIDE=1 to write the page again:
//!
//!     OPERANDI_BLESS_GUIDE=1 scripts/check-fast.sh guide_python
//!
//! - Every ```python block in the Scripting pages runs in the app as a
//!   script and must finish without an error (with the `python` feature;
//!   `scripts/merge-check.sh` runs those). A block that can't run in a test
//!   (it needs a file, or shows an error on purpose) is fenced
//!   ```python,norun instead.
use std::path::{Path, PathBuf};

const REFERENCE: &str = "../../docs/guide/src/scripting/python-reference.md";
const SCRIPTING: &str = "../../docs/guide/src/scripting";
const MODULE: &str = include_str!("../../../operandi-python/src/operandi.py");

/// One public name of `operandi.py`: `def` or `class`, with its docstring.
#[derive(Debug, PartialEq)]
struct PyName {
    /// `run(command, check=True, **args)`, or `CommandError` for a class.
    signature: String,
    doc: String,
    /// Used as `with ...:` (a context manager).
    with: bool,
    class: bool,
}

/// The module's public functions and classes, in file order. Only
/// top-level names without a leading underscore count; a docstring is the
/// triple-quoted string right under the `def`/`class` line.
fn public_names(source: &str) -> Vec<PyName> {
    let lines: Vec<&str> = source.lines().collect();
    let mut out = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        let (class, rest) = if let Some(r) = line.strip_prefix("def ") {
            (false, r)
        } else if let Some(r) = line.strip_prefix("class ") {
            (true, r)
        } else {
            continue;
        };
        if rest.starts_with('_') {
            continue;
        }
        let head = rest.trim_end_matches(':');
        let signature = if class {
            head.split('(').next().unwrap_or(head).to_owned()
        } else {
            head.to_owned()
        };
        let with = i > 0
            && lines[i - 1]
                .trim_start()
                .starts_with("@_contextlib.contextmanager");
        let mut doc = Vec::new();
        let mut j = i + 1;
        if let Some(first) = lines.get(j).map(|l| l.trim())
            && let Some(open) = first.strip_prefix("\"\"\"")
        {
            if let Some(one) = open.strip_suffix("\"\"\"") {
                doc.push(one.trim().to_owned());
            } else {
                doc.push(open.trim().to_owned());
                j += 1;
                while let Some(l) = lines.get(j) {
                    let t = l.trim();
                    if let Some(last) = t.strip_suffix("\"\"\"") {
                        doc.push(last.trim().to_owned());
                        break;
                    }
                    doc.push(t.to_owned());
                    j += 1;
                }
            }
        }
        // Paragraphs as the docstring has them: one blank line between.
        let doc = doc
            .split(|l| l.is_empty())
            .map(|para| para.join(" "))
            .filter(|p| !p.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n\n");
        out.push(PyName {
            signature,
            doc,
            with,
            class,
        });
    }
    out
}

/// Help text from the code as a table cell: `|` and `<` escaped, and the
/// rule numbers the code cites for its developers ("(UIC-163)",
/// "(IK-24, IK-40; angles in degrees)") left out, since the guide's
/// readers don't need them.
fn cell(s: &str) -> String {
    plain(s)
        .replace('|', "\\|")
        .replace('<', "&lt;")
        .replace('\n', " ")
}

/// `s` without rule numbers in brackets: a bracket of nothing but rule
/// numbers goes, with the space before it; other brackets lose just their
/// rule numbers.
fn plain(s: &str) -> String {
    let rule = |part: &str| {
        let part = part.trim();
        part.split_once('-').is_some_and(|(head, num)| {
            (1..=4).contains(&head.len())
                && head.chars().all(|c| c.is_ascii_uppercase())
                && !num.is_empty()
                && num.chars().all(|c| c.is_ascii_digit() || c == '.')
        })
    };
    let mut out = String::new();
    let mut rest = s;
    while let Some(open) = rest.find('(') {
        let Some(len) = rest[open..].find(')') else {
            break;
        };
        let inner = &rest[open + 1..open + len];
        out.push_str(&rest[..open]);
        let kept: Vec<&str> = inner
            .split(';')
            // Only clauses that are nothing but rule numbers go.
            .filter(|clause| !clause.split(',').all(rule))
            .collect();
        if kept.is_empty() {
            while out.ends_with(' ') {
                out.pop();
            }
        } else {
            out.push('(');
            out.push_str(kept.join(";").trim());
            out.push(')');
        }
        rest = &rest[open + len + 1..];
    }
    out.push_str(rest);
    out
}

fn page() -> String {
    let mut out = String::from(
        "<!-- help-ids: ; template: concept ; since: 0.1.0 -->\n\
         <!-- Written by crates/operandi-app/src/commands/guide_python_tests.rs from \
         crates/operandi-python/src/operandi.py and the command registry. Don't edit \
         by hand: run OPERANDI_BLESS_GUIDE=1 scripts/check-fast.sh guide_python -->\n\
         # Python reference\n\n\
         This page lists everything the `operandi` module offers, the commands a \
         script can ask questions with, and the commands that run scripts. It is \
         made from Operandi's own code, so it matches the version it came with. \
         For how to turn scripting on and run a script, see \
         [Python scripting](python.md).\n\n\
         ## The `operandi` module\n\n\
         `import operandi` in a script file. In the **Script Console** it is \
         already imported.\n\n",
    );
    for name in public_names(MODULE) {
        let heading = if name.class {
            format!("### `{}` (exception)\n\n", name.signature)
        } else if name.with {
            format!("### `with operandi.{}:`\n\n", name.signature)
        } else {
            format!("### `operandi.{}`\n\n", name.signature)
        };
        out.push_str(&heading);
        if !name.doc.is_empty() {
            out.push_str(&name.doc);
            out.push_str("\n\n");
        }
    }

    out.push_str(
        "## Queries\n\n\
         These commands answer a question and change nothing. Ask them with \
         `operandi.query(...)`; a JSON answer comes back as Python lists and \
         dictionaries. `item.channel` and `transform.channel` are queries only \
         when `value` is `\"?\"`.\n\n\
         | Command | What it answers | Example |\n|---|---|---|\n",
    );
    // Only the queries a script can ask: those with a headless form. The
    // others (command.list, history.max...) answer in the status line.
    let mut queries: Vec<_> = operandi_core::command_registry::COMMANDS
        .iter()
        .filter(|e| e.flags.query && operandi_core::command_exec::spec(e.name).is_some())
        .collect();
    queries.sort_by_key(|e| e.name.to_ascii_lowercase());
    for e in queries {
        out.push_str(&format!(
            "| `{}` | {} | `{}` |\n",
            e.name,
            cell(e.help),
            cell(e.example)
        ));
    }

    out.push_str(
        "\n## Script commands\n\n\
         The commands behind the **Script Console** and **System > Run Script…**. \
         You can put them on a button or a menu like any other command. A \
         script can't run these itself: a script can't start another script.\n\n\
         | Command | What it does |\n|---|---|\n",
    );
    let registry = crate::command_handlers::registry();
    for e in registry.matching("operandi.python") {
        out.push_str(&format!("| `{}` | {} |\n", cell(&e.usage()), cell(e.help)));
    }
    out.push_str(
        "\nThe bulk mesh write that `operandi.mesh_set` makes is also a command \
         of its own, for the test bridge and typed lines:\n\n\
         | Command | What it does |\n|---|---|\n",
    );
    let e = operandi_core::command_registry::find("operandi.mesh.setData")
        .expect("mesh.setData is registered");
    out.push_str(&format!("| `{}` | {} |\n", cell(&e.usage()), cell(e.help)));
    out
}

fn manifest(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

#[test]
fn guide_python_reference_matches_the_code() {
    let path = manifest(REFERENCE);
    let now = page();
    if std::env::var_os("OPERANDI_BLESS_GUIDE").is_some() {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, &now).unwrap();
        return;
    }
    let saved = std::fs::read_to_string(&path).unwrap_or_default();
    assert!(
        saved == now,
        "docs/guide/src/scripting/python-reference.md differs from the code; \
         run OPERANDI_BLESS_GUIDE=1 scripts/check-fast.sh guide_python"
    );
}

#[test]
fn guide_python_reference_reads_every_public_name() {
    let names = public_names(MODULE);
    let sig = |s: &str| names.iter().find(|n| n.signature.starts_with(s));
    let run = sig("run(").expect("run");
    assert!(run.doc.starts_with("Runs a command line"), "{run:?}");
    assert!(sig("undo_group(").is_some_and(|n| n.with));
    assert!(sig("CommandError").is_some_and(|n| n.class && !n.doc.is_empty()));
    assert!(names.iter().all(|n| !n.signature.starts_with('_')));
    // Every name has a docstring: the reference has nothing else to say.
    for n in &names {
        assert!(!n.doc.is_empty(), "{} has no docstring", n.signature);
    }
}

/// The ```python blocks of a guide page, in order.
fn python_blocks(page: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current: Option<Vec<&str>> = None;
    for line in page.lines() {
        match &mut current {
            None if line.trim_end() == "```python" => current = Some(Vec::new()),
            Some(lines) if line.trim_end() == "```" => {
                out.push(lines.join("\n"));
                current = None;
            }
            Some(lines) => lines.push(line),
            None => {}
        }
    }
    out
}

#[test]
fn guide_python_reference_leaves_out_rule_numbers() {
    assert_eq!(
        plain("Reads levels as JSON (UIC-163)."),
        "Reads levels as JSON."
    );
    assert_eq!(
        plain("Reads the graph (UIC-163; the Schematic's data)."),
        "Reads the graph (the Schematic's data)."
    );
    assert_eq!(
        plain("Nodes (RG-70, UIC-163) and wires."),
        "Nodes and wires."
    );
    assert_eq!(
        plain("Settings (IK-24, IK-40; angles in degrees)."),
        "Settings (angles in degrees)."
    );
    assert_eq!(plain("Points (x, y, z) stay."), "Points (x, y, z) stay.");
    assert_eq!(cell("file:<path.py> | x"), "file:&lt;path.py> \\| x");
}

#[test]
fn guide_python_blocks_are_found() {
    let page = "a\n```python\nprint(1)\n```\n```python,norun\nx\n```\n```python\nprint(2)\nprint(3)\n```\n";
    assert_eq!(python_blocks(page), ["print(1)", "print(2)\nprint(3)"]);
}

/// Every Scripting page's ```python blocks, as (page, block).
#[cfg_attr(not(feature = "python"), allow(dead_code))]
fn guide_examples() -> Vec<(String, String)> {
    let mut pages: Vec<PathBuf> = std::fs::read_dir(manifest(SCRIPTING))
        .expect("the Scripting pages")
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "md"))
        .collect();
    pages.sort();
    let mut out = Vec::new();
    for p in pages {
        let text = std::fs::read_to_string(&p).unwrap();
        let name = p.file_name().unwrap().to_string_lossy().into_owned();
        for block in python_blocks(&text) {
            out.push((name.clone(), block));
        }
    }
    out
}

#[cfg(feature = "python")]
#[test]
fn every_python_example_in_the_guide_runs() {
    use crate::tests::{box_app, draw};
    use eframe::egui;
    let _turn = crate::python_ui_tests::TURN
        .lock()
        .unwrap_or_else(|p| p.into_inner());
    crate::python_switch::set_for_test(true);
    let examples = guide_examples();
    assert!(examples.len() >= 5, "the Scripting pages have examples");
    for (page, code) in examples {
        let ctx = egui::Context::default();
        let mut a = box_app();
        a.python.last = None;
        a.python_start(operandi_python::Source::file_text(code.clone(), "guide.py"));
        let mut outcome = None;
        for _ in 0..2000 {
            let _ = draw(&mut a, &ctx, egui::vec2(1400., 900.), vec![]);
            if let Some(o) = a.python.last.take() {
                outcome = Some(o);
                break;
            }
        }
        let o = outcome.unwrap_or_else(|| panic!("{page}: the example didn't finish:\n{code}"));
        assert!(
            o.ok,
            "{page}: the example failed:\n{code}\n--- output\n{}\n--- error\n{}",
            o.output,
            o.error.unwrap_or_default()
        );
    }
}

/// Kits and extensions: the line that puts a script on the System menu
/// does put it there.
#[test]
fn a_form_add_line_puts_a_script_on_the_system_menu() {
    use crate::tests::{box_app, draw};
    use eframe::egui;
    crate::python_switch::set_for_test(true);
    let ctx = egui::Context::default();
    let mut a = box_app();
    let script = "operandi.python.run file:{/Users/me/Scripts/wave.py}";
    a.execute(&format!("operandi.form.add {{{script}}} form:menu.system"));
    assert!(a.last_failure.is_none(), "{}", a.status);
    let _ = draw(&mut a, &ctx, egui::vec2(1400., 900.), vec![]);
    let model = a.menu_model(&ctx);
    let system = model
        .iter()
        .find(|m| m.title == "System")
        .expect("System menu");
    let lines: Vec<String> = system
        .entries
        .iter()
        .filter_map(|e| match e {
            crate::app_menu::Entry::Item(crate::app_menu::Item {
                action: crate::app_menu::Action::Line(n),
                ..
            }) => crate::app_menu::action_line(*n),
            _ => None,
        })
        .collect();
    assert!(lines.iter().any(|l| l == script), "{lines:?}");
}
